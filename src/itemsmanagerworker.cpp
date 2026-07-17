// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "itemsmanagerworker.h"

#include <QDir>
#include <QFileInfo>
#include <QNetworkCookie>
#include <QNetworkCookieJar>
#include <QNetworkReply>
#include <QSettings>
#include <QSignalMapper>
#include <QThread>
#include <QTimer>
#include <QUrlQuery>

#include <algorithm>
#include <unordered_map>

#include "buyoutmanager.h"
#include "datastore/characterrepo.h"
#include "datastore/stashrepo.h"
#include "datastore/userstore.h"
#include "itemlocation.h"
#include "modlist.h"
#include "poe/poe_utils.h"
#include "poe/types/character.h"
#include "poe/types/item.h"
#include "poe/types/stashtab.h"
#include "ratelimit/ratelimit.h"
#include "ratelimit/ratelimitedreply.h"
#include "ratelimit/ratelimiter.h"
#include "repoe/repoe.h"
#include "util/json_readers.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/util.h"

ItemsManagerWorker::ItemsManagerWorker(QSettings &settings,
                                       BuyoutManager &buyout_manager,
                                       RateLimiter &rate_limiter,
                                       QObject *parent)
    : QObject(parent)
    , m_settings(settings)
    , m_buyout_manager(buyout_manager)
    , m_rate_limiter(rate_limiter)
    , m_type(TabSelection::Checked)
    , m_update_all(false)
    , m_need_stash_list(false)
    , m_need_character_list(false)
    , m_has_stash_list(false)
    , m_has_character_list(false)
    , m_update_tab_contents(true)
{
    spdlog::trace("ItemsManagerWorker::ItemsManagerWorker() entered");

    m_realm = m_settings.value("realm").toString();
    m_league = m_settings.value("league").toString();
    m_account = m_settings.value("account").toString();
    spdlog::trace("ItemsManagerWorker: league = {}", m_league);
    spdlog::trace("ItemsManagerWorker: account = {}", m_account);
}

ItemsManagerWorker::~ItemsManagerWorker()
{
    m_shutdown.store(true);
    if (m_parser_thread && m_parser_thread->isRunning()) {
        m_parser_thread->wait();
    }
}

void ItemsManagerWorker::UpdateRequest(TabSelection type, const std::vector<ItemLocation> &locations)
{
    spdlog::trace("ItemsManagerWorker::UpdateRequest() entered");
    m_updateRequest = true;
    m_type = type;
    m_locations = locations;
}

void ItemsManagerWorker::OnRePoEReady()
{
    spdlog::trace("ItemsManagerWorker::OnRePoEReady() entered");
    StartParseThread();
}

void ItemsManagerWorker::StartParseThread()
{
    // Create a separate thread to load the items, which allows the UI to
    // update the status bar while items are being parsed. This operation
    // can take tens of seconds or longer depending on the nubmer of tabs
    // and items.
    const QFileInfo info(m_settings.fileName());
    const QString dataDir = info.absolutePath() + "/data/";
    m_shutdown.store(false);
    m_parser_thread = QThread::create([this, dataDir]() {
        auto result = ParseCachedItems(dataDir);
        if (m_shutdown.load()) {
            return;
        }
        QMetaObject::invokeMethod(
            this,
            [this, result = std::move(result)]() mutable { OnParseCompleted(std::move(result)); },
            Qt::QueuedConnection);
    });
    connect(m_parser_thread, &QThread::finished, m_parser_thread, &QThread::deleteLater);
    m_parser_thread->start();
}

ParseResult ItemsManagerWorker::ParseCachedItems(const QString &dataDir) const
{
    spdlog::trace("ItemsManagerWorker::ParseItemMods() entered");
    ParseResult result;
    auto sendStatusUpdate = [this](ProgramState state, const QString &status) {
        emit const_cast<ItemsManagerWorker *>(this)->StatusUpdate(state, status);
    };

    // Manual-testing aid: delay each cached tab so that mid-parse GUI actions
    // (quit, refresh-while-initializing) can be exercised by hand. Unset or
    // zero in normal use.
    const int parse_delay_ms = qEnvironmentVariableIntValue("ACQ_PARSE_DELAY_MS");
    if (parse_delay_ms > 0) {
        spdlog::warn("ACQ_PARSE_DELAY_MS is set: delaying {}ms per cached tab", parse_delay_ms);
    }

    // Create a datastore to get a connection to the database.
    // NOTE: we only need read access, but this isn't enforced or checked.
    QDir data_dir{dataDir};
    UserStore userstore{data_dir, m_account};

    // Get cached characters and stash tabs.
    const auto stashes = userstore.stashes().getStashList(m_realm, m_league);
    const auto characters = userstore.characters().getCharacterList(m_realm, m_league);

    // Do not process children from unique and map stashers.
    auto special = [](const poe::StashTab &stash) {
        return stash.parent && ((stash.type == "UniqueStash") || (stash.type == "MapStash"));
    };

    result.tabs.reserve(stashes.size() + characters.size());
    for (const auto &stash : stashes) {
        if (!special(stash)) {
            result.tabs.push_back(ItemLocation(stash));
        }
    }
    for (const auto &character : characters) {
        result.tabs.push_back(ItemLocation(character, int(result.tabs.size())));
    }
    result.tabs.shrink_to_fit();

    // Save location ids.
    spdlog::trace("ItemsManagerWorker::ParseItemMods() saving location ids");
    for (const auto &tab : result.tabs) {
        result.tab_id_index.emplace(tab.id());
    }

    // Get cached items
    spdlog::trace("ItemsManagerWorker::ParseItemMods() getting cached items");

    // Get stash items.
    for (size_t i = 0; i < stashes.size(); ++i) {
        if (m_shutdown.load()) {
            return result;
        }
        if (parse_delay_ms > 0) {
            QThread::msleep(parse_delay_ms);
        }
        const auto id = stashes[i].id;
        const auto stash = userstore.stashes().getStash(id, m_realm, m_league);
        if (!stash) {
            // The row exists but its contents were never fetched: the tab
            // was listed and must stay "new" so the next content update
            // fetches it (F55).
            continue;
        }
        result.contents_known.insert(id);
        sendStatusUpdate(ProgramState::Initializing,
                         QString("Parsing items from stash %1/%2: %3 '%4'")
                             .arg(QString::number(i),
                                  QString::number(stashes.size()),
                                  stash->id,
                                  stash->name));
        ItemLocation location;
        if (!special(*stash)) {
            location = ItemLocation{*stash};
        } else {
            // Items in special tabs should use their parent's ItemLocation.
            for (const auto &tab : result.tabs) {
                if (tab.id() == stash->parent) {
                    location = tab;
                    break;
                }
            }
            if (!location.IsValid()) {
                spdlog::error("ItemsManagerWorker: could not find stash parent");
                continue;
            }
            // The items display under the parent, but they were fetched from
            // (and are keyed for replacement by) this child stash.
            location.setFetchId(stash->id);
        }
        LoadItems(*stash, location, result);
    }

    // Get character items.
    for (size_t i = 0; i < characters.size(); ++i) {
        if (m_shutdown.load()) {
            return result;
        }
        if (parse_delay_ms > 0) {
            QThread::msleep(parse_delay_ms);
        }
        const auto name = characters[i].name;
        const auto character = userstore.characters().getCharacter(name, m_realm);
        if (!character) {
            // Listed but never fetched: stays "new" (F55).
            continue;
        }
        result.contents_known.insert(characters[i].id);
        sendStatusUpdate(ProgramState::Initializing,
                         QString("Parsing items from character %1/%2: %3")
                             .arg(i)
                             .arg(characters.size())
                             .arg(name));
        ItemLocation tab{*character, int(stashes.size() + i)};
        LoadItems(*character, tab, result);
    }

    sendStatusUpdate(ProgramState::Ready,
                     QString("Parsed %1 items from %2 tabs")
                         .arg(result.items.size())
                         .arg(result.tabs.size()));

    return result;
}

void ItemsManagerWorker::OnParseCompleted(ParseResult result)
{
    m_tabs = std::move(result.tabs);
    m_items = std::move(result.items);
    m_tab_id_index = std::move(result.tab_id_index);
    m_contents_known = std::move(result.contents_known);
    m_state = WorkerState::Idle;
    // let ItemManager know that the retrieval of cached items/tabs has been completed (calls ItemsManager::OnItemsRefreshed method)
    spdlog::trace("ItemsManagerWorker::ParseItemMods() emitting ItemsRefreshed signal");
    emit ItemsRefreshed(m_items, m_tabs, true);

    if (m_updateRequest) {
        spdlog::trace("ItemsManagerWorker::ParseItemMods() triggering requested update");
        m_updateRequest = false;
        Update(m_type, m_locations);
    }
}

void ItemsManagerWorker::LoadItems(const poe::StashTab &stash,
                                   ItemLocation location,
                                   ParseResult &result) const
{
    const auto items = stash.items;
    if (!items) {
        return;
    }
    result.items.reserve(result.items.size() + items->size());
    spdlog::debug("ItemManagerWorker: loading {} items from stash # {}: {} ({})",
                  items->size(),
                  stash.index.value_or(-1),
                  stash.id,
                  stash.name);
    for (const auto &item : *items) {
        result.items.push_back(std::make_shared<Item>(item, location));
    }
}

void ItemsManagerWorker::LoadItems(const poe::Character &character,
                                   ItemLocation location,
                                   ParseResult &result) const
{
    const std::array collections{
        std::make_pair("equipment", character.equipment),
        std::make_pair("inventory", character.inventory),
        std::make_pair("rucksack", character.rucksack),
        std::make_pair("jewels", character.jewels),
    };

    for (const auto &[name, items] : collections) {
        if (!items) {
            continue;
        }
        result.items.reserve(result.items.size() + items->size());
        spdlog::debug("ItemManagerWorker: loading {} items from character {} {} ({})",
                      items->size(),
                      name,
                      character.id,
                      character.name);
        for (const auto &item : *items) {
            result.items.push_back(std::make_shared<Item>(item, location));
        }
    }
}

void ItemsManagerWorker::Update(TabSelection type, const std::vector<ItemLocation> &locations)
{
    if (!isInitialized()) {
        // tell ItemsManagerWorker to run an Update() after it's finished updating mods
        UpdateRequest(type, locations);
        spdlog::debug("Update deferred until item mods parsing is complete");
        emit NotifyUser(
            "This items worker is still initializing, but an update request has been queued.");
        return;
    }

    if (isUpdating()) {
        spdlog::warn("ItemsManagerWorker: update called while updating");
        emit NotifyUser("An update is already in progress.");
        return;
    }

    spdlog::debug("ItemsManagerWorker: updating {} tabs", type);
    if (spdlog::should_log(spdlog::level::trace)) {
        QStringList character_names;
        QStringList stash_names;
        for (const auto &location : locations) {
            switch (location.type()) {
            case ItemLocationType::CHARACTER:
                character_names.append(location.character());
                break;
            case ItemLocationType::STASH:
                stash_names.append(location.tab_label());
                break;
            }
        }
        spdlog::trace("ItemsManagerWorker: characters = {}", character_names.join(", "));
        spdlog::trace("ItemsManagerWorker: stashes = {}", stash_names.join(", "));
    }
    m_state = WorkerState::Updating;
    ++m_update_generation;
    m_request_failures = 0;
    m_update_tab_contents = (type != TabSelection::TabsOnly);

    // Reset the completion counters here so that CheckUpdateFinished() never
    // depends on leftover values from a previous update. (FetchItems() resets
    // them again before queueing requests, but it is not reached on every path.)
    m_stashes_needed = 0;
    m_stashes_received = 0;
    m_characters_needed = 0;
    m_characters_received = 0;

    // remove all pending requests
    m_queue = {};
    m_queue_id = 0;

    // Build the content selection. Nothing is culled here: tab entries are
    // reconciled against the fresh lists as they arrive, and each tab's
    // items are replaced atomically when its reply arrives, so a failed
    // update never leaves anything missing (F28).
    m_update_all = (type == TabSelection::All) || (type == TabSelection::TabsOnly);
    m_tabs_to_update.clear();
    switch (type) {
    case TabSelection::All:
        spdlog::debug("ItemsManagerWorker: updating all tabs and items.");
        break;
    case TabSelection::TabsOnly:
        spdlog::debug("ItemsManagerWorker: updating stash and character lists.");
        break;
    case TabSelection::Checked:
        // Use the buyout manager to determine which tabs are checked.
        spdlog::trace("ItemsManagerWorker: updating checked tabs.");
        for (auto const &tab : m_tabs) {
            if ((tab.IsValid()) && (m_buyout_manager.GetRefreshChecked(tab) == true)) {
                m_tabs_to_update.emplace(tab.id());
            }
        }
        break;
    case TabSelection::Selected:
        // Use the function argument to determine which tabs were selected.
        spdlog::trace("ItemsManagerWorker::Update() updating selected tabs");
        for (auto const &tab : locations) {
            if (tab.IsValid()) {
                m_tabs_to_update.emplace(tab.id());
            }
        }
        break;
    }

    // Fetch a list only if the selection needs it: always for full updates,
    // otherwise only for the location types present in the selection.
    if (m_update_all) {
        m_need_stash_list = true;
        m_need_character_list = true;
    } else {
        spdlog::debug("Updating {} tabs.", m_tabs_to_update.size());
        m_need_stash_list = false;
        m_need_character_list = false;
        for (auto const &tab : m_tabs) {
            if (m_tabs_to_update.count(tab.id()) == 0) {
                continue;
            }
            switch (tab.type()) {
            case ItemLocationType::STASH:
                m_need_stash_list = true;
                break;
            case ItemLocationType::CHARACTER:
                m_need_character_list = true;
                break;
            }
        }
    }

    m_has_stash_list = false;
    m_has_character_list = false;

    Refresh();
}

void ItemsManagerWorker::RemoveItemsFetchedBy(const QString &fetch_id)
{
    const size_t before = m_items.size();
    std::erase_if(m_items, [&fetch_id](const std::shared_ptr<Item> &item) {
        return item->location().fetch_id() == fetch_id;
    });
    spdlog::debug("ItemsManagerWorker: replacing {} items fetched by '{}'",
                  (before - m_items.size()),
                  fetch_id);
}

void ItemsManagerWorker::RebaseItemLocations(ItemLocationType type)
{
    // After a list reconciliation m_tabs carries fresh metadata, but
    // surviving items still embed the ItemLocation they were parsed with.
    // Rebase them so a renamed or moved tab is fresh everywhere the UI
    // reads a location (search buckets, headers, forum codes), not just in
    // the tab list.
    std::unordered_map<QString, const ItemLocation *> fresh_tabs;
    for (const auto &tab : m_tabs) {
        if (tab.type() == type) {
            fresh_tabs.emplace(tab.id(), &tab);
        }
    }
    for (auto &item : m_items) {
        const ItemLocation &location = item->location();
        if (location.type() != type) {
            continue;
        }
        const auto it = fresh_tabs.find(location.id());
        if (it != fresh_tabs.end()) {
            item->RebaseLocation(*it->second);
        }
    }
}

void ItemsManagerWorker::Refresh()
{
    spdlog::trace("Items Manager Worker: starting OAuth refresh");
    if (m_need_stash_list) {
        SubmitStashListRequest();
    } else if (m_need_character_list) {
        SubmitCharacterListRequest();
    }
    CheckUpdateFinished();
}

void ItemsManagerWorker::SubmitStashListRequest()
{
    const auto [endpoint, request] = poe::MakeStashListRequest(m_realm, m_league);
    spdlog::trace("ItemsManagerWorker::OAuthRefresh() requesting stash list: {}",
                  request.url().toString());
    auto reply = m_rate_limiter.Submit(endpoint, request);
    const int generation = m_update_generation;
    connect(reply,
            &RateLimitedReply::complete,
            this,
            [this, generation, reply](QNetworkReply *network_reply) {
                if (!DiscardIfStale(generation, reply, network_reply)) {
                    OnStashListReceived(network_reply);
                }
            });
}

void ItemsManagerWorker::SubmitCharacterListRequest()
{
    const auto [endpoint, request] = poe::MakeCharacterListRequest(m_realm);
    spdlog::trace("ItemsManagerWorker::OAuthRefresh() requesting character list: {}",
                  request.url().toString());
    auto reply = m_rate_limiter.Submit(endpoint, request);
    const int generation = m_update_generation;
    connect(reply,
            &RateLimitedReply::complete,
            this,
            [this, generation, reply](QNetworkReply *network_reply) {
                if (!DiscardIfStale(generation, reply, network_reply)) {
                    OnCharacterListReceived(network_reply);
                }
            });
}

void ItemsManagerWorker::ProcessTab(const poe::StashTab &tab, int &count)
{
    // The modern API documents stash ids as 10 hexadecimal digits; longer
    // ids were a legacy-API artifact. Warn if one ever shows up.
    if (tab.id.size() > 10) {
        spdlog::warn("Stash tab unique id is longer than 10 characters: {}", tab.name);
    }

    ItemLocation location(tab);

    // Guard against duplicates within one list (and re-processed children).
    if (m_tab_id_index.count(location.id()) > 0) {
        return;
    }

    // Add this tab with fresh metadata (name, colour, position).
    m_tabs.push_back(location);
    m_tab_id_index.insert(location.id());

    // Fetch this tab's contents if it is part of the update selection, or
    // if its contents were never successfully fetched. Keying on
    // contents-known rather than list membership keeps a new tab "new"
    // across a failed first fetch — list receipt alone (which persists
    // metadata) must not consume newness (F55).
    const bool selected = m_update_all || (m_tabs_to_update.count(location.id()) > 0)
                          || (m_contents_known.count(location.id()) == 0);
    if (m_update_tab_contents && selected) {
        ++count;
        const auto [endpoint, request] = poe::MakeStashRequest(m_realm, m_league, location.id());
        QueueRequest(endpoint, request, location);
    }

    // Process any children.
    if (tab.children) {
        // These children need to be added to the database, since they weren't included
        // in the original list.
        emit stashListReceived(*tab.children, m_realm, m_league);
        for (const auto &child : *tab.children) {
            ProcessTab(child, count);
        }
    }
}

void ItemsManagerWorker::OnStashListReceived(QNetworkReply *reply)
{
    spdlog::trace("ItemsManagerWorker::OnOAuthStashListReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    if (reply->error() != QNetworkReply::NoError) {
        spdlog::warn("Aborting update because there was an error fetching the stash list: {}",
                     reply->errorString());
        ++m_request_failures;
        m_has_stash_list = true;
        m_has_character_list = true;
        m_queue = {};
        CheckUpdateFinished();
        return;
    }
    const QByteArray bytes = reply->readAll();
    const auto result = json::readStashListWrapper(bytes);
    if (!result) {
        spdlog::error("ItemsManagerWorker: unable to parse stash list");
        ++m_request_failures;
        m_has_stash_list = true;
        m_has_character_list = true;
        m_queue = {};
        CheckUpdateFinished();
        return;
    }

    const auto &stashes = result->stashes;

    emit stashListReceived(stashes, m_realm, m_league);
    // Only a fresh top-level list may drive datastore deletion — never the
    // partial stashListReceived re-emits ProcessTab makes for folder
    // children (F53).
    emit stashListReplaced(stashes, m_realm, m_league);

    spdlog::debug("Received stash list, there are {} stash tabs", stashes.size());

    // The fresh list is authoritative for stash tabs: rebuild the stash
    // entries of m_tabs from it (fresh metadata, server-deleted tabs
    // dropped), keeping character entries untouched.
    std::vector<ItemLocation> kept_tabs;
    kept_tabs.reserve(m_tabs.size());
    for (auto const &tab : m_tabs) {
        if (tab.type() != ItemLocationType::STASH) {
            kept_tabs.push_back(tab);
        }
    }
    m_tabs = std::move(kept_tabs);
    m_tab_id_index.clear();
    for (auto const &tab : m_tabs) {
        m_tab_id_index.insert(tab.id());
    }

    // Queue stash tab requests.
    int tabs_requested = 0;
    for (const auto &tab : stashes) {
        // This will process tabs recursively.
        ProcessTab(tab, tabs_requested);
    }
    spdlog::debug("Requesting {} out of {} stash tabs", tabs_requested, stashes.size());

    // Drop items belonging to stash tabs the server no longer lists.
    const size_t before = m_items.size();
    std::erase_if(m_items, [this](const std::shared_ptr<Item> &item) {
        const ItemLocation &location = item->location();
        return (location.type() == ItemLocationType::STASH)
               && (m_tab_id_index.count(location.id()) == 0);
    });
    if (before > m_items.size()) {
        spdlog::debug("ItemsManagerWorker: dropped {} items from deleted stash tabs",
                      (before - m_items.size()));
    }

    m_has_stash_list = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (m_need_character_list && !m_has_character_list) {
        SubmitCharacterListRequest();
    } else {
        spdlog::trace("ItemsManagerWorker::OnOAuthStashListReceived() fetching items");
        FetchItems();
    }
    CheckUpdateFinished();
}

void ItemsManagerWorker::OnCharacterListReceived(QNetworkReply *reply)
{
    spdlog::trace("ItemsManagerWorker::OnOAuthCharacterListReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    spdlog::trace("OAuth character list received");
    if (reply->error() != QNetworkReply::NoError) {
        spdlog::warn("Aborting update because there was an error fetching the character list: {}",
                     reply->errorString());
        ++m_request_failures;
        m_has_stash_list = true;
        m_has_character_list = true;
        m_queue = {};
        CheckUpdateFinished();
        return;
    }
    const QByteArray bytes = reply->readAll();
    const auto result = json::readCharacterListWrapper(bytes);
    if (!result) {
        spdlog::error("ItemsManagerWorker: unable to parse character list");
        ++m_request_failures;
        m_has_stash_list = true;
        m_has_character_list = true;
        m_queue = {};
        CheckUpdateFinished();
        return;
    }

    const auto &characters = result->characters;

    emit characterListReceived(characters, m_realm);
    // The fresh list is authoritative for the datastore too (F53).
    emit characterListReplaced(characters, m_realm);

    // The fresh list is authoritative for characters: rebuild the character
    // entries of m_tabs from it, keeping stash entries untouched. This also
    // fixes the old skip-check, which compared character *names* against an
    // index keyed by character *ids* and so never matched (F48): a partial
    // update used to re-add and re-fetch every character in the league.
    std::vector<ItemLocation> kept_tabs;
    kept_tabs.reserve(m_tabs.size());
    for (auto const &tab : m_tabs) {
        if (tab.type() != ItemLocationType::CHARACTER) {
            kept_tabs.push_back(tab);
        }
    }
    m_tabs = std::move(kept_tabs);
    m_tab_id_index.clear();
    for (auto const &tab : m_tabs) {
        m_tab_id_index.insert(tab.id());
    }

    int requested_character_count = 0;

    for (auto &character : characters) {
        const QString name = character.name;
        const QString realm = character.realm;
        const QString league = character.league.value_or("");
        if (realm != m_realm) {
            spdlog::trace("Skipping {} because this character is not in realm {}", name, m_realm);
            continue;
        }
        if (league != m_league) {
            spdlog::trace("Skipping {} because this character is not in league {}", name, m_league);
            continue;
        }
        ItemLocation location(character, int(m_tabs.size()));
        if (m_tab_id_index.count(location.id()) > 0) {
            spdlog::trace("Skipping {} because it appears twice in the character list.", name);
            continue;
        }
        m_tabs.push_back(location);
        m_tab_id_index.insert(location.id());

        // Fetch this character's items if it is part of the update
        // selection, or if its contents were never successfully fetched
        // (same contents-known rule as ProcessTab, F55).
        const bool selected = m_update_all || (m_tabs_to_update.count(location.id()) > 0)
                              || (m_contents_known.count(location.id()) == 0);
        if (m_update_tab_contents && selected) {
            ++requested_character_count;
            const auto [endpoint, request] = poe::MakeCharacterRequest(m_realm, name);
            QueueRequest(endpoint, request, location);
        }
    }
    spdlog::debug("There are {} characters to update in '{}'", requested_character_count, m_league);

    // Drop items belonging to characters the server no longer lists.
    const size_t before = m_items.size();
    std::erase_if(m_items, [this](const std::shared_ptr<Item> &item) {
        const ItemLocation &location = item->location();
        return (location.type() == ItemLocationType::CHARACTER)
               && (m_tab_id_index.count(location.id()) == 0);
    });
    if (before > m_items.size()) {
        spdlog::debug("ItemsManagerWorker: dropped {} items from deleted characters",
                      (before - m_items.size()));
    }

    m_has_character_list = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!m_need_stash_list || m_has_stash_list) {
        spdlog::trace("ItemsManagerWorker::OnOAuthCharacterListReceived() fetching items");
        FetchItems();
    }
    CheckUpdateFinished();
}

void ItemsManagerWorker::OnStashReceived(QNetworkReply *reply, const ItemLocation &location)
{
    spdlog::trace("ItemsManagerWorker::OnOAuthStashReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    spdlog::trace("OAuth stash recieved");
    if (reply->error() != QNetworkReply::NoError) {
        spdlog::warn("Aborting update because there was an error fetching the stash: {}",
                     reply->errorString());
        ++m_request_failures;
        ++m_stashes_received;
        m_stashes_needed = m_stashes_received;
        m_characters_needed = m_characters_received;
        m_queue = {};
        SendStatusUpdate();
        CheckUpdateFinished();
        return;
    }
    const QByteArray bytes = reply->readAll();
    const auto result = json::readStashWrapper(bytes);
    if (!result) {
        spdlog::error("ItemsManagerWorker: unable to parse stash");
        ++m_request_failures;
        ++m_stashes_received;
        m_stashes_needed = m_stashes_received;
        m_characters_needed = m_characters_received;
        m_queue = {};
        SendStatusUpdate();
        CheckUpdateFinished();
        return;
    }

    const auto &stash = *result->stash;

    emit stashReceived(*result->stash, m_realm, m_league);

    // The fetch landed, so this tab is no longer "new" (F55).
    m_contents_known.insert(location.id());

    // Atomically replace whatever this request fetched last time: for a
    // normal tab that is the tab's items, for a child of a special tab it
    // is just that child's share of the parent location's items.
    RemoveItemsFetchedBy(location.fetch_id());

    if (stash.items) {
        const auto &items = *stash.items;
        if (items.size() > 0) {
            ParseItems(items, location);
        } else {
            spdlog::debug("Stash 'items' does not contain any items: {}", location.GetHeader());
        }
    } else {
        spdlog::debug("Stash does not have an 'items' array: {}", location.GetHeader());
    }

    ++m_stashes_received;
    SendStatusUpdate();

    bool get_children{false};

    if (stash.type == "MapStash") {
        get_children = m_settings.value("get_map_stashes", false).toBool();
    } else if (stash.type == "UniqueStash") {
        get_children = m_settings.value("get_unique_stashes", false).toBool();
    } else if (stash.type == "Folder") {
        get_children = true;
    }

    if (get_children && stash.children) {
        spdlog::debug("ItemsManagerWorker: getting children ({}) of {} '{}' ({})",
                      stash.children->size(),
                      stash.type,
                      stash.name,
                      stash.id);
        for (const auto &child : *stash.children) {
            // F49 tripwire: a child that is already a known tab may also
            // have been queued from the stash list, in which case this
            // request fetches it a second time (during a partial refresh a
            // known-but-unselected child is never queued from the list, so
            // this can be its only fetch). Warn (visible in the Event Log)
            // so a live account with folder tabs can confirm or refute the
            // finding.
            if (m_tab_id_index.count(child.id) > 0) {
                spdlog::warn("F49: child '{}' ({}) of {} '{}' is already a known tab; "
                             "its contents may be fetched twice this update",
                             child.name,
                             child.id,
                             stash.type,
                             stash.name);
            }
            const auto [endpoint,
                        request] = poe::MakeStashRequest(m_realm, m_league, stash.id, child.id);
            // The child's items display under the parent's location, but
            // they are fetched (and atomically replaced) by the child's id.
            ItemLocation child_location = location;
            child_location.setFetchId(child.id);
            QueueRequest(endpoint, request, child_location);
            ++m_stashes_needed;
        }
    }

    // The parent's reply is authoritative for its children: drop cached
    // items that display under this tab but were fetched from a child it
    // no longer lists — or whose fetching is disabled in the settings.
    // Their fetch ids are never re-fetched, so nothing else would ever
    // replace or remove them, not even a full refresh.
    if (location.fetch_id() == location.id()) {
        std::set<QString> expected_fetch_ids{location.id()};
        QStringList child_ids;
        if (get_children && stash.children) {
            for (const auto &child : *stash.children) {
                expected_fetch_ids.insert(child.id);
                child_ids.append(child.id);
            }
        }
        // Mirror the reconcile into the datastore, but only for Map/Unique
        // parents — their child rows exist solely under parent replies
        // (F53). Folder children are governed by the top-level list, and a
        // folder's own reply carries no children in the live API (F49), so
        // keying off a folder reply would wipe its legitimate child rows.
        if ((stash.type == "MapStash") || (stash.type == "UniqueStash")) {
            emit stashChildrenReplaced(location.id(), child_ids, m_realm, m_league);
        }
        const size_t before = m_items.size();
        std::erase_if(m_items, [&](const std::shared_ptr<Item> &item) {
            const ItemLocation &item_location = item->location();
            return (item_location.type() == ItemLocationType::STASH)
                   && (item_location.id() == location.id())
                   && (expected_fetch_ids.count(item_location.fetch_id()) == 0);
        });
        if (before > m_items.size()) {
            spdlog::debug("ItemsManagerWorker: dropped {} ghost child items from {}",
                          (before - m_items.size()),
                          location.GetHeader());
        }
    }

    SubmitNextItemRequest();
    CheckUpdateFinished();
}

void ItemsManagerWorker::OnCharacterReceived(QNetworkReply *reply, const ItemLocation &location)
{
    spdlog::trace("ItemsManagerWorker::OnOAuthCharacterReceived() entered");

    auto sender = qobject_cast<RateLimitedReply *>(QObject::sender());
    sender->deleteLater();
    reply->deleteLater();

    spdlog::trace("OAuth character recieved");
    if (reply->error() != QNetworkReply::NoError) {
        spdlog::warn("Aborting update because there was an error fetching the character: {}",
                     reply->errorString());
        ++m_request_failures;
        ++m_characters_received;
        m_stashes_needed = m_stashes_received;
        m_characters_needed = m_characters_received;
        m_queue = {};
        SendStatusUpdate();
        CheckUpdateFinished();
        return;
    }
    const QByteArray bytes = reply->readAll();
    const auto result = json::readCharacterWrapper(bytes);
    if (!result) {
        spdlog::error("ItemsManagerWorker: unable to parse character");
        ++m_request_failures;
        ++m_characters_received;
        m_stashes_needed = m_stashes_received;
        m_characters_needed = m_characters_received;
        m_queue = {};
        SendStatusUpdate();
        CheckUpdateFinished();
        return;
    } else if (!result->character) {
        spdlog::error("ItemsManagerWorker: character is empty");
        ++m_request_failures;
        ++m_characters_received;
        m_stashes_needed = m_stashes_received;
        m_characters_needed = m_characters_received;
        m_queue = {};
        SendStatusUpdate();
        CheckUpdateFinished();
        return;
    }

    const auto &character = *result->character;

    emit characterReceived(character, m_realm);

    // The fetch landed, so this character is no longer "new" (F55).
    m_contents_known.insert(location.id());

    // Atomically replace this character's items.
    RemoveItemsFetchedBy(location.fetch_id());

    const auto collections = {character.equipment,
                              character.inventory,
                              character.rucksack,
                              character.jewels};

    for (const auto &items : collections) {
        if (items) {
            ParseItems(*items, location);
        }
    }

    ++m_characters_received;
    SendStatusUpdate();

    SubmitNextItemRequest();
    CheckUpdateFinished();
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
}

void ItemsManagerWorker::FetchItems()
{
    spdlog::trace("ItemsManagerWorker::FetchItems() entered");

    if (!m_update_tab_contents) {
        spdlog::trace("ItemsManagerWorker: not fetching items.");
        CheckUpdateFinished();
        return;
    }

    m_stashes_needed = 0;
    m_stashes_received = 0;

    m_characters_needed = 0;
    m_characters_received = 0;

    std::queue<ItemsRequest> requests = m_queue;
    QString tab_titles;
    while (!requests.empty()) {
        const ItemsRequest request = requests.front();
        requests.pop();
        switch (request.location.type()) {
        case ItemLocationType::STASH:
            ++m_stashes_needed;
            break;
        case ItemLocationType::CHARACTER:
            ++m_characters_needed;
            break;
        }
        tab_titles += request.location.GetHeader() + " ";
    }

    SendStatusUpdate();

    spdlog::debug("Requested {} stashes and {} characters.", m_stashes_needed, m_characters_needed);
    spdlog::debug("Tab titles: {}", tab_titles);

    SubmitNextItemRequest();
    CheckUpdateFinished();
}

void ItemsManagerWorker::SubmitNextItemRequest()
{
    if (m_state != WorkerState::Updating || m_request_failures > 0 || m_queue.empty()) {
        return;
    }

    ItemsRequest request = m_queue.front();
    m_queue.pop();

    const ItemLocation location = request.location;
    std::function<void(QNetworkReply *)> callback;

    switch (location.type()) {
    case ItemLocationType::STASH:
        callback = [=, this](QNetworkReply *reply) { OnStashReceived(reply, location); };
        break;
    case ItemLocationType::CHARACTER:
        callback = [=, this](QNetworkReply *reply) { OnCharacterReceived(reply, location); };
        break;
    }

    auto reply = m_rate_limiter.Submit(request.endpoint, request.network_request);
    const int generation = m_update_generation;
    connect(reply,
            &RateLimitedReply::complete,
            this,
            [this, generation, reply, callback](QNetworkReply *network_reply) {
                if (!DiscardIfStale(generation, reply, network_reply)) {
                    callback(network_reply);
                }
            });
}

bool ItemsManagerWorker::DiscardIfStale(int generation,
                                        RateLimitedReply *reply,
                                        QNetworkReply *network_reply)
{
    // A reply is stale if it belongs to an earlier update, or if no update is
    // running at all: every request is submitted during an update, a
    // successful finish requires all of them to have been counted, and a
    // failed update abandons whatever is still in flight (F28).
    if ((m_state == WorkerState::Updating) && (generation == m_update_generation)) {
        return false;
    }
    spdlog::debug("ItemsManagerWorker: discarding a stale reply from update {} (current is {}): {}",
                  generation,
                  m_update_generation,
                  network_reply->url().toString());
    reply->deleteLater();
    network_reply->deleteLater();
    return true;
}

void ItemsManagerWorker::SendStatusUpdate()
{
    spdlog::trace("ItemsManagerWorker::SendStatusUpdate() entered");

    QString message;
    if ((m_stashes_needed > 0) && (m_characters_needed > 0)) {
        message = QString("Receieved %1/%2 stash tabs and %3/%4 character locations")
                      .arg(QString::number(m_stashes_received),
                           QString::number(m_stashes_needed),
                           QString::number(m_characters_received),
                           QString::number(m_characters_needed));
    } else if (m_stashes_needed > 0) {
        message = QString("Received %1/%2 stash tabs")
                      .arg(QString::number(m_stashes_received), QString::number(m_stashes_needed));
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

void ItemsManagerWorker::ParseItems(const std::vector<poe::Item> &items,
                                    const ItemLocation &base_location)
{
    for (auto &item : items) {
        const ItemLocation location = base_location.getItemLocation(item);
        m_items.push_back(std::make_shared<Item>(item, location));
        if (item.socketedItems) {
            ParseItems(*item.socketedItems, location);
        }
    }
}

void ItemsManagerWorker::CheckUpdateFinished()
{
    if (m_state != WorkerState::Updating) {
        return;
    }

    const bool lists_received = (!m_need_stash_list || m_has_stash_list)
                                && (!m_need_character_list || m_has_character_list);
    const bool items_received = (m_stashes_received == m_stashes_needed)
                                && (m_characters_received == m_characters_needed);
    if (!lists_received || !items_received) {
        return;
    }

    if (m_request_failures == 0) {
        spdlog::trace("ItemsManagerWorker::CheckUpdateFinished() finishing update");
        FinishUpdate();
    } else {
        emit StatusUpdate(ProgramState::Ready,
                          QString("Update failed: %1 requests failed")
                              .arg(QString::number(m_request_failures)));
        m_state = WorkerState::Idle;
        spdlog::debug("Update failed.");
    }
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

    // Rebase surviving items' locations onto the fresh tab metadata only
    // now that the update has succeeded: the emitted Items share Item
    // objects with ItemsManager and the UI, so rebasing any earlier would
    // mutate the already-published snapshot mid-update — and a terminal
    // failure would leave it mutated, with no emit to rebuild the search
    // buckets around the new metadata.
    RebaseItemLocations(ItemLocationType::STASH);
    RebaseItemLocations(ItemLocationType::CHARACTER);

    // Sort tabs.
    std::sort(begin(m_tabs), end(m_tabs));

    // Sort items.
    std::sort(begin(m_items),
              end(m_items),
              [](const std::shared_ptr<Item> &a, const std::shared_ptr<Item> &b) {
                  return *a < *b;
              });

    // Let everyone know the update is done.
    spdlog::trace("ItemsManagerWorker::FinishUpdate() emitting ItemsRefreshed");
    emit ItemsRefreshed(m_items, m_tabs, false);

    m_state = WorkerState::Idle;
    spdlog::debug("Update finished.");
}
