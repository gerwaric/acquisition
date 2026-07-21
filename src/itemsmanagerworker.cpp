// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "itemsmanagerworker.h"

#include <QCoroFuture>

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
#include "poe/poeapiclient.h"
#include "poe/types/character.h"
#include "poe/types/item.h"
#include "poe/types/stashtab.h"
#include "repoe/repoe.h"
#include "util/json_readers.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/util.h"

namespace {

    // The Internal FetchError a per-fetch task's catch-all synthesizes from an
    // exceptional future (R5-1): it routes an unexpected throw through the same
    // failure branch a value-level failure takes, so the update aborts cleanly
    // instead of wedging on a completion that never counts.
    RateLimit::FetchError MakeInternalError(const QString &message)
    {
        RateLimit::FetchError error;
        error.kind = RateLimit::FetchError::Kind::Internal;
        error.message = message;
        return error;
    }

} // namespace

ItemsManagerWorker::ItemsManagerWorker(QSettings &settings,
                                       BuyoutManager &buyout_manager,
                                       PoeApiClient &api,
                                       QObject *parent)
    : QObject(parent)
    , m_settings(settings)
    , m_buyout_manager(buyout_manager)
    , m_api(api)
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
    // Read on the main thread: the parser thread must not touch the shared
    // QSettings instance, which the UI writes concurrently.
    const bool get_map_stashes = m_settings.value("get_map_stashes", false).toBool();
    const bool get_unique_stashes = m_settings.value("get_unique_stashes", false).toBool();
    m_shutdown.store(false);
    m_parser_thread = QThread::create([this, dataDir, get_map_stashes, get_unique_stashes]() {
        auto result = ParseCachedItems(dataDir, get_map_stashes, get_unique_stashes);
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

ParseResult ItemsManagerWorker::ParseCachedItems(const QString &dataDir,
                                                 bool get_map_stashes,
                                                 bool get_unique_stashes) const
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
    std::vector<std::pair<QString, QStringList>> required_children;
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
        // A Map/Unique parent's saved reply records the children a fetch of
        // it implies; collect them so completeness can be checked below.
        const bool children_enabled = ((stash->type == "MapStash") && get_map_stashes)
                                      || ((stash->type == "UniqueStash") && get_unique_stashes);
        if (children_enabled && stash->children && !stash->children->empty()) {
            QStringList child_ids;
            for (const auto &child : *stash->children) {
                child_ids.append(child.id);
            }
            required_children.emplace_back(id, child_ids);
        }
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

    // A Map/Unique parent's contents are complete only if every enabled
    // child fetch also landed: a cached parent with a missing child row
    // must stay "new" so the next content update refetches it and
    // re-queues its children — they never appear in a top-level list, so
    // nothing else would retry them (F55).
    for (const auto &[parent_id, child_ids] : required_children) {
        for (const auto &child_id : child_ids) {
            if (result.contents_known.count(child_id) == 0) {
                result.contents_known.erase(parent_id);
                break;
            }
        }
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
    // A fresh stop_source per update (D2): every facade call below takes this
    // update's token, and a terminal failure request_stop()s it. The previous
    // update's source (stopped or not) is dropped here, so the new update's
    // calls carry a distinct, unstopped token — which is what makes the token
    // the update's sole identity under the post-await check (W-IDENTITY).
    m_stop_source = std::stop_source{};
    m_request_failures = 0;
    m_update_tab_contents = (type != TabSelection::TabsOnly);

    // Reset the completion counters here, once per update: this is their only
    // reset. Each lane's LaunchContent() accumulates into them (never resets),
    // so a fresh update must not inherit a previous one's totals.
    m_stashes_needed = 0;
    m_stashes_received = 0;
    m_characters_needed = 0;
    m_characters_received = 0;

    // Counts left over from a failed update are stale: this update launches
    // its own child fetches afresh.
    m_pending_children.clear();

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

    // Every per-update counter and selection flag above is initialized before
    // the root task launches its first fetch: a ready/fail-fast future can run
    // its continuation synchronously during launch (S1-6/IR2), so nothing it
    // touches may still be uninitialized. Launch the owned root task last.
    m_update_task = RunUpdate();
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

QCoro::Task<> ItemsManagerWorker::RunUpdate()
{
    // The root update task (D6): it launches the update's required list(s) and
    // then reconciles the terminal state; the per-fetch tasks self-drive from
    // there. 5B keeps list submission serial — the stash list first, the
    // character list from its handler — exactly as the old Refresh() did; 5C
    // launches both without awaiting one another. This body never co_awaits,
    // so it runs synchronously to completion on assignment; the co_return
    // makes it a coroutine so the root catch-all and the owned handle exist.
    //
    // No exception ever escapes a root coroutine (R5-1): a throw in
    // orchestration itself aborts and finalizes the update rather than
    // unwinding into the caller (ItemsManager / the UI).
    try {
        spdlog::trace("ItemsManagerWorker: starting the update");
        // Root orchestration fault site (test-only; no-op in production).
        FireFaultHook();
        // Launch every required list without awaiting one another (D6): with
        // ordinary pending futures both are concurrently outstanding, resolving
        // F56's one-lane serialization. Each submission is made unconditionally
        // even if the first ran a ready/fail-fast handler inline that already
        // stopped the token — the second list simply carries that stopped token
        // and completes through the ordinary cancellation path. Every per-update
        // counter was initialized in Update() before this launch (IR2/S1-6).
        if (m_need_stash_list) {
            SubmitStashListRequest();
        }
        if (m_need_character_list) {
            SubmitCharacterListRequest();
        }
        CheckUpdateFinished();
    } catch (const std::exception &e) {
        spdlog::error("ItemsManagerWorker: the update orchestration threw: {}", e.what());
        AbortUpdate();
    } catch (...) {
        spdlog::error("ItemsManagerWorker: the update orchestration threw an unknown exception");
        AbortUpdate();
    }
    co_return;
}

void ItemsManagerWorker::AbortUpdate()
{
    // The single, idempotent terminal transition for a failed update. Every
    // failure — a list error, a content error, an exceptional future, a throw in
    // orchestration or a handler — routes here. It is deliberately INDEPENDENT of
    // the completion counters: the first terminal failure returns the worker to
    // idle immediately (D6), abandoning in-flight siblings as stopped stragglers
    // that resolve Canceled later. It must never rewrite the counters to force
    // CheckUpdateFinished()'s success-equality — doing so drove `needed` backward
    // below `received` and made the reported progress non-monotonic (P-STATUS).
    // Success uses the counter equality in CheckUpdateFinished(); failure uses
    // this direct path, so the two never contend.
    if (m_state != WorkerState::Updating) {
        // Already terminal: a later straggler or a second failure must not emit a
        // second Ready/failure transition (P-FAILURE) or reset a live update.
        return;
    }
    m_stop_source.request_stop();
    // A failed update aborts on the first failure, so the count is at least one;
    // a handler that recorded nothing still yields a truthful "1 failed".
    if (m_request_failures == 0) {
        ++m_request_failures;
    }
    emit StatusUpdate(ProgramState::Ready,
                      QString("Update failed: %1 requests failed")
                          .arg(QString::number(m_request_failures)));
    m_state = WorkerState::Idle;
    spdlog::debug("Update failed.");
}

void ItemsManagerWorker::StopUpdateForFailure()
{
    m_stop_source.request_stop();
    FireFaultHook();
}

void ItemsManagerWorker::FireFaultHook()
{
    if (m_fault_hook) {
        // One-shot: consume before invoking so a re-entrant path cannot fire it
        // again, and so it never fires on the recovery update.
        auto hook = std::move(m_fault_hook);
        m_fault_hook = nullptr;
        hook();
    }
}

void ItemsManagerWorker::SubmitStashListRequest()
{
    spdlog::trace("ItemsManagerWorker: requesting the stash list");
    m_fetch_tasks.push_back(FetchStashList(m_stop_source.get_token()));
}

void ItemsManagerWorker::SubmitCharacterListRequest()
{
    spdlog::trace("ItemsManagerWorker: requesting the character list");
    m_fetch_tasks.push_back(FetchCharacterList(m_stop_source.get_token()));
}

// Each per-fetch task wraps its ENTIRE body in a catch-all (R5-1): nothing —
// not an exceptional facade future, not the OnX handler, not a nested launch,
// not a connected persistence slot — may escape into the unawaited task, where
// it would vanish silently (S1-8) and wedge the update on a completion that
// never counts. There are two layers:
//   * the inner try converts an exceptional facade future into an ordinary
//     Internal failure value, so it flows through the handler's failure branch;
//   * the outer try contains anything the handler (or a launch it triggers)
//     throws, and forces a terminal AbortUpdate() so the update cannot wedge.
// The post-await identity check (IR2) gates the handler on the token alone: a
// future can resolve an instant before request_stop(), and its consumer must
// not touch state that now belongs to a later update. The token IS the update's
// identity — a straggler from a failed update carries that update's stopped
// token, so `!token.stop_requested()` discards it (W-IDENTITY). The sweep is
// scheduled unconditionally on every exit so no completed handle leaks.
QCoro::Task<> ItemsManagerWorker::FetchStashList(std::stop_token token)
{
    try {
        Result<poe::StashListWrapper> result;
        try {
            // Move the payload out (S1-7): the worker is the single consumer.
            auto future = m_api.listStashes(m_realm, m_league, token);
            result = co_await qCoro(future).takeResult();
        } catch (...) {
            result = std::unexpected(MakeInternalError("the stash list fetch threw"));
        }
        if (!token.stop_requested()) {
            OnStashListReceived(result);
        }
    } catch (...) {
        spdlog::error("ItemsManagerWorker: a stash list continuation threw; aborting the update");
        // A handler that reached this catch ran because the token was live at the
        // post-await gate, so it belongs to the active update — even if its own
        // failure branch stopped the token before throwing. Abort it. A resumed
        // straggler is gated out above and never enters the handler, so it never
        // reaches here; and AbortUpdate() guards on WorkerState and is idempotent,
        // so an already-terminal update is left untouched.
        AbortUpdate();
    }
    ScheduleSweep();
}

QCoro::Task<> ItemsManagerWorker::FetchCharacterList(std::stop_token token)
{
    try {
        Result<poe::CharacterListWrapper> result;
        try {
            auto future = m_api.listCharacters(m_realm, token);
            result = co_await qCoro(future).takeResult();
        } catch (...) {
            result = std::unexpected(MakeInternalError("the character list fetch threw"));
        }
        if (!token.stop_requested()) {
            OnCharacterListReceived(result);
        }
    } catch (...) {
        spdlog::error(
            "ItemsManagerWorker: a character list continuation threw; aborting the update");
        AbortUpdate();
    }
    ScheduleSweep();
}

QCoro::Task<> ItemsManagerWorker::FetchStash(ItemLocation location,
                                             QString stash_id,
                                             QString substash_id,
                                             std::stop_token token)
{
    try {
        Result<poe::StashWrapper> result;
        try {
            auto future = m_api.getStash(m_realm, m_league, stash_id, substash_id, token);
            result = co_await qCoro(future).takeResult();
        } catch (...) {
            result = std::unexpected(MakeInternalError("the stash fetch threw"));
        }
        if (!token.stop_requested()) {
            OnStashReceived(result, location);
        }
    } catch (...) {
        spdlog::error("ItemsManagerWorker: a stash continuation threw; aborting the update");
        AbortUpdate();
    }
    ScheduleSweep();
}

QCoro::Task<> ItemsManagerWorker::FetchCharacter(ItemLocation location,
                                                 QString name,
                                                 std::stop_token token)
{
    try {
        Result<poe::CharacterWrapper> result;
        try {
            auto future = m_api.getCharacter(m_realm, name, token);
            result = co_await qCoro(future).takeResult();
        } catch (...) {
            result = std::unexpected(MakeInternalError("the character fetch threw"));
        }
        if (!token.stop_requested()) {
            OnCharacterReceived(result, location);
        }
    } catch (...) {
        spdlog::error("ItemsManagerWorker: a character continuation threw; aborting the update");
        AbortUpdate();
    }
    ScheduleSweep();
}

void ItemsManagerWorker::ProcessTab(const poe::StashTab &tab, std::vector<ItemsRequest> &batch)
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
        batch.push_back(ItemsRequest{StashFetch{location.id(), {}}, location});
    }

    // Process any children.
    if (tab.children) {
        // These children need to be added to the database, since they weren't included
        // in the original list.
        emit stashListReceived(*tab.children, m_realm, m_league);
        for (const auto &child : *tab.children) {
            ProcessTab(child, batch);
        }
    }
}

void ItemsManagerWorker::OnStashListReceived(const Result<poe::StashListWrapper> &result)
{
    spdlog::trace("ItemsManagerWorker::OnStashListReceived() entered");

    if (!result) {
        spdlog::warn("Aborting update because there was an error fetching the stash list ({}): {}",
                     RateLimit::ToString(result.error().kind),
                     result.error().message);
        // First-failure terminal (D2/D6): stop the token so any sibling launched
        // concurrently — the character list or its content — resolves Canceled,
        // then take the direct terminal path. AbortUpdate() returns the worker to
        // idle immediately without touching the progress counters, so a six-tab
        // batch reporting 0/6 does not lurch to 1/1 (P-STATUS).
        StopUpdateForFailure();
        AbortUpdate();
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

    // Accumulate this list's whole content batch (D6): ProcessTab appends each
    // selected tab — and, recursively, its folder children — in source
    // traversal order.
    std::vector<ItemsRequest> batch;
    for (const auto &tab : stashes) {
        // This will process tabs recursively.
        ProcessTab(tab, batch);
    }
    spdlog::debug("Requesting {} out of {} stash tabs", batch.size(), stashes.size());

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

    // Launch this list's complete stash content batch now (D6): it is never
    // held behind the character list, which RunUpdate already requested and
    // whose own content launches from its handler. ProcessTab accumulated the
    // batch above in source traversal order; LaunchContent launches it all at
    // once.
    LaunchContent(std::move(batch));
    CheckUpdateFinished();
}

void ItemsManagerWorker::OnCharacterListReceived(const Result<poe::CharacterListWrapper> &result)
{
    spdlog::trace("ItemsManagerWorker::OnCharacterListReceived() entered");

    if (!result) {
        spdlog::warn("Aborting update because there was an error fetching the character list "
                     "({}): {}",
                     RateLimit::ToString(result.error().kind),
                     result.error().message);
        // First-failure terminal (D2/D6): stop the token so the concurrently
        // launched stash list or its content resolves Canceled, then take the
        // direct terminal path — see the stash-list branch above.
        StopUpdateForFailure();
        AbortUpdate();
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

    std::vector<ItemsRequest> batch;

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
            batch.push_back(ItemsRequest{CharacterFetch{name}, location});
        }
    }
    spdlog::debug("There are {} characters to update in '{}'", batch.size(), m_league);

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

    // Launch this list's complete character content batch now (D6): it is
    // independent of the stash lane, so it is never held behind the stash list.
    LaunchContent(std::move(batch));
    CheckUpdateFinished();
}

void ItemsManagerWorker::OnStashReceived(const Result<poe::StashWrapper> &result,
                                         const ItemLocation &location)
{
    spdlog::trace("ItemsManagerWorker::OnStashReceived() entered");

    if (!result || !result->stash) {
        if (!result) {
            spdlog::warn("Aborting update because there was an error fetching the stash ({}): {}",
                         RateLimit::ToString(result.error().kind),
                         result.error().message);
        } else {
            spdlog::error("ItemsManagerWorker: stash is empty");
        }
        // First-failure terminal (D2/D6): stop the token and take the direct
        // terminal path. The progress counters are left untouched — this failed
        // fetch is not counted as received, and the batch's `needed` total is not
        // snapped down, so reported progress stays monotonic (P-STATUS).
        StopUpdateForFailure();
        AbortUpdate();
        return;
    }

    const auto &stash = *result->stash;

    emit stashReceived(*result->stash, m_realm, m_league);

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

    std::vector<ItemsRequest> batch;
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
            // The child's items display under the parent's location, but
            // they are fetched (and atomically replaced) by the child's id.
            // Accumulate the whole child batch first; LaunchContent below counts
            // it into m_stashes_needed before launching any of it, so a ready
            // child future cannot complete against a half-tallied count.
            ItemLocation child_location = location;
            child_location.setFetchId(child.id);
            batch.push_back(ItemsRequest{StashFetch{stash.id, child.id}, child_location});
        }
    }

    // A tab's contents count as known only once everything a fetch of it
    // implies has landed. For a Map/Unique parent that includes every
    // enabled child fetch, so completion is deferred to the last child
    // reply — marking the parent at its own reply would let a failed child
    // fetch strand the children, since they never appear in a top-level
    // list to be retried (F55).
    if (location.fetch_id() == location.id()) {
        const int queued_children = (get_children && stash.children) ? int(stash.children->size())
                                                                     : 0;
        if (queued_children > 0) {
            // Starting a child-fetch cycle makes the parent incomplete
            // until the last child lands — an already-known parent whose
            // reply introduces a child that then fails must not stay
            // known, or the child would never be retried. The cost of a
            // mid-cycle terminal failure is one redundant refetch of the
            // parent and its children on the next update.
            m_contents_known.erase(location.id());
            m_pending_children[location.id()] = queued_children;
        } else {
            m_contents_known.insert(location.id());
        }
    } else {
        const auto pending = m_pending_children.find(location.id());
        if ((pending != m_pending_children.end()) && (--pending->second <= 0)) {
            m_pending_children.erase(pending);
            m_contents_known.insert(location.id());
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

    // Launch the child batch (D6): reply-discovered Map/Unique children go out
    // as one complete batch, with m_pending_children[parent] already set above
    // so a ready child's completion sees the pending count. An empty batch
    // (a normal tab, or a parent whose children are disabled) makes this a
    // no-op.
    LaunchContent(std::move(batch));
    CheckUpdateFinished();
}

void ItemsManagerWorker::OnCharacterReceived(const Result<poe::CharacterWrapper> &result,
                                             const ItemLocation &location)
{
    spdlog::trace("ItemsManagerWorker::OnCharacterReceived() entered");

    if (!result || !result->character) {
        if (!result) {
            spdlog::warn("Aborting update because there was an error fetching the character "
                         "({}): {}",
                         RateLimit::ToString(result.error().kind),
                         result.error().message);
        } else {
            spdlog::error("ItemsManagerWorker: character is empty");
        }
        // First-failure terminal (D2/D6): stop the token and take the direct
        // terminal path, leaving the progress counters untouched (P-STATUS).
        StopUpdateForFailure();
        AbortUpdate();
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

    // No follow-up launch (D6): a character reply discovers no children, and its
    // whole content batch went out at once when the character list arrived.
    CheckUpdateFinished();
}

void ItemsManagerWorker::LaunchContent(std::vector<ItemsRequest> batch)
{
    // The whole batch goes out at once (D6, F56): no worker-side pacing, no
    // one-at-a-time submission. The caller owns the batch as a local vector, so a
    // ready fetch whose handler discovers and launches its own child batch inline
    // builds a separate local batch — there is no shared queue for the inline
    // re-entry to mutate mid-iteration.
    if (batch.empty()) {
        // A TabsOnly update (no content) or a lane with nothing to fetch.
        return;
    }

    // Initialize the entire batch's needed counters BEFORE the first launch
    // (IR2/S1-6): a ready or fail-fast future runs its completion inline in the
    // launch loop below, and it must never observe a needed count still being
    // tallied. These accumulate across the update's lanes — the stash list, the
    // character list, and each parent reply's child batch each add their own
    // and never reset a sibling lane's progress (D6).
    for (const auto &request : batch) {
        switch (request.location.type()) {
        case ItemLocationType::STASH:
            ++m_stashes_needed;
            break;
        case ItemLocationType::CHARACTER:
            ++m_characters_needed;
            break;
        }
    }
    SendStatusUpdate();
    spdlog::debug("Launching a batch of {} item requests.", batch.size());

    const std::stop_token token = m_stop_source.get_token();

    // Each entry names the resource, not how to fetch it; launch an owned
    // per-fetch task per entry, in the batch's source traversal order (D6's
    // per-lane FIFO). Every handle lives in m_fetch_tasks — no fire-and-forget.
    for (const auto &request : batch) {
        const ItemLocation location = request.location;
        std::visit(
            [&](const auto &what) {
                using What = std::decay_t<decltype(what)>;
                if constexpr (std::is_same_v<What, StashFetch>) {
                    m_fetch_tasks.push_back(
                        FetchStash(location, what.stash_id, what.substash_id, token));
                } else {
                    m_fetch_tasks.push_back(FetchCharacter(location, what.name, token));
                }
            },
            request.what);
    }
}

void ItemsManagerWorker::ScheduleSweep()
{
    // Coalesce: many completions in one event-loop turn queue a single sweep.
    if (m_sweep_scheduled) {
        return;
    }
    m_sweep_scheduled = true;
    if (m_sweep_observer) {
        ++m_sweep_observer->scheduled;
    }
    // Queued, so the sweep runs on a later turn — outside the completing
    // coroutine whose final_suspend has not yet released its own handle
    // (D6/R5-1): finalization must never destroy the frame it is running in.
    QMetaObject::invokeMethod(this, [this]() { SweepTasks(); }, Qt::QueuedConnection);
}

void ItemsManagerWorker::SweepTasks()
{
    m_sweep_scheduled = false;
    int destroyed = 0;
    // Destroy completed handles only. isReady() is true for a finished (or
    // empty) coroutine; a suspended straggler is not ready and survives —
    // destroying its handle would detach the frame, not stop it (S1-1/S1-4),
    // and it would still resume later.
    std::erase_if(m_fetch_tasks, [&destroyed](const QCoro::Task<> &task) {
        if (task.isReady()) {
            ++destroyed;
            return true;
        }
        return false;
    });
    if (m_sweep_observer) {
        ++m_sweep_observer->executed;
        m_sweep_observer->destroyed += destroyed;
        m_sweep_observer->live_after = m_fetch_tasks.size();
    }
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
    // The SUCCESS terminal only. A failed update never reaches its counter
    // equality — the first failure routes through AbortUpdate() and sets Idle
    // directly (D6) — so by the time this runs while still Updating, no failure
    // has occurred. It is called after every completion that could reconcile the
    // counters; when they line up with both required lists received, the update
    // succeeded.
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

    spdlog::trace("ItemsManagerWorker::CheckUpdateFinished() finishing update");
    FinishUpdate();
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
