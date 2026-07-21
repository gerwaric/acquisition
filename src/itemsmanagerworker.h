// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QCoroTask>

#include <QNetworkCookie>
#include <QObject>
#include <QPointer>
#include <QString>
#include <QStringList>

#include <atomic>
#include <expected>
#include <functional>
#include <map>
#include <set>
#include <stop_token>
#include <variant>
#include <vector>

#include "item.h"
#include "poe/types/character.h"
#include "poe/types/stashtab.h"
#include "ratelimit/fetcherror.h"
#include "util/programstate.h"
#include "util/util.h"

class QSettings;
class QSignalMapper;
class QThread;
class QTimer;

class BuyoutManager;
class DataStore;
class ItemLocation;
class NetworkManager;
class PoeApiClient;
class RePoE;

// What a content fetch asks the API for. The worker names the resource; the
// facade builds the request (phase 4b) — above this line nothing sees a
// QNetworkRequest.
struct StashFetch
{
    QString stash_id;
    QString substash_id;
};

struct CharacterFetch
{
    QString name;
};

// One entry in a content batch: what to fetch and where its items belong. A
// batch is a plain local vector built inside the handler that discovered it
// (the stash list, the character list, or a parent reply's children) and
// launched all at once by LaunchContent — there is no persistent worker queue
// (F56).
struct ItemsRequest
{
    std::variant<StashFetch, CharacterFetch> what;
    ItemLocation location;
};

struct ParseResult
{
    std::vector<ItemLocation> tabs;
    Items items;
    std::set<QString> tab_id_index;
    // Ids whose contents exist in the datastore — a strict subset of
    // tab_id_index, since a tab can be listed (metadata saved) without its
    // items ever having been fetched (F55).
    std::set<QString> contents_known;
};

// Read-only observation of the deferred task sweep (network-redesign phase 5,
// verification §2). Injected by the worker suite; null in production. The
// worker only ever writes to it — the seam observes lifecycle, it never drives
// it. W-SWEEP asserts that a completion schedules a sweep that runs on a later
// event-loop turn and destroys only completed (ready) per-fetch handles.
struct WorkerSweepObserver
{
    int scheduled = 0;     // ScheduleSweep() actually queued a sweep (coalesced)
    int executed = 0;      // SweepTasks() ran
    int destroyed = 0;     // ready handles erased, summed across every sweep
    size_t live_after = 0; // per-fetch handles still held after the last sweep
};

class ItemsManagerWorker : public QObject
{
    Q_OBJECT
public:
    explicit ItemsManagerWorker(QSettings &m_settings,
                                BuyoutManager &buyout_manager,
                                PoeApiClient &api,
                                QObject *parent = nullptr);
    ~ItemsManagerWorker() override;

    void UpdateRequest(TabSelection type, const std::vector<ItemLocation> &locations);

    // The single test-only seam into the worker (network-redesign phase 5,
    // verification §2). Only forward-declared here and DEFINED in a test-only
    // header (tests/workertestaccess.h), so it grants the worker suite private
    // access without adding any test surface — production code cannot reach it,
    // and the worker's public API carries no standing *ForTest / Set* methods.
    friend class WorkerTestAccess;

    // The setting values are parameters because this runs on the parser
    // thread: QSettings is reentrant but not thread-safe for one shared
    // instance, and the UI writes these keys on the main thread. The
    // caller must read them on the thread that owns m_settings.
    ParseResult ParseCachedItems(const QString &dataDir,
                                 bool get_map_stashes,
                                 bool get_unique_stashes) const;

signals:
    void ItemsRefreshed(const Items &items,
                        const std::vector<ItemLocation> &tabs,
                        bool initial_refresh);
    void StatusUpdate(ProgramState state, const QString &status);
    void NotifyUser(const QString &message);

    void characterListReceived(const std::vector<poe::Character> &characters, const QString &realm);
    void characterReceived(const poe::Character &character, const QString &realm);
    void stashListReceived(const std::vector<poe::StashTab> &stashes,
                           const QString &realm,
                           const QString &league);
    void stashReceived(const poe::StashTab &stash, const QString &realm, const QString &league);

    // Authoritative-list signals (F53): emitted only for a fresh top-level
    // list (never for ProcessTab's folder-children re-emits of
    // stashListReceived, which are partial and must not drive deletion) so
    // the datastore can drop rows the server no longer lists.
    void stashListReplaced(const std::vector<poe::StashTab> &stashes,
                           const QString &realm,
                           const QString &league);
    void characterListReplaced(const std::vector<poe::Character> &characters, const QString &realm);
    // Emitted when a parent stash reply arrives (F53): the reply is
    // authoritative for the parent's children, so the datastore can drop
    // child rows it no longer lists. child_ids is empty when child
    // fetching is disabled in the settings.
    void stashChildrenReplaced(const QString &parent_id,
                               const QStringList &child_ids,
                               const QString &realm,
                               const QString &league);

public slots:
    void OnRePoEReady();
    void Update(Util::TabSelection type, const std::vector<ItemLocation> &tab_names = {});

private:
    // Every handler takes one already-classified result (phase 4b): the
    // transport check, the status check, and the parse the handlers used to
    // do all live below the facade now, so a failure of any flavour arrives
    // as the same unexpected value.
    template<typename T>
    using Result = std::expected<T, RateLimit::FetchError>;

    void OnStashListReceived(const Result<poe::StashListWrapper> &result);
    void OnStashReceived(const Result<poe::StashWrapper> &result, const ItemLocation &location);
    void OnCharacterListReceived(const Result<poe::CharacterListWrapper> &result);
    void OnCharacterReceived(const Result<poe::CharacterWrapper> &result,
                             const ItemLocation &location);

    enum class WorkerState { Initializing, Idle, Updating };

    bool isInitialized() const { return m_state != WorkerState::Initializing; }
    bool isUpdating() const { return m_state == WorkerState::Updating; }
    void StartParseThread();
    void OnParseCompleted(ParseResult result);
    void LoadItems(const poe::Character &character,
                   ItemLocation location,
                   ParseResult &result) const;
    void LoadItems(const poe::StashTab &stash, ItemLocation location, ParseResult &result) const;
    void RemoveItemsFetchedBy(const QString &fetch_id);
    void RebaseItemLocations(ItemLocationType type);
    void SubmitStashListRequest();
    void SubmitCharacterListRequest();
    // Launch a whole content batch at once (D6, F56): no worker-side pacing, no
    // one-at-a-time submission. The batch is a local vector the caller built —
    // one policy lane's worth of fetches (a list's content, or a parent reply's
    // discovered children) in source traversal order. The whole batch's needed
    // counters are initialized before its first launch, because a ready/fail-fast
    // future runs its completion synchronously inside the launch loop (IR2/S1-6).
    void LaunchContent(std::vector<ItemsRequest> batch);

    // The root orchestration (D6, rev. 10): it launches the update's required
    // list(s) without awaiting one another and returns. It is an ordinary
    // synchronous method, NOT a coroutine — the fan-out is a counter-driven join,
    // not a linear await (content and reply-discovered child batches are
    // discovered dynamically and cannot be a static co_await sequence), so it
    // never suspends and there is no asynchronous lifetime for an owned task
    // handle to manage. Only the per-fetch tasks suspend, and they are owned in
    // m_fetch_tasks (R4-3). Its whole body is wrapped in a catch-all (R5-1) so a
    // throw in orchestration aborts and finalizes rather than escaping into the
    // caller; a plain function's try/catch does this exactly as a coroutine's
    // would, since nothing here suspends. Each list handler launches its own
    // complete content batch (character content is never held behind the stash
    // list), each per-fetch coroutine self-drives, and the completion that
    // reconciles the counters finalizes the update. The m_has_*_list /
    // m_need_*_list flags stay because a counter-driven join cannot tell a
    // required-but-empty list from one that has not arrived yet; they are the
    // minimal list-arrival state, not a residual callback pyramid.
    void RunUpdate();

    // Per-fetch tasks (D6): each co_awaits one facade future via
    // qCoro(future).takeResult(), checks the update token immediately after
    // (post-await identity invariant, IR2), then hands the result to the
    // matching handler. The token is the update's sole identity: a straggler
    // from a failed update carries that update's stopped token, so the post-await
    // check discards it before it can touch a later update's state (W-IDENTITY).
    // A per-fetch catch-all turns an exceptional future (R5-1) into an ordinary
    // Internal failure so it flows through the handler's failure branch and takes
    // the direct terminal abort, instead of escaping into the unawaited task and
    // wedging the update. Every handle is owned in m_fetch_tasks — no
    // fire-and-forget.
    QCoro::Task<> FetchStashList(std::stop_token token);
    QCoro::Task<> FetchCharacterList(std::stop_token token);
    QCoro::Task<> FetchStash(ItemLocation location,
                             QString stash_id,
                             QString substash_id,
                             std::stop_token token);
    QCoro::Task<> FetchCharacter(ItemLocation location, QString name, std::stop_token token);

    // The single, idempotent terminal transition for a failed update (R5-1):
    // every failure — a list/content error, an exceptional future, or a throw in
    // orchestration or a handler — routes here. It requests stop, records at
    // least one request failure, and returns the worker to Idle immediately,
    // abandoning in-flight siblings as stopped stragglers. It
    // is deliberately INDEPENDENT of the completion counters: success reconciles
    // the counters in CheckUpdateFinished(); failure takes this direct path, so
    // the two never contend and a failed fetch never rewrites the counters
    // (which drove reported progress backward, P-STATUS).
    void AbortUpdate();

    // First-failure stop for the value-level failure branches: stop the update
    // token, then fire the test fault hook (a no-op in production) at the point
    // after stopping the token but before the handler finishes its failure
    // bookkeeping and its direct AbortUpdate() — the window in which the update
    // is stopped but still active, which the catch-alls must still recognize as
    // theirs to abort.
    void StopUpdateForFailure();

    // Invoke the test fault hook once if armed (test-only; may throw — that is
    // the point). See SetFaultHook.
    void FireFaultHook();

    // Queue a deferred, coalesced reclaim of completed per-fetch handles. Runs
    // outside any completing coroutine (D6/R5-1): the completion that
    // finalizes the update must not destroy the frame it is running in. Only
    // ready (finished) handles are destroyed — destroying a suspended
    // straggler's handle would detach, not stop, it (S1-1/S1-4).
    void ScheduleSweep();
    void SweepTasks();

    void SendStatusUpdate();
    void ParseItems(const std::vector<poe::Item> &items, const ItemLocation &base_location);
    void CheckUpdateFinished();
    void FinishUpdate();

    void ProcessTab(const poe::StashTab &tab, std::vector<ItemsRequest> &batch);

    QSettings &m_settings;
    BuyoutManager &m_buyout_manager;
    PoeApiClient &m_api;

    QString m_realm;
    QString m_league;
    QString m_account;

    std::vector<ItemLocation> m_tabs;

    Items m_items;

    size_t m_stashes_needed{0};
    size_t m_stashes_received{0};

    size_t m_characters_needed{0};
    size_t m_characters_received{0};

    std::set<QString> m_tab_id_index;

    // Ids whose contents have actually been fetched — seeded from the
    // datastore at startup, extended on every successful stash/character
    // reply, never consumed by list receipt alone. The always-fetch check
    // keys on this instead of list membership, so a new tab whose first
    // fetch failed (or never ran) stays "new" until a fetch lands (F55).
    std::set<QString> m_contents_known;

    // Child fetches still outstanding per Map/Unique parent: the parent
    // joins m_contents_known only when its last enabled child fetch lands,
    // so a failed child fetch leaves the parent "new" and the children get
    // retried — they never appear in a top-level list, so nothing else
    // would ever refetch them (F55).
    std::map<QString, int> m_pending_children;

    WorkerState m_state{WorkerState::Initializing};
    QPointer<QThread> m_parser_thread;
    std::atomic<bool> m_shutdown{false};

    bool m_updateRequest{false};
    TabSelection m_type;
    std::vector<ItemLocation> m_locations;
    size_t m_request_failures{0};

    // The current update's content selection: refresh everything
    // (All/TabsOnly), or only the tabs/characters whose ids are listed.
    // Tabs not previously known (brand new on the server) are always
    // fetched.
    bool m_update_all;
    std::set<QString> m_tabs_to_update;

    bool m_need_stash_list;
    bool m_need_character_list;

    bool m_has_stash_list;
    bool m_has_character_list;

    bool m_update_tab_contents;

    // One stop_source per update (D2): every facade call in the update takes
    // its token, and a terminal failure request_stop()s it. Reset to a fresh
    // source at the start of each Update(), so the next update's calls carry a
    // distinct, unstopped token. This token IS the update's identity: the
    // post-await check discards any resumed straggler whose token is stopped,
    // which replaces the deleted generation guard (W-IDENTITY, verification §5).
    std::stop_source m_stop_source;

    // Read-only sweep observation (verification §2); null in production.
    WorkerSweepObserver *m_sweep_observer{nullptr};

    // Test-only one-shot fault injection (verification §3); null in production.
    std::function<void()> m_fault_hook;

    // Set while a deferred sweep is already queued, so many completions in one
    // turn coalesce into a single SweepTasks() run.
    bool m_sweep_scheduled{false};

    // Owned per-fetch coroutine handles (D6/R4-3). Declared LAST so they
    // destruct FIRST: at worker destruction a suspended frame is detached
    // (S1-1), and it must be released before any member its frame could still
    // observe. m_fetch_tasks holds every per-fetch handle until the deferred
    // sweep reclaims the completed ones. The root orchestration (RunUpdate) is a
    // synchronous method that never suspends, so it needs no owned handle here
    // (rev. 10).
    std::vector<QCoro::Task<>> m_fetch_tasks;
};
