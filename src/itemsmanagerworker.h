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
#include <queue>
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

// What a queued fetch asks the API for. The worker names the resource; the
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

struct ItemsRequest
{
    int id{-1};
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

    // Test-only: inject a read-only sweep observer (verification §2). The
    // worker writes counts as it schedules/runs sweeps; nothing here changes
    // production behavior when the observer is null.
    void SetSweepObserver(WorkerSweepObserver *observer) { m_sweep_observer = observer; }

    // Test-only fault injection (verification §3). Fires once, at the next
    // fault site reached — the root orchestration body (RunUpdate) or a failure
    // handler immediately after it stops the token, before it sets completion
    // flags. A test arms it to prove each catch-all contains a throw and drives
    // a terminal AbortUpdate() from a stopped-but-still-active update, without
    // depending on Qt's undefined slot-throwing behavior. Null in production.
    void SetFaultHook(std::function<void()> hook) { m_fault_hook = std::move(hook); }

    // Test-only: how many per-fetch task handles the worker still holds. This
    // reaches zero only after every future the worker awaited — including any
    // stopped old-update straggler — has settled and the deferred sweep has
    // drained, not merely when an update reaches its terminal state. (In 5C an
    // aborted update's stopped straggler handles intentionally outlive it.)
    // Ordinary fixtures assert this is zero at teardown (verification §2).
    size_t OutstandingFetchTasksForTest() const { return m_fetch_tasks.size(); }
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
    void QueueRequest(std::variant<StashFetch, CharacterFetch> what, const ItemLocation &location);
    void SubmitStashListRequest();
    void SubmitCharacterListRequest();
    // Drain the accumulated request queue and launch the whole batch at once
    // (D6, F56): no worker-side pacing, no one-at-a-time submission. Called once
    // per policy lane as its list result is processed, and again for each
    // parent reply's discovered child batch. The whole batch's needed counters
    // are initialized before its first launch, because a ready/fail-fast future
    // runs its completion synchronously inside the launch loop (IR2/S1-6).
    void LaunchQueuedContent();
    bool IsStale(int generation) const;

    // The root update task (D6): launches the update's required list(s) and
    // reconciles the terminal state through the completion counters. Owns no
    // flow control of its own — the per-fetch tasks self-drive. Its whole body
    // is wrapped in a catch-all so a throw in orchestration itself aborts and
    // finalizes rather than escaping (R5-1 root catch-all). It launches every
    // required list without awaiting one another (D6); each list handler then
    // launches its own complete content batch, so character content is never
    // held behind the stash list.
    QCoro::Task<> RunUpdate();

    // Per-fetch tasks (D6): each co_awaits one facade future via
    // qCoro(future).takeResult(), checks the update token immediately after
    // (post-await identity invariant, IR2), then hands the result to the
    // matching handler. A per-fetch catch-all turns an exceptional future
    // (R5-1) into an ordinary Internal failure so it counts its completion and
    // enters the first-failure stop path instead of wedging the update. Every
    // handle is owned in m_fetch_tasks — no fire-and-forget.
    QCoro::Task<> FetchStashList(int generation, std::stop_token token);
    QCoro::Task<> FetchCharacterList(int generation, std::stop_token token);
    QCoro::Task<> FetchStash(ItemLocation location,
                             QString stash_id,
                             QString substash_id,
                             int generation,
                             std::stop_token token);
    QCoro::Task<> FetchCharacter(ItemLocation location,
                                 QString name,
                                 int generation,
                                 std::stop_token token);

    // Unconditional terminal transition for the exceptional paths (R5-1): stop
    // the token, record a failure, drop queued work, and force both
    // list-received flags and the content counters to a state where
    // CheckUpdateFinished() must take the terminal (failed) branch — so a throw
    // in orchestration or in a fetch continuation can never leave the update
    // wedged in Updating waiting on a list/completion that will never arrive.
    // The ordinary value-level failure branches keep their own precise
    // bookkeeping; this is the catch-all's blunt, always-terminal fallback.
    void AbortUpdate();

    // First-failure stop for the value-level failure branches: stop the update
    // token, then fire the test fault hook (a no-op in production) at the point
    // between stopping the token and setting completion flags — the window in
    // which the update is stopped but still active, which the catch-alls must
    // still recognize as theirs to abort.
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

    void ProcessTab(const poe::StashTab &tab, int &count);

    QSettings &m_settings;
    BuyoutManager &m_buyout_manager;
    PoeApiClient &m_api;

    QString m_realm;
    QString m_league;
    QString m_account;

    std::vector<ItemLocation> m_tabs;
    std::queue<ItemsRequest> m_queue;

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

    // Incremented by every update; reply handlers discard replies whose
    // generation is not the currently running update's (F28).
    int m_update_generation{0};

    bool m_updateRequest{false};
    TabSelection m_type;
    std::vector<ItemLocation> m_locations;
    size_t m_request_failures{0};

    int m_queue_id{0};

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
    // distinct, unstopped token. In 5B this overlaps the still-present
    // generation guard (IsStale); 5D deletes the generation guard once the
    // post-await token check is mutation-proven to replace it.
    std::stop_source m_stop_source;

    // Read-only sweep observation (verification §2); null in production.
    WorkerSweepObserver *m_sweep_observer{nullptr};

    // Test-only one-shot fault injection (verification §3); null in production.
    std::function<void()> m_fault_hook;

    // Set while a deferred sweep is already queued, so many completions in one
    // turn coalesce into a single SweepTasks() run.
    bool m_sweep_scheduled{false};

    // Owned coroutine handles (D6/R4-3). Declared LAST so they destruct FIRST:
    // at worker destruction a suspended frame is detached (S1-1), and it must
    // be released before any member its frame could still observe. m_update_task
    // is the single root handle (reused each update); m_fetch_tasks holds every
    // per-fetch handle until the deferred sweep reclaims the completed ones.
    QCoro::Task<> m_update_task;
    std::vector<QCoro::Task<>> m_fetch_tasks;
};
