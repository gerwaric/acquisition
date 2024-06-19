/*
	Copyright 2014 Ilya Zhuravlev

	This file is part of Acquisition.

	Acquisition is free software: you can redistribute it and/or modify
	it under the terms of the GNU General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	Acquisition is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU General Public License for more details.

	You should have received a copy of the GNU General Public License
	along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "itemsmanager.h"

#include <QMessageBox>
#include <QNetworkCookie>
#include <QSettings>

#include "QsLog.h"

#include "application.h"
#include "buyoutmanager.h"
#include "datastore.h"
#include "item.h"
#include "itemsmanagerworker.h"
#include "shop.h"
#include "util.h"
#include "mainwindow.h"
#include "modlist.h"
#include "filters.h"

ItemsManager::ItemsManager(QObject* parent,
	QSettings& settings,
	QNetworkAccessManager& network_manager,
	BuyoutManager& buyout_manager,
	DataStore& datastore,
	RateLimiter& rate_limiter)
	:
	settings_(settings),
	network_manager_(network_manager),
	buyout_manager_(buyout_manager),
	datastore_(datastore),
	rate_limiter_(rate_limiter),
	auto_update_timer_(std::make_unique<QTimer>())
{
	const int interval = settings_.value("autoupdate_interval", 30).toInt();
	auto_update_timer_->setSingleShot(false);
	auto_update_timer_->setInterval(interval * 60 * 1000);
	connect(auto_update_timer_.get(), &QTimer::timeout, this, &ItemsManager::OnAutoRefreshTimer);
}

ItemsManager::~ItemsManager() {}

void ItemsManager::Start(PoeApiMode mode) {
	worker_ = std::make_unique<ItemsManagerWorker>(this,
		settings_,
		network_manager_,
		buyout_manager_,
		datastore_,
		rate_limiter_, mode);
	connect(this, &ItemsManager::UpdateSignal, worker_.get(), &ItemsManagerWorker::Update);
	connect(worker_.get(), &ItemsManagerWorker::StatusUpdate, this, &ItemsManager::OnStatusUpdate);
	connect(worker_.get(), &ItemsManagerWorker::ItemsRefreshed, this, &ItemsManager::OnItemsRefreshed);
	worker_->Init();
}

void ItemsManager::OnStatusUpdate(ProgramState state, const QString& status) {
	emit StatusUpdate(state, status);
}

void ItemsManager::ApplyAutoTabBuyouts() {
	// Can handle everything related to auto-tab pricing here.
	// 1. First format we need to honor is ascendency pricing formats which is top priority and overrides other types
	// 2. Second priority is to honor manual user pricing
	// 3. Third priority it to apply pricing based on ideally user specified formats (doesn't exist yet)

	// Loop over all tabs, create buyout based on tab name which applies auto-pricing policies
	for (auto const& loc : buyout_manager_.GetStashTabLocations()) {
		auto tab_label = loc.get_tab_label();
		Buyout buyout = buyout_manager_.StringToBuyout(tab_label);
		if (buyout.IsActive()) {
			buyout_manager_.SetTab(loc.GetUniqueHash(), buyout);
		};
	};

	// Need to compress tab buyouts here, as the tab names change we accumulate and save BO's
	// for tabs that no longer exist I think.
	buyout_manager_.CompressTabBuyouts();
}

void ItemsManager::ApplyAutoItemBuyouts() {
	// Loop over all items, check for note field with pricing and apply
	for (auto const& item : items_) {
		auto const& note = item->note();
		if (!note.empty()) {
			Buyout buyout = buyout_manager_.StringToBuyout(note);
			// This line may look confusing, buyout returns an active buyout if game
			// pricing was found or a default buyout (inherit) if it was not.
			// If there is a currently valid note we want to apply OR if
			// old note no longer is valid (so basically clear pricing)
			if (buyout.IsActive() || buyout_manager_.Get(*item).IsGameSet()) {
				buyout_manager_.Set(*item, buyout);
			};
		};
	};

	// Commenting this out for robustness (iss381) to make it as unlikely as possible that users
	// pricing data will be removed.  Side effect is that stale pricing data will pile up and
	// could be applied to future items with the same hash (which includes tab name).
	// bo.CompressItemBuyouts(items_);
}

void ItemsManager::PropagateTabBuyouts() {
	buyout_manager_.ClearRefreshLocks();
	for (auto& item_ptr : items_) {
		Item& item = *item_ptr;
		std::string hash = item.location().GetUniqueHash();
		auto item_bo = buyout_manager_.Get(item);
		auto tab_bo = buyout_manager_.GetTab(hash);

		if (item_bo.IsInherited()) {
			if (tab_bo.IsActive()) {
				// Any propagation from tab price to item price should include this bit set
				tab_bo.inherited = true;
				tab_bo.last_update = QDateTime::currentDateTime();
				buyout_manager_.Set(item, tab_bo);
			} else {
				// This effectively 'clears' buyout by setting back to 'inherit' state.
				buyout_manager_.Set(item, Buyout());
			};
		};

		// If any savable bo's are set on an item or the tab then lock the refresh state.
		// Skip remove-only tabs because they are not editable, nor indexed for trade now.
		if (item.location().removeonly() == false) {
			if (buyout_manager_.Get(item).RequiresRefresh() || tab_bo.RequiresRefresh()) {
				buyout_manager_.SetRefreshLocked(item.location());
			};
		};
	};
}

void ItemsManager::OnItemsRefreshed(const Items& items, const std::vector<ItemLocation>& tabs, bool initial_refresh) {
	items_ = items;

	QLOG_INFO() << "There are" << items_.size() << "items and" << tabs.size() << "tabs after the refresh.";
	int n = 0;
	for (const auto& item : items) {
		if (item->category().empty()) {
			QLOG_TRACE() << "Unable to categorize" << item->PrettyName();
			++n;
		};
	};
	if (n > 0) {
		QLOG_INFO() << "There are" << n << " uncategorized items.";
	};

	buyout_manager_.SetStashTabLocations(tabs);
	MigrateBuyouts();
	ApplyAutoTabBuyouts();
	ApplyAutoItemBuyouts();
	PropagateTabBuyouts();

	emit ItemsRefreshed(initial_refresh);
}

void ItemsManager::Update(TabSelection::Type type, const std::vector<ItemLocation>& locations) {
	if (!isInitialized()) {
		// tell ItemsManagerWorker to run an Update() after it's finished updating mods
		worker_->UpdateRequest(type, locations);
		QLOG_DEBUG() << "Update deferred until item mods parsing is complete";
		QMessageBox::information(nullptr,
			"Acquisition",
			"This items worker is still initializing, but an update request has been queued.",
			QMessageBox::Ok,
			QMessageBox::Ok);
	} else if (isUpdating()) {
		QMessageBox::information(nullptr,
			"Acquisition",
			"An update is already in progress.",
			QMessageBox::Ok,
			QMessageBox::Ok);
	} else {
		emit UpdateSignal(type, locations);
	};
}

void ItemsManager::SetAutoUpdate(bool update) {
	settings_.setValue("autoupdate", update);
	if (update) {
		auto_update_timer_->start();
	} else {
		auto_update_timer_->stop();
	};
}

void ItemsManager::SetAutoUpdateInterval(int minutes) {
	settings_.setValue("autoupdate_interval", minutes);
	auto_update_timer_->setInterval(minutes * 60 * 1000);
}

void ItemsManager::OnAutoRefreshTimer() {
	Update(TabSelection::Checked);
}

void ItemsManager::MigrateBuyouts() {
	int db_version = datastore_.GetInt("db_version");
	// Don't migrate twice
	if (db_version == 4) {
		return;
	};
	for (auto& item : items_) {
		buyout_manager_.MigrateItem(*item);
	};
	buyout_manager_.Save();
	datastore_.SetInt("db_version", 4);
}
