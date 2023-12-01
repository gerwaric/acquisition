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

#include <QNetworkCookie>
#include <QThread>
#include <stdexcept>

#include "QsLog.h"

#include "application.h"
#include "buyout.h"
#include "buyoutmanager.h"
#include "datastore.h"
#include "itemsmanagerworker.h"
#include "porting.h"
#include "shop.h"
#include "util.h"
#include "mainwindow.h"
#include "filters.h"
#include "itemcategories.h"
#include "ratelimit.h"
#include "repoe.h"

ItemsManager::ItemsManager(Application& app) :
	auto_update_timer_(std::make_unique<QTimer>()),
	app_(app)
{
	auto_update_interval_ = std::stoi(app_.data().Get("autoupdate_interval", "60"));
	auto_update_ = app_.data().GetBool("autoupdate", false);
	SetAutoUpdateInterval(auto_update_interval_);
	connect(auto_update_timer_.get(), &QTimer::timeout, this, &ItemsManager::OnAutoRefreshTimer);
}

ItemsManager::~ItemsManager() {
	/*
	if (thread_ != nullptr) {
		thread_->quit();
		thread_->wait();
	};
	*/
}

void ItemsManager::Start() {

	worker_ = std::make_unique<ItemsManagerWorker>(app_);
	connect(worker_.get(), &ItemsManagerWorker::StatusUpdate, this, &ItemsManager::OnStatusUpdate);
	connect(worker_.get(), &ItemsManagerWorker::RateLimitStatusUpdate, this, &ItemsManager::OnRateLimitStatusUpdate);
	connect(worker_.get(), &ItemsManagerWorker::ItemsRefreshed, this, &ItemsManager::OnItemsRefreshed);

	repoe_ = std::make_unique<RePoE::Updater>(this);
	connect(repoe_.get(), &RePoE::Updater::GetRequest, &app_.rate_limiter(), &RateLimit::RateLimiter::Submit);
	connect(repoe_.get(), &RePoE::Updater::ItemClassesUpdate, this, &ItemsManager::OnItemClassesUpdate);
	connect(repoe_.get(), &RePoE::Updater::BaseTypesUpdate, this, &ItemsManager::OnItemBaseTypesUpdate);
	connect(repoe_.get(), &RePoE::Updater::StatTranslationsUpdate, worker_.get(), &ItemsManagerWorker::OnStatTranslationsReceived);

	connect(this, &ItemsManager::UpdateSignal, worker_.get(), &ItemsManagerWorker::Update);

	emit repoe_->RequestItemClasses();
}

void ItemsManager::OnStatusUpdate(const CurrentStatusUpdate& status) {
	emit StatusUpdate(status);
}

void ItemsManager::OnRateLimitStatusUpdate(const QString& status) {
	emit RateLimitStatusUpdate(status);
}

void ItemsManager::OnItemClassesUpdate(const RePoE::ItemClasses& item_classes) {

	categories_.clear();
	itemClassKeyToValue.clear();
	itemClassValueToKey.clear();

	for (const auto& it : item_classes) {
		const std::string& key = it.first;
		const std::string& value = it.second.name;
		categories_.insert(QString::fromStdString(value));
		itemClassKeyToValue[key] = value;
		itemClassValueToKey[value] = key;
	};
	categories_.insert(QString::fromStdString(CategorySearchFilter::k_Default));

	emit repoe_->RequestBaseTypes();
}

void ItemsManager::OnItemBaseTypesUpdate(const RePoE::BaseTypes& base_types) {
	itemBaseType_NameToClass.clear();
	for (const auto& it : base_types) {
		const std::string& item_class = it.second.item_class;
		const std::string& name = it.second.name;
		itemBaseType_NameToClass[name] = item_class;
	};
	emit repoe_->RequestStatTranslations();
}

void ItemsManager::ApplyAutoTabBuyouts() {
	// Can handle everything related to auto-tab pricing here.
	// 1. First format we need to honor is ascendency pricing formats which is top priority and overrides other types
	// 2. Second priority is to honor manual user pricing
	// 3. Third priority it to apply pricing based on ideally user specified formats (doesn't exist yet)

	// Loop over all tabs, create buyout based on tab name which applies auto-pricing policies
	auto& bo = app_.buyout_manager();
	for (auto const& loc : app_.buyout_manager().GetStashTabLocations()) {
		auto tab_label = loc.name();
		Buyout buyout = bo.StringToBuyout(tab_label);
		if (buyout.IsActive()) {
			bo.SetTab(loc.id(), buyout);
		};
	};

	// Need to compress tab buyouts here, as the tab names change we accumulate and save BO's
	// for tabs that no longer exist I think.
	bo.CompressTabBuyouts();
}

void ItemsManager::ApplyAutoItemBuyouts() {
	// Loop over all items, check for note field with pricing and apply
	auto& bo = app_.buyout_manager();
	for (auto const& item : items_) {
		auto const& note = item->note();
		if (!note.empty()) {
			Buyout buyout = bo.StringToBuyout(note);
			// This line may look confusing, buyout returns an active buyout if game
			// pricing was found or a default buyout (inherit) if it was not.
			// If there is a currently valid note we want to apply OR if
			// old note no longer is valid (so basically clear pricing)
			if (buyout.IsActive() || bo.Get(*item).IsGameSet()) {
				bo.Set(*item, buyout);
			};
		};
	};

	// Commenting this out for robustness (iss381) to make it as unlikely as possible that users
	// pricing data will be removed.  Side effect is that stale pricing data will pile up and
	// could be applied to future items with the same hash (which includes tab name).
	// bo.CompressItemBuyouts(items_);
}

void ItemsManager::PropagateTabBuyouts() {
	auto& bo = app_.buyout_manager();
	bo.ClearRefreshLocks();
	for (auto& item_ptr : items_) {
		Item& item = *item_ptr;
		std::string hash = item.location().id();
		auto item_bo = bo.Get(item);
		auto tab_bo = bo.GetTab(hash);

		if (item_bo.IsInherited()) {
			if (tab_bo.IsActive()) {
				// Any propagation from tab price to item price should include this bit set
				tab_bo.inherited = true;
				tab_bo.last_update = QDateTime::currentDateTime();
				bo.Set(item, tab_bo);
			} else {
				// This effectively 'clears' buyout by setting back to 'inherit' state.
				bo.Set(item, Buyout());
			};
		};

		// If any savable bo's are set on an item or the tab then lock the refresh state.
		// Skip remove-only tabs because they are not editable, nor indexed for trade now.
		if (item.location().removeonly() == false) {
			if (bo.Get(item).RequiresRefresh() || tab_bo.RequiresRefresh()) {
				bo.SetRefreshLocked(item.location());
			};
		};
	};
}

void ItemsManager::OnItemsRefreshed(const Items& items, const std::vector<ItemLocation>& tabs, bool initial_refresh) {
	items_ = items;

	QLOG_DEBUG() << "Number of items refreshed: " << items_.size() << "; Number of tabs refreshed: " << tabs.size() << "; Initial Refresh: " << initial_refresh;

	app_.buyout_manager().SetStashTabLocations(tabs);
	ApplyAutoTabBuyouts();
	ApplyAutoItemBuyouts();
	PropagateTabBuyouts();

	emit ItemsRefreshed(initial_refresh);
}

void ItemsManager::Update(TabSelection::Type type, const std::vector<ItemLocation>& locations) {
	emit UpdateSignal(type, locations);
}

void ItemsManager::SetAutoUpdate(bool update) {
	app_.data().SetBool("autoupdate", update);
	auto_update_ = update;
	if (!auto_update_) {
		auto_update_timer_->stop();
	} else {
		// to start timer
		SetAutoUpdateInterval(auto_update_interval_);
	};
}

void ItemsManager::SetAutoUpdateInterval(int minutes) {
	app_.data().Set("autoupdate_interval", std::to_string(minutes));
	auto_update_interval_ = minutes;
	if (auto_update_) {
		auto_update_timer_->start(auto_update_interval_ * 60 * 1000);
	};
}

void ItemsManager::OnAutoRefreshTimer() {
	Update(TabSelection::Checked);
}
