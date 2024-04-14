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

#include "application.h"
#include "datastore.h"
#include "util.h"
#include "mainwindow.h"
#include "buyoutmanager.h"
#include "modlist.h"
#include "ratelimit.h"
#include "oauth.h"

using RateLimit::RateLimiter;

const char* kStashItemsUrl = "https://www.pathofexile.com/character-window/get-stash-items";
const char* kCharacterItemsUrl = "https://www.pathofexile.com/character-window/get-items";
const char* kGetCharactersUrl = "https://www.pathofexile.com/character-window/get-characters";
const char* kMainPage = "https://www.pathofexile.com/";
//While the page does say "get passive skills", it seems to only send socketed jewels
const char* kCharacterSocketedJewels = "https://www.pathofexile.com/character-window/get-passive-skills";

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
	test_mode_(false),
	rate_limiter_(app.rate_limiter()),
	total_completed_(-1),
	total_needed_(-1),
	requests_completed_(-1),
	requests_needed_(-1),
	initialized_(false),
	updating_(false),
	cancel_update_(false),
	updateRequest_(false),
	type_(TabSelection::Type::Checked),
	queue_id_(-1),
	first_fetch_tab_id_(-1)
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

	//rate_limiter_ = std::make_unique<RateLimiter>(app_);
	connect(&rate_limiter_, &RateLimiter::StatusUpdate, this, &ItemsManagerWorker::OnRateLimitStatusUpdate);

	updating_ = true;

	QNetworkRequest PoE_item_classes_request = QNetworkRequest(QUrl(QString(kRePoE_item_classes)));
	rate_limiter_.Submit(PoE_item_classes_request,
		[=](QNetworkReply* reply) {
			OnItemClassesReceived(reply);
		});
}

void ItemsManagerWorker::OnRateLimitStatusUpdate(const RateLimit::StatusInfo& update) {
	emit RateLimitStatusUpdate(update);
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
		QByteArray bytes = reply->readAll();
		emit ItemBaseTypesUpdate(bytes);
	};
	mods.clear();
	QStringList StatTranslationUrls = QStringList(REPOE_STAT_TRANSLATIONS);
	UpdateModList(StatTranslationUrls);
}

void ItemsManagerWorker::ParseItemMods() {
	InitModlist();
	tabs_.clear();
	tabs_signature_.clear();
	tab_id_index_.clear();

	//Get cached tabs (item tabs not search tabs)
	for (ItemLocationType type : {ItemLocationType::STASH, ItemLocationType::CHARACTER}) {
		std::string tabs = app_.data().GetTabs(type);
		if (tabs.empty()) {
			QLOG_DEBUG() << "Cache contains no" << type << "locations";
			continue;
		};
		rapidjson::Document doc;
		if (doc.Parse(tabs.c_str()).HasParseError()) {
			QLOG_ERROR() << "Malformed tabs data (" << rapidjson::GetParseError_En(doc.GetParseError()) << "):" << tabs.c_str();
			continue;
		};
		tabs_signature_ = CreateTabsSignatureVector(tabs);
		for (auto& tab : doc) {
			//constructor values to fill in
			size_t index;
			std::string tabUniqueId, name;
			int r, g, b;

			switch (type) {
			case ItemLocationType::STASH:
				if (tab_id_index_.count(tab["id"].GetString())) {
					QLOG_ERROR() << "Duplicated tab found while loading data:" << tab["id"].GetString();
					continue;
				};
				if (!tab.HasMember("n") || !tab["n"].IsString()) {
					QLOG_ERROR() << "Malformed tabs data doesn't contain its name (field 'n'):" << tabs.c_str();
					continue;
				};
				index = tab["i"].GetInt();
				tabUniqueId = tab["id"].GetString();
				name = tab["n"].GetString();
				r = tab["colour"]["r"].GetInt();
				g = tab["colour"]["g"].GetInt();
				b = tab["colour"]["b"].GetInt();
				break;
			case ItemLocationType::CHARACTER:
				if (tab_id_index_.count(tab["name"].GetString())) {
					continue;
				};
				if (tab.HasMember("i")) {
					index = tab["i"].GetInt();
				} else {
					index = tabs_.size();
				};
				tabUniqueId = tab["name"].GetString();
				name = tab["name"].GetString();
				r = 0;
				g = 0;
				b = 0;
				break;
			};
			ItemLocation loc(static_cast<int>(index), tabUniqueId, name, type, r, g, b);
			loc.set_json(tab, doc.GetAllocator());
			tabs_.push_back(loc);
			tab_id_index_.insert(loc.get_tab_uniq_id());
		};
	};

	items_.clear();

	// Get cached items
	for (int i = 0; i < tabs_.size(); i++) {
		auto tab = tabs_[i];
		std::string items = app_.data().GetItems(tab);
		if (items.size() != 0) {
			rapidjson::Document doc;
			doc.Parse(items.c_str());
			for (auto item = doc.Begin(); item != doc.End(); ++item) {
				// Create a new location and make sure location-related information
				// such as x and y are pulled from the item json.
				ItemLocation loc = tab;
				loc.FromItemJson(*item);
				items_.push_back(std::make_shared<Item>(*item, loc));
			};
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

void ItemsManagerWorker::UpdateModList(QStringList StatTranslationUrls) {
	if (StatTranslationUrls.isEmpty()) {
		// Create a separate thread to load the items, which allows the UI to
		// update the status bar while items are being parsed. This operation
		// can take tens of seconds or longer depending on the nubmer of tabs
		// and items.
		QThread* parser = QThread::create([=]() { ParseItemMods(); });
		parser->start();
	} else {
		QUrl url = QUrl(StatTranslationUrls.takeFirst());
		QNetworkRequest PoE_stat_translations_request = QNetworkRequest(url);
		rate_limiter_.Submit(PoE_stat_translations_request,
			[=](QNetworkReply* reply) {
				OnStatTranslationsReceived(reply);
				UpdateModList(StatTranslationUrls);
			});
	};
}

void ItemsManagerWorker::OnStatTranslationsReceived(QNetworkReply* reply) {
	QLOG_TRACE() << "Stat translations received:" << reply->request().url().toString();

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

	//mods.clear();
	std::set<std::string> stat_strings;

	for (auto& translation : doc) {
		for (auto& stat : translation["English"]) {
			if (stat.HasMember("is_markup") && (stat["is_markup"].GetBool() == true)) {
				// This was added with the change to process json files inside
				// the stat_translations directory. In this case, the necropolis
				// mods from 3.24 have some kind of duplicate formatting with
				// markup that acquisition has not had to deal with before.
				//
				// It's possible this is true for other files in the stat_translations
				// folder, but acquisition has never needed to load modifiers from those
				// files before.
				continue;
			};
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

	for (const std::string& stat_string : stat_strings) {
		mods.push_back({ stat_string });
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

	// remove all pending requests
	queue_ = std::queue<ItemsRequest>();
	queue_id_ = 0;

	tabs_as_string_ = "";
	selected_character_ = "";

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

	// first, download the main page because it's the only way to know which character is selected
	QNetworkRequest main_page_request = QNetworkRequest(QUrl(kMainPage));
	rate_limiter_.Submit(main_page_request,
		[=](QNetworkReply* reply) {
			OnMainPageReceived(reply);
		});
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
void ItemsManagerWorker::OnMainPageReceived(QNetworkReply* reply) {
	QLOG_TRACE() << "Main page received.";

	if (reply->error()) {
		QLOG_WARN() << "Couldn't fetch main page: " << reply->url().toDisplayString() << " due to error: " << reply->errorString();
	} else {
		std::string page(reply->readAll().constData());
		selected_character_ = Util::FindTextBetween(page, "C({\"name\":\"", "\",\"class");
		if (selected_character_.empty()) {
			// If the user is using POESESSID, then we should expect to find the character name.
			// If the uses is using OAuth, then we might not find the character name if they user
			// is not logged into pathofexile.com using the browser they authenticated with.
			if (app_.oauth_manager().access_token().isEmpty() == true) {
				QLOG_WARN() << "Couldn't extract currently selected character name from GGG homepage (maintenence?) Text was: " << page.c_str();
			};
		};
	};

	// now get character list
	QNetworkRequest characters_request = QNetworkRequest(QUrl(kGetCharactersUrl));
	rate_limiter_.Submit(characters_request,
		[=](QNetworkReply* reply) {
			OnCharacterListReceived(reply);
		});
}

void ItemsManagerWorker::OnCharacterListReceived(QNetworkReply* reply) {
	QLOG_TRACE() << "Character list received.";
	QByteArray bytes = reply->readAll();
	rapidjson::Document doc;
	doc.Parse(bytes.constData());

	if (reply->error()) {
		QLOG_WARN() << "Couldn't fetch character list: " << reply->url().toDisplayString()
			<< " due to error: " << reply->errorString() << " Aborting update.";
		updating_ = false;
		return;
	};

	if (doc.HasParseError() || !doc.IsArray()) {
		QLOG_ERROR() << "Received invalid reply instead of character list:" << bytes.constData();
		if (doc.HasParseError()) {
			QLOG_ERROR() << "The error was" << rapidjson::GetParseError_En(doc.GetParseError());
		};
		QLOG_ERROR() << "";
		QLOG_ERROR() << "(Maybe you need to log in to the website manually and accept new Terms of Service?)";
		updating_ = false;
		return;
	};

	QLOG_DEBUG() << "Received character list, there are" << doc.Size() << "characters across all leagues.";

	int total_character_count = 0;
	int requested_character_count = 0;
	for (auto& character : doc) {
		const std::string name = character["name"].GetString();
		if (!character.HasMember("league") || !character.HasMember("name") || !character["league"].IsString() || !character["name"].IsString()) {
			QLOG_ERROR() << "Malformed character entry for" << name.c_str() << ": the reply may be invalid : " << bytes.constData();
			continue;
		};
		if (character["league"].GetString() != app_.league()) {
			QLOG_DEBUG() << "Skipping" << name.c_str() << "because this character is not in" << app_.league().c_str();
			continue;
		};
		++total_character_count;
		if (tab_id_index_.count(name) > 0) {
			QLOG_DEBUG() << "Skipping" << name.c_str() << "because this item is not being refreshed.";
			continue;
		};
		const int tab_count = static_cast<int>(tabs_.size());
		ItemLocation location;
		location.set_type(ItemLocationType::CHARACTER);
		location.set_character(name);
		location.set_json(character, doc.GetAllocator());
		location.set_tab_id(tab_count);
		tabs_.push_back(location);
		++requested_character_count;

		//Queue request for items on character in character's stash
		QueueRequest(MakeCharacterRequest(name), location);

		//Queue request for jewels in character's passive tree
		QueueRequest(MakeCharacterPassivesRequest(name), location);
	}
	QLOG_DEBUG() << "There are" << requested_character_count << "characters to update in" << app_.league().c_str();

	CurrentStatusUpdate status;
	status.state = ProgramState::CharactersReceived;
	status.total = total_character_count;
	emit StatusUpdate(status);

	QNetworkRequest tab_request = MakeTabRequest(first_fetch_tab_id_, true);
	rate_limiter_.Submit(tab_request,
		[=](QNetworkReply* reply) {
			OnFirstTabReceived(reply);
		});
}

QNetworkRequest ItemsManagerWorker::MakeTabRequest(int tab_index, bool tabs) {
	QUrlQuery query;
	query.addQueryItem("league", app_.league().c_str());
	query.addQueryItem("tabs", tabs ? "1" : "0");
	query.addQueryItem("tabIndex", std::to_string(tab_index).c_str());
	query.addQueryItem("accountName", app_.email().c_str());

	QUrl url(kStashItemsUrl);
	url.setQuery(query);
	return QNetworkRequest(url);
}

QNetworkRequest ItemsManagerWorker::MakeCharacterRequest(const std::string& name) {
	QUrlQuery query;
	query.addQueryItem("character", name.c_str());
	query.addQueryItem("accountName", app_.email().c_str());

	QUrl url(kCharacterItemsUrl);
	url.setQuery(query);
	return QNetworkRequest(url);
}

QNetworkRequest ItemsManagerWorker::MakeCharacterPassivesRequest(const std::string& name) {
	QUrlQuery query;
	query.addQueryItem("character", name.c_str());
	query.addQueryItem("accountName", app_.email().c_str());

	QUrl url(kCharacterSocketedJewels);
	url.setQuery(query);
	return QNetworkRequest(url);
}

void ItemsManagerWorker::QueueRequest(const QNetworkRequest& request, const ItemLocation& location) {
	QLOG_DEBUG() << "Queued (" << queue_id_ + 1 << ") -- " << location.GetHeader().c_str();
	ItemsRequest items_request;
	items_request.network_request = request;
	items_request.id = queue_id_++;
	items_request.location = location;
	queue_.push(items_request);
}

void ItemsManagerWorker::FetchItems() {
	std::string tab_titles;
	const size_t count = queue_.size();
	for (int i = 0; i < count; ++i) {
		// Take the next request out of the queue.
		ItemsRequest request = queue_.front();
		queue_.pop();

		// Pass the request to the rate limiter.
		QNetworkRequest fetch_request = request.network_request;
		ItemLocation location = request.location;
		rate_limiter_.Submit(fetch_request,
			[=](QNetworkReply* reply) {
				OnTabReceived(reply, location);
			});

		// Keep track of the tabs requested.
		tab_titles += request.location.GetHeader() + " ";
	};
	QLOG_DEBUG() << "Created" << count << "requests:" << tab_titles.c_str();
	requests_needed_ = count;
	requests_completed_ = 0;
}

void ItemsManagerWorker::OnFirstTabReceived(QNetworkReply* reply) {
	QLOG_TRACE() << "First tab received.";

	QByteArray bytes = reply->readAll();
	rapidjson::Document doc;
	doc.Parse(bytes.constData());

	if (!doc.IsObject()) {
		QLOG_ERROR() << "Can't even fetch first tab. Failed to update items.";
		updating_ = false;
		return;
	};

	if (doc.HasMember("error")) {
		QLOG_ERROR() << "Aborting update since first fetch failed due to 'error':" << Util::RapidjsonSerialize(doc["error"]).c_str();
		updating_ = false;
		return;
	};

	if (!doc.HasMember("tabs") || doc["tabs"].Size() == 0) {
		QLOG_ERROR() << "There are no tabs, this should not happen, bailing out.";
		updating_ = false;
		return;
	};

	QLOG_DEBUG() << "Received tabs list, there are" << doc["tabs"].Size() << "tabs";
	tabs_as_string_ = Util::RapidjsonSerialize(doc["tabs"]);
	tabs_signature_ = CreateTabsSignatureVector(tabs_as_string_);

	// Remember old tab headers before clearing tabs
	std::set<std::string> old_tab_headers;
	for (auto const& tab : tabs_) {
		old_tab_headers.insert(tab.GetHeader());
	};

	// Force refreshes for any stash tabs that were moved or renamed.
	for (auto const& tab : tabs_) {
		if (!old_tab_headers.count(tab.GetHeader())) {
			QLOG_DEBUG() << "Forcing refresh of moved or renamed tab: " << tab.GetHeader().c_str();
			QueueRequest(MakeTabRequest(tab.get_tab_id(), true), tab);
		};
	};

	// Queue stash tab requests.
	for (auto& tab : doc["tabs"]) {

		std::string label = tab["n"].GetString();
		const int index = tab["i"].GetInt();

		// Skip hidden tabs.
		if (doc["tabs"][index].HasMember("hidden") && doc["tabs"][index]["hidden"].GetBool()) {
			continue;
		};

		// Skip tabs that are in the index; they are not being refreshed.
		const char* tab_id = tab["id"].GetString();
		if (tab_id_index_.count(tab_id) > 0) {
			continue;
		};

		// Create and save the tab location object.
		const int r = tab["colour"]["r"].GetInt();
		const int g = tab["colour"]["g"].GetInt();
		const int b = tab["colour"]["b"].GetInt();
		ItemLocation location(index, tab_id, label, ItemLocationType::STASH, r, g, b);
		location.set_json(tab, doc.GetAllocator());
		tabs_.push_back(location);
		tab_id_index_.insert(tab_id);

		// Submit a request for this tab.
		QueueRequest(MakeTabRequest(location.get_tab_id(), true), location);
	};

	total_needed_ = queue_.size();
	total_completed_ = 0;
	FetchItems();
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

void ItemsManagerWorker::OnTabReceived(QNetworkReply* network_reply, ItemLocation location) {
	QLOG_DEBUG() << "Received a reply for" << location.GetHeader().c_str();

	QByteArray bytes = network_reply->readAll();
	rapidjson::Document doc;
	doc.Parse(bytes.constData());

	bool error = false;
	if (!doc.IsObject()) {
		QLOG_WARN() << "Got a non-object response";
		error = true;
	} else if (doc.HasMember("error")) {
		// this can happen if user is browsing stash in background and we can't know about it
		QLOG_WARN() << "Got 'error' instead of stash tab contents: " << Util::RapidjsonSerialize(doc["error"]).c_str();
		error = true;
	};

	// We index expected tabs and their locations as part of the first fetch.  It's possible for users
	// to move or rename tabs during the update which will result in the item data being out-of-sync with
	// expected index/tab name map.  We need to detect this case and abort the update.
	if (!cancel_update_ && !error && (location.get_type() == ItemLocationType::STASH)) {
		cancel_update_ = TabsChanged(doc, network_reply, location);
	};

	// re-queue a failed request
	if (error) {
		QueueRequest(network_reply->request(), location);
	};

	++requests_completed_;

	if (!error) {
		++total_completed_;
	};

	if (requests_completed_ == requests_needed_) {
		if (cancel_update_) {
			updating_ = false;
		};
	};

	CurrentStatusUpdate status = CurrentStatusUpdate();
	status.state = ProgramState::ItemsReceive;
	status.progress = total_completed_;
	status.total = total_needed_;
	if (total_completed_ == total_needed_) {
		status.state = ProgramState::ItemsCompleted;
	};
	if (cancel_update_) {
		status.state = ProgramState::UpdateCancelled;
	};
	emit StatusUpdate(status);

	if (error) {
		return;
	};

	ParseItems(&doc["items"], location, doc.GetAllocator());

	if ((total_completed_ == total_needed_) && !cancel_update_) {
		FinishUpdate();
		PreserveSelectedCharacter();
	};
}

bool ItemsManagerWorker::TabsChanged(rapidjson::Document& doc, QNetworkReply* network_reply, ItemLocation& location) {

	if (!doc.HasMember("tabs") || doc["tabs"].Size() == 0) {
		QLOG_ERROR() << "Full tab information missing from stash tab fetch.  Cancelling update. Full fetch URL: "
			<< network_reply->request().url().toDisplayString();
		return true;
	};

	std::string tabs_as_string = Util::RapidjsonSerialize(doc["tabs"]);
	auto tabs_signature_current = CreateTabsSignatureVector(tabs_as_string);
	auto tab_id = location.get_tab_id();
	if (tabs_signature_[tab_id] != tabs_signature_current[tab_id]) {

		std::string reason;
		if (tabs_signature_current.size() != tabs_signature_.size()) {
			reason += "[Tab size mismatch:"
				+ std::to_string(tabs_signature_current.size())
				+ " != " + std::to_string(tabs_signature_.size()) + "]";
		};

		auto& x = tabs_signature_current[tab_id];
		auto& y = tabs_signature_[tab_id];
		reason += "[tab_index=" + std::to_string(tab_id)
			+ "/" + std::to_string(tabs_signature_current.size())
			+ "(#" + std::to_string(tab_id + 1) + ")]";

		if (x.first != y.first) {
			reason += "[name:" + x.first + " != " + y.first + "]";
		};
		if (x.second != y.second) {
			reason += "[id:" + x.second + " != " + y.second + "]";
		};

		QLOG_ERROR() << "You renamed or re-ordered tabs in game while acquisition was in the middle of the update,"
			<< " aborting to prevent synchronization problems and pricing data loss. Mismatch reason(s) -> "
			<< reason.c_str() << ". For request: " << network_reply->request().url().toDisplayString();
		return true;
	};
	return false;
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
	std::map<ItemLocationType, QStringList> tabsPerType;
	for (auto const& tab : tabs_) {
		const ItemLocationType& tab_type = tab.get_type();
		tabsPerType[tab_type].push_back(QString::fromStdString(tab.get_json()));
	};

	// Map locations to a list of items in that location.
	std::map<ItemLocation, QStringList> itemsPerLoc;
	for (auto const& item : items_) {
		const ItemLocation& item_location = item->location();
		itemsPerLoc[item_location].push_back(QString::fromStdString(item->json()));
	};

	// Save tabs by tab type.
	for (auto const& tab : tabsPerType) {
		const ItemLocationType& tab_type = tab.first;
		const QString tab_json = "[" + tab.second.join(",") + "]";
		app_.data().SetTabs(tab_type, tab_json.toStdString());
	};

	// Save items by location.
	for (auto const& items : itemsPerLoc) {
		const ItemLocation& items_location = items.first;
		const QString items_json = "[" + items.second.join(",") + "]";
		app_.data().SetItems(items_location, items_json.toStdString());
	};

	// Let everyone know the update is done.
	emit ItemsRefreshed(items_, tabs_, false);

	updating_ = false;
	QLOG_DEBUG() << "Update finished.";
}

void ItemsManagerWorker::PreserveSelectedCharacter() {
	if (selected_character_.empty()) {
		QLOG_DEBUG() << "Cannot preserve selected character: no character selected";
		return;
	};
	QLOG_DEBUG() << "Preserving selected character:" << selected_character_.c_str();
	// The act of making this request sets the active character.
	// We don't need to to anything with the reply.
	QNetworkRequest character_request = MakeCharacterRequest(selected_character_);
	rate_limiter_.Submit(character_request, [](QNetworkReply*) {});
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

