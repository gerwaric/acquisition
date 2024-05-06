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

#include "legacyitemsworker.h"

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

LegacyItemsWorker::LegacyItemsWorker(Application& app) : ItemsManagerWorker(app),
	total_completed_(-1),
	total_needed_(-1),
	requests_completed_(-1),
	requests_needed_(-1),
	queue_id_(-1)
{}

LegacyItemsWorker::~LegacyItemsWorker()
{}

void LegacyItemsWorker::DoUpdate() {

	// remove all pending requests
	queue_ = std::queue<ItemsRequest>();
	queue_id_ = 0;

	tabs_as_string_ = "";
	selected_character_ = "";

	// first, download the main page because it's the only way to know which character is selected
	QNetworkRequest main_page_request = QNetworkRequest(QUrl(kMainPage));
	rate_limiter_.Submit(main_page_request,
		[=](QNetworkReply* reply) {
			OnMainPageReceived(reply);
		});
}

void LegacyItemsWorker::OnMainPageReceived(QNetworkReply* reply) {
	QLOG_TRACE() << "Main page received.";

	if (reply->error()) {
		QLOG_WARN() << "Couldn't fetch main page: " << reply->url().toDisplayString() << " due to error: " << reply->errorString();
	} else {
		std::string page(reply->readAll().constData());
		selected_character_ = Util::FindTextBetween(page, "C({\"name\":\"", "\",\"class");
		selected_character_ = Util::ConvertAsciiToUtf(selected_character_);
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

void LegacyItemsWorker::OnCharacterListReceived(QNetworkReply* reply) {
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

QNetworkRequest LegacyItemsWorker::MakeTabRequest(int tab_index, bool tabs) {
	QUrlQuery query;
	query.addQueryItem("league", QString::fromUtf8(app_.league()));
	query.addQueryItem("tabs", tabs ? "1" : "0");
	query.addQueryItem("tabIndex", std::to_string(tab_index).c_str());
	query.addQueryItem("accountName", QString::fromUtf8(app_.email()));

	QUrl url(kStashItemsUrl);
	url.setQuery(query);
	return QNetworkRequest(url);
}

QNetworkRequest LegacyItemsWorker::MakeCharacterRequest(const std::string& name) {
	QUrlQuery query;
	query.addQueryItem("character", QString::fromUtf8(name));
	query.addQueryItem("accountName", QString::fromUtf8(app_.email()));

	QUrl url(kCharacterItemsUrl);
	url.setQuery(query);
	return QNetworkRequest(url);
}

QNetworkRequest LegacyItemsWorker::MakeCharacterPassivesRequest(const std::string& name) {
	QUrlQuery query;
	query.addQueryItem("character", QString::fromUtf8(name));
	query.addQueryItem("accountName", QString::fromUtf8(app_.email()));

	QUrl url(kCharacterSocketedJewels);
	url.setQuery(query);
	return QNetworkRequest(url);
}

void LegacyItemsWorker::QueueRequest(const QNetworkRequest& request, const ItemLocation& location) {
	QLOG_DEBUG() << "Queued (" << queue_id_ + 1 << ") -- " << location.GetHeader().c_str();
	ItemsRequest items_request;
	items_request.network_request = request;
	items_request.id = queue_id_++;
	items_request.location = location;
	queue_.push(items_request);
}

void LegacyItemsWorker::FetchItems() {
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

void LegacyItemsWorker::OnFirstTabReceived(QNetworkReply* reply) {
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

void LegacyItemsWorker::ParseItems(rapidjson::Value* value_ptr, ItemLocation base_location, rapidjson_allocator& alloc) {
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

void LegacyItemsWorker::OnTabReceived(QNetworkReply* network_reply, ItemLocation location) {
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
		PreserveSelectedCharacter();
		FinishUpdate();
	};
}

bool LegacyItemsWorker::TabsChanged(rapidjson::Document& doc, QNetworkReply* network_reply, ItemLocation& location) {

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

void LegacyItemsWorker::PreserveSelectedCharacter() {
	if (selected_character_.empty()) {
		QLOG_DEBUG() << "Cannot preserve selected character: no character selected";
		return;
	};
	QLOG_DEBUG() << "Preserving selected character:" << QString::fromUtf8(selected_character_);
	// The act of making this request sets the active character.
	// We don't need to to anything with the reply.
	QNetworkRequest character_request = MakeCharacterRequest(selected_character_);
	rate_limiter_.Submit(character_request, [](QNetworkReply*) {});
}

std::vector<std::pair<std::string, std::string> > LegacyItemsWorker::CreateTabsSignatureVector(std::string tabs) {
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
