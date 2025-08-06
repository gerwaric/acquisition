/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include <rapidjson/document.h>
#include <rapidjson/error/en.h>

#include <datastore/datastore.h>
#include <ratelimit/ratelimit.h>
#include <ratelimit/ratelimitedreply.h>
#include <ratelimit/ratelimiter.h>
#include <ui/mainwindow.h>
#include <util/oauthmanager.h>
#include <util/rapidjson_util.h>
#include <util/repoe.h>
#include <util/spdlog_qt.h>
#include <util/util.h>

#include "application.h"
#include "buyoutmanager.h"
#include "itemcategories.h"
#include "modlist.h"
#include "network_info.h"

using rapidjson::HasArray;

constexpr const char *kStashItemsUrl
    = "https://www.pathofexile.com/character-window/get-stash-items";
constexpr const char *kCharacterItemsUrl = "https://www.pathofexile.com/character-window/get-items";
constexpr const char *kGetCharactersUrl
    = "https://www.pathofexile.com/character-window/get-characters";
constexpr const char *kMainPage = "https://www.pathofexile.com/";
//While the page does say "get passive skills", it seems to only send socketed jewels
constexpr const char *kCharacterSocketedJewels
    = "https://www.pathofexile.com/character-window/get-passive-skills";

constexpr const char *kPOE_trade_stats = "https://www.pathofexile.com/api/trade/data/stats";

constexpr const char *kOauthListStashesEndpoint = "List Stashes";
constexpr const char *kOAuthListStashesUrl = "https://api.pathofexile.com/stash";

constexpr const char *kOAuthListCharactersEndpoint = "List Characters";
constexpr const char *kOAuthListCharactersUrl = "https://api.pathofexile.com/character";

constexpr const char *kOAuthGetStashEndpoint = "Get Stash";
constexpr const char *kOAuthGetStashUrl = "https://api.pathofexile.com/stash";

constexpr const char *kOAuthGetCharacterEndpoint = "Get Character";
constexpr const char *kOAuthGetCharacterUrl = "https://api.pathofexile.com/character";

constexpr std::array CHARACTER_ITEM_FIELDS = {"equipment", "inventory", "rucksack", "jewels"};

ItemsManagerWorker::ItemsManagerWorker(QSettings &settings,
                                       QNetworkAccessManager &network_manager,
                                       RePoE &repoe,
                                       BuyoutManager &buyout_manager,
                                       DataStore &datastore,
                                       RateLimiter &rate_limiter,
                                       POE_API mode)
    : m_settings(settings)
    , m_network_manager(network_manager)
    , m_repoe(repoe)
    , m_datastore(datastore)
    , m_buyout_manager(buyout_manager)
    , m_rate_limiter(rate_limiter)
    , m_mode(mode)
    , m_test_mode(false)
    , m_stashes_needed(0)
    , m_stashes_received(0)
    , m_characters_needed(0)
    , m_characters_received(0)
    , m_initialized(false)
    , m_updating(false)
    , m_cancel_update(false)
    , m_updateRequest(false)
    , m_type(TabSelection::Checked)
    , m_queue_id(-1)
    , m_first_stash_request_index(-1)
    , m_need_stash_list(false)
    , m_need_character_list(false)
    , m_has_stash_list(false)
    , m_has_character_list(false)
    , m_update_tab_contents(true)
{
    spdlog::trace("ItemsManagerWorker::ItemsManagerWorker() entered");
}

void ItemsManagerWorker::UpdateRequest(TabSelection type, const std::vector<ItemLocation> &locations)
{
    spdlog::trace("ItemsManagerWorker::UpdateRequest() entered");
    m_updateRequest = true;
    m_type = type;
    m_locations = locations;
}

void ItemsManagerWorker::Init()
{
    spdlog::trace("ItemsManagerWorker::Init() entered");
    if (m_updating) {
        spdlog::warn("ItemsManagerWorker::Init() called while updating, skipping Mod List Update");
        return;
    }

    m_realm = m_settings.value("realm").toString();
    m_league = m_settings.value("league").toString();
    m_account = m_settings.value("account").toString();
    m_updating = true;
    spdlog::trace("ItemsManagerWorker::Init() league = {}", m_league);
    spdlog::trace("ItemsManagerWorker::Init() account = {}", m_account);

    if (m_repoe.IsInitialized()) {
        spdlog::debug("RePoE data is available.");
        OnRePoEReady();
    } else {
        spdlog::debug("Waiting for RePoE data.");
        connect(&m_repoe, &RePoE::finished, this, &ItemsManagerWorker::OnRePoEReady);
    }
}

void ItemsManagerWorker::OnRePoEReady()
{
    spdlog::trace("ItemsManagerWorker::OnRePoEReady() entered");
    // Create a separate thread to load the items, which allows the UI to
    // update the status bar while items are being parsed. This operation
    // can take tens of seconds or longer depending on the nubmer of tabs
    // and items.
    QThread *parser = QThread::create([=]() { ParseItemMods(); });
    parser->start();
}

void ItemsManagerWorker::ParseItemMods()
{
    spdlog::trace("ItemsManagerWorker::ParseItemMods() entered");
    m_tabs.clear();
    m_tabs_signature.clear();
    m_tab_id_index.clear();

    // Get cached tabs (item tabs not search tabs)
    for (ItemLocationType type : {ItemLocationType::STASH, ItemLocationType::CHARACTER}) {
        spdlog::trace("ItemsManagerWorker::ParseItemMods() getting cached tabs for {}", type);
        Locations tabs = m_datastore.GetTabs(type);
        m_tabs.reserve(m_tabs.size() + tabs.size());
        for (const auto &tab : tabs) {
            m_tabs.push_back(tab);
        }
    }

    // Save location ids.
    spdlog::trace("ItemsManagerWorker::ParseItemMods() saving location ids");
    for (const auto &tab : m_tabs) {
        m_tab_id_index.emplace(tab.get_tab_uniq_id());
    }

    // Build the signature vector.
    spdlog::trace("ItemsManangerWorker::ParseItemMods() building tabs signature");
    m_tabs_signature.reserve(m_tabs.size());
    for (const auto &tab : m_tabs) {
        const QString tab_name = tab.get_tab_label();
        const QString tab_id = QString::number(tab.get_tab_id());
        m_tabs_signature.emplace_back(tab_name, tab_id);
    }

    // Get cached items
    spdlog::trace("ItemsManagerWorker::ParseItemMods() getting cached items");
    for (int i = 0; i < m_tabs.size(); i++) {
        auto &tab = m_tabs[i];
        Items tab_items = m_datastore.GetItems(tab);
        m_items.reserve(m_items.size() + tab_items.size());
        spdlog::trace("ItemsManagerWorker::ParseItemMods() got {} items from {}",
                      tab_items.size(),
                      tab.GetHeader());
        for (const auto &tab_item : tab_items) {
            m_items.push_back(tab_item);
        }
        emit StatusUpdate(ProgramState::Initializing,
                          QString("Parsing items in %1/%2 tabs")
                              .arg(QString::number(i + 1), QString::number(m_tabs.size())));
    }
    emit StatusUpdate(ProgramState::Ready,
                      QString("Parsed items from %1 tabs").arg(QString::number(m_tabs.size())));

    m_initialized = true;
    m_updating = false;

    // let ItemManager know that the retrieval of cached items/tabs has been completed (calls ItemsManager::OnItemsRefreshed method)
    spdlog::trace("ItemsManagerWorker::ParseItemMods() emitting ItemsRefreshed signal");
    emit ItemsRefreshed(m_items, m_tabs, true);

    if (m_updateRequest) {
        spdlog::trace("ItemsManagerWorker::ParseItemMods() triggering requested update");
        m_updateRequest = false;
        Update(m_type, m_locations);
    }
}

void ItemsManagerWorker::Update(TabSelection type, const std::vector<ItemLocation> &locations)
{
    if (m_updating) {
        spdlog::warn("ItemsManagerWorker: update called while updating");
        return;
    }

    spdlog::debug("ItemsManagerWorker: updating {} tabs", type);
    if (spdlog::should_log(spdlog::level::trace)) {
        QStringList character_names;
        QStringList stash_names;
        for (const auto &location : locations) {
            switch (location.get_type()) {
            case ItemLocationType::CHARACTER:
                character_names.append(location.get_character());
                break;
            case ItemLocationType::STASH:
                stash_names.append(location.get_tab_label());
                break;
            }
        }
        spdlog::trace("ItemsManagerWorker: characters = {}", character_names.join(", "));
        spdlog::trace("ItemsManagerWorker: stashes = {}", stash_names.join(", "));
    }
    m_updating = true;
    m_cancel_update = false;
    m_update_tab_contents = (type != TabSelection::TabsOnly);

    // remove all pending requests
    m_queue = {};
    m_queue_id = 0;

    m_selected_character.clear();

    m_need_stash_list = false;
    m_need_character_list = false;

    m_first_stash_request_index = -1;
    m_first_character_request_name.clear();

    if (type == TabSelection::TabsOnly) {
        spdlog::debug("ItemsManagerWorker: updating stash and character lists.");
        m_tabs.clear();
        m_tab_id_index.clear();
        m_need_stash_list = true;
        m_need_character_list = true;
        m_first_stash_request_index = 0;
    } else if (type == TabSelection::All) {
        spdlog::debug("ItemsManagerWorker: updating all tabs and items.");
        m_tabs.clear();
        m_tab_id_index.clear();
        m_items.clear();
        m_first_stash_request_index = 0;
        m_need_stash_list = true;
        m_need_character_list = true;
    } else {
        // Build a list of tabs to update.
        std::set<QString> tabs_to_update = {};
        switch (type) {
        case TabSelection::Checked:
            // Use the buyout manager to determine which tabs are check.
            spdlog::trace("ItemsManagerWorker: updating checked tabs.");
            for (auto const &tab : m_tabs) {
                if ((tab.IsValid()) && (m_buyout_manager.GetRefreshChecked(tab) == true)) {
                    tabs_to_update.emplace(tab.get_tab_uniq_id());
                }
            }
            break;
        case TabSelection::Selected:
            // Use the function argument to determine which tabs were selected.
            spdlog::trace("ItemsManagerWorker::Update() updating selected tabs");
            for (auto const &tab : locations) {
                if (tab.IsValid()) {
                    tabs_to_update.emplace(tab.get_tab_uniq_id());
                }
            }
            break;
        case TabSelection::All:
        case TabSelection::TabsOnly:
            // These cases are handled by the enclosing if.
            spdlog::error("ItemsManagerWorker: this code should have been unreachable??");
            break;
        }
        // Remove the tabs that will be updated, and all the items linked to those tabs.
        spdlog::debug("Updating {} tabs.", tabs_to_update.size());
        RemoveUpdatingTabs(tabs_to_update);
        RemoveUpdatingItems(tabs_to_update);
        m_need_stash_list = (m_first_stash_request_index >= 0);
        m_need_character_list = !m_first_character_request_name.isEmpty();
    }

    m_has_stash_list = false;
    m_has_character_list = false;
    m_requested_locations.clear();

    switch (m_mode) {
    case POE_API::LEGACY:
        LegacyRefresh();
        break;
    case POE_API::OAUTH:
        OAuthRefresh();
        break;
    }
}

void ItemsManagerWorker::RemoveUpdatingTabs(const std::set<QString> &tab_ids)
{
    spdlog::trace("ItemsManagerWorker::RemoveUpdatingTabs() entered");
    if (tab_ids.empty()) {
        spdlog::error("No tabs to remove.");
        return;
    }

    // Keep tabs that are not being updated.
    std::vector<ItemLocation> current_tabs = m_tabs;
    m_tabs.clear();
    m_tab_id_index.clear();
    for (auto &tab : current_tabs) {
        const QString tab_uid = tab.get_tab_uniq_id();
        bool save_tab = (tab_ids.count(tab_uid) == 0);
        if (save_tab) {
            m_tabs.push_back(tab);
            m_tab_id_index.insert(tab_uid);
        } else {
            switch (tab.get_type()) {
            case ItemLocationType::STASH:
                if (m_first_stash_request_index < 0) {
                    m_first_stash_request_index = tab.get_tab_id();
                }
                break;
            case ItemLocationType::CHARACTER:
                if (m_first_character_request_name.isEmpty()) {
                    m_first_character_request_name = tab.get_character();
                }
                break;
            }
        }
    }
    spdlog::debug("Keeping {} tabs and culling {}",
                  m_tabs.size(),
                  (current_tabs.size() - m_tabs.size()));
}

void ItemsManagerWorker::RemoveUpdatingItems(const std::set<QString> &tab_ids)
{
    spdlog::trace("ItemsManagerWorker::RemoveUpdatingItems() entered");
    // Keep items with locations that are not being updated.
    if (tab_ids.empty()) {
        spdlog::error("No tabs to remove items from.");
        return;
    }
    Items current_items = m_items;
    m_items.clear();
    for (auto const &item : current_items) {
        const ItemLocation &tab = item.get()->location();
        bool save_item = (tab_ids.count(tab.get_tab_uniq_id()) == 0);
        if (save_item) {
            m_items.push_back(item);
        }
    }
    spdlog::debug("Keeping {} items and culling {}",
                  m_items.size(),
                  (current_items.size() - m_items.size()));
}

void ItemsManagerWorker::LegacyRefresh()
{
    spdlog::trace("Items Manager Worker: starting legacy refresh");
    if (m_need_stash_list) {
        // This queues stash tab requests.
        QNetworkRequest tab_request = MakeLegacyTabRequest(m_first_stash_request_index, true);
        spdlog::trace("ItemsManagerWorker::LegacyRefresh() requesting stash list: {}",
                      tab_request.url().toString());
        auto reply = m_rate_limiter.Submit(kStashItemsUrl, tab_request);
        connect(reply,
                &RateLimitedReply::complete,
                this,
                &ItemsManagerWorker::OnFirstLegacyTabReceived);
    }
    if (m_need_character_list) {
        // Before listing characters we download the main page because it's the only way
        // to know which character is selected (doesn't apply to OAuth api).
        QNetworkRequest main_page_request = QNetworkRequest(QUrl(kMainPage));
        spdlog::trace("ItemsManagerWorker::LegacyRefresh() requesting main page to capture "
                      "selected character: {}",
                      main_page_request.url().toString());
        main_page_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
        QNetworkReply *submit = m_network_manager.get(main_page_request);
        connect(submit,
                &QNetworkReply::finished,
                this,
                &ItemsManagerWorker::OnLegacyMainPageReceived);
    }
}

void ItemsManagerWorker::OAuthRefresh()
{
    spdlog::trace("Items Manager Worker: starting OAuth refresh");
    if (m_need_stash_list) {
        const auto request = MakeOAuthStashListRequest(m_realm, m_league);
        spdlog::trace("ItemsManagerWorker::OAuthRefresh() requesting stash list: {}",
                      request.url().toString());
        auto reply = m_rate_limiter.Submit(kOauthListStashesEndpoint, request);
        connect(reply,
                &RateLimitedReply::complete,
                this,
                &ItemsManagerWorker::OnOAuthStashListReceived);
    }
    if (m_need_character_list) {
        const auto request = MakeOAuthCharacterListRequest(m_realm);
        spdlog::trace("ItemsManagerWorker::OAuthRefresh() requesting character list: {}",
                      request.url().toString());
        auto submit = m_rate_limiter.Submit(kOAuthListCharactersEndpoint, request);
        connect(submit,
                &RateLimitedReply::complete,
                this,
                &ItemsManagerWorker::OnOAuthCharacterListReceived);
    }
}

QNetworkRequest ItemsManagerWorker::MakeOAuthStashListRequest(const QString &realm,
                                                              const QString &league)
{
    QString url(kOAuthListStashesUrl);
    if (realm != "pc") {
        url += "/" + realm;
    }
    url += "/" + league;
    return QNetworkRequest(QUrl(url));
}

QNetworkRequest ItemsManagerWorker::MakeOAuthCharacterListRequest(const QString &realm)
{
    QString url(kOAuthListCharactersUrl);
    if (realm != "pc") {
        url += "/" + realm;
    }
    return QNetworkRequest(QUrl(url));
}

QNetworkRequest ItemsManagerWorker::MakeOAuthStashRequest(const QString &realm,
                                                          const QString &league,
                                                          const QString &stash_id,
                                                          const QString &substash_id)
{
    QString url(kOAuthGetStashUrl);
    if (realm != "pc") {
        url += "/" + realm;
    };
    url += "/" + league;
    url += "/" + stash_id;
    if (!substash_id.isEmpty()) {
        url += "/" + substash_id;
    }
    return QNetworkRequest(QUrl(url));
}

QNetworkRequest ItemsManagerWorker::MakeOAuthCharacterRequest(const QString &realm,
                                                              const QString &name)
{
    QString url(kOAuthGetCharacterUrl);
    if (realm != "pc") {
        url += "/" + realm;
    }
    url += "/" + name;
    return QNetworkRequest(QUrl(url));
}

bool ItemsManagerWorker::IsOAuthTabValid(rapidjson::Value &tab)
{
    // Get the name of the stash tab.
    if (!HasString(tab, "name")) {
        spdlog::error("The stash tab does not have a name");
        return false;
    }

    // Skip hidden tabs.
    if (HasBool(tab, "hidden") && tab["hidden"].GetBool()) {
        spdlog::debug("The stash tab is hidden: {}", tab["name"].GetString());
        return false;
    }

    // Get the unique id.
    if (!HasString(tab, "id")) {
        spdlog::error("The stash tab does not have a unique id: {}", tab["name"].GetString());
        return false;
    }

    // Get the index of this stash tab.
    if (!HasInt(tab, "index")) {
        spdlog::error("The stash tab does not have an index: {}", tab["name"].GetString());
        return false;
    }

    // Get the type of this stash tab.
    if (!HasString(tab, "type")) {
        spdlog::error("The stash tab does not have a type: {}", tab["name"].GetString());
        return false;
    }

    return true;
}

void ItemsManagerWorker::ProcessOAuthTab(rapidjson::Value &tab,
                                         int &count,
                                         rapidjson_allocator &alloc)
{
    // Skip this tab if it doesn't pass sanity checks.
    if (!IsOAuthTabValid(tab)) {
        return;
    }

    // Get tab info.
    const QString tab_name = tab["name"].GetString();
    const int tab_index = tab["index"].GetInt();
    const QString tab_type = tab["type"].GetString();

    // The unique id for stash tabs returned from the legacy API
    // need to be trimmed to 10 characters.
    QString tab_id = tab["id"].GetString();
    if (tab_id.size() > 10) {
        spdlog::debug("Trimming tab unique id: {}", tab_name);
        tab_id = tab_id.first(10);
    }

    if (m_tab_id_index.count(tab_id) == 0) {
        // Create this tab.
        int r = 0, g = 0, b = 0;
        Util::GetTabColor(tab, r, g, b);
        ItemLocation location(
            tab_index, tab_id, tab_name, ItemLocationType::STASH, tab_type, r, g, b, tab, alloc);

        // Add this tab.
        m_tabs.push_back(location);
        m_tab_id_index.insert(tab_id);

        // Submit a request for this tab.
        if (m_update_tab_contents) {
            ++count;
            const auto uid = location.get_tab_uniq_id();
            QNetworkRequest request = MakeOAuthStashRequest(m_realm, m_league, uid);
            QueueRequest(kOAuthGetStashEndpoint, request, location);
        }

        // Process any children.
        if (tab.HasMember("children")) {
            for (auto &child : tab["children"]) {
                ProcessOAuthTab(child, count, alloc);
            }
        }
    }
}

void ItemsManagerWorker::OnOAuthStashListReceived(QNetworkReply *reply)
{
    spdlog::trace("ItemsManagerWorker::OnOAuthStashListReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    if (reply->error() != QNetworkReply::NoError) {
        spdlog::warn("Aborting update because there was an error fetching the stash list: {}",
                     reply->errorString());
        m_updating = false;
        return;
    }
    const QByteArray bytes = reply->readAll();

    rapidjson::Document doc;
    doc.Parse(bytes);
    if (doc.HasParseError()) {
        spdlog::error("Error parsing the stash list: {}",
                      rapidjson::GetParseError_En(doc.GetParseError()));
        m_updating = false;
        return;
    }

    if (!doc.IsObject() || !HasArray(doc, "stashes")) {
        spdlog::error("The stash list is invalid: {}", bytes);
        m_updating = false;
        return;
    }
    const auto &stashes = doc["stashes"].GetArray();

    spdlog::debug("Received stash list, there are {} stash tabs", stashes.Size());

    m_tabs_signature = CreateTabsSignatureVector(stashes);

    // Remember old tab headers before clearing tabs
    std::set<QString> old_tab_headers;
    for (auto const &tab : m_tabs) {
        old_tab_headers.emplace(tab.GetHeader());
    }

    // Force refreshes for any stash tabs that were moved or renamed.
    if (m_update_tab_contents) {
        for (auto const &tab : m_tabs) {
            if (!old_tab_headers.count(tab.GetHeader())) {
                spdlog::debug("Forcing refresh of moved or renamed tab: {}", tab.GetHeader());
                QNetworkRequest request = MakeOAuthStashRequest(m_realm,
                                                                m_league,
                                                                tab.get_tab_uniq_id());
                QueueRequest(kOAuthGetStashEndpoint, request, tab);
            }
        }
    }

    int tabs_requested = 0;

    auto &alloc = doc.GetAllocator();

    // Queue stash tab requests.
    for (rapidjson::Value &tab : stashes) {
        // This will process tabs recursively.
        ProcessOAuthTab(tab, tabs_requested, alloc);
    }
    spdlog::debug("Requesting {} out of {} stash tabs", tabs_requested, stashes.Size());

    m_has_stash_list = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!m_need_character_list || m_has_character_list) {
        spdlog::trace("ItemsManagerWorker::OnOAuthStashListReceived() fetching items");
        FetchItems();
    }
}

void ItemsManagerWorker::OnOAuthCharacterListReceived(QNetworkReply *reply)
{
    spdlog::trace("ItemsManagerWorker::OnOAuthCharacterListReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    spdlog::trace("OAuth character list received");
    if (reply->error() != QNetworkReply::NoError) {
        spdlog::warn("Aborting update because there was an error fetching the character list: {}",
                     reply->errorString());
        m_updating = false;
        return;
    }
    const QByteArray bytes = reply->readAll();

    rapidjson::Document doc;
    doc.Parse(bytes);
    if (doc.HasParseError()) {
        spdlog::error("Error parsing the character list: {}",
                      rapidjson::GetParseError_En(doc.GetParseError()));
        m_updating = false;
        return;
    }

    if (!doc.IsObject() || !HasArray(doc, "characters")) {
        spdlog::error("The characters list is invalid: {}", bytes);
        m_updating = false;
        return;
    }

    const auto &characters = doc["characters"].GetArray();
    int requested_character_count = 0;
    for (auto &character : characters) {
        if (!HasString(character, "name")) {
            spdlog::error("The character does not have a name. The reply may be invalid: {}", bytes);
            continue;
        }
        const QString name = character["name"].GetString();
        if (!HasString(character, "realm")) {
            spdlog::error("The character does not have a realm. The reply may be invalid: {}",
                          bytes);
        }
        const QString realm = character["realm"].GetString();
        if (!HasString(character, "league")) {
            spdlog::error("Malformed character entry for {}: the reply may be invalid: {}",
                          name,
                          bytes);
            continue;
        }
        const QString league = character["league"].GetString();
        if (realm != m_realm) {
            spdlog::trace("Skipping {} because this character is not in realm {}", name, m_realm);
            continue;
        }
        if (league != m_league) {
            spdlog::trace("Skipping {} because this character is not in league {}", name, m_league);
            continue;
        }
        if (m_tab_id_index.count(name) > 0) {
            spdlog::trace("Skipping {} because this item is not being refreshed.", name);
            continue;
        }
        const int tab_count = static_cast<int>(m_tabs.size());
        ItemLocation location(tab_count,
                              "",
                              name,
                              ItemLocationType::CHARACTER,
                              "",
                              0,
                              0,
                              0,
                              character,
                              doc.GetAllocator());
        m_tabs.push_back(location);

        // Queue character request if needed.
        if (m_update_tab_contents) {
            ++requested_character_count;
            QNetworkRequest request = MakeOAuthCharacterRequest(m_realm, name);
            QueueRequest(kOAuthGetCharacterEndpoint, request, location);
        }
    }
    spdlog::debug("There are {} characters to update in '{}'", requested_character_count, m_league);

    m_has_character_list = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!m_need_stash_list || m_has_stash_list) {
        spdlog::trace("ItemsManagerWorker::OnOAuthCharacterListReceived() fetching items");
        FetchItems();
    }
}

void ItemsManagerWorker::OnOAuthStashReceived(QNetworkReply *reply, const ItemLocation &location)
{
    spdlog::trace("ItemsManagerWorker::OnOAuthStashReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    spdlog::trace("OAuth stash recieved");
    if (reply->error() != QNetworkReply::NoError) {
        spdlog::warn("Aborting update because there was an error fetching the stash: {}",
                     reply->errorString());
        m_updating = false;
        return;
    }
    const QByteArray bytes = reply->readAll();

    rapidjson::Document doc;
    doc.Parse(bytes);
    if (doc.HasParseError()) {
        spdlog::error("Error parsing the stash: {}",
                      rapidjson::GetParseError_En(doc.GetParseError()));
        m_updating = false;
        return;
    }

    if (!HasObject(doc, "stash")) {
        spdlog::error("Error parsing the stash: 'stash' field was missing.");
        m_updating = false;
        return;
    }
    auto &stash = doc["stash"];

    if (HasArray(stash, "items")) {
        auto &items = stash["items"];
        if (items.GetArray().Size() > 0) {
            ParseItems(items, location, doc.GetAllocator());
        } else {
            spdlog::debug("Stash 'items' does not contain any items: {}", location.GetHeader());
        }
    } else {
        spdlog::debug("Stash does not have an 'items' array: {}", location.GetHeader());
    }

    ++m_stashes_received;
    SendStatusUpdate();

    if ((m_stashes_received == m_stashes_needed) && (m_characters_received == m_characters_needed)
        && !m_cancel_update) {
        spdlog::trace("ItemsManagerWorker::OnOAuthStashReceived() finishing update");
        FinishUpdate();
    }
}

void ItemsManagerWorker::OnOAuthCharacterReceived(QNetworkReply *reply, const ItemLocation &location)
{
    spdlog::trace("ItemsManagerWorker::OnOAuthCharacterReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    spdlog::trace("OAuth character recieved");
    if (reply->error() != QNetworkReply::NoError) {
        spdlog::warn("Aborting update because there was an error fetching the character: {}",
                     reply->errorString());
        m_updating = false;
        return;
    }
    const QByteArray bytes = reply->readAll();

    rapidjson::Document doc;
    doc.Parse(bytes);
    if (doc.HasParseError()) {
        spdlog::error("Error parsing the character: {}",
                      rapidjson::GetParseError_En(doc.GetParseError()));
        m_updating = false;
        return;
    }

    if (!HasObject(doc, "character")) {
        spdlog::error("The reply to a character request did not contain a character object.");
        m_updating = false;
        return;
    }

    auto character = doc["character"].GetObj();
    for (const auto &field : CHARACTER_ITEM_FIELDS) {
        if (character.HasMember(field)) {
            ParseItems(character[field], location, doc.GetAllocator());
        }
    }

    ++m_characters_received;
    SendStatusUpdate();

    if ((m_stashes_received == m_stashes_needed) && (m_characters_received == m_characters_needed)
        && !m_cancel_update) {
        spdlog::trace("ItemsManagerWorker::OnOAuthCharacterReceived() finishing update");
        FinishUpdate();
    }
}

void ItemsManagerWorker::OnLegacyMainPageReceived()
{
    spdlog::trace("ItemsManagerWorker::OnLegacyMainPageReceived() entered");

    QNetworkReply *reply = qobject_cast<QNetworkReply *>(QObject::sender());
    if (reply->error()) {
        spdlog::warn("Couldn't fetch main page: {} due to error: {}",
                     reply->url().toDisplayString(),
                     reply->errorString());
    } else {
        QString page(reply->readAll().constData());
        m_selected_character = Util::FindTextBetween(page, "C({\"name\":\"", "\",\"class");
        if (m_selected_character.isEmpty()) {
            spdlog::warn("Couldn't extract currently selected character name from GGG homepage "
                         "(maintenence?) Text was: {}",
                         page);
        }
    }
    reply->deleteLater();

    QNetworkRequest characters_request = MakeLegacyCharacterListRequest();
    spdlog::trace("ItemsManagerWorker::OnLegacyMainPageReceived() requesting characters: {}",
                  characters_request.url().toString());
    auto submit = m_rate_limiter.Submit(kGetCharactersUrl, characters_request);
    connect(submit,
            &RateLimitedReply::complete,
            this,
            &ItemsManagerWorker::OnLegacyCharacterListReceived);
}

void ItemsManagerWorker::OnLegacyCharacterListReceived(QNetworkReply *reply)
{
    spdlog::trace("ItemsManagerWorker::OnLegacyCharacterListReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();
    if (reply->error()) {
        spdlog::warn("Couldn't fetch character list: {} due to error: {}. Aborting update.",
                     reply->url().toDisplayString(),
                     reply->errorString());
        m_updating = false;
        reply->deleteLater();
        return;
    }

    const QByteArray bytes = reply->readAll();
    rapidjson::Document doc;
    doc.Parse(bytes.constData());
    if (doc.HasParseError() || !doc.IsArray()) {
        spdlog::error("Received invalid reply instead of character list: {}", bytes.constData());
        if (doc.HasParseError()) {
            spdlog::error("The error was {}", rapidjson::GetParseError_En(doc.GetParseError()));
        };
        spdlog::error("");
        spdlog::error(
            "(Maybe you need to log in to the website manually and accept new Terms of Service?)");
        m_updating = false;
        return;
    }

    spdlog::debug("Received character list, there are {} characters across all leagues.",
                  doc.Size());

    auto &alloc = doc.GetAllocator();

    int requested_character_count = 0;
    for (auto &character : doc) {
        if (!HasString(character, "name")) {
            spdlog::error("The legacy character does not have a name. The reply may be invalid: {}",
                          bytes);
            continue;
        }
        const QString name = character["name"].GetString();
        if (!HasString(character, "league")) {
            spdlog::error("Malformed legacy character entry for {}: the reply may be invalid: {}",
                          name,
                          bytes);
            continue;
        }
        const QString league = character["league"].GetString();
        if (league != m_league) {
            spdlog::debug("Skipping {} because this character is not in league {}", name, m_league);
            continue;
        }
        if (m_tab_id_index.count(name) > 0) {
            spdlog::debug("Skipping legacy character {} because this item is not being refreshed.",
                          name);
            continue;
        }
        const int tab_count = static_cast<int>(m_tabs.size());
        const auto loc_type = ItemLocationType::CHARACTER;
        ItemLocation location(tab_count, "", name, loc_type, "", 0, 0, 0, character, alloc);
        m_tabs.push_back(location);

        if (m_update_tab_contents) {
            ++requested_character_count;

            //Queue request for items on character in character's stash
            QueueRequest(kCharacterItemsUrl, MakeLegacyCharacterRequest(name), location);

            //Queue request for jewels in character's passive tree
            QueueRequest(kCharacterSocketedJewels, MakeLegacyPassivesRequest(name), location);
        }
    }
    spdlog::debug("There are {}characters to update in {}", requested_character_count, m_league);

    m_has_character_list = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!m_need_stash_list || m_has_stash_list) {
        spdlog::trace("ItemsManagerWorker::OnLegacyCharacterListReceived() fetching items");
        FetchItems();
    }
}

QNetworkRequest ItemsManagerWorker::MakeLegacyCharacterListRequest()
{
    spdlog::trace("ItemsManagerWorker::MakeLegacyCharacterListRequest() entered");

    QUrlQuery query;
    query.addQueryItem("accountName", m_account);
    query.addQueryItem("realm", m_realm);

    QUrl url(kGetCharactersUrl);
    url.setQuery(query);
    return QNetworkRequest(url);
}

QNetworkRequest ItemsManagerWorker::MakeLegacyTabRequest(int tab_index, bool tabs)
{
    spdlog::trace("ItemsManagerWorker::MakeLegacyTabRequest() entered");

    if (tab_index < 0) {
        spdlog::error("MakeLegacyTabRequest: invalid tab_index = {}", tab_index);
    }
    QUrlQuery query;
    query.addQueryItem("accountName", m_account);
    query.addQueryItem("realm", m_realm);
    query.addQueryItem("league", m_league);
    query.addQueryItem("tabs", tabs ? "1" : "0");
    query.addQueryItem("tabIndex", QString::number(tab_index));

    QUrl url(kStashItemsUrl);
    url.setQuery(query);
    return QNetworkRequest(url);
}

QNetworkRequest ItemsManagerWorker::MakeLegacyCharacterRequest(const QString &name)
{
    spdlog::trace("ItemsManagerWorker::MakeLegacyCharacterRequest() entered");

    if (name.isEmpty()) {
        spdlog::error("MakeLegacyCharacterRequest: invalid name = '" + name + "'");
    }
    QUrlQuery query;
    query.addQueryItem("accountName", m_account);
    query.addQueryItem("realm", m_realm);
    query.addQueryItem("character", name);

    QUrl url(kCharacterItemsUrl);
    url.setQuery(query);
    return QNetworkRequest(url);
}

QNetworkRequest ItemsManagerWorker::MakeLegacyPassivesRequest(const QString &name)
{
    spdlog::trace("ItemsManagerWorker::MakeLegacyPassivesRequest() entered");

    if (name.isEmpty()) {
        spdlog::error("MakeLegacyPassivesRequest: invalid name = '" + name + "'");
    }
    QUrlQuery query;
    query.addQueryItem("accountName", m_account);
    query.addQueryItem("realm", m_realm);
    query.addQueryItem("character", name);

    QUrl url(kCharacterSocketedJewels);
    url.setQuery(query);
    return QNetworkRequest(url);
}

void ItemsManagerWorker::QueueRequest(const QString &endpoint,
                                      const QNetworkRequest &request,
                                      const ItemLocation &location)
{
    spdlog::trace("Queued ({}) -- {}", (m_queue_id + 1), location.GetHeader());
    ItemsRequest items_request;
    items_request.endpoint = endpoint;
    items_request.network_request = request;
    items_request.id = m_queue_id++;
    items_request.location = location;
    m_queue.push(items_request);
    m_requested_locations.insert(location);
}

void ItemsManagerWorker::FetchItems()
{
    spdlog::trace("ItemsManagerWorker::FetchItems() entered");

    if (!m_update_tab_contents) {
        spdlog::trace("ItemsManagerWorker: not fetching items.");
        FinishUpdate();
        return;
    }

    m_stashes_needed = 0;
    m_stashes_received = 0;

    m_characters_needed = 0;
    m_characters_received = 0;

    QString tab_titles;
    while (!m_queue.empty()) {
        // Take the next request out of the queue.
        ItemsRequest request = m_queue.front();
        m_queue.pop();

        // Setup the right callback for this endpoint.
        const ItemLocation location = request.location;
        const QString endpoint = request.endpoint;
        std::function<void(QNetworkReply *)> callback;

        if (endpoint == kStashItemsUrl) {
            callback = [=](QNetworkReply *reply) { OnLegacyTabReceived(reply, location); };
            ++m_stashes_needed;
        } else if ((endpoint == kCharacterItemsUrl) || (endpoint == kCharacterSocketedJewels)) {
            callback = [=](QNetworkReply *reply) { OnLegacyTabReceived(reply, location); };
            ++m_characters_needed;
        } else if (endpoint == kOAuthGetStashEndpoint) {
            callback = [=](QNetworkReply *reply) { OnOAuthStashReceived(reply, location); };
            ++m_stashes_needed;
        } else if (endpoint == kOAuthGetCharacterEndpoint) {
            callback = [=](QNetworkReply *reply) { OnOAuthCharacterReceived(reply, location); };
            ++m_characters_needed;
        } else {
            spdlog::error("FetchItems(): invalid endpoint: {}", request.endpoint);
        }

        // Pass the request to the rate limiter.
        auto submit = m_rate_limiter.Submit(request.endpoint, request.network_request);
        connect(submit, &RateLimitedReply::complete, this, callback);

        // Keep track of the tabs requested.
        tab_titles += request.location.GetHeader() + " ";
    }

    SendStatusUpdate();

    spdlog::debug("Requested {} stashes and {} characters.", m_stashes_needed, m_characters_needed);
    spdlog::debug("Tab titles: {}", tab_titles);

    // Make sure we cancel the update if there was nothing to do.
    // (Discovered this was necessary when trying to refresh a single unique stashtab).
    if ((m_stashes_needed == 0) && (m_characters_needed == 0)) {
        m_updating = false;
    }
}

bool ItemsManagerWorker::IsLegacyTabValid(rapidjson::Value &tab)
{
    if (!HasString(tab, "n")) {
        spdlog::error("Legacy tab does not have name");
        return false;
    }

    if (!HasInt(tab, "i")) {
        spdlog::error("Legacy tab does not have an index: {}", tab["n"].GetString());
        return false;
    }

    // Skip hidden tabs.
    if (HasBool(tab, "hidden") && tab["hidden"].GetBool()) {
        spdlog::debug("The legacy tab is hidden: {}", tab["n"].GetString());
        return false;
    }

    // Skip tabs that are in the index; they are not being refreshed.
    if (!HasString(tab, "id")) {
        spdlog::error("The legacy tab does not have a unique id: {}", tab["n"].GetString());
        return false;
    }
    QString tab_id = tab["id"].GetString();
    if (tab_id.size() > 10) {
        // The unique id for stash tabs returned from the legacy API
        // need to be trimmed to 10 characters.
        spdlog::debug("Trimming legacy tab unique id: {}", tab["n"].GetString());
        tab_id = tab_id.first(10);
    }

    // Get the type of this stash tab.
    if (!HasString(tab, "type")) {
        spdlog::error("The stash tab does not have a type: {}", tab["n"].GetString());
        return false;
    }

    return true;
}

void ItemsManagerWorker::ProcessLegacyTab(rapidjson::Value &tab,
                                          int &count,
                                          rapidjson_allocator &alloc)
{
    if (!IsLegacyTabValid(tab)) {
        return;
    }

    const QString label = tab["n"].GetString();
    const QString tab_type = tab["type"].GetString();
    const int index = tab["i"].GetInt();

    // The unique id for stash tabs returned from the legacy API
    // need to be trimmed to 10 characters.
    QString tab_id = tab["id"].GetString();
    if (tab_id.size() > 10) {
        tab_id = tab_id.first(10);
    }

    if (m_tab_id_index.count(tab_id) == 0) {
        // Create this tab.
        int r = 0, g = 0, b = 0;
        Util::GetTabColor(tab, r, g, b);
        ItemLocation
            location(index, tab_id, label, ItemLocationType::STASH, tab_type, r, g, b, tab, alloc);

        // Add this tab.
        m_tabs.push_back(location);
        m_tab_id_index.insert(tab_id);

        // Submit a request for this tab.
        if (m_update_tab_contents) {
            ++count;
            QNetworkRequest request = MakeLegacyTabRequest(location.get_tab_id(), true);
            QueueRequest(kStashItemsUrl, request, location);
        }

        // Process any children.
        if (tab.HasMember("children")) {
            for (auto &child : tab["children"]) {
                ProcessLegacyTab(child, count, alloc);
            }
        }
    }
}

void ItemsManagerWorker::OnFirstLegacyTabReceived(QNetworkReply *reply)
{
    spdlog::trace("ItemsManagerWorker::OnFirstLegacyTabReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    spdlog::trace("First legacy tab received.");
    rapidjson::Document doc;
    QByteArray bytes = reply->readAll();
    doc.Parse(bytes.constData());

    if (!doc.IsObject()) {
        spdlog::error("Can't even fetch first legacy tab. Failed to update items.");
        m_updating = false;
        return;
    }

    if (doc.HasMember("error")) {
        spdlog::error("Aborting legacy update since first fetch failed due to 'error': {}",
                      Util::RapidjsonSerialize(doc["error"]));
        m_updating = false;
        return;
    }

    if (!HasArray(doc, "tabs") || doc["tabs"].Size() == 0) {
        spdlog::error("There are no legacy tabs, this should not happen, bailing out.");
        m_updating = false;
        return;
    }

    auto &tabs = doc["tabs"];

    spdlog::debug("Received legacy tabs list, there are {} tabs", tabs.Size());
    m_tabs_signature = CreateTabsSignatureVector(tabs);

    // Remember old tab headers before clearing tabs
    std::set<QString> old_tab_headers;
    for (auto const &tab : m_tabs) {
        old_tab_headers.emplace(tab.GetHeader());
    }

    // Force refreshes for any stash tabs that were moved or renamed.
    if (m_update_tab_contents) {
        for (auto const &tab : m_tabs) {
            if (!old_tab_headers.count(tab.GetHeader())) {
                spdlog::debug("Forcing refresh of moved or renamed tab: {}", tab.GetHeader());
                QueueRequest(kStashItemsUrl, MakeLegacyTabRequest(tab.get_tab_id(), true), tab);
            }
        }
    }

    // Queue stash tab requests.
    int count = 0;
    auto &alloc = doc.GetAllocator();
    for (auto &tab : tabs) {
        ProcessLegacyTab(tab, count, alloc);
    }

    m_has_stash_list = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!m_need_character_list || m_has_character_list) {
        spdlog::trace("ItemsManagerWorker::OnFirstLegacyTabReceived() fetching items");
        FetchItems();
    }
}

void ItemsManagerWorker::SendStatusUpdate()
{
    spdlog::trace("ItemsManagerWorker::SendStatusUpdate() entered");

    if (m_cancel_update) {
        emit StatusUpdate(ProgramState::Ready, "Update cancelled.");
    } else {
        QString message;
        if ((m_stashes_needed > 0) && (m_characters_needed > 0)) {
            message = QString("Receieved %1/%2 stash tabs and %3/%4 character locations")
                          .arg(QString::number(m_stashes_received),
                               QString::number(m_stashes_needed),
                               QString::number(m_characters_received),
                               QString::number(m_characters_needed));
        } else if (m_stashes_needed > 0) {
            message = QString("Received %1/%2 stash tabs")
                          .arg(QString::number(m_stashes_received),
                               QString::number(m_stashes_needed));
        } else if (m_characters_needed > 0) {
            message = QString("Received %1/%2 character locations")
                          .arg(QString::number(m_characters_received),
                               QString::number(m_characters_needed));
        } else if (!m_update_tab_contents) {
            message = "Received tab lists";
        } else {
            message = "Received nothing; needed nothing.";
        }
        emit StatusUpdate(ProgramState::Busy, message);
    }
}

void ItemsManagerWorker::ParseItems(rapidjson::Value &value,
                                    const ItemLocation &base_location,
                                    rapidjson_allocator &alloc)
{
    ItemLocation location = base_location;

    for (auto &item : value) {
        // Make sure location data from the item like x and y is brought over to the location object.
        location.FromItemJson(item);
        location.ToItemJson(&item, alloc);
        m_items.push_back(std::make_shared<Item>(item, location));
        if (HasArray(item, "socketedItems")) {
            location.set_socketed(true);
            ParseItems(item["socketedItems"], location, alloc);
            location.set_socketed(false);
        }
    }
}

void ItemsManagerWorker::OnLegacyTabReceived(QNetworkReply *reply, const ItemLocation &location)
{
    spdlog::trace("ItemsManagerWorker::OnLegacyTabReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    spdlog::debug("Legacy tab receivevd: {}", location.GetHeader());
    rapidjson::Document doc;
    QByteArray bytes = reply->readAll();
    doc.Parse(bytes.constData());

    bool error = false;
    if (!doc.IsObject()) {
        spdlog::error("Legacy tab is non-object response for: {}", location.GetHeader());
        error = true;
    } else if (doc.HasMember("error")) {
        // this can happen if user is browsing stash in background and we can't know about it
        spdlog::error("Legacy tab has 'error' instead of stash tab contents for: {}",
                      location.GetHeader());
        spdlog::error("The error is: {}", Util::RapidjsonSerialize(doc["error"]));
        error = true;
    }

    // We index expected tabs and their locations as part of the first fetch.  It's possible for users
    // to move or rename tabs during the update which will result in the item data being out-of-sync with
    // expected index/tab name map.  We need to detect this case and abort the update.
    if (!m_cancel_update && !error && (location.get_type() == ItemLocationType::STASH)) {
        m_cancel_update = TabsChanged(doc, reply, location);
    }

    switch (location.get_type()) {
    case ItemLocationType::STASH:
        ++m_stashes_received;
        break;
    case ItemLocationType::CHARACTER:
        ++m_characters_received;
        break;
    default:
        spdlog::error("OnLegacyTabReceived: invalid location type {}", location.get_type());
    }
    SendStatusUpdate();

    if ((m_stashes_received == m_stashes_needed) && (m_characters_received == m_characters_needed)) {
        if (m_cancel_update) {
            spdlog::trace("ItemsManagerWorker::OnLegacyTabReceived() cancelling update");
            m_updating = false;
        }
    }

    if (error) {
        return;
    }

    if (!HasArray(doc, "items")) {
        spdlog::debug("Legacy stash does not have an 'items' array: {}", location.GetHeader());
    } else {
        auto &items = doc["items"];
        if (items.Size() == 0) {
            spdlog::debug("Legacy stash 'items' is empty: {}", location.GetHeader());
        } else {
            ParseItems(items, location, doc.GetAllocator());
        }
    }

    if ((m_stashes_received == m_stashes_needed) && (m_characters_received == m_characters_needed)
        && !m_cancel_update) {
        spdlog::trace("ItemsManagerWorker::OnLegacyTabReceived() finishing update");
        FinishUpdate();
        PreserveSelectedCharacter();
    }
}

bool ItemsManagerWorker::TabsChanged(rapidjson::Document &doc,
                                     QNetworkReply *network_reply,
                                     const ItemLocation &location)
{
    spdlog::trace("ItemsManagerWorker::TabsChanged() entered");

    if (!doc.HasMember("tabs") || doc["tabs"].Size() == 0) {
        spdlog::error("Full tab information missing from stash tab fetch.  Cancelling update. Full "
                      "fetch URL: {}",
                      network_reply->request().url().toDisplayString());
        return true;
    }

    auto tabs_signature_current = CreateTabsSignatureVector(doc["tabs"]);
    auto tab_id = location.get_tab_id();
    if (m_tabs_signature[tab_id] != tabs_signature_current[tab_id]) {
        QString reason;
        if (tabs_signature_current.size() != m_tabs_signature.size()) {
            reason += "[Tab size mismatch:" + std::to_string(tabs_signature_current.size())
                      + " != " + std::to_string(m_tabs_signature.size()) + "]";
        }

        auto &x = tabs_signature_current[tab_id];
        auto &y = m_tabs_signature[tab_id];
        reason += "[tab_index=" + std::to_string(tab_id) + "/"
                  + std::to_string(tabs_signature_current.size()) + "(#"
                  + std::to_string(tab_id + 1) + ")]";

        if (x.first != y.first) {
            reason += "[name:" + x.first + " != " + y.first + "]";
        }
        if (x.second != y.second) {
            reason += "[id:" + x.second + " != " + y.second + "]";
        }

        spdlog::error("You renamed or re-ordered tabs in game while acquisition was in the middle "
                      "of the update,"
                      " aborting to prevent synchronization problems and pricing data loss. "
                      "Mismatch reason(s) -> {}."
                      " For request: {}",
                      reason,
                      network_reply->request().url().toDisplayString());
        return true;
    }
    return false;
}

void ItemsManagerWorker::FinishUpdate()
{
    spdlog::trace("ItemsManagerWorker::FinishUpdate() entered");

    // It's possible that we receive character vs stash tabs out of order, or users
    // move items around in a tab and we get them in a different order. For
    // consistency we want to present the tab data in a deterministic way to the rest
    // of the application.  Especially so we don't try to update shop when nothing actually
    // changed.  So sort m_items here before emitting and then generate
    // item list as strings.

    QString message;
    if ((m_stashes_received > 0) && (m_characters_received > 0)) {
        message = QString("Received %1 stash tabs and %2 character locations")
                      .arg(QString::number(m_stashes_received),
                           QString::number(m_characters_received));
    } else if (m_stashes_received > 0) {
        message = QString("Received %1 stash tabs").arg(QString::number(m_stashes_received));
    } else if (m_characters_received > 0) {
        message = QString("Received %1 character locations")
                      .arg(QString::number(m_characters_received));
    } else if (!m_update_tab_contents) {
        message = "Received tab lists";
    } else {
        message = "Received nothing";
    }

    emit StatusUpdate(ProgramState::Ready, message);

    // Sort tabs.
    std::sort(begin(m_tabs), end(m_tabs));

    // Maps location type (CHARACTER or STASH) to a list of all the tabs of that type
    std::map<ItemLocationType, Locations> tabsPerType;
    for (auto &tab : m_tabs) {
        tabsPerType[tab.get_type()].push_back(tab);
    }

    // Save tabs by tab type.
    for (auto const &pair : tabsPerType) {
        const auto &location_type = pair.first;
        const auto &tabs = pair.second;
        m_datastore.SetTabs(location_type, tabs);
    }

    if (m_update_tab_contents) {
        // Clear out all requested locations before updating them with the new items.
        for (const auto &location : m_requested_locations) {
            m_datastore.SetItems(location, {});
        }

        // Sort items.
        std::sort(begin(m_items),
                  end(m_items),
                  [](const std::shared_ptr<Item> &a, const std::shared_ptr<Item> &b) {
                      return *a < *b;
                  });

        // Map locations to a list of items in that location.
        std::map<ItemLocation, Items> itemsPerLoc;
        for (auto &item : m_items) {
            itemsPerLoc[item->location()].push_back(item);
        }

        // Save items by location.
        for (auto const &pair : itemsPerLoc) {
            const auto &location = pair.first;
            const auto &items = pair.second;
            m_datastore.SetItems(location, items);
        }
    }

    // Let everyone know the update is done.
    spdlog::trace("ItemsManagerWorker::FinishUpdate() emitting ItemsRefreshed");
    emit ItemsRefreshed(m_items, m_tabs, false);

    m_updating = false;
    spdlog::debug("Update finished.");
}

void ItemsManagerWorker::PreserveSelectedCharacter()
{
    spdlog::trace("ItemsManagerWorker::PreserveSelectedCharacter() entered");

    if (m_selected_character.isEmpty()) {
        spdlog::debug("Cannot preserve selected character: no character selected");
        return;
    }
    spdlog::debug("Preserving selected character: {}", m_selected_character);
    // The act of making this request sets the active character.
    // We don't need to to anything with the reply.
    QNetworkRequest character_request = MakeLegacyCharacterRequest(m_selected_character);
    auto submit = m_rate_limiter.Submit(kCharacterItemsUrl, character_request);
    connect(submit, &RateLimitedReply::complete, this, [=](QNetworkReply *reply) {
        reply->deleteLater();
    });
}

ItemsManagerWorker::TabsSignatureVector ItemsManagerWorker::CreateTabsSignatureVector(
    const rapidjson::Value &tabs)
{
    spdlog::trace("ItemsManagerWorker::CreateTabsSignatureVector() entered");

    if (!tabs.IsArray()) {
        spdlog::error("Tabs list is invalid: cannot create signature vector");
        return {};
    }

    const bool legacy = tabs[0].HasMember("n");
    const char *n = legacy ? "n" : "name";
    const char *id = "id";
    TabsSignatureVector signature;
    for (auto &tab : tabs) {
        QString name = HasString(tab, n) ? tab[n].GetString() : "UNKNOWN_NAME";
        QString uid = HasString(tab, id) ? tab[id].GetString() : "UNKNOWN_ID";
        if (!tab.HasMember("class")) {
            // The stash tab unique id is only ten characters, but legacy tabs
            // return a much longer value. GGG confirmed that taking the first
            // ten characters is the right thing to do.
            if (uid.size() > 10) {
                uid = uid.first(10);
            }
        }
        signature.emplace_back(name, uid);
    }
    return signature;
}
