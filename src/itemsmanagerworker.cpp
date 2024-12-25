/*
    Copyright (C) 2014-2024 Acquisition Contributors

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
#include <QSettings>
#include <QSignalMapper>
#include <QThread>
#include <QTimer>
#include <QUrlQuery>

#include <algorithm>

#include <QsLog/QsLog.h>
#include <rapidjson/document.h>
#include <rapidjson/error/en.h>
#include <boost/algorithm/string.hpp>

#include "application.h"
#include "datastore/datastore.h"
#include "itemcategories.h"
#include "util/util.h"
#include "ui/mainwindow.h"
#include "network_info.h"
#include "buyoutmanager.h"
#include "modlist.h"
#include "util/rapidjson_util.h"
#include "ratelimit/ratelimit.h"
#include "ratelimit/ratelimiter.h"
#include "util/repoe.h"
#include "util/oauthmanager.h"

using rapidjson::HasArray;

constexpr const char* kStashItemsUrl = "https://www.pathofexile.com/character-window/get-stash-items";
constexpr const char* kCharacterItemsUrl = "https://www.pathofexile.com/character-window/get-items";
constexpr const char* kGetCharactersUrl = "https://www.pathofexile.com/character-window/get-characters";
constexpr const char* kMainPage = "https://www.pathofexile.com/";
//While the page does say "get passive skills", it seems to only send socketed jewels
constexpr const char* kCharacterSocketedJewels = "https://www.pathofexile.com/character-window/get-passive-skills";

constexpr const char* kPOE_trade_stats = "https://www.pathofexile.com/api/trade/data/stats";

constexpr const char* kOauthListStashesEndpoint = "GET /stash/<league>";
constexpr const char* kOAuthListStashesUrl = "https://api.pathofexile.com/stash";

constexpr const char* kOAuthListCharactersEndpoint = "GET /character";
constexpr const char* kOAuthListCharactersUrl = "https://api.pathofexile.com/character";

constexpr const char* kOAuthGetStashEndpoint = "GET /stash/<league>/<stash_id>[/<substash_id>]";
constexpr const char* kOAuthGetStashUrl = "https://api.pathofexile.com/stash";

constexpr const char* kOAuthGetCharacterEndpoint = "GET /character/<name>";
constexpr const char* kOAuthGetCharacterUrl = "https://api.pathofexile.com/character";

constexpr std::array CHARACTER_ITEM_FIELDS = {
    "equipment",
    "inventory",
    "rucksack",
    "jewels"
};

ItemsManagerWorker::ItemsManagerWorker(
    QSettings& settings,
    QNetworkAccessManager& network_manager,
    RePoE& repoe,
    BuyoutManager& buyout_manager,
    DataStore& datastore,
    RateLimiter& rate_limiter,
    POE_API mode)
    : settings_(settings)
    , network_manager_(network_manager)
    , repoe_(repoe)
    , datastore_(datastore)
    , buyout_manager_(buyout_manager)
    , rate_limiter_(rate_limiter)
    , mode_(mode)
    , test_mode_(false)
    , stashes_needed_(0)
    , stashes_received_(0)
    , characters_needed_(0)
    , characters_received_(0)
    , initialized_(false)
    , updating_(false)
    , cancel_update_(false)
    , updateRequest_(false)
    , type_(TabSelection::Type::Checked)
    , queue_id_(-1)
    , first_stash_request_index_(-1)
    , need_stash_list_(false)
    , need_character_list_(false)
    , has_stash_list_(false)
    , has_character_list_(false)
{
    QLOG_TRACE() << "ItemsManagerWorker::ItemsManagerWorker() entered";
}

void ItemsManagerWorker::UpdateRequest(TabSelection::Type type, const std::vector<ItemLocation>& locations) {
    QLOG_TRACE() << "ItemsManagerWorker::UpdateRequest() entered";
    updateRequest_ = true;
    type_ = type;
    locations_ = locations;
}

void ItemsManagerWorker::Init() {
    QLOG_TRACE() << "ItemsManagerWorker::Init() entered";
    if (updating_) {
        QLOG_WARN() << "ItemsManagerWorker::Init() called while updating, skipping Mod List Update";
        return;
    };

    realm_ = settings_.value("realm").toString().toStdString();
    league_ = settings_.value("league").toString().toStdString();
    account_ = settings_.value("account").toString().toStdString();
    updating_ = true;
    QLOG_TRACE() << "ItemsManagerWorker::Init() league =" << league_;
    QLOG_TRACE() << "ItemsManagerWorker::Init() account =" << account_;

    if (repoe_.IsInitialized()) {
        QLOG_DEBUG() << "RePoE data is available.";
        OnRePoEReady();
    } else {
        QLOG_DEBUG() << "Waiting for RePoE data.";
        connect(&repoe_, &RePoE::finished, this, &ItemsManagerWorker::OnRePoEReady);
    };
}

void ItemsManagerWorker::OnRePoEReady() {
    QLOG_TRACE() << "ItemsManagerWorker::OnRePoEReady() entered";
    // Create a separate thread to load the items, which allows the UI to
    // update the status bar while items are being parsed. This operation
    // can take tens of seconds or longer depending on the nubmer of tabs
    // and items.
    QThread* parser = QThread::create(
        [=]() {
            ParseItemMods();
        });
    parser->start();
};

void ItemsManagerWorker::ParseItemMods() {
    QLOG_TRACE() << "ItemsManagerWorker::ParseItemMods() entered";
    tabs_.clear();
    tabs_signature_.clear();
    tab_id_index_.clear();

    // Get cached tabs (item tabs not search tabs)
    for (ItemLocationType type : {ItemLocationType::STASH, ItemLocationType::CHARACTER}) {
        QLOG_TRACE() << "ItemsManagerWorker::ParseItemMods() getting cached tabs for" << type;
        Locations tabs = datastore_.GetTabs(type);
        tabs_.reserve(tabs_.size() + tabs.size());
        for (const auto& tab : tabs) {
            tabs_.push_back(tab);
        };
    };

    // Save location ids.
    QLOG_TRACE() << "ItemsManagerWorker::ParseItemMods() saving location ids";
    for (const auto& tab : tabs_) {
        tab_id_index_.insert(tab.get_tab_uniq_id());
    };

    // Build the signature vector.
    QLOG_TRACE() << "ItemsManangerWorker::ParseItemMods() building tabs signature";
    tabs_signature_.reserve(tabs_.size());
    for (const auto& tab : tabs_) {
        const std::string tab_name = tab.get_tab_label();
        const std::string tab_id = QString::number(tab.get_tab_id()).toStdString();
        tabs_signature_.push_back({ tab_name, tab_id });
    };

    // Get cached items
    QLOG_TRACE() << "ItemsManagerWorker::ParseItemMods() getting cached items";
    for (int i = 0; i < tabs_.size(); i++) {
        auto& tab = tabs_[i];
        Items tab_items = datastore_.GetItems(tab);
        items_.reserve(items_.size() + tab_items.size());
        QLOG_TRACE() << "ItemsManagerWorker::ParseItemMods() got" << tab_items.size() << "items from" << tab.GetHeader();
        for (const auto& tab_item : tab_items) {
            items_.push_back(tab_item);
        };
        emit StatusUpdate(
            ProgramState::Initializing,
            QString("Parsing items in %1/%2 tabs").arg(
                QString::number(i + 1),
                QString::number(tabs_.size())));
    };
    emit StatusUpdate(
        ProgramState::Ready,
        QString("Parsed items from %1 tabs").arg(
            QString::number(tabs_.size())));

    initialized_ = true;
    updating_ = false;

    // let ItemManager know that the retrieval of cached items/tabs has been completed (calls ItemsManager::OnItemsRefreshed method)
    QLOG_TRACE() << "ItemsManagerWorker::ParseItemMods() emitting ItemsRefreshed signal";
    emit ItemsRefreshed(items_, tabs_, true);

    if (updateRequest_) {
        QLOG_TRACE() << "ItemsManagerWorker::ParseItemMods() triggering requested update";
        updateRequest_ = false;
        Update(type_, locations_);
    };
}

void ItemsManagerWorker::Update(TabSelection::Type type, const std::vector<ItemLocation>& locations) {
    QLOG_TRACE() << "ItemsManagerWorker::Update() entered";
    if (updating_) {
        QLOG_WARN() << "ItemsManagerWorker::Update called while updating";
        return;
    };
    QLOG_DEBUG() << "Updating" << type << "stash tabs";
    updating_ = true;
    cancel_update_ = false;

    // remove all pending requests
    queue_ = {};
    queue_id_ = 0;

    selected_character_ = "";

    need_stash_list_ = false;
    need_character_list_ = false;

    first_stash_request_index_ = -1;
    first_character_request_name_ = "";

    if (type == TabSelection::All) {
        QLOG_DEBUG() << "Updating all tabs and items.";
        tabs_.clear();
        tab_id_index_.clear();
        items_.clear();
        first_stash_request_index_ = 0;
        need_stash_list_ = true;
        need_character_list_ = true;
    } else {
        // Build a list of tabs to update.
        std::set<std::string> tabs_to_update = {};
        switch (type) {
        case TabSelection::Checked:
            // Use the buyout manager to determine which tabs are check.
            QLOG_TRACE() << "ItemsManagerWorker::Update() updating checked tabs";
            for (auto const& tab : tabs_) {
                if ((tab.IsValid()) && (buyout_manager_.GetRefreshChecked(tab) == true)) {
                    tabs_to_update.insert(tab.get_tab_uniq_id());
                };
            };
            break;
        case TabSelection::Selected:
            // Use the function argument to determine which tabs were selected.
            QLOG_TRACE() << "ItemsManagerWorker::Update() updating selected tabs";
            for (auto const& tab : locations) {
                if (tab.IsValid()) {
                    tabs_to_update.insert(tab.get_tab_uniq_id());
                };
            };
            break;
        case TabSelection::All:
            // This case is handled by the enclosing if.
            break;
        };
        // Remove the tabs that will be updated, and all the items linked to those tabs.
        QLOG_DEBUG() << "Updating" << tabs_to_update.size() << " tabs.";
        RemoveUpdatingTabs(tabs_to_update);
        RemoveUpdatingItems(tabs_to_update);
        need_stash_list_ = (first_stash_request_index_ >= 0);
        need_character_list_ = !first_character_request_name_.empty();
    };

    has_stash_list_ = false;
    has_character_list_ = false;

    switch (mode_) {
    case POE_API::LEGACY: LegacyRefresh(); break;
    case POE_API::OAUTH: OAuthRefresh(); break;
    };
}

void ItemsManagerWorker::RemoveUpdatingTabs(const std::set<std::string>& tab_ids) {
    QLOG_TRACE() << "ItemsManagerWorker::RemoveUpdatingTabs() entered";
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
        } else {
            switch (tab.get_type()) {
            case ItemLocationType::STASH:
                if (first_stash_request_index_ < 0) {
                    first_stash_request_index_ = tab.get_tab_id();
                };
                break;
            case ItemLocationType::CHARACTER:
                if (first_character_request_name_.empty()) {
                    first_character_request_name_ = tab.get_character();
                };
                break;
            };
        };
    };
    QLOG_DEBUG() << "Keeping" << tabs_.size() << "tabs and culling" << (current_tabs.size() - tabs_.size());
}

void ItemsManagerWorker::RemoveUpdatingItems(const std::set<std::string>& tab_ids) {
    QLOG_TRACE() << "ItemsManagerWorker::RemoveUpdatingItems() entered";
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

void ItemsManagerWorker::LegacyRefresh() {
    QLOG_TRACE() << "ItemsManagerWorker::LegacyRefresh() entered";
    if (need_stash_list_) {
        // This queues stash tab requests.
        QNetworkRequest tab_request = MakeLegacyTabRequest(first_stash_request_index_, true);
        QLOG_TRACE() << "ItemsManagerWorker::LegacyRefresh() requesting stash list:" << tab_request.url().toString();
        auto reply = rate_limiter_.Submit(kStashItemsUrl, tab_request);
        connect(reply, &RateLimit::RateLimitedReply::complete, this, &ItemsManagerWorker::OnFirstLegacyTabReceived);
    };
    if (need_character_list_) {
        // Before listing characters we download the main page because it's the only way
        // to know which character is selected (doesn't apply to OAuth api).
        QNetworkRequest main_page_request = QNetworkRequest(QUrl(kMainPage));
        QLOG_TRACE() << "ItemsManagerWorker::LegacyRefresh() requesting main page to capture selected character:" << main_page_request.url().toString();
        main_page_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
        QNetworkReply* submit = network_manager_.get(main_page_request);
        connect(submit, &QNetworkReply::finished, this, &ItemsManagerWorker::OnLegacyMainPageReceived);
    };
}

void ItemsManagerWorker::OAuthRefresh() {
    QLOG_TRACE() << "ItemsManagerWorker::OAuthRefresh() entered";
    if (need_stash_list_) {
        const auto request = MakeOAuthStashListRequest(realm_, league_);
        QLOG_TRACE() << "ItemsManagerWorker::OAuthRefresh() requesting stash list:" << request.url().toString();
        auto reply = rate_limiter_.Submit(kOauthListStashesEndpoint, request);
        connect(reply, &RateLimit::RateLimitedReply::complete, this, &ItemsManagerWorker::OnOAuthStashListReceived);
    };
    if (need_character_list_) {
        const auto request = MakeOAuthCharacterListRequest(realm_);
        QLOG_TRACE() << "ItemsManagerWorker::OAuthRefresh() requesting character list:" << request.url().toString();
        auto submit = rate_limiter_.Submit(kOAuthListCharactersEndpoint, request);
        connect(submit, &RateLimit::RateLimitedReply::complete, this, &ItemsManagerWorker::OnOAuthCharacterListReceived);
    };
}

QNetworkRequest ItemsManagerWorker::MakeOAuthStashListRequest(
    const std::string& realm,
    const std::string& league)
{
    QLOG_TRACE() << "ItemsManagerWorker::MakeOAuthStashListRequest() entered";
    QString url(kOAuthListStashesUrl);
    if (realm != "pc") {
        url += "/" + realm;
    };
    url += "/" + league;
    return QNetworkRequest(QUrl(url));
}

QNetworkRequest ItemsManagerWorker::MakeOAuthCharacterListRequest(
    const std::string& realm)
{
    QLOG_TRACE() << "ItemsManagerWorker::MakeOAuthCharacterListRequest() entered";
    QString url(kOAuthListCharactersUrl);
    if (realm != "pc") {
        url += "/" + realm;
    };
    return QNetworkRequest(QUrl(url));
}

QNetworkRequest ItemsManagerWorker::MakeOAuthStashRequest(
    const std::string& realm,
    const std::string& league,
    const std::string& stash_id,
    const std::string& substash_id)
{
    QLOG_TRACE() << "ItemsManagerWorker::MakeOAuthStashRequest() entered";
    QString url(kOAuthGetStashUrl);
    if (realm != "pc") {
        url += "/" + realm;
    };
    url += "/" + league;
    url += "/" + stash_id;
    if (!substash_id.empty()) {
        url += "/" + substash_id;
    };
    return QNetworkRequest(QUrl(url));
}

QNetworkRequest ItemsManagerWorker::MakeOAuthCharacterRequest(
    const std::string& realm,
    const std::string& name)
{
    QLOG_TRACE() << "ItemsManagerWorker::MakeOAuthCharacterRequest() entered";
    QString url(kOAuthGetCharacterUrl);
    if (realm != "pc") {
        url += "/" + realm;
    };
    url += "/" + name;
    return QNetworkRequest(QUrl(url));
}

void ItemsManagerWorker::OnOAuthStashListReceived(QNetworkReply* reply) {
    QLOG_TRACE() << "ItemsManagerWorker::OnOAuthStashListReceived() entered";

    auto sender = qobject_cast<RateLimit::RateLimitedReply*>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    if (reply->error() != QNetworkReply::NoError) {
        QLOG_WARN() << "Aborting update because there was an error fetching the stash list:" << reply->errorString();
        updating_ = false;
        return;
    };
    const QByteArray bytes = reply->readAll();

    rapidjson::Document doc;
    doc.Parse(bytes);
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Error parsing the stash list:" << rapidjson::GetParseError_En(doc.GetParseError());
        updating_ = false;
        return;
    };

    if (!doc.IsObject() || !HasArray(doc, "stashes")) {
        QLOG_ERROR() << "The stash list is invalid:" << bytes;
        updating_ = false;
        return;
    };
    const auto& stashes = doc["stashes"].GetArray();

    QLOG_DEBUG() << "Received stash list, there are" << stashes.Size() << "stash tabs";

    tabs_signature_ = CreateTabsSignatureVector(stashes);

    // Remember old tab headers before clearing tabs
    std::set<std::string> old_tab_headers;
    for (auto const& tab : tabs_) {
        old_tab_headers.insert(tab.GetHeader());
    };

    // Force refreshes for any stash tabs that were moved or renamed.
    for (auto const& tab : tabs_) {
        if (!old_tab_headers.count(tab.GetHeader())) {
            QLOG_DEBUG() << "Forcing refresh of moved or renamed tab: " << tab.GetHeader().c_str();
            QNetworkRequest request = MakeOAuthStashRequest(realm_, league_, tab.get_tab_uniq_id());
            QueueRequest(kOAuthGetStashEndpoint, request, tab);
        };
    };

    int tabs_requested = 0;

    // Queue stash tab requests.
    for (auto& tab : stashes) {

        const auto x = Util::RapidjsonSerialize(tab);

        // Get the name of the stash tab.
        if (!HasString(tab, "name")) {
            QLOG_ERROR() << "The stash tab does not have a name";
            continue;
        };
        const std::string tab_name = tab["name"].GetString();

        // Skip hidden tabs.
        if (HasBool(tab, "hidden") && tab["hidden"].GetBool()) {
            QLOG_DEBUG() << "The stash tab is hidden:" << tab_name;
            continue;
        };

        // Get the unique id.
        if (!HasString(tab, "id")) {
            QLOG_ERROR() << "The stash tab does not have a unique id:" << tab_name;
            continue;
        }
        std::string tab_id = tab["id"].GetString();
        if (tab_id.size() > 10) {
            // The unique id for stash tabs returned from the legacy API
            // need to be trimmed to 10 characters.
            QLOG_DEBUG() << "Trimming tab unique id:" << tab_name;
            tab_id = tab_id.substr(0, 10);
        };

        // Skip tabs that are in the index; they are not being refreshed.
        if (tab_id_index_.count(tab_id) > 0) {
            QLOG_TRACE() << "ItemsManagerWorker::OnOAuthStashListReceived() skipping tab:" << tab_name;
            continue;
        };

        // Get the index of this stash tab.
        if (!HasInt(tab, "index")) {
            QLOG_ERROR() << "The stash tab does not have an index:" << tab_name;
            continue;
        };
        const int tab_index = tab["index"].GetInt();

        // Get the type of this stash tab.
        if (!HasString(tab, "type")) {
            QLOG_ERROR() << "The stash tab does not have a type:" << tab_name;
            continue;
        };
        const std::string tab_type = tab["type"].GetString();

        ++tabs_requested;

        // Get the stash tab color.
        int r = 0, g = 0, b = 0;
        Util::GetTabColor(tab, r, g, b);

        // Create and save the tab location object.
        ItemLocation location(tab_index, tab_id, tab_name, ItemLocationType::STASH, tab_type, r, g, b, tab, doc.GetAllocator());
        tabs_.push_back(location);
        tab_id_index_.insert(tab_id);

        // Submit a request for this tab.
        QNetworkRequest request = MakeOAuthStashRequest(realm_, league_, location.get_tab_uniq_id());
        QueueRequest(kOAuthGetStashEndpoint, request, location);
    };
    QLOG_DEBUG() << "Requesting" << tabs_requested << "out of" << stashes.Size() << "stash tabs";

    has_stash_list_ = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!need_character_list_ || has_character_list_) {
        QLOG_TRACE() << "ItemsManagerWorker::OnOAuthStashListReceived() fetching items";
        FetchItems();
    };
}

void ItemsManagerWorker::OnOAuthCharacterListReceived(QNetworkReply* reply) {
    QLOG_TRACE() << "ItemsManagerWorker::OnOAuthCharacterListReceived() entered";

    auto sender = qobject_cast<RateLimit::RateLimitedReply*>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    QLOG_TRACE() << "OAuth character list received";
    if (reply->error() != QNetworkReply::NoError) {
        QLOG_WARN() << "Aborting update because there was an error fetching the character list:" << reply->errorString();
        updating_ = false;
        return;
    };
    const QByteArray bytes = reply->readAll();

    rapidjson::Document doc;
    doc.Parse(bytes);
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Error parsing the character list:" << rapidjson::GetParseError_En(doc.GetParseError());
        updating_ = false;
        return;
    };

    if (!doc.IsObject() || !HasArray(doc, "characters")) {
        QLOG_ERROR() << "The characters list is invalid:" << bytes;
        updating_ = false;
        return;
    };

    const auto& characters = doc["characters"].GetArray();
    int requested_character_count = 0;
    for (auto& character : characters) {
        if (!HasString(character, "name")) {
            QLOG_ERROR() << "The character does not have a name. The reply may be invalid:" << bytes;
            continue;
        };
        const std::string name = character["name"].GetString();
        if (!HasString(character, "realm")) {
            QLOG_ERROR() << "The character does not have a realm. The reply may be invalid:" << bytes;
        };
        const std::string realm = character["realm"].GetString();
        if (!HasString(character, "league")) {
            QLOG_ERROR() << "Malformed character entry for" << name << ": the reply may be invalid: " << bytes;
            continue;
        };
        const std::string league = character["league"].GetString();
        if (realm != realm_) {
            QLOG_DEBUG() << "Skipping" << name << "because this character is not in realm" << realm_;
            continue;
        };
        if (league != league_) {
            QLOG_DEBUG() << "Skipping" << name << "because this character is not in leage" << league_;
            continue;
        };
        if (tab_id_index_.count(name) > 0) {
            QLOG_DEBUG() << "Skipping" << name << "because this item is not being refreshed.";
            continue;
        };
        const int tab_count = static_cast<int>(tabs_.size());
        ItemLocation location(tab_count, "", name, ItemLocationType::CHARACTER, "", 0, 0, 0, character, doc.GetAllocator());
        tabs_.push_back(location);
        ++requested_character_count;

        QNetworkRequest request = MakeOAuthCharacterRequest(realm_, name);
        QueueRequest(kOAuthGetCharacterEndpoint, request, location);
    }
    QLOG_DEBUG() << "There are" << requested_character_count << "characters to update in" << league_.c_str();

    has_character_list_ = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!need_stash_list_ || has_stash_list_) {
        QLOG_TRACE() << "ItemsManagerWorker::OnOAuthCharacterListReceived() fetching items";
        FetchItems();
    };
}

void ItemsManagerWorker::OnOAuthStashReceived(QNetworkReply* reply, ItemLocation location) {
    QLOG_TRACE() << "ItemsManagerWorker::OnOAuthStashReceived() entered";

    auto sender = qobject_cast<RateLimit::RateLimitedReply*>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    QLOG_TRACE() << "OAuth stash recieved";
    if (reply->error() != QNetworkReply::NoError) {
        QLOG_WARN() << "Aborting update because there was an error fetching the stash:" << reply->errorString();
        updating_ = false;
        return;
    };
    const QByteArray bytes = reply->readAll();

    rapidjson::Document doc;
    doc.Parse(bytes);
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Error parsing the stash:" << rapidjson::GetParseError_En(doc.GetParseError());
        updating_ = false;
        return;
    };

    if (!HasObject(doc, "stash")) {
        QLOG_ERROR() << "Error parsing the stash: 'stash' field was missing.";
        updating_ = false;
        return;
    };
    auto& stash = doc["stash"];

    if (HasArray(stash, "items")) {
        auto& items = stash["items"];
        if (items.GetArray().Size() > 0) {
            ParseItems(items, location, doc.GetAllocator());
        } else {
            QLOG_DEBUG() << "Stash 'items' does not contain any items:" << location.GetHeader();
        };
    } else {
        QLOG_DEBUG() << "Stash does not have an 'items' array:" << location.GetHeader();
    };

    ++stashes_received_;
    SendStatusUpdate();

    if ((stashes_received_ == stashes_needed_) && (characters_received_ == characters_needed_) && !cancel_update_) {
        QLOG_TRACE() << "ItemsManagerWorker::OnOAuthStashReceived() finishing update";
        FinishUpdate();
    };
}

void ItemsManagerWorker::OnOAuthCharacterReceived(QNetworkReply* reply, ItemLocation location) {
    QLOG_TRACE() << "ItemsManagerWorker::OnOAuthCharacterReceived() entered";

    auto sender = qobject_cast<RateLimit::RateLimitedReply*>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    QLOG_TRACE() << "OAuth character recieved";
    if (reply->error() != QNetworkReply::NoError) {
        QLOG_WARN() << "Aborting update because there was an error fetching the character:" << reply->errorString();
        updating_ = false;
        return;
    };
    const QByteArray bytes = reply->readAll();

    rapidjson::Document doc;
    doc.Parse(bytes);
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Error parsing the character:" << rapidjson::GetParseError_En(doc.GetParseError());
        updating_ = false;
        return;
    };

    if (!HasObject(doc, "character")) {
        QLOG_ERROR() << "The reply to a character request did not contain a character object.";
        updating_ = false;
        return;
    };

    auto character = doc["character"].GetObj();
    for (const auto& field : CHARACTER_ITEM_FIELDS) {
        if (character.HasMember(field)) {
            ParseItems(character[field], location, doc.GetAllocator());
        };
    };

    ++characters_received_;
    SendStatusUpdate();

    if ((stashes_received_ == stashes_needed_) && (characters_received_ == characters_needed_) && !cancel_update_) {
        QLOG_TRACE() << "ItemsManagerWorker::OnOAuthCharacterReceived() finishing update";
        FinishUpdate();
    };
}

void ItemsManagerWorker::OnLegacyMainPageReceived() {
    QLOG_TRACE() << "ItemsManagerWorker::OnLegacyMainPageReceived() entered";

    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    if (reply->error()) {
        QLOG_WARN() << "Couldn't fetch main page: " << reply->url().toDisplayString() << " due to error: " << reply->errorString();
    } else {
        std::string page(reply->readAll().constData());
        selected_character_ = Util::FindTextBetween(page, "C({\"name\":\"", "\",\"class");
        selected_character_ = Util::ConvertAsciiToUtf(selected_character_);
        if (selected_character_.empty()) {
            QLOG_WARN() << "Couldn't extract currently selected character name from GGG homepage (maintenence?) Text was: " << page.c_str();
        };
    };
    reply->deleteLater();

    QNetworkRequest characters_request = QNetworkRequest(QUrl(kGetCharactersUrl));
    QLOG_TRACE() << "ItemsManagerWorker::OnLegacyMainPageReceived() requesting characters:" << characters_request.url().toString();
    auto submit = rate_limiter_.Submit(kGetCharactersUrl, characters_request);
    connect(submit, &RateLimit::RateLimitedReply::complete, this, &ItemsManagerWorker::OnLegacyCharacterListReceived);
}

void ItemsManagerWorker::OnLegacyCharacterListReceived(QNetworkReply* reply) {
    QLOG_TRACE() << "ItemsManagerWorker::OnLegacyCharacterListReceived() entered";

    auto sender = qobject_cast<RateLimit::RateLimitedReply*>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();
    if (reply->error()) {
        QLOG_WARN() << "Couldn't fetch character list: " << reply->url().toDisplayString()
            << " due to error: " << reply->errorString() << " Aborting update.";
        updating_ = false;
        reply->deleteLater();
        return;
    };

    const QByteArray bytes = reply->readAll();
    rapidjson::Document doc;
    doc.Parse(bytes.constData());
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

    int requested_character_count = 0;
    for (auto& character : doc) {
        if (!HasString(character, "name")) {
            QLOG_ERROR() << "The legacy character does not have a name. The reply may be invalid:" << bytes;
            continue;
        };
        const std::string name = character["name"].GetString();
        if (!HasString(character, "league")) {
            QLOG_ERROR() << "Malformed legacy character entry for" << name << ": the reply may be invalid: " << bytes;
            continue;
        };
        const std::string league = character["league"].GetString();
        if (league != league_) {
            QLOG_DEBUG() << "Skipping" << name << "because this character is not in" << league_;
            continue;
        };
        if (tab_id_index_.count(name) > 0) {
            QLOG_DEBUG() << "Skipping legacy character" << name << "because this item is not being refreshed.";
            continue;
        };
        const int tab_count = static_cast<int>(tabs_.size());
        ItemLocation location(tab_count, "", name, ItemLocationType::CHARACTER, "", 0, 0, 0, character, doc.GetAllocator());
        tabs_.push_back(location);
        ++requested_character_count;

        //Queue request for items on character in character's stash
        QueueRequest(kCharacterItemsUrl, MakeLegacyCharacterRequest(name), location);

        //Queue request for jewels in character's passive tree
        QueueRequest(kCharacterSocketedJewels, MakeLegacyPassivesRequest(name), location);
    }
    QLOG_DEBUG() << "There are" << requested_character_count << "characters to update in" << league_.c_str();

    has_character_list_ = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!need_stash_list_ || has_stash_list_) {
        QLOG_TRACE() << "ItemsManagerWorker::OnLegacyCharacterListReceived() fetching items";
        FetchItems();
    };
}

QNetworkRequest ItemsManagerWorker::MakeLegacyTabRequest(int tab_index, bool tabs) {
    QLOG_TRACE() << "ItemsManagerWorker::MakeLegacyTabRequest() entered";

    if (tab_index < 0) {
        QLOG_ERROR() << "MakeLegacyTabRequest: invalid tab_index =" << tab_index;
    };
    QUrlQuery query;
    query.addQueryItem("league", QString::fromUtf8(league_));
    query.addQueryItem("tabs", tabs ? "1" : "0");
    query.addQueryItem("tabIndex", std::to_string(tab_index).c_str());

    QUrl url(kStashItemsUrl);
    url.setQuery(query);
    return QNetworkRequest(url);
}

QNetworkRequest ItemsManagerWorker::MakeLegacyCharacterRequest(const std::string& name) {
    QLOG_TRACE() << "ItemsManagerWorker::MakeLegacyCharacterRequest() entered";

    if (name.empty()) {
        QLOG_ERROR() << "MakeLegacyCharacterRequest: invalid name = '" + name + "'";
    };
    QUrlQuery query;
    query.addQueryItem("character", QString::fromUtf8(name));

    QUrl url(kCharacterItemsUrl);
    url.setQuery(query);
    return QNetworkRequest(url);
}

QNetworkRequest ItemsManagerWorker::MakeLegacyPassivesRequest(const std::string& name) {
    QLOG_TRACE() << "ItemsManagerWorker::MakeLegacyPassivesRequest() entered";

    if (name.empty()) {
        QLOG_ERROR() << "MakeLegacyPassivesRequest: invalid name = '" + name + "'";
    };
    QUrlQuery query;
    query.addQueryItem("character", QString::fromUtf8(name));

    QUrl url(kCharacterSocketedJewels);
    url.setQuery(query);
    return QNetworkRequest(url);
}

void ItemsManagerWorker::QueueRequest(const QString& endpoint, const QNetworkRequest& request, const ItemLocation& location) {
    QLOG_TRACE() << "ItemsManagerWorker::QueueRequest() entered";

    QLOG_DEBUG() << "Queued (" << queue_id_ + 1 << ") -- " << QString::fromUtf8(location.GetHeader());
    ItemsRequest items_request;
    items_request.endpoint = endpoint;
    items_request.network_request = request;
    items_request.id = queue_id_++;
    items_request.location = location;
    queue_.push(items_request);
}

void ItemsManagerWorker::FetchItems() {
    QLOG_TRACE() << "ItemsManagerWorker::FetchItems() entered";

    stashes_needed_ = 0;
    stashes_received_ = 0;

    characters_needed_ = 0;
    characters_received_ = 0;

    std::string tab_titles;
    while (!queue_.empty()) {

        // Take the next request out of the queue.
        ItemsRequest request = queue_.front();
        queue_.pop();

        // Setup the right callback for this endpoint.
        const ItemLocation location = request.location;
        const QString endpoint = request.endpoint;
        std::function<void(QNetworkReply*)> callback;

        if (endpoint == kStashItemsUrl) {
            callback = [=](QNetworkReply* reply) { OnLegacyTabReceived(reply, location); };
            ++stashes_needed_;
        } else if ((endpoint == kCharacterItemsUrl) || (endpoint == kCharacterSocketedJewels)) {
            callback = [=](QNetworkReply* reply) { OnLegacyTabReceived(reply, location); };
            ++characters_needed_;
        } else if (endpoint == kOAuthGetStashEndpoint) {
            callback = [=](QNetworkReply* reply) { OnOAuthStashReceived(reply, location); };
            ++stashes_needed_;
        } else if (endpoint == kOAuthGetCharacterEndpoint) {
            callback = [=](QNetworkReply* reply) { OnOAuthCharacterReceived(reply, location); };
            ++characters_needed_;
        } else {
            QLOG_ERROR() << "FetchItems(): invalid endpoint:" << request.endpoint;
        };

        // Pass the request to the rate limiter.
        auto submit = rate_limiter_.Submit(request.endpoint, request.network_request);
        connect(submit, &RateLimit::RateLimitedReply::complete, this, callback);

        // Keep track of the tabs requested.
        tab_titles += request.location.GetHeader() + " ";
    };

    SendStatusUpdate();

    QLOG_DEBUG() << "Requested" << stashes_needed_ << "stashes and" << characters_needed_ << "characters.";
    QLOG_DEBUG() << "Tab titles:" << tab_titles;
}

void ItemsManagerWorker::OnFirstLegacyTabReceived(QNetworkReply* reply) {
    QLOG_TRACE() << "ItemsManagerWorker::OnFirstLegacyTabReceived() entered";

    auto sender = qobject_cast<RateLimit::RateLimitedReply*>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    QLOG_TRACE() << "First legacy tab received.";
    rapidjson::Document doc;
    QByteArray bytes = reply->readAll();
    doc.Parse(bytes.constData());

    if (!doc.IsObject()) {
        QLOG_ERROR() << "Can't even fetch first legacy tab. Failed to update items.";
        updating_ = false;
        return;
    };

    if (doc.HasMember("error")) {
        QLOG_ERROR() << "Aborting legacy update since first fetch failed due to 'error':" << Util::RapidjsonSerialize(doc["error"]).c_str();
        updating_ = false;
        return;
    };

    if (!HasArray(doc, "tabs") || doc["tabs"].Size() == 0) {
        QLOG_ERROR() << "There are no legacy tabs, this should not happen, bailing out.";
        updating_ = false;
        return;
    };

    auto& tabs = doc["tabs"];

    QLOG_DEBUG() << "Received legacy tabs list, there are" << tabs.Size() << "tabs";
    tabs_signature_ = CreateTabsSignatureVector(tabs);

    // Remember old tab headers before clearing tabs
    std::set<std::string> old_tab_headers;
    for (auto const& tab : tabs_) {
        old_tab_headers.insert(tab.GetHeader());
    };

    // Force refreshes for any stash tabs that were moved or renamed.
    for (auto const& tab : tabs_) {
        if (!old_tab_headers.count(tab.GetHeader())) {
            QLOG_DEBUG() << "Forcing refresh of moved or renamed tab: " << tab.GetHeader().c_str();
            QueueRequest(kStashItemsUrl, MakeLegacyTabRequest(tab.get_tab_id(), true), tab);
        };
    };

    // Queue stash tab requests.
    for (auto& tab : tabs) {

        if (!HasString(tab, "n")) {
            QLOG_ERROR() << "Legacy tab does not have name";
            continue;
        };
        const std::string label = tab["n"].GetString();

        if (!HasInt(tab, "i")) {
            QLOG_ERROR() << "Legacy tab does not have an index:" << label;
            continue;
        };
        const int index = tab["i"].GetInt();

        // Skip hidden tabs.
        if (HasBool(tabs[index], "hidden") && tabs[index]["hidden"].GetBool()) {
            QLOG_DEBUG() << "The legacy tab is hidden:" << label;
            continue;
        };

        // Skip tabs that are in the index; they are not being refreshed.
        if (!HasString(tab, "id")) {
            QLOG_ERROR() << "The legacy tab does not have a unique id:" << label;
            continue;
        };
        std::string tab_id = tab["id"].GetString();
        if (tab_id.size() > 10) {
            // The unique id for stash tabs returned from the legacy API
            // need to be trimmed to 10 characters.
            QLOG_DEBUG() << "Trimming legacy tab unique id:" << label;
            tab_id = tab_id.substr(0, 10);
        };
        if (tab_id_index_.count(tab_id) > 0) {
            continue;
        };

        // Get the type of this stash tab.
        if (!HasString(tab, "type")) {
            QLOG_ERROR() << "The stash tab does not have a type:" << label;
            continue;
        };
        const std::string tab_type = tab["type"].GetString();

        // Get the stash tab color;
        int r = 0, g = 0, b = 0;
        Util::GetTabColor(tab, r, g, b);

        // Create and save the tab location object.
        ItemLocation location(index, tab_id, label, ItemLocationType::STASH, tab_type, r, g, b, tab, doc.GetAllocator());
        tabs_.push_back(location);
        tab_id_index_.insert(tab_id);

        // Submit a request for this tab.
        QueueRequest(kStashItemsUrl, MakeLegacyTabRequest(location.get_tab_id(), true), location);
    };

    has_stash_list_ = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!need_character_list_ || has_character_list_) {
        QLOG_TRACE() << "ItemsManagerWorker::OnFirstLegacyTabReceived() fetching items";
        FetchItems();
    };
}

void ItemsManagerWorker::SendStatusUpdate() {
    QLOG_TRACE() << "ItemsManagerWorker::SendStatusUpdate() entered";

    if (cancel_update_) {
        emit StatusUpdate(ProgramState::Ready, "Update cancelled.");
    } else {
        QString message;
        if ((stashes_needed_ > 0) && (characters_needed_ > 0)) {
            message = QString("Receieved %1/%2 stash tabs and %3/%4 character locations").arg(
                QString::number(stashes_received_),
                QString::number(stashes_needed_),
                QString::number(characters_received_),
                QString::number(characters_needed_));
        } else if (stashes_needed_ > 0) {
            message = QString("Received %1/%2 stash tabs").arg(
                QString::number(stashes_received_),
                QString::number(stashes_needed_));
        } else if (characters_needed_ > 0) {
            message = QString("Received %1/%2 character locations").arg(
                QString::number(characters_received_),
                QString::number(characters_needed_));
        } else {
            message = "Received nothing; needed nothing.";
        };
        emit StatusUpdate(ProgramState::Busy, message);
    };
}

void ItemsManagerWorker::ParseItems(rapidjson::Value& value, ItemLocation base_location, rapidjson_allocator& alloc) {
    QLOG_TRACE() << "ItemsManagerWorker::ParseItems() entered";

    for (auto& item : value) {
        // Make sure location data from the item like x and y is brought over to the location object.
        base_location.FromItemJson(item);
        base_location.ToItemJson(&item, alloc);
        items_.push_back(std::make_shared<Item>(item, base_location));
        if (HasArray(item, "socketedItems")) {
            base_location.set_socketed(true);
            ParseItems(item["socketedItems"], base_location, alloc);
            base_location.set_socketed(false);
        };
    };
}

void ItemsManagerWorker::OnLegacyTabReceived(QNetworkReply* reply, ItemLocation location) {
    QLOG_TRACE() << "ItemsManagerWorker::OnLegacyTabReceived() entered";

    auto sender = qobject_cast<RateLimit::RateLimitedReply*>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    QLOG_DEBUG() << "Legacy tab receivevd:" << location.GetHeader();
    rapidjson::Document doc;
    QByteArray bytes = reply->readAll();
    doc.Parse(bytes.constData());

    bool error = false;
    if (!doc.IsObject()) {
        QLOG_ERROR() << "Legacy tab is non-object response for:" << location.GetHeader();
        error = true;
    } else if (doc.HasMember("error")) {
        // this can happen if user is browsing stash in background and we can't know about it
        QLOG_ERROR() << "Legacy tab has 'error' instead of stash tab contents for: " << location.GetHeader();
        QLOG_ERROR() << "The error is:" << Util::RapidjsonSerialize(doc["error"]).c_str();
        error = true;
    };

    // We index expected tabs and their locations as part of the first fetch.  It's possible for users
    // to move or rename tabs during the update which will result in the item data being out-of-sync with
    // expected index/tab name map.  We need to detect this case and abort the update.
    if (!cancel_update_ && !error && (location.get_type() == ItemLocationType::STASH)) {
        cancel_update_ = TabsChanged(doc, reply, location);
    };

    switch (location.get_type()) {
    case ItemLocationType::STASH: ++stashes_received_; break;
    case ItemLocationType::CHARACTER: ++characters_received_; break;
    default:
        QLOG_ERROR() << "OnLegacyTabReceived: invalid location type" << location.get_type();
    };
    SendStatusUpdate();

    if ((stashes_received_ == stashes_needed_) && (characters_received_ == characters_needed_)) {
        if (cancel_update_) {
            QLOG_TRACE() << "ItemsManagerWorker::OnLegacyTabReceived() cancelling update";
            updating_ = false;
        };
    };

    if (error) {
        return;
    };

    if (!HasArray(doc, "items")) {
        QLOG_DEBUG() << "Legacy stash does not have an 'items' array:" << location.GetHeader();
    } else {
        auto& items = doc["items"];
        if (items.Size() == 0) {
            QLOG_DEBUG() << "Legacy stash 'items' is empty:" << location.GetHeader();
        } else {
            ParseItems(items, location, doc.GetAllocator());
        };
    };

    if ((stashes_received_ == stashes_needed_) && (characters_received_ == characters_needed_) && !cancel_update_) {
        QLOG_TRACE() << "ItemsManagerWorker::OnLegacyTabReceived() finishing update";
        FinishUpdate();
        PreserveSelectedCharacter();
    };
}

bool ItemsManagerWorker::TabsChanged(rapidjson::Document& doc, QNetworkReply* network_reply, ItemLocation& location) {
    QLOG_TRACE() << "ItemsManagerWorker::TabsChanged() entered";

    if (!doc.HasMember("tabs") || doc["tabs"].Size() == 0) {
        QLOG_ERROR() << "Full tab information missing from stash tab fetch.  Cancelling update. Full fetch URL: "
            << network_reply->request().url().toDisplayString();
        return true;
    };

    auto tabs_signature_current = CreateTabsSignatureVector(doc["tabs"]);
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
    QLOG_TRACE() << "ItemsManagerWorker::FinishUpdate() entered";

    // It's possible that we receive character vs stash tabs out of order, or users
    // move items around in a tab and we get them in a different order. For
    // consistency we want to present the tab data in a deterministic way to the rest
    // of the application.  Especially so we don't try to update shop when nothing actually
    // changed.  So sort items_ here before emitting and then generate
    // item list as strings.

    QString message;
    if ((stashes_received_ > 0) && (characters_received_ > 0)) {
        message = QString("Received %1 stash tabs and %2 character locations").arg(
            QString::number(stashes_received_),
            QString::number(characters_received_));
    } else if (stashes_received_ > 0) {
        message = QString("Received %1 stash tabs").arg(
            QString::number(stashes_received_));
    } else if (characters_received_ > 0) {
        message = QString("Received %1 character locations").arg(
            QString::number(characters_received_));
    } else {
        message = "Received nothing.";
    }

    emit StatusUpdate(ProgramState::Ready, message);

    // Sort tabs.
    std::sort(begin(tabs_), end(tabs_));

    // Sort items.
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
        datastore_.SetTabs(location_type, tabs);
    };

    // Save items by location.
    for (auto const& pair : itemsPerLoc) {
        const auto& location = pair.first;
        const auto& items = pair.second;
        datastore_.SetItems(location, items);
    };

    // Let everyone know the update is done.
    QLOG_TRACE() << "ItemsManagerWorker::FinishUpdate() emitting ItemsRefreshed";
    emit ItemsRefreshed(items_, tabs_, false);

    updating_ = false;
    QLOG_DEBUG() << "Update finished.";
}

void ItemsManagerWorker::PreserveSelectedCharacter() {
    QLOG_TRACE() << "ItemsManagerWorker::PreserveSelectedCharacter() entered";

    if (selected_character_.empty()) {
        QLOG_DEBUG() << "Cannot preserve selected character: no character selected";
        return;
    };
    QLOG_DEBUG() << "Preserving selected character:" << QString::fromUtf8(selected_character_);
    // The act of making this request sets the active character.
    // We don't need to to anything with the reply.
    QNetworkRequest character_request = MakeLegacyCharacterRequest(selected_character_);
    auto submit = rate_limiter_.Submit(kCharacterItemsUrl, character_request);
    connect(submit, &RateLimit::RateLimitedReply::complete, this,
        [=](QNetworkReply* reply) {
            reply->deleteLater();
        });
}

ItemsManagerWorker::TabsSignatureVector ItemsManagerWorker::CreateTabsSignatureVector(const rapidjson::Value& tabs) {
    QLOG_TRACE() << "ItemsManagerWorker::CreateTabsSignatureVector() entered";

    const bool legacy = tabs[0].HasMember("n");
    const char* n = legacy ? "n" : "name";
    const char* id = "id";
    TabsSignatureVector signature;
    for (auto& tab : tabs) {
        std::string name = HasString(tab, n) ? tab[n].GetString() : "UNKNOWN_NAME";
        std::string uid = HasString(tab, id) ? tab[id].GetString() : "UNKNOWN_ID";
        if (!tab.HasMember("class")) {
            // The stash tab unique id is only ten characters, but legacy tabs
            // return a much longer value. GGG confirmed that taking the first
            // ten characters is the right thing to do.
            if (uid.size() > 10) {
                uid = uid.substr(0, 10);
            };
        };
        signature.emplace_back(name, uid);
    };
    return signature;
}

