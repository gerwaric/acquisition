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

#include "itemsmanagerworker.h"

#include <QNetworkReply>
#include "QsLog.h"
#include <algorithm>
#include "rapidjson/error/en.h"

#include "application.h"
#include "datastore.h"
#include "mainwindow.h"
#include "modlist.h"
#include "ratelimit.h"

using RateLimit::RateLimiter;

const char* kPOE_trade_stats = "https://www.pathofexile.com/api/trade/data/stats";
const char* kRePoE_item_classes = "https://raw.githubusercontent.com/lvlvllvlvllvlvl/RePoE/master/RePoE/data/item_classes.min.json";
const char* kRePoE_item_base_types = "https://raw.githubusercontent.com/lvlvllvlvllvlvl/RePoE/master/RePoE/data/base_items.min.json";

// Modifiers from this list of files will be loaded in order from first to last.
const QStringList REPOE_STAT_TRANSLATIONS = {
	"https://raw.githubusercontent.com/lvlvllvlvllvlvl/RePoE/master/RePoE/data/stat_translations.min.json",
	"https://raw.githubusercontent.com/lvlvllvlvllvlvl/RePoE/master/RePoE/data/stat_translations/necropolis.min.json"
};

ItemsManagerWorker::ItemsManagerWorker(Application& app) :
	app_(app),
	rate_limiter_(app.rate_limiter()),
	initialized_(false),
	updating_(false),
	cancel_update_(false),
	updateRequest_(false),
	type_(TabSelection::Type::Checked),
	first_fetch_tab_id_(-1)
{}

ItemsManagerWorker::~ItemsManagerWorker()
{}

void ItemsManagerWorker::UpdateRequest(TabSelection::Type type, const std::vector<ItemLocation>& locations) {
	updateRequest_ = true;
	type_ = type;
	locations_ = locations;
}

void ItemsManagerWorker::Init() {

	if (updating_) {
		QLOG_WARN() << "ItemsManagerWorker::Init() called while updating, skipping Mod List Update";
		return;
	};

	updating_ = true;

	QNetworkRequest PoE_item_classes_request = QNetworkRequest(QUrl(QString(kRePoE_item_classes)));
	rate_limiter_.Submit(PoE_item_classes_request,
		[=](QNetworkReply* reply) {
			OnItemClassesReceived(reply);
		});
}

void ItemsManagerWorker::OnItemClassesReceived(QNetworkReply* reply) {
	if (reply->error()) {
		QLOG_ERROR() << "Couldn't fetch RePoE Item Classes:" << reply->url().toDisplayString()
			<< "due to error:" << reply->errorString() << "The type dropdown will remain empty.";
	} else {
		QLOG_DEBUG() << "Item classes received.";
		const QByteArray bytes = reply->readAll();
		emit ItemClassesUpdate(bytes);
	};
	QNetworkRequest PoE_item_base_types_request = QNetworkRequest(QUrl(QString(kRePoE_item_base_types)));
	rate_limiter_.Submit(PoE_item_base_types_request,
		[=](QNetworkReply* reply) {
			OnItemBaseTypesReceived(reply);
		});
}

void ItemsManagerWorker::OnItemBaseTypesReceived(QNetworkReply* reply) {
	if (reply->error()) {
		QLOG_ERROR() << "Couldn't fetch RePoE Item Base Types:" << reply->url().toDisplayString()
			<< "due to error:" << reply->errorString() << "The type dropdown will remain empty.";
	} else {
		QLOG_DEBUG() << "Item base types received.";
		const QByteArray bytes = reply->readAll();
		emit ItemBaseTypesUpdate(bytes);
	};
	InitStatTranslations();
	QStringList StatTranslationUrls = QStringList(REPOE_STAT_TRANSLATIONS);
	UpdateModList(StatTranslationUrls);
}

void ItemsManagerWorker::UpdateModList(QStringList StatTranslationUrls) {
	if (StatTranslationUrls.isEmpty() == false) {
		QUrl url = QUrl(StatTranslationUrls.takeFirst());
		QNetworkRequest PoE_stat_translations_request = QNetworkRequest(url);
		rate_limiter_.Submit(PoE_stat_translations_request,
			[=](QNetworkReply* reply) {
				OnStatTranslationsReceived(reply);
				UpdateModList(StatTranslationUrls);
			});
	} else {
		// Create a separate thread to load the items, which allows the UI to
		// update the status bar while items are being parsed. This operation
		// can take tens of seconds or longer depending on the nubmer of tabs
		// and items.
		QThread* parser = QThread::create(
			[=]() {
				InitModList();
				ParseItemMods();
			});
		parser->start();
	};
}

void ItemsManagerWorker::OnStatTranslationsReceived(QNetworkReply* reply) {
	QLOG_TRACE() << "Stat translations received:" << reply->request().url().toString();
	if (reply->error()) {
		QLOG_ERROR() << "Couldn't fetch RePoE Stat Translations: " << reply->url().toDisplayString()
			<< " due to error: " << reply->errorString() << " Aborting update.";
		return;
	};
	const QByteArray bytes = reply->readAll();
	emit StatTranslationsUpdate(bytes);
}

void ItemsManagerWorker::ParseItemMods() {
	tabs_.clear();
	tabs_signature_.clear();
	tab_id_index_.clear();

	//Get cached tabs (item tabs not search tabs)
	for (ItemLocationType type : {ItemLocationType::STASH, ItemLocationType::CHARACTER}) {
		Locations tabs = app_.data().GetTabs(type);
		tabs_.reserve(tabs_.size() + tabs.size());
		for (const auto& tab : tabs) {
			tabs_.push_back(tab);
		};
	};

	// Save location ids.
	for (const auto& tab : tabs_) {
		tab_id_index_.insert(tab.get_tab_uniq_id());
	};

	// Build the signature vector.
	tabs_signature_.reserve(tabs_.size());
	for (const auto& tab : tabs_) {
		const std::string tab_name = tab.get_tab_label();
		const std::string tab_id = QString::number(tab.get_tab_id()).toStdString();
		tabs_signature_.push_back({ tab_name, tab_id });
	};

	// Get cached items
	for (int i = 0; i < tabs_.size(); i++) {
		auto& tab = tabs_[i];
		Items tab_items = app_.data().GetItems(tab);
		items_.reserve(items_.size() + tab_items.size());
		for (const auto& tab_item : tab_items) {
			items_.push_back(tab_item);
		};
		CurrentStatusUpdate status;
		status.state = ProgramState::ItemsRetrieved;
		status.progress = static_cast<size_t>(i) + 1;
		status.total = tabs_.size();
		emit StatusUpdate(status);
	};

	CurrentStatusUpdate status;
	status.state = ProgramState::ItemsCompleted;
	status.progress = tabs_.size();
	status.total = tabs_.size();
	emit StatusUpdate(status);

	initialized_ = true;
	updating_ = false;

	// let ItemManager know that the retrieval of cached items/tabs has been completed (calls ItemsManager::OnItemsRefreshed method)
	emit ItemsRefreshed(items_, tabs_, true);

	if (updateRequest_) {
		updateRequest_ = false;
		Update(type_, locations_);
	};
}

void ItemsManagerWorker::Update(TabSelection::Type type, const std::vector<ItemLocation>& locations) {
	if (updating_) {
		QLOG_WARN() << "ItemsManagerWorker::Update called while updating";
		return;
	};
	QLOG_DEBUG() << "Updating" << type << "stash tabs";
	updating_ = true;
	cancel_update_ = false;

	first_fetch_tab_ = "";
	first_fetch_tab_id_ = -1;

	if (type == TabSelection::All) {
		QLOG_DEBUG() << "Updating all tabs and items.";
		tabs_.clear();
		tab_id_index_.clear();
		items_.clear();
		first_fetch_tab_id_ = 0;
	} else {
		// Build a list of tabs to update.
		std::set<std::string> tabs_to_update = {};
		switch (type) {
		case TabSelection::Checked:
			// Use the buyout manager to determine which tabs are check.
			for (auto const& tab : tabs_) {
				if ((tab.IsValid()) && (app_.buyout_manager().GetRefreshChecked(tab) == true)) {
					tabs_to_update.insert(tab.get_tab_uniq_id());
				};
			};
			break;
		case TabSelection::Selected:
			// Use the function argument to determine which tabs were selected.
			for (auto const& tab : locations) {
				if (tab.IsValid()) {
					tabs_to_update.insert(tab.get_tab_uniq_id());
				};
			};
			break;
		};
		// Remove the tabs that will be updated, and all the items linked to those tabs.
		QLOG_DEBUG() << "Updating" << tabs_to_update.size() << " tabs.";
		RemoveUpdatingTabs(tabs_to_update);
		RemoveUpdatingItems(tabs_to_update);
	};

	if (first_fetch_tab_id_ < 0) {
		QLOG_WARN() << "Requesting tab index 0 because there are no known tabs to update.";
		first_fetch_tab_ = "<UNKNOWN>";
		first_fetch_tab_id_ = 0;
	};

	DoUpdate();
}


void ItemsManagerWorker::RemoveUpdatingTabs(const std::set<std::string>& tab_ids) {
	if (tab_ids.empty()) {
		QLOG_ERROR() << "No tabs to remove.";
		return;
	};

	// Keep tabs that are not being updated.
	std::vector<ItemLocation> current_tabs = tabs_;
	bool need_first = true;
	tabs_.clear();
	tab_id_index_.clear();
	for (auto& tab : current_tabs) {
		const std::string tab_id = tab.get_tab_uniq_id();
		bool save_tab = (tab_ids.count(tab.get_tab_uniq_id()) == 0);
		if (save_tab) {
			tabs_.push_back(tab);
			tab_id_index_.insert(tab.get_tab_uniq_id());
		} else if (need_first) {
			first_fetch_tab_ = tab.get_tab_uniq_id();
			first_fetch_tab_id_ = tab.get_tab_id();
			need_first = false;
		};
	};
	QLOG_DEBUG() << "Keeping" << tabs_.size() << "tabs and culling" << (current_tabs.size() - tabs_.size());
}

void ItemsManagerWorker::RemoveUpdatingItems(const std::set<std::string>& tab_ids) {
	// Keep items with locations that are not being updated.
	if (tab_ids.empty()) {
		QLOG_ERROR() << "No tabs to remove items from.";
		return;
	};
	Items current_items = items_;
	items_.clear();
	for (auto const& item : current_items) {
		const ItemLocation& tab = item.get()->location();
		bool save_item = (tab_ids.count(tab.get_tab_uniq_id()) == 0);
		if (save_item) {
			items_.push_back(item);
		};
	};
	QLOG_DEBUG() << "Keeping" << items_.size() << "items and culling" << (current_items.size() - items_.size());
}

void ItemsManagerWorker::ParseItems(rapidjson::Value* value_ptr, ItemLocation base_location, rapidjson_allocator& alloc) {
	auto& value = *value_ptr;

	for (auto& item : value) {
		// Make sure location data from the item like x and y is brought over to the location object.
		base_location.FromItemJson(item);
		base_location.ToItemJson(&item, alloc);
		items_.push_back(std::make_shared<Item>(item, base_location));
		if (item.HasMember("socketedItems") && item["socketedItems"].IsArray()) {
			base_location.set_socketed(true);
			ParseItems(&item["socketedItems"], base_location, alloc);
		};
	}
}

void ItemsManagerWorker::FinishUpdate() {
	// It's possible that we receive character vs stash tabs out of order, or users
	// move items around in a tab and we get them in a different order. For
	// consistency we want to present the tab data in a deterministic way to the rest
	// of the application.  Especially so we don't try to update shop when nothing actually
	// changed.  So sort items_ here before emitting and then generate
	// item list as strings.

	std::sort(begin(items_), end(items_),
		[](const std::shared_ptr<Item>& a, const std::shared_ptr<Item>& b) {
			return *a < *b;
		});

	// Maps location type (CHARACTER or STASH) to a list of all the tabs of that type
	std::map<ItemLocationType, Locations> tabsPerType;
	for (auto& tab : tabs_) {
		tabsPerType[tab.get_type()].push_back(tab);
	};

	// Map locations to a list of items in that location.
	std::map<ItemLocation, Items> itemsPerLoc;
	for (auto& item : items_) {
		itemsPerLoc[item->location()].push_back(item);
	};

	// Save tabs by tab type.
	for (auto const& pair : tabsPerType) {
		const auto& location_type = pair.first;
		const auto& tabs = pair.second;
		app_.data().SetTabs(location_type, tabs);
	};

	// Save items by location.
	for (auto const& pair : itemsPerLoc) {
		const auto& location = pair.first;
		const auto& items = pair.second;
		app_.data().SetItems(location, items);
	};

	// Let everyone know the update is done.
	emit ItemsRefreshed(items_, tabs_, false);

	updating_ = false;
	QLOG_DEBUG() << "Update finished.";
}

std::vector<std::pair<std::string, std::string> > ItemsManagerWorker::CreateTabsSignatureVector(std::string tabs) {
	std::vector<std::pair<std::string, std::string> > tmp;
	rapidjson::Document doc;

	if (doc.Parse(tabs.c_str()).HasParseError()) {
		QLOG_ERROR() << "Error creating signature vector from tabs data ("
			<< rapidjson::GetParseError_En(doc.GetParseError()) << "):" << tabs.c_str();
	} else {
		for (auto& tab : doc) {
			std::string name = (tab.HasMember("n") && tab["n"].IsString()) ? tab["n"].GetString() : "UNKNOWN_NAME";
			std::string uid = (tab.HasMember("id") && tab["id"].IsString()) ? tab["id"].GetString() : "UNKNOWN_ID";
			tmp.emplace_back(name, uid);
		};
	};
	return tmp;
}
