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

#include "application.h"
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
#include "ui/mainwindow.h"
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
    , m_first_stash_request_index(-1)
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
    m_updating = true;
    spdlog::trace("ItemsManagerWorker: league = {}", m_league);
    spdlog::trace("ItemsManagerWorker: account = {}", m_account);
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
    // Create a separate thread to load the items, which allows the UI to
    // update the status bar while items are being parsed. This operation
    // can take tens of seconds or longer depending on the nubmer of tabs
    // and items.
    QThread *parser = QThread::create([=, this]() { ParseItemMods(); });
    parser->start();
}

void ItemsManagerWorker::ParseItemMods()
{
    spdlog::trace("ItemsManagerWorker::ParseItemMods() entered");

    // Create a datastore to get a connection to the database.
    // NOTE: we only need read access, but this isn't enforced or checked.
    const QFileInfo info(m_settings.fileName());
    QDir data_dir{info.absolutePath() + "/data/"};
    UserStore userstore{data_dir, m_account};

    // Get cached characters and stash tabs.
    const auto stashes = userstore.stashes().getStashList(m_realm, m_league);
    const auto characters = userstore.characters().getCharacterList(m_realm);

    m_tabs.clear();
    m_tabs.reserve(stashes.size() + characters.size());
    for (const auto &stash : stashes) {
        if (stash.parent) {
            // Do not add substashes from these special stashes.
            if ((stash.type == "DivinationCardStash") || (stash.type == "FlaskStash")
                || (stash.type == "MapStash") || (stash.type == "UniqueStash")) {
                continue;
            }
        }
        m_tabs.emplace_back(stash);
    }
    for (const auto &character : characters) {
        // The character list has all characters for all leagues, so
        // we have to look for just the league we care about.
        if (character.league == m_league) {
            m_tabs.emplace_back(character, m_tabs.size());
        }
    }
    m_tabs.shrink_to_fit();

    // Save location ids.
    spdlog::trace("ItemsManagerWorker::ParseItemMods() saving location ids");
    m_tab_id_index.clear();
    for (const auto &tab : m_tabs) {
        m_tab_id_index.emplace(tab.get_tab_uniq_id());
    }

    // Build the signature vector.
    spdlog::trace("ItemsManangerWorker::ParseItemMods() building tabs signature");
    m_tabs_signature.clear();
    m_tabs_signature.reserve(m_tabs.size());
    for (const auto &tab : m_tabs) {
        const QString tab_name = tab.get_tab_label();
        const QString tab_id = QString::number(tab.get_tab_id());
        m_tabs_signature.emplace_back(tab_name, tab_id);
    }

    // Get cached items
    m_items.clear();
    spdlog::trace("ItemsManagerWorker::ParseItemMods() getting cached items");
    for (size_t i = 0; i < m_tabs.size(); i++) {
        const ItemLocation &tab = m_tabs[i];
        switch (tab.get_type()) {
        case ItemLocationType::STASH: {
            const auto stash = userstore.stashes().getStash(tab.get_tab_uniq_id(),
                                                            m_realm,
                                                            m_league);
            if (stash) {
                ParseItems(*stash, tab);
                // We have to make separate requests for the special tabs.
                // This is messy and needs to be reworked.
                if (stash->children) {
                    if ((stash->type == "DivinationCardStash") || (stash->type == "FlaskStash")
                        || (stash->type == "MapStash") || (stash->type == "UniqueStash")) {
                        for (const auto &child : *stash->children) {
                            const auto child_stash = userstore.stashes().getStash(child.id,
                                                                                  m_realm,
                                                                                  m_league);
                            if (child_stash->items) {
                                ParseItems(*child_stash, tab);
                            }
                        }
                    }
                }
            }
        } break;
        case ItemLocationType::CHARACTER: {
            const auto character = userstore.characters().getCharacter(tab.get_character(), m_realm);
            if (character) {
                ParseItems(*character, tab);
            }
        } break;
        }
        emit StatusUpdate(ProgramState::Initializing,
                          QString("Parsing items in %1/%2 tabs").arg(i).arg(m_tabs.size()));
    }
    emit StatusUpdate(ProgramState::Ready,
                      QString("Parsed %1 items from %2 tabs").arg(m_items.size()).arg(m_tabs.size()));

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

void ItemsManagerWorker::ParseItems(const poe::StashTab &stash, ItemLocation location)
{
    if (stash.items) {
        const auto &items = *stash.items;
        m_items.reserve(m_items.size() + items.size());
        spdlog::debug("ItemManagerWorker: loading {} items from stash {} ({})",
                      items.size(),
                      stash.id,
                      stash.name);
        for (const auto &item : items) {
            m_items.push_back(std::make_shared<Item>(item, location));
        }
    }
}

void ItemsManagerWorker::ParseItems(const poe::Character &character, ItemLocation location)
{
    const std::array collections{std::make_pair("equipment", character.equipment),
                                 std::make_pair("inventory", character.inventory),
                                 std::make_pair("rucksack", character.rucksack),
                                 std::make_pair("jewels", character.jewels)};
    for (const auto &[name, items] : collections) {
        if (items) {
            m_items.reserve(m_items.size() + items->size());
            spdlog::debug("ItemManagerWorker: loading {} items from character {} {} ({})",
                          items->size(),
                          name,
                          character.id,
                          character.name);
            for (const auto &item : *items) {
                m_items.push_back(std::make_shared<Item>(item, location));
            }
        }
    }
}

void ItemsManagerWorker::Update(TabSelection type, const std::vector<ItemLocation> &locations)
{
    if (!isInitialized()) {
        // tell ItemsManagerWorker to run an Update() after it's finished updating mods
        UpdateRequest(type, locations);
        spdlog::debug("Update deferred until item mods parsing is complete");
        QMessageBox::information(
            nullptr,
            "Acquisition",
            "This items worker is still initializing, but an update request has been queued.",
            QMessageBox::Ok,
            QMessageBox::Ok);
        return;
    }

    if (isUpdating()) {
        spdlog::warn("ItemsManagerWorker: update called while updating");
        QMessageBox::information(nullptr,
                                 "Acquisition",
                                 "An update is already in progress.",
                                 QMessageBox::Ok,
                                 QMessageBox::Ok);
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

    OAuthRefresh();
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

void ItemsManagerWorker::OAuthRefresh()
{
    spdlog::trace("Items Manager Worker: starting OAuth refresh");
    if (m_need_stash_list) {
        const auto [endpoint, request] = poe::MakeStashListRequest(m_realm, m_league);
        spdlog::trace("ItemsManagerWorker::OAuthRefresh() requesting stash list: {}",
                      request.url().toString());
        auto reply = m_rate_limiter.Submit(endpoint, request);
        connect(reply, &RateLimitedReply::complete, this, &ItemsManagerWorker::OnStashListReceived);
    }
    if (m_need_character_list) {
        const auto [endpoint, request] = poe::MakeCharacterListRequest(m_realm);
        spdlog::trace("ItemsManagerWorker::OAuthRefresh() requesting character list: {}",
                      request.url().toString());
        auto submit = m_rate_limiter.Submit(endpoint, request);
        connect(submit,
                &RateLimitedReply::complete,
                this,
                &ItemsManagerWorker::OnCharacterListReceived);
    }
}

void ItemsManagerWorker::ProcessTab(const poe::StashTab &tab, int &count)
{
    // Get tab info.

    // The unique id for stash tabs returned from the legacy API
    // need to be trimmed to 10 characters.
    QString tab_id = tab.id;
    if (tab_id.size() > 10) {
        spdlog::debug("Trimming tab unique id: {}", tab.name);
        tab_id = tab_id.first(10);
    }

    if (m_tab_id_index.count(tab_id) == 0) {
        // Create this tab.
        ItemLocation location(tab);

        // Add this tab.
        m_tabs.push_back(location);
        m_tab_id_index.insert(tab_id);

        // Submit a request for this tab.
        if (m_update_tab_contents) {
            ++count;
            const auto uid = location.get_tab_uniq_id();
            const auto [endpoint, request] = poe::MakeStashRequest(m_realm, m_league, uid);
            QueueRequest(endpoint, request, location);
        }

        // Process any children.
        if (tab.children) {
            for (const auto &child : *tab.children) {
                ProcessTab(child, count);
            }
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
        m_updating = false;
        return;
    }
    const QByteArray bytes = reply->readAll();
    const std::string_view sv{bytes.constBegin(), size_t(bytes.size())};

    const auto result = glz::read_json<poe::StashListWrapper>(sv);
    if (!result) {
        const auto msg = glz::format_error(result.error(), sv);
        spdlog::error("ItemsManagerWorker: unable to parse stash list: {}", msg);
        m_updating = false;
        return;
    }

    const auto &stashes = result->stashes;

    emit stashListReceived(stashes, m_realm, m_league);

    spdlog::debug("Received stash list, there are {} stash tabs", stashes.size());

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
                const auto [endpoint, request] = poe::MakeStashRequest(m_realm,
                                                                       m_league,
                                                                       tab.get_tab_uniq_id());
                QueueRequest(endpoint, request, tab);
            }
        }
    }

    // Queue stash tab requests.
    int tabs_requested = 0;
    for (const auto &tab : stashes) {
        // This will process tabs recursively.
        ProcessTab(tab, tabs_requested);
    }
    spdlog::debug("Requesting {} out of {} stash tabs", tabs_requested, stashes.size());

    m_has_stash_list = true;

    // Check to see if we can start sending queued requests to fetch items yet.
    if (!m_need_character_list || m_has_character_list) {
        spdlog::trace("ItemsManagerWorker::OnOAuthStashListReceived() fetching items");
        FetchItems();
    }
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
        m_updating = false;
        return;
    }
    const QByteArray bytes = reply->readAll();
    const std::string_view sv{bytes.constBegin(), size_t(bytes.size())};

    const auto result = glz::read_json<poe::CharacterListWrapper>(sv);
    if (!result) {
        const auto msg = glz::format_error(result.error(), sv);
        spdlog::error("ItemsManagerWorker: unable to parse character list: {}", msg);
        m_updating = false;
        return;
    }

    const auto &characters = result->characters;

    emit characterListReceived(characters, m_realm);

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
        if (m_tab_id_index.count(name) > 0) {
            spdlog::trace("Skipping {} because this item is not being refreshed.", name);
            continue;
        }
        ItemLocation location(character, int(m_tabs.size()));
        m_tabs.push_back(location);

        // Queue character request if needed.
        if (m_update_tab_contents) {
            ++requested_character_count;
            const auto [endpoint, request] = poe::MakeCharacterRequest(m_realm, name);
            QueueRequest(endpoint, request, location);
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
        m_updating = false;
        return;
    }
    const QByteArray bytes = reply->readAll();
    const std::string_view sv{bytes.constBegin(), size_t(bytes.size())};

    const auto result = glz::read_json<poe::StashWrapper>(sv);
    if (!result) {
        const auto msg = glz::format_error(result.error(), sv);
        spdlog::error("ItemsManagerWorker: unable to parse stash: {}", msg);
        return;
    } else if (!result->stash) {
        spdlog::error("ItemsManagerWorker: emtpy stash repsonse received");
        return;
    }

    const auto &stash = *result->stash;

    emit stashReceived(*result->stash, m_realm, m_league);

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

    if (stash.type == "DivinationCardStash") {
        get_children = m_settings.value("get_divination_stashes", false).toBool();
    } else if (stash.type == "FlaskStash") {
        get_children = m_settings.value("get_flask_stashes", false).toBool();
    } else if (stash.type == "MapStash") {
        get_children = m_settings.value("get_map_stashes", false).toBool();
    } else if (stash.type == "UniqueStash") {
        get_children = m_settings.value("get_unique_stashes", false).toBool();
    }

    if (get_children && stash.children) {
        spdlog::debug("ItemsManagerWorker: getting {} children of {} '{}' ({})",
                      stash.children->size(),
                      stash.type,
                      stash.name,
                      stash.id);
        for (const auto &child : *stash.children) {
            const auto [endpoint, request] = poe::MakeStashRequest(m_realm,
                                                                   m_league,
                                                                   stash.id,
                                                                   child.id);
            auto submit = m_rate_limiter.Submit(endpoint, request);
            connect(submit, &RateLimitedReply::complete, this, [=, this](QNetworkReply *reply) {
                OnStashReceived(reply, location);
            });
            ++m_stashes_needed;
        }
    }

    if ((m_stashes_received == m_stashes_needed) && (m_characters_received == m_characters_needed)
        && !m_cancel_update) {
        spdlog::trace("ItemsManagerWorker::OnOAuthStashReceived() finishing update");
        FinishUpdate();
    }
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
        m_updating = false;
        return;
    }
    const QByteArray bytes = reply->readAll();
    const size_t nbytes = static_cast<size_t>(bytes.size());
    const std::string_view sv{bytes.constBegin(), nbytes};

    const auto result = glz::read_json<poe::CharacterWrapper>(sv);
    if (!result) {
        const auto msg = glz::format_error(result.error(), sv);
        spdlog::error("ItemsManagerWorker: unable to parse character: {}", msg);
        return;
    } else if (!result->character) {
        spdlog::error("ItemsManagerWorker: empty character received");
        return;
    }

    const auto &character = *result->character;

    emit characterReceived(character, m_realm);

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

    if ((m_stashes_received == m_stashes_needed) && (m_characters_received == m_characters_needed)
        && !m_cancel_update) {
        spdlog::trace("ItemsManagerWorker::OnOAuthCharacterReceived() finishing update");
        FinishUpdate();
    }
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

        switch (location.get_type()) {
        case ItemLocationType::STASH:
            callback = [=, this](QNetworkReply *reply) { OnStashReceived(reply, location); };
            ++m_stashes_needed;
            break;
        case ItemLocationType::CHARACTER:
            callback = [=, this](QNetworkReply *reply) { OnCharacterReceived(reply, location); };
            ++m_characters_needed;
            break;
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

void ItemsManagerWorker::ParseItems(const std::vector<poe::Item> &items,
                                    const ItemLocation &base_location)
{
    ItemLocation location = base_location;

    for (auto &item : items) {
        // Make sure location data from the item like x and y is brought over to the location object.
        location.FromItem(item);
        //location.ToItemJson(&item, alloc);
        m_items.push_back(std::make_shared<Item>(item, location));
        if (item.socketedItems) {
            location.set_socketed(true);
            ParseItems(*item.socketedItems, location);
            location.set_socketed(false);
        }
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

    m_updating = false;
    spdlog::debug("Update finished.");
}

ItemsManagerWorker::TabsSignatureVector ItemsManagerWorker::CreateTabsSignatureVector(
    const std::vector<poe::StashTab> &tabs)
{
    spdlog::trace("ItemsManagerWorker::CreateTabsSignatureVector() entered");

    TabsSignatureVector signature;
    for (auto &tab : tabs) {
        signature.emplace_back(tab.name, tab.id);
    }
    return signature;
}
