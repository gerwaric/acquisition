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

#include <QNetworkAccessManager>
#include <QNetworkCookie>
#include <QNetworkCookieJar>
#include <QNetworkReply>
#include <QSignalMapper>
#include "QsLog.h"
#include <QTimer>
#include <QUrlQuery>
#include <algorithm>
#include "rapidjson/document.h"
#include "rapidjson/error/en.h"
#include <boost/algorithm/string.hpp>

#include "json_struct/json_struct.h"
#include "poe_api/poe_character.h"
#include "poe_api/poe_stash.h"

#include "application.h"
#include "datastore.h"
#include "util.h"
#include "mainwindow.h"
#include "buyoutmanager.h"
#include "modlist.h"
#include "ratelimit.h"
#include "oauth.h"

using RateLimit::RateLimiter;

const char* kPOE_trade_stats = "https://www.pathofexile.com/api/trade/data/stats";

const char* kRePoE_stat_translations = "https://raw.githubusercontent.com/brather1ng/RePoE/master/RePoE/data/stat_translations.min.json";
const char* kRePoE_item_classes = "https://raw.githubusercontent.com/brather1ng/RePoE/master/RePoE/data/item_classes.min.json";
const char* kRePoE_item_base_types = "https://raw.githubusercontent.com/brather1ng/RePoE/master/RePoE/data/base_items.min.json";

ItemsManagerWorker::ItemsManagerWorker(Application& app) :
	app_(app),
	requests_needed_(0),
	requests_completed_(0),
	initialized_(false),
	updating_(false),
	cancel_update_(false),
	updateRequest_(false),
	type_(TabSelection::Type::Checked)
{
	// This is how we make rate-limited requests.
	connect(this, &ItemsManagerWorker::GetRequest, &app_.rate_limiter(), &RateLimit::RateLimiter::Submit);

	// This is how we get the status of rate limiting.
	connect(&app_.rate_limiter(), &RateLimiter::StatusUpdate, this, &ItemsManagerWorker::OnRateLimitStatusUpdate);
}

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
	emit GetRequest("<NONE>", PoE_item_classes_request,
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
		QByteArray bytes = reply->readAll();
		emit ItemClassesUpdate(bytes);
	};
	QNetworkRequest PoE_item_base_types_request = QNetworkRequest(QUrl(QString(kRePoE_item_base_types)));
	emit GetRequest("<NONE>", PoE_item_base_types_request,
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
		QByteArray bytes = reply->readAll();
		emit ItemBaseTypesUpdate(bytes);
	};
	// Update the mods list.
	QNetworkRequest PoE_stat_translations_request = QNetworkRequest(QUrl(QString(kRePoE_stat_translations)));
	emit GetRequest("<NONE>", PoE_stat_translations_request,
		[=](QNetworkReply* reply) {
			OnStatTranslationsReceived(reply);
		});
}

void ItemsManagerWorker::OnStatTranslationsReceived(QNetworkReply* reply) {
	if (reply->error()) {
		QLOG_ERROR() << "Couldn't fetch RePoE Stat Translations: " << reply->url().toDisplayString()
			<< " due to error: " << reply->errorString() << " Aborting update.";
		return;
	};

	QByteArray bytes = reply->readAll();
	rapidjson::Document doc;
	doc.Parse(bytes.constData());

	if (doc.HasParseError()) {
		QLOG_ERROR() << "Couldn't properly parse Stat Translations from RePoE, canceling Mods Update";
		return;
	};

	mods.clear();
	std::set<std::string> stat_strings;

	for (auto& translation : doc) {
		for (auto& stat : translation["English"]) {
			std::vector<std::string> formats;
			for (auto& format : stat["format"]) {
				formats.push_back(format.GetString());
			};
			std::string stat_string = stat["string"].GetString();
			if (formats[0].compare("ignore") != 0) {
				for (int i = 0; i < formats.size(); i++) {
					std::string searchString = "{" + std::to_string(i) + "}";
					boost::replace_all(stat_string, searchString, formats[i]);
				};
			};
			if (stat_string.length() > 0) {
				stat_strings.insert(stat_string);
			};
		};
	};

	for (std::string stat_string : stat_strings) {
		mods.push_back({ stat_string });
	};

	// Create a separate thread to load the items, which allows the UI to
	// update the status bar while items are being parsed. This operation
	// can take tens of seconds or longer depending on the nubmer of tabs
	// and items.
	QThread* parser = QThread::create([=]() { ParseItemMods(); });
	parser->start();
}

void ItemsManagerWorker::ParseItemMods() {
	InitModlist();
	tabs_.clear();
	tab_id_index_.clear();
	items_.clear();

	// Get stash tabs and character locations from the data store
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

	// Get cached items
	QLOG_INFO() << "Loading cached items from" << tabs_.size() << "locations.";
	for (int i = 0; i < tabs_.size(); i++) {
		auto tab = tabs_[i];
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
	requests_needed_ = 0;
	requests_completed_ = 0;
	updating_ = true;
	cancel_update_ = false;

	if (type == TabSelection::All) {
		QLOG_DEBUG() << "Updating all tabs and items.";
		tabs_.clear();
		tab_id_index_.clear();
		items_.clear();
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

	// Now we can start the update, so let's get the characters.
	PoE::ListCharacters(this,
		[=](const PoE::ListCharactersResult& result) {
			OnCharacterListReceived(result);
		});
}

void ItemsManagerWorker::RemoveUpdatingTabs(const std::set<std::string>& tab_ids) {
	if (tab_ids.empty()) {
		QLOG_ERROR() << "No tabs to remove.";
		return;
	};

	// Keep tabs that are not being updated.
	std::vector<ItemLocation> current_tabs = tabs_;
	tabs_.clear();
	tab_id_index_.clear();
	for (auto& tab : current_tabs) {
		const std::string tab_id = tab.get_tab_uniq_id();
		bool save_tab = (tab_ids.count(tab.get_tab_uniq_id()) == 0);
		if (save_tab) {
			tabs_.push_back(tab);
			tab_id_index_.insert(tab.get_tab_uniq_id());
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

void ItemsManagerWorker::OnCharacterListReceived(const PoE::ListCharactersResult& result) {

	QLOG_DEBUG() << "Character list received with" << result.characters.size() << "characters (all leagues).";

	characters_to_request_.clear();

	int total_character_count = 0;
	int requested_character_count = 0;
	for (auto& character : result.characters) {
		const std::string& name = character.name;
		if (name.empty()) {
			QLOG_ERROR() << "Character name is empty!";
			break;
		};
		if (!character.league) {
			QLOG_ERROR() << "Skipping" << name << "because `league` is missing";
			continue;
		};
		if (character.league.value() != app_.league()) {
			continue;
		};
		++total_character_count;
		if (tab_id_index_.count(character.id) > 0) {
			QLOG_DEBUG() << "Skipping" << name << "because this character is not being refreshed.";
			continue;
		};

		// Use a legay character until we are ready to swap over completely.
		const PoE::LegacyCharacter legacy_character(character);

		const int tab_count = static_cast<int>(tabs_.size());
		ItemLocation location(character);
		location.set_json(JS::serializeStruct(legacy_character));
		tabs_.push_back(location);
		++requested_character_count;

		characters_to_request_.insert(character.name);
	};
	QLOG_DEBUG() << "There are" << requested_character_count << "characters to update in" << app_.league().c_str();

	CurrentStatusUpdate status;
	status.state = ProgramState::CharactersReceived;
	status.total = total_character_count;
	emit StatusUpdate(status);

	PoE::ListStashes(this,
		[=](const PoE::ListStashesResult& result) {
			OnStashListReceived(result);
		},
		app_.league());
}

void ItemsManagerWorker::OnStashListReceived(const PoE::ListStashesResult& result) {
	QLOG_TRACE() << "Stash list received with" << result.stashes.size() << "stash tabs.";

	stashes_to_request_.clear();

	// Queue stash tab requests.
	for (auto& tab : result.stashes) {

		// Skip tabs that are in the index; they are not being refreshed.
		if (tab_id_index_.count(tab.id) > 0) {
			continue;
		};

		// Create a legacy tab object for use until we swap over completely.
		const PoE::LegacyStashTab legacy_tab(tab);

		// Create and save the tab location object.
		ItemLocation location(tab);
		location.set_json(JS::serializeStruct(legacy_tab));
		tabs_.push_back(location);
		tab_id_index_.insert(tab.id);
		stashes_to_request_.insert(tab.id);
	};
	FetchItems();
}

void ItemsManagerWorker::FetchItems() {

	QLOG_DEBUG() << "Fetching items from"
		<< stashes_to_request_.size() << "stashes and"
		<< characters_to_request_.size() << "characters.";

	requests_needed_ = stashes_to_request_.size() + characters_to_request_.size();
	requests_completed_ = 0;

	if (requests_needed_ == 0) {
		QLOG_ERROR() << "Nothing to fetch.";
		updating_ = false;
		return;
	};

	for (auto& stashtab_id : stashes_to_request_) {
		PoE::GetStash(this,
			[=](const PoE::GetStashResult& result) {
				OnStashReceived(result);
			},
			app_.league(),
				stashtab_id);
	};
	for (auto& character_name : characters_to_request_) {
		PoE::GetCharacter(this,
			[=](const PoE::GetCharacterResult& result) {
				OnCharacterReceived(result);
			},
			character_name);
	};
}

void ItemsManagerWorker::OnStashReceived(const PoE::GetStashResult& result) {

	stashes_to_request_.erase(result.stash.id);

	ItemLocation* location = nullptr;
	for (auto& loc : tabs_) {
		if (loc.get_tab_uniq_id() == result.stash.id) {
			location = &loc;
		};
	};
	if (location == nullptr) {
		QLOG_ERROR() << "OnStashReceived(): unable to match a location to stash:" << result.stash.name;
		return;
	} else {
		if (result.stash.items) {
			AddItems(result.stash.items.value(), *location);
		};
	};
	FinishRequest();
}

void ItemsManagerWorker::OnCharacterReceived(const PoE::GetCharacterResult& result) {

	characters_to_request_.erase(result.character.name);

	ItemLocation* location = nullptr;
	for (auto& loc : tabs_) {
		if (loc.get_tab_uniq_id() == result.character.id) {
			location = &loc;
		};
	};
	if (location == nullptr) {
		QLOG_ERROR() << "OnStashReceived(): unable to find location.";
		return;
	} else {
		if (result.character.equipment) {
			AddItems(result.character.equipment.value(), *location);
		};
		if (result.character.inventory) {
			AddItems(result.character.inventory.value(), *location);
		};
		if (result.character.jewels) {
			AddItems(result.character.jewels.value(), *location);
		};
	};
	FinishRequest();
}

void ItemsManagerWorker::AddItems(const std::vector<PoE::Item>& items, const ItemLocation& location) {
	ItemLocation base_location = location;
	for (auto& item : items) {
		const std::string json = JS::serializeStruct(item);
		rapidjson::Document doc;
		doc.Parse(json.c_str());
		if (doc.HasParseError()) {
			QLOG_ERROR() << "AddItems: error parsing item:"
				<< rapidjson::GetParseError_En(doc.GetParseError()) << ":" << json;
			continue;
		};
		base_location.FromItemJson(doc);
		base_location.ToItemJson(&doc, doc.GetAllocator());
		auto legacy_item = std::make_shared<Item>(doc, base_location);
		items_.push_back(legacy_item);
		if (item.socketedItems) {
			base_location.set_socketed(true);
			AddItems(item.socketedItems.value(), base_location);
		};
	};
}

void ItemsManagerWorker::FinishRequest() {

	// Mark one more request as complete.
	++requests_completed_;

	// Update the program status.
	CurrentStatusUpdate status = CurrentStatusUpdate();
	status.state = ProgramState::ItemsReceive;
	status.progress = requests_completed_;
	status.total = requests_needed_;
	if (requests_completed_ == requests_needed_) {
		status.state = ProgramState::ItemsCompleted;
	};
	if (cancel_update_) {
		status.state = ProgramState::UpdateCancelled;
	};
	emit StatusUpdate(status);

	// Don't do anything else if we are still waiting for reqeusts.
	if (requests_needed_ == requests_completed_) {
		FinishUpdate();
	};
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
	std::map<ItemLocationType, DataStore::LocationList> tabsPerType;
	for (const auto& tab : tabs_) {
		tabsPerType[tab.get_type()].push_back(&tab);
	};

	// Map locations to a list of items in that location.
	std::map<ItemLocation, DataStore::ItemList> itemsPerLoc;
	for (const auto& item : items_) {
		itemsPerLoc[item->location()].push_back(item.get());
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


void ItemsManagerWorker::OnRateLimitStatusUpdate(const QString& status) {
	emit RateLimitStatusUpdate(status);
}