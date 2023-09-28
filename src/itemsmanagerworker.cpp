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
#include "tabcache.h"
#include "mainwindow.h"
#include "buyoutmanager.h"
#include "filesystem.h"
#include "modlist.h"
#include "network_info.h"
#include "ratelimit.h"

const char *kStashItemsUrl = "https://www.pathofexile.com/character-window/get-stash-items";
const char *kCharacterItemsUrl = "https://www.pathofexile.com/character-window/get-items";
const char *kGetCharactersUrl = "https://www.pathofexile.com/character-window/get-characters";
const char *kMainPage = "https://www.pathofexile.com/";
//While the page does say "get passive skills", it seems to only send socketed jewels
const char *kCharacterSocketedJewels = "https://www.pathofexile.com/character-window/get-passive-skills";

const char *kPOE_trade_stats = "https://www.pathofexile.com/api/trade/data/stats";

const char *kRePoE_stat_translations = "https://raw.githubusercontent.com/brather1ng/RePoE/master/RePoE/data/stat_translations.min.json";
const char *kRePoE_item_classes = "https://raw.githubusercontent.com/brather1ng/RePoE/master/RePoE/data/item_classes.min.json";
const char *kRePoE_item_base_types = "https://raw.githubusercontent.com/brather1ng/RePoE/master/RePoE/data/base_items.min.json";

ItemsManagerWorker::ItemsManagerWorker(Application &app, QThread *thread) :
	data_(app.data()),
	league_(app.league()),
	updating_(false),
	bo_manager_(app.buyout_manager()),
	account_name_(app.email()),
    rate_limiter_(app.rate_limiter())
{
	QUrl poe(kMainPage);

	QDir cache_path{std::string{Filesystem::UserDir() + "/tabcache/" + account_name_ + "/" + league_}.c_str()};

	QLOG_DEBUG() << "Cache directory: " << cache_path.path();

	tab_cache_->setCacheDirectory(cache_path.path());
	tab_cache_->setMaximumCacheSize(kMaxCacheSize);

	// setCache takes ownership of tab_cache ptr so we don't need to destruct it
	network_manager_.setCache(tab_cache_);
	network_manager_.cookieJar()->setCookiesFromUrl(app.logged_in_nm().cookieJar()->cookiesForUrl(poe), poe);
	network_manager_.moveToThread(thread);
}

ItemsManagerWorker::~ItemsManagerWorker() {
}

void ItemsManagerWorker::Init(){
	if (updating_) {
		QLOG_WARN() << "ItemsManagerWorker::Init() called while updating, skipping Mod List Update";
		return;
	}

	updating_ = true;

	QNetworkRequest PoE_item_classes_request = QNetworkRequest(QUrl(QString(kRePoE_item_classes)));
	rate_limiter_.Submit(network_manager_, PoE_item_classes_request,
		[=](QNetworkReply* reply) {
			OnItemClassesReceived(reply);
		});
}

void ItemsManagerWorker::CheckForViolation(QNetworkReply* reply) {
    if (reply->hasRawHeader("Retry-After")) {
        QLOG_ERROR() << "unnhandled rate limit violation in reply to" << reply->url().toDisplayString();
        for (auto pair : reply->rawHeaderPairs()) {
            QLOG_ERROR() << "\n\t" << pair.first << "=" << pair.second;
        };
        RateLimit::Error("unhandled rate limit violation!");
    };
}

void ItemsManagerWorker::OnItemClassesReceived(QNetworkReply *reply){

    QLOG_TRACE() << "Item classes received.";
    CheckForViolation(reply);

	if (reply->error()) {
		QLOG_ERROR() << "Couldn't fetch RePoE Item Classes: " << reply->url().toDisplayString()
					<< " due to error: " << reply->errorString() << " The type dropdown will remain empty.";
	} else {
		QByteArray bytes = reply->readAll();
		emit ItemClassesUpdate(bytes);
	}

	QNetworkRequest PoE_item_base_types_request = QNetworkRequest(QUrl(QString(kRePoE_item_base_types)));
    rate_limiter_.Submit(network_manager_, PoE_item_base_types_request,
		[=](QNetworkReply* reply) {
			OnItemBaseTypesReceived(reply);
		});
}

void ItemsManagerWorker::OnItemBaseTypesReceived(QNetworkReply* reply){

    QLOG_TRACE() << "Item base types received.";
    CheckForViolation(reply);

	if (reply->error()) {
		QLOG_ERROR() << "Couldn't fetch RePoE Item Base Types: " << reply->url().toDisplayString()
					<< " due to error: " << reply->errorString() << " The type dropdown will remain empty.";
	} else {
		QByteArray bytes = reply->readAll();
		emit ItemBaseTypesUpdate(bytes);
	}

	UpdateModList();
}

void ItemsManagerWorker::ParseItemMods() {
	tabs_.clear();
	tab_id_index_.clear();

	//Get cached tabs (item tabs not search tabs)
	for(ItemLocationType type : {ItemLocationType::STASH, ItemLocationType::CHARACTER}){
		std::string tabs = data_.GetTabs(type);
		tabs_signature_ = CreateTabsSignatureVector(tabs);
		if (tabs.size() != 0) {
			rapidjson::Document doc;
			if (doc.Parse(tabs.c_str()).HasParseError()) {
				QLOG_ERROR() << "Malformed tabs data:" << tabs.c_str() << "The error was"
					<< rapidjson::GetParseError_En(doc.GetParseError());
				continue;
			}
			for (auto &tab : doc) {
				//constructor values to fill in
				int index;
				std::string tabUniqueId, name;
				int r, g, b;

				if(type == ItemLocationType::STASH){
					if(tab_id_index_.count(tab["id"].GetString())){
					   continue;
					}
					if (!tab.HasMember("n") || !tab["n"].IsString()) {
						QLOG_ERROR() << "Malformed tabs data:" << tabs.c_str() << "Tab doesn't contain its name (field 'n').";
						continue;
					}

					index = tab["i"].GetInt();
					tabUniqueId = tab["id"].GetString();
					name = tab["n"].GetString();
					r = tab["colour"]["r"].GetInt();
					g = tab["colour"]["g"].GetInt();
					b = tab["colour"]["b"].GetInt();
				} else {
					if(tab_id_index_.count(tab["name"].GetString())){
					   continue;
					}

					if(tab.HasMember("i"))
						index = tab["i"].GetInt();
					else
						index = tabs_.size();

					tabUniqueId = tab["name"].GetString();
					name = tab["name"].GetString();
					r = 0;
					g = 0;
					b = 0;
				}

				ItemLocation loc(index, tabUniqueId, name, type, r, g, b);
				loc.set_json(tab, doc.GetAllocator());
				tabs_.push_back(loc);
				tab_id_index_.insert(loc.get_tab_uniq_id());
			}
		}
	}

	items_.clear();

	//Get cached items
	for (int i = 0; i < tabs_.size(); i++){
		auto tab = tabs_[i];

		std::string items = data_.GetItems(tab);
		if (items.size() != 0) {
			rapidjson::Document doc;
			doc.Parse(items.c_str());
			for (auto item = doc.Begin(); item != doc.End(); ++item)
				items_.push_back(std::make_shared<Item>(*item, tab));
		}

		CurrentStatusUpdate status;
		status.state = ProgramState::ItemsRetrieved;
		status.progress = i + 1;
		status.total = tabs_.size();

		emit StatusUpdate(status);
	}

	//let ItemManager know that the retrieval of cached items/tabs has been completed (calls ItemsManager::OnItemsRefreshed method)
	emit ItemsRefreshed(items_, tabs_, true);
}

void ItemsManagerWorker::UpdateModList(){
	modsUpdating_ = true;

	QNetworkRequest PoE_stat_translations_request = QNetworkRequest(QUrl(QString(kRePoE_stat_translations)));
	rate_limiter_.Submit(network_manager_, PoE_stat_translations_request,
		[=](QNetworkReply* reply) {
			OnStatTranslationsReceived(reply);
		});
}

void ItemsManagerWorker::OnStatTranslationsReceived(QNetworkReply* reply){

    QLOG_TRACE() << "Stat translations received.";
    CheckForViolation(reply);

	if (reply->error()) {
		QLOG_ERROR() << "Couldn't fetch RePoE Stat Translations: " << reply->url().toDisplayString()
					<< " due to error: " << reply->errorString() << " Aborting update.";
		modsUpdating_ = false;
		return;
	} else {
		QByteArray bytes = reply->readAll();
		rapidjson::Document doc;
		doc.Parse(bytes.constData());

		if (doc.HasParseError()) {
			QLOG_ERROR() << "Couldn't properly parse Stat Translations from RePoE, canceling Mods Update";
			modsUpdating_ = false;
			return;
		}

		mods.clear();
		std::set<std::string> stat_strings;

		for (auto &translation : doc){
			for (auto &stat : translation["English"]){
				std::vector<std::string> formats;
				for (auto &format : stat["format"])
					formats.push_back(format.GetString());

				std::string stat_string = stat["string"].GetString();
				if(formats[0].compare("ignore") != 0){
					for (int i = 0; i < formats.size(); i++) {
						std::string searchString = "{" + std::to_string(i) + "}";
						boost::replace_all(stat_string, searchString, formats[i]);
					}
				}

				if (stat_string.length() > 0)
					stat_strings.insert(stat_string);
			}
		}

		for (std::string stat_string : stat_strings) {
			mods.push_back({ stat_string });
		}

		InitModlist();
	}

	ParseItemMods();

	modsUpdating_ = false;
	initialModUpdateCompleted_ = true;
	updating_ = false;

	if (updateRequest_){
		updateRequest_ = false;
		Update(type_, locations_);
	}

	reply->deleteLater();
}

void ItemsManagerWorker::Update(TabSelection::Type type, const std::vector<ItemLocation> &locations) {
	if (updating_) {
		QLOG_WARN() << "ItemsManagerWorker::Update called while updating";
		return;
	}

	selected_tabs_.clear();
	for (auto const &tab: locations) {
		selected_tabs_.insert(tab.get_tab_uniq_id());
	}

	tab_selection_ = type;

    switch (tab_selection_) {
    case TabSelection::Type::All: QLOG_DEBUG() << "Updating all stash tabs"; break;
    case TabSelection::Type::Checked: QLOG_DEBUG() << "Updating checked stash tabs"; break;
    case TabSelection::Type::Selected: QLOG_DEBUG() << "Updating selected stash tabs"; break;
    };
	updating_ = true;

	cancel_update_ = false;
	// remove all pending requests
	queue_ = std::queue<ItemsRequest>();
	queue_id_ = 0;
	replies_.clear();
	items_.clear();
	tabs_as_string_ = "";
	selected_character_ = "";

	// first, download the main page because it's the only way to know which character is selected
	QNetworkRequest main_page_request = Request(QUrl(kMainPage), ItemLocation(), TabCache::Refresh);
	main_page_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
	rate_limiter_.Submit(network_manager_, main_page_request,
		[=](QNetworkReply* reply) {
			OnMainPageReceived(reply);
		});
}

void ItemsManagerWorker::OnMainPageReceived(QNetworkReply* reply) {

    QLOG_TRACE() << "Main page received.";
    CheckForViolation(reply);

	if (reply->error()) {
		QLOG_WARN() << "Couldn't fetch main page: " << reply->url().toDisplayString() << " due to error: " << reply->errorString();
	} else {
		std::string page(reply->readAll().constData());

		selected_character_ = Util::FindTextBetween(page, "C({\"name\":\"", "\",\"class");
		if (selected_character_.empty()) {
			QLOG_WARN() << "Couldn't extract currently selected character name from GGG homepage (maintenence?) Text was: " << page.c_str();
		}
	}

	// now get character list
	QNetworkRequest characters_request = Request(QUrl(kGetCharactersUrl), ItemLocation(), TabCache::Refresh);
	characters_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
	rate_limiter_.Submit(network_manager_, characters_request,
		[=](QNetworkReply* reply) {
			OnCharacterListReceived(reply);
		});
}

void ItemsManagerWorker::OnCharacterListReceived(QNetworkReply* reply) {
    
    QLOG_TRACE() << "Character list received.";
    CheckForViolation(reply);

	QByteArray bytes = reply->readAll();
	rapidjson::Document doc;
	doc.Parse(bytes.constData());

	if (reply->error()) {
		QLOG_WARN() << "Couldn't fetch character list: " << reply->url().toDisplayString()
					<< " due to error: " << reply->errorString() << " Aborting update.";
		updating_ = false;
		return;
	} else {
		if (doc.HasParseError() || !doc.IsArray()) {
			QLOG_ERROR() << "Received invalid reply instead of character list. The reply was: "
						 << bytes.constData();
			if (doc.HasParseError()) {
				QLOG_ERROR() << "The error was" << rapidjson::GetParseError_En(doc.GetParseError());
			}
			QLOG_ERROR() << "";
			QLOG_ERROR() << "(Maybe you need to log in to the website manually and accept new Terms of Service?)";
			updating_ = false;
			return;
		}
	}

	QLOG_DEBUG() << "Received character list, there are" << doc.Size() << "characters";

	//clear out ItemLocationType::CHARACTER tabs from tabs_
	std::vector<ItemLocation> tmpLocs = tabs_;
	for(auto &tab : tmpLocs){
		if(tab.get_type() == ItemLocationType::CHARACTER){
			tabs_.erase(std::remove(tabs_.begin(), tabs_.end(), tab), tabs_.end());
			tab_id_index_.erase(tab.get_tab_uniq_id());
		}
	}

	auto char_count = 0;
	for (auto &character : doc) {
		if (!character.HasMember("league") || !character.HasMember("name") || !character["league"].IsString() || !character["name"].IsString()) {
			QLOG_ERROR() << "Malformed character entry, the reply is most likely invalid" << bytes.constData();
			continue;
		}
		if (character["league"].GetString() == league_) {
			char_count++;
			std::string name = character["name"].GetString();
			ItemLocation location;
			location.set_type(ItemLocationType::CHARACTER);
			location.set_character(name);
			location.set_json(character, doc.GetAllocator());
			location.set_tab_id(tabs_.size());
			tabs_.push_back(location);
			//Queue request for items on character in character's stash
			QueueRequest(MakeCharacterRequest(name, location), location);

			//Queue request for jewels in character's passive tree
			QueueRequest(MakeCharacterPassivesRequest(name, location), location);
		}
	}
	CurrentStatusUpdate status;
	status.state = ProgramState::CharactersReceived;
	status.total = char_count;

	emit StatusUpdate(status);

	if (char_count == 0) {
		updating_ = false;
		return;
	}

	// Fetch a single tab and also request tabs list.  We can fetch any tab here with tabs list
	// appended, so prefer one that the user has already 'checked'.  Default to index '1' which is
	// first user visible tab.
	first_fetch_tab_ = "";
	ItemLocation tabToReq;
	if (tab_selection_ == TabSelection::Checked) {
		for (auto const &tab : tabs_) {
			if (bo_manager_.GetRefreshChecked(tab)) {
				first_fetch_tab_ = tab.get_tab_uniq_id();
				tabToReq = tab;
				break;
			}
		}
	}
	// If we're refreshing a manual selection of tabs choose one of them to save a tab fetch
	if (tab_selection_ == TabSelection::Selected) {
		for (auto const &tab : tabs_) {
			if (selected_tabs_.count(tab.get_tab_uniq_id())) {
				first_fetch_tab_ = tab.get_tab_uniq_id();
				tabToReq = tab;
				break;
			}
		}
	}

	QNetworkRequest tab_request = MakeTabRequest(tabToReq.get_tab_id(), ItemLocation(), true, true);
	rate_limiter_.Submit(network_manager_, tab_request,
		[=](QNetworkReply* reply) {
			OnFirstTabReceived(reply);
		});

}

QNetworkRequest ItemsManagerWorker::MakeTabRequest(int tab_index, const ItemLocation &location, bool tabs, bool refresh) {
	QUrlQuery query;
	query.addQueryItem("league", league_.c_str());
	query.addQueryItem("tabs", tabs ? "1" : "0");
	query.addQueryItem("tabIndex", std::to_string(tab_index).c_str());
	query.addQueryItem("accountName", account_name_.c_str());

	QUrl url(kStashItemsUrl);
	url.setQuery(query);

	// If refresh is explicity request then force unconditionally
	TabCache::Flags flags = (refresh) ? TabCache::Refresh : TabCache::None;

	QNetworkRequest tab_request = Request(url, location, flags);
	tab_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
	return tab_request;
}

QNetworkRequest ItemsManagerWorker::MakeCharacterRequest(const std::string &name, const ItemLocation &location) {
	QUrlQuery query;
	query.addQueryItem("character", name.c_str());
	query.addQueryItem("accountName", account_name_.c_str());

	QUrl url(kCharacterItemsUrl);
	url.setQuery(query);

	return Request(url, location, TabCache::None);
}

QNetworkRequest ItemsManagerWorker::MakeCharacterPassivesRequest(const std::string &name, const ItemLocation &location) {
	QUrlQuery query;
	query.addQueryItem("character", name.c_str());
	query.addQueryItem("accountName", account_name_.c_str());

	QUrl url(kCharacterSocketedJewels);
	url.setQuery(query);

	return Request(url, location, TabCache::None);
}

QNetworkRequest ItemsManagerWorker::Request(QUrl url, const ItemLocation &location, TabCache::Flags flags) {
	switch (tab_selection_) {
	case TabSelection::All:
		flags |= TabCache::Refresh;
		break;
	case TabSelection::Checked:
		if (!location.IsValid() || bo_manager_.GetRefreshChecked(location))
			flags |= TabCache::Refresh;
		break;
	case TabSelection::Selected:
		if (!location.IsValid() || selected_tabs_.count(location.get_tab_uniq_id()))
			flags |= TabCache::Refresh;
		break;
	}
	return tab_cache_->Request(url, flags);
}

void ItemsManagerWorker::QueueRequest(const QNetworkRequest &request, const ItemLocation &location) {
	QLOG_DEBUG() << "Queued (" << queue_id_ + 1 << ") -- " << location.GetHeader().c_str();
	ItemsRequest items_request;
	items_request.network_request = request;
	items_request.id = queue_id_++;
	items_request.location = location;
	queue_.push(items_request);
}

void ItemsManagerWorker::FetchItems(int limit) {
	std::string tab_titles;
	int count = std::min(limit, static_cast<int>(queue_.size()));
	for (int i = 0; i < count; ++i) {
		ItemsRequest request = queue_.front();
		queue_.pop();

		QNetworkRequest fetch_request = request.network_request;
		fetch_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
		int id = request.id;
		ItemLocation location = request.location;
		rate_limiter_.Submit(network_manager_, fetch_request,
			[=](QNetworkReply* reply) {
				OnTabReceived(reply, id, location);
			});

		ItemsReply reply;
		reply.network_reply = nullptr;
		reply.request = request;
		replies_[request.id] = reply;

		tab_titles += request.location.GetHeader() + " ";
	}
	QLOG_DEBUG() << "Created" << count << "requests:" << tab_titles.c_str();
	requests_needed_ = count;
	requests_completed_ = 0;
	cached_requests_completed_ = 0;
}

void ItemsManagerWorker::OnFirstTabReceived(QNetworkReply* reply) {
	
    QLOG_TRACE() << "First tab received.";
    CheckForViolation(reply);

	QByteArray bytes = reply->readAll();
	rapidjson::Document doc;
	doc.Parse(bytes.constData());

	if (!doc.IsObject()) {
		QLOG_ERROR() << "Can't even fetch first tab. Failed to update items.";
		updating_ = false;
		return;
	}

	if (doc.HasMember("error")) {
		QLOG_ERROR() << "Aborting update since first fetch failed due to 'error': " << Util::RapidjsonSerialize(doc["error"]).c_str();
		updating_ = false;
		return;
	}

	if (!doc.HasMember("tabs") || doc["tabs"].Size() == 0) {
		QLOG_WARN() << "There are no tabs, this should not happen, bailing out.";
		updating_ = false;
		return;
	}

	tabs_as_string_ = Util::RapidjsonSerialize(doc["tabs"]);
	tabs_signature_ = CreateTabsSignatureVector(tabs_as_string_);

	QLOG_DEBUG() << "Received tabs list, there are" << doc["tabs"].Size() << "tabs";

	std::set<std::string> old_tab_headers;
	for (auto const &tab: tabs_) {
		// Remember old tab headers before clearing tabs
		old_tab_headers.insert(tab.GetHeader());
	}

	//clear out ItemLocationType::STASH tabs from tabs_
	std::vector<ItemLocation> tmpLocs = tabs_;
	for(auto &tab : tmpLocs){
		if(tab.get_type() == ItemLocationType::STASH){
			tabs_.erase(std::remove(tabs_.begin(), tabs_.end(), tab), tabs_.end());
			tab_id_index_.erase(tab.get_tab_uniq_id());
		}
	}

	// Create tab location objects
	for (auto &tab : doc["tabs"]) {
		std::string label = tab["n"].GetString();
		auto index = tab["i"].GetInt();
		// Ignore hidden locations
		if (!doc["tabs"][index].HasMember("hidden") || !doc["tabs"][index]["hidden"].GetBool()){
			int r = tab["colour"]["r"].GetInt();
			int g = tab["colour"]["g"].GetInt();
			int b = tab["colour"]["b"].GetInt();

			ItemLocation loc(index, tab["id"].GetString(), label, ItemLocationType::STASH, r, g, b);
			loc.set_json(tab, doc.GetAllocator());
			tabs_.push_back(loc);
			tab_id_index_.insert(tab["id"].GetString());
		}
	}

	// Queue requests for Stash tabs
	for (auto const &tab: tabs_) {
		bool refresh = false;

		// Force refreshes for any tabs that were moved or renamed regardless of what user
		// requests for refresh.
		if (!old_tab_headers.count(tab.GetHeader())) {
			QLOG_DEBUG() << "Forcing refresh of moved or renamed tab: " << tab.GetHeader().c_str();
			refresh = true;
		}
		if(tab.get_type() == ItemLocationType::STASH)
			QueueRequest(MakeTabRequest(tab.get_tab_id(), tab, true, refresh), tab);
	}

	total_needed_ = queue_.size();
	total_completed_ = 0;
	total_cached_ = reply->attribute(QNetworkRequest::SourceIsFromCacheAttribute).toBool() ? 1:0;

	FetchItems(kThrottleRequests - 1);

}

void ItemsManagerWorker::ParseItems(rapidjson::Value *value_ptr, ItemLocation base_location, rapidjson_allocator &alloc) {
	auto &value = *value_ptr;

	std::string test = Util::RapidjsonSerialize(value);
	if(base_location.get_type() == ItemLocationType::CHARACTER){
	   // QLOG_DEBUG() << test.c_str();
	}

	for (auto &item : value) {
		//ItemLocation location(base_location);
		base_location.FromItemJson(item);
		base_location.ToItemJson(&item, alloc);
		items_.push_back(std::make_shared<Item>(item, base_location));
		base_location.set_socketed(true);
		if (item.HasMember("socketedItems") && item["socketedItems"].IsArray()){
			ParseItems(&item["socketedItems"], base_location, alloc);
		}
	}
}

void ItemsManagerWorker::OnTabReceived(QNetworkReply* network_reply, int request_id, ItemLocation location) {

    QLOG_TRACE() << "Tab received:" << location.GetHeader().c_str();
    CheckForViolation(network_reply);

	if (!replies_.count(request_id)) {
		QLOG_WARN() << "Received a reply for request" << request_id << "that was not requested.";
		return;
	}

	bool reply_from_cache = network_reply->attribute(QNetworkRequest::SourceIsFromCacheAttribute).toBool();

	if (reply_from_cache) {
		QLOG_DEBUG() << "Received a cached reply for" << location.GetHeader().c_str();
		++cached_requests_completed_;
		++total_cached_;
	} else {
		QLOG_DEBUG() << "Received a reply for" << location.GetHeader().c_str();
	}

	QByteArray bytes = network_reply->readAll();
	rapidjson::Document doc;
	doc.Parse(bytes.constData());

	bool error = false;
	if (!doc.IsObject()) {
		QLOG_WARN() << request_id << "got a non-object response";
		error = true;
	} else if (doc.HasMember("error")) {
		// this can happen if user is browsing stash in background and we can't know about it
		QLOG_WARN() << request_id << "got 'error' instead of stash tab contents: " << Util::RapidjsonSerialize(doc["error"]).c_str();
		error = true;
	}

	// We index expected tabs and their locations as part of the first fetch.  It's possible for users
	// to move or rename tabs during the update which will result in the item data being out-of-sync with
	// expected index/tab name map.  We need to detect this case and abort the update.
	if (!cancel_update_ && !error && (location.get_type() == ItemLocationType::STASH)) {
		if (!doc.HasMember("tabs") || doc["tabs"].Size() == 0) {
			QLOG_ERROR() << "Full tab information missing from stash tab fetch.  Cancelling update. Full fetch URL: "
						 << network_reply->request().url().toDisplayString();
			cancel_update_ = true;
		} else {
			std::string tabs_as_string = Util::RapidjsonSerialize(doc["tabs"]);
			auto tabs_signature_current = CreateTabsSignatureVector(tabs_as_string);

			auto tab_id = location.get_tab_id();
			if (tabs_signature_[tab_id] != tabs_signature_current[tab_id]) {
				if (reply_from_cache) {
					// Here we unexpectedly are seeing a cached document that is out-of-sync with current tab state
					// This is not fatal but unexpected as we shouldn't get here if everything else is done right.
					// If we do see, set 'error' condition which causes us to flush from catch and re-fetch from server.
					QLOG_WARN() << "Unexpected hit on stale cached tab.  Flushing and re-fetching request: "
								<< network_reply->request().url().toDisplayString();
					error = true;
					// Isn't really cached since we're erroring out and replaying so fix up stats
					total_cached_--;
				} else {
					std::string reason;
					if (tabs_signature_current.size() != tabs_signature_.size())
						reason += "[Tab size mismatch:" + std::to_string(tabs_signature_current.size()) + " != "
								+ std::to_string(tabs_signature_.size()) + "]";

					auto &x = tabs_signature_current[tab_id];
					auto &y = tabs_signature_[tab_id];
					reason += "[tab_index=" + std::to_string(tab_id) + "/" + std::to_string(tabs_signature_current.size()) + "(#" + std::to_string(tab_id+1) + ")]";
					if (x.first != y.first)
						reason += "[name:" + x.first + " != " + y.first + "]";
					if (x.second != y.second)
						reason += "[id:" + x.second + " != " + y.second + "]";

					QLOG_ERROR() << "You renamed or re-ordered tabs in game while acquisition was in the middle of the update,"
								 << " aborting to prevent synchronization problems and pricing data loss. Mismatch reason(s) -> "
								 << reason.c_str() << ". For request: " << network_reply->request().url().toDisplayString();
					cancel_update_ = true;
				}
			}
		}
	}

	// re-queue a failed request
	if (error) {
		// We can 'cache' error response document so make sure we remove it
		// before reque
		tab_cache_->remove(network_reply->request().url());
		QueueRequest(network_reply->request(), location);
	}

	++requests_completed_;

	if (!error)
		++total_completed_;

	bool throttled = false;

	if (requests_completed_ == requests_needed_) {
		if (cancel_update_) {
			updating_ = false;
		} else if (queue_.size() > 0) {
			if (cached_requests_completed_ > 0) {
				// We basically don't want cached requests to count against throttle limit
				// so if we did get any cached requests fetch up to that number without a
				// large delay
				QTimer::singleShot(1, [&]() { FetchItems(cached_requests_completed_); });
			} else {
				throttled = true;
				QLOG_DEBUG() << "Sleeping one minute to prevent throttling.";
				QTimer::singleShot(kThrottleSleep * 1000, this, SLOT(FetchItems()));
			}
		}
	}
	CurrentStatusUpdate status = CurrentStatusUpdate();
	status.state = throttled ? ProgramState::ItemsPaused : ProgramState::ItemsReceive;
	status.progress = total_completed_;
	status.total = total_needed_;
	status.cached = total_cached_;
	if (total_completed_ == total_needed_)
		status.state = ProgramState::ItemsCompleted;
	if (cancel_update_)
		status.state = ProgramState::UpdateCancelled;
	emit StatusUpdate(status);

	if (error)
		return;

	ParseItems(&doc["items"], location, doc.GetAllocator());

	if ((total_completed_ == total_needed_) && !cancel_update_) {
		// It's possible that we receive character vs stash tabs out of order, or users
		// move items around in a tab and we get them in a different order. For
		// consistency we want to present the tab data in a deterministic way to the rest
		// of the application.  Especially so we don't try to update shop when nothing actually
		// changed.  So sort items_ here before emitting and then generate
		// item list as strings.

		std::sort(begin(items_), end(items_), [](const std::shared_ptr<Item> &a, const std::shared_ptr<Item> &b){
			return *a < *b;
		});

		std::map<ItemLocationType, QStringList> tabsPerType;
		//categorize tabs by tab type
		for(auto const tab : tabs_){
			tabsPerType[tab.get_type()].push_back(tab.get_json().c_str());
		}

		//loop through each item, put it in a set that equals its location
		std::map<ItemLocation, QStringList> itemsPerLoc;
		for (auto const &item: items_) {
			itemsPerLoc[item->location()].push_back(item->json().c_str());
		}

		//save tabs
		std::map<ItemLocationType, QStringList>::iterator tabIter;
		for ( tabIter = tabsPerType.begin(); tabIter != tabsPerType.end(); tabIter++ ){
		   data_.SetTabs(tabIter->first, std::string("[") + tabIter->second.join(",").toStdString() + "]");
		}

		//save items
		std::map<ItemLocation, QStringList>::iterator itemIter;
		for ( itemIter = itemsPerLoc.begin(); itemIter != itemsPerLoc.end(); itemIter++ )
		{
		   data_.SetItems(itemIter->first, std::string("[") + itemIter->second.join(",").toStdString() + "]");
		}

		// all requests completed
		emit ItemsRefreshed(items_, tabs_, false);

		updating_ = false;
		QLOG_DEBUG() << "Finished updating stash.";

		// if we're at the verge of getting throttled, sleep so we don't
		if (requests_completed_ == kThrottleRequests)
			QTimer::singleShot(kThrottleSleep, this, SLOT(PreserveSelectedCharacter()));
		else
			PreserveSelectedCharacter();
	}
}

void ItemsManagerWorker::PreserveSelectedCharacter() {
	if (selected_character_.empty())
		return;
	QNetworkRequest character_request = MakeCharacterRequest(selected_character_, ItemLocation());
	rate_limiter_.Submit(network_manager_, character_request, [](QNetworkReply*) {});
}


std::vector<std::pair<std::string, std::string> > ItemsManagerWorker::CreateTabsSignatureVector(std::string tabs) {
	std::vector<std::pair<std::string, std::string> > tmp;
	rapidjson::Document doc;

	if (doc.Parse(tabs.c_str()).HasParseError()) {
		QLOG_ERROR() << "Malformed tabs data:" << tabs.c_str() << "The error was"
			<< rapidjson::GetParseError_En(doc.GetParseError());
	} else {
		for (auto &tab : doc) {
			std::string name = (tab.HasMember("n") && tab["n"].IsString()) ? tab["n"].GetString(): "UNKNOWN_NAME";
			std::string uid = (tab.HasMember("id") && tab["id"].IsString()) ? tab["id"].GetString(): "UNKNOWN_ID";
			tmp.emplace_back(name,uid);
		}
	}
	return tmp;
}

