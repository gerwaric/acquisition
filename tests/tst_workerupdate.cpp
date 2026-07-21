#include <QtTest/QtTest>

#include <QEventLoop>
#include <QSettings>

#include <stdexcept>
#include <stop_token>

#include "datastore/characterrepo.h"
#include "datastore/stashrepo.h"
#include "datastore/userstore.h"
#include "fakeapiclient.h"
#include "itemsmanagerworker.h"
#include "testfixtures.h"
#include "util/json_readers.h"
#include "util/networkmanager.h"

// Offline update-cycle tests for ItemsManagerWorker, driven through a fake
// typed API facade (items-pipeline M1; moved off the byte-crafting network
// fake in network-redesign phase 4b). These pin the reachable F28 semantics —
// a failed update loses nothing and the next one starts clean (the stale-reply
// half became unreachable under the future boundary; see
// failedUpdateDoesNotLeakIntoTheNext) — and the F48 no-duplicate-characters
// fix, none of which can be produced reliably by hand against the live API.
//
// The worker's request building and endpoint labels are not pinned here;
// they belong to the facade and are pinned in tst_poeapiclient. What these
// tests pin is what the worker asked for, in domain terms.

// NOTE: this class must be declared before the JSON helpers below: moc's
// lexer treats the // in a raw string literal (the icon URL) as a comment
// and loses any Q_OBJECT class that follows.
class WorkerUpdateTest : public QObject
{
    Q_OBJECT

private slots:
    void failedUpdateKeepsItemsIntact();
    void failedUpdateDoesNotLeakIntoTheNext();
    void partialUpdateDoesNotDuplicateCharacters();
    void renamedTabMetadataRefreshesWithoutFetch();
    void vanishedMapChildItemsAreRemovedOnParentRefresh();
    void failedUpdateDoesNotRebasePublishedLocations();
    void listedButNeverFetchedTabIsFetchedOnNextUpdate();
    void failedFirstFetchDoesNotConsumeNewness();
    void failedChildFetchKeepsParentNew();
    void cachedParentWithMissingChildRowStaysNew();
    void knownParentWithNewFailedChildIsRetried();
    void datastoreFollowsServerDeletions();
    void datastoreFollowsCharacterDeletions();

    // Direct FakePoeApiClient harness pins. The 5A worker passes only default
    // tokens (D7), so the stopped-straggler and cross-update-overlap paths the
    // shared-token worker will exercise in 5B/5D cannot yet be driven through
    // the worker; these prove the harness capability against the fake directly.
    void fakeSettlesStoppedStragglersCanceledExactlyOnce();
    void fakeTreatsCrossUpdateIdentityOverlapAsLive();

    // Phase 5B — task lifecycle, update identity, and the deferred sweep,
    // while submission is still serial (verification §3, W-TOKEN/W-INIT/
    // W-THROW/W-SWEEP). These install the coroutine foundation; they do not
    // yet prove the generation guard is replaced (that is W-IDENTITY, 5D).
    void everyCallInAnUpdateSharesOneTokenAndTheNextUpdateGetsAFreshOne(); // W-TOKEN
    void readyFirstListRunsInlineWithoutCorruptingCounters();              // W-INIT (success)
    void readyFirstListErrorAbortsLeavingTheSecondListAStraggler();        // W-INIT (error)
    void exceptionalFetchIsCaughtCountedAndAbortsTheUpdate();              // W-THROW
    void handlerExceptionAbortsToIdleInsteadOfWedging();                   // W-THROW (handler)
    void rootOrchestrationExceptionAbortsToIdle();                         // W-THROW (root)
    void completionsScheduleADeferredSweepThatReclaimsFinishedHandles();   // W-SWEEP

    // Phase 5C — staged batching (verification §3). These prove F56 is resolved:
    // both lists and each lane's whole content batch go out without worker-side
    // serialization, folder and Map children batch correctly, source order is
    // preserved per lane, and a stopped sibling resumes safely.
    void stagedBatchesRunConcurrentlyPreservingLaneOrder();             // W-F56-*
    void stoppedSiblingResumesCanceledAndMutatesNothing();              // W-ER1
    void listFailureWithSiblingContentInFlightTerminatesCleanly();      // W-ER1 (list lane)
    void readyContentBatchInitializesCountsBeforeInlineCompletion();    // W-INIT (content)
    void readyContentErrorStillLaunchesTheRestOfTheBatch();             // W-INIT (content error)
    void readyChildCompletesInlineWithoutCorruptingParentBookkeeping(); // W-INIT (child)
    void readyChildErrorStillLaunchesTheRestAndParentStaysRetryable();  // W-INIT (child error)
    void failureKeepsProgressMonotonicWithOneTerminalTransition();      // P-STATUS / P-FAILURE
    void tabsOnlyRequestsBothListsButNoContent();                       // P-SELECTION (TabsOnly)
};

namespace {

    constexpr const char *kRealm = "pc";
    constexpr const char *kLeague = "Worker Update League";

    QString itemJson(const QString &id)
    {
        return QString(R"({
            "verified": true,
            "w": 1,
            "h": 1,
            "icon": "https://web.poecdn.com/image/test.png",
            "id": "%1",
            "name": "",
            "typeLine": "Test Item",
            "baseType": "Test Item",
            "identified": true,
            "ilvl": 1,
            "x": 0,
            "y": 0
        })")
            .arg(id);
    }

    QString joinItems(const QStringList &item_ids)
    {
        QStringList parts;
        for (const auto &id : item_ids) {
            parts.append(itemJson(id));
        }
        return parts.join(",");
    }

    QByteArray stashJson(const QString &id,
                         const QString &name,
                         unsigned index,
                         const std::optional<QStringList> &item_ids = {},
                         const QString &type = "PremiumStash",
                         const QString &parent = {},
                         const QList<QByteArray> &children = {})
    {
        QString json = QString(R"({"id":"%1","name":"%2","type":"%3","index":%4)")
                           .arg(id, name, type, QString::number(index));
        if (!parent.isEmpty()) {
            json += QString(R"(,"parent":"%1")").arg(parent);
        }
        if (item_ids) {
            json += QString(R"(,"items":[%1])").arg(joinItems(*item_ids));
        }
        if (!children.isEmpty()) {
            QByteArrayList list(children.cbegin(), children.cend());
            json += R"(,"children":[)" + list.join(",") + "]";
        }
        json += "}";
        return json.toUtf8();
    }

    // The JSON helpers stay because the datastore seeding path needs them.
    // The network side does not: the worker suite drives a typed facade
    // fake, so responses are domain objects, not bytes. These turn one into
    // the other so a test can express both sides the same way.
    poe::StashTab stashOf(const QByteArray &json)
    {
        const auto stash = json::readStash(json);
        if (!stash) {
            qFatal("test bug: stashOf() could not read the stash json");
        }
        return *stash;
    }

    std::vector<poe::StashTab> stashList(const QList<QByteArray> &stashes)
    {
        std::vector<poe::StashTab> list;
        for (const auto &stash : stashes) {
            list.push_back(stashOf(stash));
        }
        return list;
    }

    QByteArray characterJson(const QString &id,
                             const QString &name,
                             const std::optional<QStringList> &item_ids = {})
    {
        QString json
            = QString(
                  R"({"id":"%1","name":"%2","realm":"%3","class":"Witch","league":"%4","level":1,"experience":0)")
                  .arg(id, name, kRealm, kLeague);
        if (item_ids) {
            json += QString(R"(,"equipment":[%1])").arg(joinItems(*item_ids));
        }
        json += "}";
        return json.toUtf8();
    }

    poe::Character characterOf(const QByteArray &json)
    {
        const auto character = json::readCharacter(json);
        if (!character) {
            qFatal("test bug: characterOf() could not read the character json");
        }
        return *character;
    }

    std::vector<poe::Character> characterList(const QList<QByteArray> &characters)
    {
        std::vector<poe::Character> list;
        for (const auto &character : characters) {
            list.push_back(characterOf(character));
        }
        return list;
    }

    QStringList sortedItemIds(const Items &items)
    {
        QStringList ids;
        for (const auto &item : items) {
            ids.append(item->id());
        }
        ids.sort();
        return ids;
    }

    // A global event filter that flags any event delivered while it is
    // installed. Installed on the application object, it observes events for
    // every object — including QEvent::DeferredDelete deliveries, whose effect
    // QEventLoop::processEvents() cannot report. The drain uses it to declare
    // quiescence only when a full pass delivered nothing through EITHER the
    // processEvents phase or the deferred-delete flush.
    struct EventActivityFilter : QObject
    {
        bool active = false;
        bool eventFilter(QObject *, QEvent *) override
        {
            active = true;
            return false; // observe only; never consume
        }
    };

    // A worker wired to a fake typed facade, with the last ItemsRefreshed
    // emission captured for inspection.
    //
    // The facade's continuations are context-bound, so a delivery lands on a
    // later event-loop pass rather than inside the call. Each deliver* helper
    // names the call by domain identity (never a submission index), settles
    // it, then drains, so every assertion after it still sees the worker's
    // settled state — and if the drain were ever insufficient, the call-count
    // assertion that follows fails loudly rather than silently passing.
    struct WorkerFixture
    {
        explicit WorkerFixture(const QString &account)
            : settings(bm.tempDir.filePath("settings.ini"), QSettings::IniFormat)
            , unused_limiter(network)
            , api(unused_limiter)
        {
            settings.setValue("account", account);
            settings.setValue("realm", kRealm);
            settings.setValue("league", kLeague);
            settings.sync();
        }

        // Verification §2: an ordinary worker fixture must leave no fake call
        // pending and no worker resumption queued. Every update here runs to a
        // terminal state — a refresh emit or a failure — so once the fixture
        // goes out of scope, draining to quiescence must reach idle and leave
        // nothing awaiting a result. Draining first is essential: a fake call is
        // settled before its consumer resumes, so a bare pendingCount() could
        // read zero with a resumption still queued. The check runs while `f` is
        // still a live local inside the test method, so a leak is attributed to
        // that test.
        ~WorkerFixture()
        {
            QVERIFY(drainUntilIdle());
            QCOMPARE(api.pendingCount(), size_t(0));
            // Once every fake call has settled and the deferred sweep has drained,
            // no per-fetch handle remains (verification §2). A stopped straggler's
            // handle intentionally OUTLIVES its update — an aborted update goes
            // idle immediately, leaving the sibling suspended — but the test
            // settles it Canceled before teardown, at which point its consumer
            // resumes and the sweep reclaims it. A non-zero count here means a
            // handle leaked or a straggler was never settled.
            if (worker) {
                QCOMPARE(worker->OutstandingFetchTasksForTest(), size_t(0));
            }
        }

        // Call after seeding the datastore: the worker reads the cache on
        // construction paths keyed off the settings file's directory.
        void start()
        {
            worker = std::make_unique<ItemsManagerWorker>(settings, *bm.manager, api);
            worker->SetSweepObserver(&sweep);
            QObject::connect(worker.get(),
                             &ItemsManagerWorker::ItemsRefreshed,
                             worker.get(),
                             [this](const Items &items,
                                    const std::vector<ItemLocation> &tabs,
                                    bool initial) {
                                 last_items = items;
                                 last_tabs = tabs;
                                 last_initial = initial;
                                 ++refresh_count;
                             });
            // Capture every StatusUpdate with the counter snapshot live at its
            // emission, so a test can pin monotonic progress and the single
            // terminal Ready/failure transition (P-STATUS/P-FAILURE).
            QObject::connect(worker.get(),
                             &ItemsManagerWorker::StatusUpdate,
                             worker.get(),
                             [this](ProgramState state, const QString &message) {
                                 status_updates.push_back(
                                     {state, message, worker->ProgressCountersForTest()});
                             });
            worker->OnRePoEReady();
        }

        QString dataDir() const { return bm.tempDir.filePath("data"); }

        // --- what the worker asked for ------------------------------------

        // A submission count (no over-fetching, no premature next request),
        // not a request identity: individual calls are addressed below by what
        // the worker asked for.
        size_t callCount() const { return api.callCount(); }

        // Identity assertions: does the worker have a call of this shape still
        // outstanding? Each folds in the kind and every distinguishing field,
        // so `hasPendingStash("stashaaaa1")` also asserts it is a top-level
        // GetStash (empty substash) for that id.
        bool hasPendingStashList() const { return api.hasPendingStashList(kRealm, kLeague); }
        bool hasPendingCharacterList() const { return api.hasPendingCharacterList(kRealm); }
        bool hasPendingStash(const QString &stash_id, const QString &substash_id = {}) const
        {
            return api.hasPendingStash(stash_id, substash_id);
        }
        bool hasPendingCharacter(const QString &name) const
        {
            return api.hasPendingCharacter(name);
        }

        // --- delivery, addressed by domain identity -----------------------

        void deliverStashList(std::vector<poe::StashTab> stashes)
        {
            api.resolveStashList(api.pendingStashList(kRealm, kLeague), std::move(stashes));
            QVERIFY(drainUntilIdle());
        }

        void deliverCharacterList(std::vector<poe::Character> characters)
        {
            api.resolveCharacterList(api.pendingCharacterList(kRealm), std::move(characters));
            QVERIFY(drainUntilIdle());
        }

        void deliverStash(const QString &stash_id, poe::StashTab stash)
        {
            api.resolveStash(api.pendingStash(stash_id), std::move(stash));
            QVERIFY(drainUntilIdle());
        }

        // A child fetch: same parent stash id, a non-empty substash id.
        void deliverChild(const QString &stash_id, const QString &substash_id, poe::StashTab stash)
        {
            api.resolveStash(api.pendingStash(stash_id, substash_id), std::move(stash));
            QVERIFY(drainUntilIdle());
        }

        void deliverCharacter(const QString &name, poe::Character character)
        {
            api.resolveCharacter(api.pendingCharacter(name), std::move(character));
            QVERIFY(drainUntilIdle());
        }

        // Settle every in-flight sibling a first-failure abort left behind, then
        // drain (verification §2, W-ER1). Under batching a terminal failure
        // returns the worker to idle immediately (D6), leaving any concurrently
        // launched request outstanding with the now-stopped shared token.
        // Production settles those through the limiter's cancellation/transfer
        // timeout; here the test settles them deterministically Canceled. Their
        // consumers resume, see the stopped token, mutate nothing, and their
        // handles are swept — so an ordinary fixture reaches zero pending at
        // teardown. Returns how many stragglers were settled so a test can pin
        // it. A clean single-fetch failure leaves none and this is a no-op.
        size_t settleStragglers()
        {
            const size_t settled = api.settleStoppedStragglers();
            if (!drainUntilIdle()) {
                qFatal("settleStragglers: the worker never reached quiescence");
            }
            return settled;
        }

        // Fail an outstanding stash fetch. The kind is immaterial to every pin
        // here — the worker treats each non-Canceled failure the same way — so
        // these read as "this fetch died" rather than naming a transport error.
        void failStash(const QString &stash_id,
                       const QString &substash_id = {},
                       RateLimit::FetchError::Kind kind = RateLimit::FetchError::Kind::Network)
        {
            api.reject(api.pendingStash(stash_id, substash_id), kind);
            QVERIFY(drainUntilIdle());
        }

        // Fail an outstanding top-level list fetch (the stash or character
        // list), for the scenarios where a list dies while the OTHER lane's
        // content is already in flight.
        void failStashList(RateLimit::FetchError::Kind kind = RateLimit::FetchError::Kind::Network)
        {
            api.reject(api.pendingStashList(kRealm, kLeague), kind);
            QVERIFY(drainUntilIdle());
        }

        // Settle an outstanding fetch with an EXCEPTIONAL future: the worker's
        // co_await rethrows, exercising the per-fetch catch-all (W-THROW).
        void throwStash(const QString &stash_id, const QString &substash_id = {})
        {
            api.throwAt(api.pendingStash(stash_id, substash_id));
            QVERIFY(drainUntilIdle());
        }

        void throwCharacter(const QString &name)
        {
            api.throwAt(api.pendingCharacter(name));
            QVERIFY(drainUntilIdle());
        }

        // Pump queued coroutine resumptions and any deferred deletions until the
        // event loop is quiescent — a full pass that delivers nothing — or the
        // bound trips. Condition-driven on the loop emptying (not a fixed pass
        // count): it neither under-drains a multi-hop resumption nor assumes the
        // chain settles on the first turn, and it flushes deleteLater() work
        // posted outside a running loop, which processEvents alone does not.
        //
        // Quiescence is judged by a global event filter, not by
        // processEvents()'s return value: a deferred delete flushed by
        // sendPostedEvents() may run a destructor that schedules more work, and
        // processEvents() cannot report that. So each pass runs both phases with
        // the filter armed and only counts as empty when the filter saw no event
        // at all — otherwise another pass runs.
        //
        // This is what makes zero-pending meaningful at teardown: a fake call is
        // marked settled before its consumer's queued .then/co_await resumes, so
        // pendingCount() can read zero with a resumption still queued. Draining
        // to quiescence first closes that window. Returns false if the loop
        // never settled — a runaway resumption the caller surfaces via QVERIFY.
        [[nodiscard]] static bool drainUntilIdle()
        {
            constexpr int kMaxPasses = 1000;
            EventActivityFilter filter;
            QCoreApplication::instance()->installEventFilter(&filter);
            bool idle = false;
            for (int pass = 0; pass < kMaxPasses; ++pass) {
                filter.active = false;
                QEventLoop loop;
                loop.processEvents(QEventLoop::AllEvents);
                QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
                if (!filter.active) {
                    idle = true;
                    break;
                }
            }
            QCoreApplication::instance()->removeEventFilter(&filter);
            return idle;
        }

        BuyoutManagerFixture bm;
        QSettings settings;
        NetworkManager network;
        // Never called: FakePoeApiClient overrides every method. It exists
        // only to satisfy the facade's base constructor.
        RateLimiter unused_limiter;
        FakePoeApiClient api;
        // Declared before `worker` so it outlives it: the worker holds a raw
        // pointer to it and writes sweep counts through it (W-SWEEP).
        WorkerSweepObserver sweep;
        std::unique_ptr<ItemsManagerWorker> worker;

        Items last_items;
        std::vector<ItemLocation> last_tabs;
        bool last_initial{false};
        int refresh_count{0};

        // Every StatusUpdate the worker emitted, with the counter snapshot live
        // when it fired. Progress messages report received/needed; the terminal
        // is a single Ready transition (success or failure).
        struct StatusRecord
        {
            ProgramState state;
            QString message;
            ItemsManagerWorker::ProgressForTest counters;
        };
        std::vector<StatusRecord> status_updates;

        // How many Ready transitions have been emitted since `from` — used to pin
        // that an update produces exactly one terminal Ready (P-FAILURE).
        int readyTransitionsSince(size_t from) const
        {
            int n = 0;
            for (size_t i = from; i < status_updates.size(); ++i) {
                if (status_updates[i].state == ProgramState::Ready) {
                    ++n;
                }
            }
            return n;
        }
    };

    // Wire the worker's persistence signals to a store the way
    // Application does, including the F53 reconciliation connections.
    void connectPersistence(ItemsManagerWorker *worker, UserStore &store)
    {
        auto stashes = &store.stashes();
        auto characters = &store.characters();
        QObject::connect(worker,
                         &ItemsManagerWorker::stashListReceived,
                         stashes,
                         &StashRepo::saveStashList);
        QObject::connect(worker, &ItemsManagerWorker::stashReceived, stashes, &StashRepo::saveStash);
        QObject::connect(worker,
                         &ItemsManagerWorker::characterListReceived,
                         characters,
                         &CharacterRepo::saveCharacterList);
        QObject::connect(worker,
                         &ItemsManagerWorker::characterReceived,
                         characters,
                         &CharacterRepo::saveCharacter);
        QObject::connect(worker,
                         &ItemsManagerWorker::stashListReplaced,
                         stashes,
                         &StashRepo::reconcileStashList);
        QObject::connect(worker,
                         &ItemsManagerWorker::characterListReplaced,
                         characters,
                         &CharacterRepo::reconcileCharacterList);
        QObject::connect(worker,
                         &ItemsManagerWorker::stashChildrenReplaced,
                         stashes,
                         &StashRepo::reconcileStashChildren);
    }

    QStringList storedStashIds(UserStore &store)
    {
        QStringList ids;
        for (const auto &stash : store.stashes().getStashList(kRealm, kLeague)) {
            ids.append(stash.id);
        }
        ids.sort();
        return ids;
    }

} // namespace

// A terminal failure mid-update must lose nothing: items already replaced
// stay replaced, items whose fetch never landed stay cached (F28).
void WorkerUpdateTest::failedUpdateKeepsItemsIntact()
{
    WorkerFixture f("worker-update-account-1");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1", "a2"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    QVERIFY(tab_a && tab_b);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-1");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QVERIFY(f.last_initial);
    QCOMPARE(f.last_items.size(), size_t(3));

    const std::vector<poe::StashTab> fresh_list = stashList(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("stashbbbb1", "Tab B", 1)});

    // A full update launches both lists at once (D6): neither is held behind
    // the other.
    f.worker->Update(TabSelection::All);
    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingStashList());
    QVERIFY(f.hasPendingCharacterList());

    // The stash list's whole content batch — both tabs — goes out at once, not
    // one request at a time.
    f.deliverStashList(fresh_list);
    QCOMPARE(f.callCount(), size_t(4));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("stashbbbb1"));
    f.deliverCharacterList({});
    QCOMPARE(f.callCount(), size_t(4)); // the empty character list adds no content

    // Tab A lands with a new set of items...
    f.deliverStash("stashaaaa1",
                   stashOf(stashJson("stashaaaa1",
                                     "Tab A",
                                     0,
                                     QStringList{"a1-new", "a2-new", "a3-new"})));

    // ...then tab B's fetch dies. It was the only sibling still in flight, so
    // the update terminates with nothing left stranded and no emit.
    QVERIFY(f.hasPendingStash("stashbbbb1"));
    f.failStash("stashbbbb1");
    QCOMPARE(f.refresh_count, 1);

    // A follow-up update of tab A alone succeeds and shows that the failed
    // update lost nothing: tab B's cached item is still present.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.callCount(), size_t(5));
    f.deliverStashList(fresh_list);

    QCOMPARE(f.callCount(), size_t(6));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    f.deliverStash("stashaaaa1",
                   stashOf(stashJson("stashaaaa1",
                                     "Tab A",
                                     0,
                                     QStringList{"a1-new", "a2-new", "a3-new"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1-new", "a2-new", "a3-new", "b1"}));
}

// A terminal failure ends the update losing nothing, and the next update
// starts from a clean slate and succeeds — no failure count or in-flight work
// carries across the boundary (F28).
//
// This test used to also pin the stale-reply half of F28 by delivering the
// same recorded reply three times — once while the worker was idle, once
// mid-update-2 — which the legacy signal boundary allowed. That half is no
// longer reachable through this single-tab scenario: each fetch settles
// exactly once, and this update never has a sibling in flight when it aborts.
// The cross-update straggler the batch worker CAN produce — a stopped old
// request that resumes while a new update is active — is pinned separately by
// W-ER1 and (in 5D) W-IDENTITY. See the F28 entry in docs/cleanup/findings.md.
void WorkerUpdateTest::failedUpdateDoesNotLeakIntoTheNext()
{
    WorkerFixture f("worker-update-account-2");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    QVERIFY(tab_a);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-2");
        QVERIFY(store.stashes().saveStashList({*tab_a}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_items.size(), size_t(1));

    const ItemLocation loc_a(*tab_a);
    const std::vector<poe::StashTab> fresh_list = stashList({stashJson("stashaaaa1", "Tab A", 0)});

    // Update 1: the item fetch for tab A fails, terminating the update.
    f.worker->Update(TabSelection::Selected, {loc_a});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverStashList(fresh_list);
    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    f.failStash("stashaaaa1");
    QCOMPARE(f.refresh_count, 1);

    // Nothing from update 1 is left in flight, and the failure did not
    // disturb the cached items.
    QCOMPARE(f.callCount(), size_t(2));
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1"}));

    // Update 2 starts clean: the failure count from update 1 must not carry
    // over, or it would abort update 2 with no emit.
    f.worker->Update(TabSelection::Selected, {loc_a});
    QCOMPARE(f.callCount(), size_t(3));
    f.deliverStashList(fresh_list);
    QCOMPARE(f.callCount(), size_t(4));
    f.deliverStash("stashaaaa1",
                   stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1-after"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1-after"}));
}

// A partial update that includes a character must fetch only the selected
// characters and keep exactly one tab entry per character. The old
// skip-check compared names against an id-keyed index and never matched,
// so every character in the league was re-added and re-fetched (F48).
void WorkerUpdateTest::partialUpdateDoesNotDuplicateCharacters()
{
    WorkerFixture f("worker-update-account-3");

    const auto char_1 = json::readCharacter(
        characterJson("charid0001", "CharOne", QStringList{"c1-item"}));
    const auto char_2 = json::readCharacter(
        characterJson("charid0002", "CharTwo", QStringList{"c2-item"}));
    QVERIFY(char_1 && char_2);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-3");
        QVERIFY(store.characters().saveCharacterList({*char_1, *char_2}));
        QVERIFY(store.characters().saveCharacter(*char_1));
        QVERIFY(store.characters().saveCharacter(*char_2));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_tabs.size(), size_t(2));
    QCOMPARE(f.last_items.size(), size_t(2));

    // Refresh only CharOne. The fresh list mentions both characters, with
    // CharTwo renamed in-game (same id, new name).
    f.worker->Update(TabSelection::Selected, {ItemLocation(*char_1, 0)});
    QCOMPARE(f.callCount(), size_t(1));
    QVERIFY(f.hasPendingCharacterList());
    f.deliverCharacterList(characterList(
        {characterJson("charid0001", "CharOne"), characterJson("charid0002", "CharTwoRenamed")}));

    // Only the selected character is fetched.
    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingCharacter("CharOne"));
    f.deliverCharacter("CharOne",
                       characterOf(
                           characterJson("charid0001", "CharOne", QStringList{"c1-item-new"})));

    QCOMPARE(f.refresh_count, 2);

    // Exactly one tab entry per character, and CharTwo's cached item intact.
    QCOMPARE(f.last_tabs.size(), size_t(2));
    QStringList tab_ids;
    for (const auto &tab : f.last_tabs) {
        tab_ids.append(tab.id());
    }
    tab_ids.sort();
    QCOMPARE(tab_ids, QStringList({"charid0001", "charid0002"}));
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"c1-item-new", "c2-item"}));

    // The unfetched character's surviving item carries the fresh name —
    // forum codes and sort order read the item's embedded location.
    const auto item_c2 = std::find_if(f.last_items.cbegin(),
                                      f.last_items.cend(),
                                      [](const std::shared_ptr<Item> &item) {
                                          return item->id() == "c2-item";
                                      });
    QVERIFY(item_c2 != f.last_items.cend());
    QCOMPARE((*item_c2)->location().character(), QString("CharTwoRenamed"));
}

// Tab-list reconciliation gives every listed tab fresh metadata, even tabs
// outside the update selection: a tab renamed or moved in-game updates its
// label and position on any partial refresh, with no extra fetch (the M1
// behavior change that absorbed the F15 sketch), and its cached items are
// kept. The surviving items' embedded locations are rebased too — search
// buckets are built from item locations, so a stale embedded copy would
// show the old name (and a moved tab would bucket twice, once per index).
void WorkerUpdateTest::renamedTabMetadataRefreshesWithoutFetch()
{
    WorkerFixture f("worker-update-account-4");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Old Name", 0, QStringList{"a1"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    QVERIFY(tab_a && tab_b);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-4");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_items.size(), size_t(2));

    // Refresh only tab B; the fresh list shows tab A renamed and moved
    // in-game (index 0 -> 2).
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_b)});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverStashList(
        stashList({stashJson("stashbbbb1", "Tab B", 1), stashJson("stashaaaa1", "New Name", 2)}));

    // Only tab B is fetched; the rename costs no extra request.
    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingStash("stashbbbb1"));
    f.deliverStash("stashbbbb1",
                   stashOf(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1-new"})));

    QCOMPARE(f.refresh_count, 2);

    // Tab A carries the new label and position, and its cached item survived.
    const auto renamed = std::find_if(f.last_tabs.cbegin(),
                                      f.last_tabs.cend(),
                                      [](const ItemLocation &tab) {
                                          return tab.id() == "stashaaaa1";
                                      });
    QVERIFY(renamed != f.last_tabs.cend());
    QCOMPARE(renamed->tab_label(), QString("New Name"));
    QCOMPARE(renamed->tab_index(), 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "b1-new"}));

    // The surviving item's embedded location was rebased to match.
    const auto item_a = std::find_if(f.last_items.cbegin(),
                                     f.last_items.cend(),
                                     [](const std::shared_ptr<Item> &item) {
                                         return item->id() == "a1";
                                     });
    QVERIFY(item_a != f.last_items.cend());
    QCOMPARE((*item_a)->location().tab_label(), QString("New Name"));
    QCOMPARE((*item_a)->location().tab_index(), 2);
}

// A parent's reply is authoritative for its children: cached items fetched
// from a Map/Unique child the parent no longer lists must be removed when
// the parent is refreshed. Without this reconciliation they would survive
// every refresh, even a full one — the per-reply replacement only erases
// fetch ids it is about to re-fetch, and the tab-list cleanup keys on the
// display id, which for child items is the (still-listed) parent.
void WorkerUpdateTest::vanishedMapChildItemsAreRemovedOnParentRefresh()
{
    WorkerFixture f("worker-update-account-5");
    f.settings.setValue("get_map_stashes", true);

    // Cached state: a MapStash whose child holds one item. Child stashes
    // are cached under their own ids with a parent reference, and are not
    // part of the stash list (matching the live API).
    const auto parent_tab = json::readStash(
        stashJson("mapstash01", "Maps", 0, QStringList{"p1"}, "MapStash"));
    const auto child_tab = json::readStash(
        stashJson("mapchild01", "Tier 1", 0, QStringList{"m1"}, "MapStash", "mapstash01"));
    QVERIFY(parent_tab && child_tab);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-5");
        QVERIFY(store.stashes().saveStashList({*parent_tab}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*parent_tab, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*child_tab, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"m1", "p1"}));
    QCOMPARE(f.last_tabs.size(), size_t(1));

    // Refresh the map stash. Its reply lists a different child: the old
    // child's cached item must go, the new child's contents arrive.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*parent_tab)});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverStashList(stashList({stashJson("mapstash01", "Maps", 0, {}, "MapStash")}));

    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingStash("mapstash01"));
    f.deliverStash(
        "mapstash01",
        stashOf(stashJson("mapstash01",
                          "Maps",
                          0,
                          QStringList{"p1-new"},
                          "MapStash",
                          {},
                          {stashJson("mapchild02", "Tier 2", 0, {}, "MapStash", "mapstash01")})));

    // The new child is fetched by its own id...
    QCOMPARE(f.callCount(), size_t(3));
    QVERIFY(f.hasPendingStash("mapstash01", "mapchild02"));
    f.deliverChild(
        "mapstash01",
        "mapchild02",
        stashOf(stashJson("mapchild02", "Tier 2", 0, QStringList{"m2"}, "MapStash", "mapstash01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"m2", "p1-new"}));

    // ...and its items display under the parent tab.
    const auto item_m2 = std::find_if(f.last_items.cbegin(),
                                      f.last_items.cend(),
                                      [](const std::shared_ptr<Item> &item) {
                                          return item->id() == "m2";
                                      });
    QVERIFY(item_m2 != f.last_items.cend());
    QCOMPARE((*item_m2)->location().id(), QString("mapstash01"));
    QCOMPARE((*item_m2)->location().fetch_id(), QString("mapchild02"));
}

// The emitted Items share Item objects with ItemsManager and the UI, so
// rebasing surviving item locations may only happen when an update
// finishes successfully (in FinishUpdate, just before the emit). Rebasing
// at list receipt would mutate the already-published snapshot mid-update —
// and a terminal failure would leave it mutated indefinitely, with no emit
// to rebuild the search buckets around the new metadata.
void WorkerUpdateTest::failedUpdateDoesNotRebasePublishedLocations()
{
    WorkerFixture f("worker-update-account-6");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Old Name", 0, QStringList{"a1"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    QVERIFY(tab_a && tab_b);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-6");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    // Hold the published item the way ItemsManager and the UI do: a
    // shared_ptr into the emitted snapshot.
    std::shared_ptr<Item> item_a;
    for (const auto &item : f.last_items) {
        if (item->id() == "a1") {
            item_a = item;
        }
    }
    QVERIFY(item_a);

    const std::vector<poe::StashTab> fresh_list = stashList(
        {stashJson("stashaaaa1", "New Name", 0), stashJson("stashbbbb1", "Tab B", 1)});

    // Refresh only tab B; the fresh list renames tab A, then tab B's fetch
    // dies, terminating the update without an emit.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_b)});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverStashList(fresh_list);
    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingStash("stashbbbb1"));
    f.failStash("stashbbbb1");
    QCOMPARE(f.refresh_count, 1);

    // The published snapshot is untouched by the failed update.
    QCOMPARE(item_a->location().tab_label(), QString("Old Name"));

    // A subsequent successful update applies the rename to the same shared
    // object the UI holds.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_b)});
    QCOMPARE(f.callCount(), size_t(3));
    f.deliverStashList(fresh_list);
    QCOMPARE(f.callCount(), size_t(4));
    f.deliverStash("stashbbbb1",
                   stashOf(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1-new"})));
    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(item_a->location().tab_label(), QString("New Name"));
}

// A tab whose metadata was cached but whose contents were never fetched —
// the durable footprint of an update that died between list receipt and
// the tab's first item request — must still count as "new" after a
// restart, so the next content update fetches it (F55).
void WorkerUpdateTest::listedButNeverFetchedTabIsFetchedOnNextUpdate()
{
    WorkerFixture f("worker-update-account-7");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    const auto tab_n = json::readStash(stashJson("stashnnnn1", "New Tab", 1));
    QVERIFY(tab_a && tab_n);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-7");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_n}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        // tab N: listed, never fetched.
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_tabs.size(), size_t(2));
    QCOMPARE(f.last_items.size(), size_t(1));

    // Refresh only tab A: tab N must be fetched anyway, because its
    // contents were never seen.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverStashList(
        stashList({stashJson("stashaaaa1", "Tab A", 0), stashJson("stashnnnn1", "New Tab", 1)}));

    // Tab A (selected) and tab N (never fetched, so still new) launch together
    // as one batch — N is not held behind A.
    QCOMPARE(f.callCount(), size_t(3));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("stashnnnn1"));
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    f.deliverStash("stashnnnn1", stashOf(stashJson("stashnnnn1", "New Tab", 1, QStringList{"n1"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "n1"}));
}

// The ledger-specified F55 pin: an update that fails after list receipt
// but before a newly discovered tab's first reply must not consume the
// tab's newness — the next partial refresh still fetches it. (Before the
// fix, list receipt alone marked the tab previously-known, so a later
// partial refresh skipped it and published the tab empty.)
void WorkerUpdateTest::failedFirstFetchDoesNotConsumeNewness()
{
    WorkerFixture f("worker-update-account-8");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    QVERIFY(tab_a);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-8");
        QVERIFY(store.stashes().saveStashList({*tab_a}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    const std::vector<poe::StashTab> list_with_new_tab = stashList(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("stashnnnn1", "New Tab", 1)});

    // Update 1: the list reveals new tab N; its first fetch dies.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverStashList(list_with_new_tab);
    // Tab A (selected) and new tab N launch together.
    QCOMPARE(f.callCount(), size_t(3));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("stashnnnn1"));
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    // N's first fetch dies; A has already landed, so nothing else is stranded.
    f.failStash("stashnnnn1");
    QCOMPARE(f.refresh_count, 1);

    // Update 2 selects only tab A again: tab N is still new and must be
    // fetched.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.callCount(), size_t(4));
    f.deliverStashList(list_with_new_tab);
    QCOMPARE(f.callCount(), size_t(6));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("stashnnnn1"));
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    f.deliverStash("stashnnnn1", stashOf(stashJson("stashnnnn1", "New Tab", 1, QStringList{"n1"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "n1"}));
}

// A Map/Unique parent's contents are complete only when every enabled
// child fetch lands: marking the parent known at its own reply would let
// a failed child fetch strand the children, since they never appear in a
// top-level list to be retried (F55, review follow-up).
void WorkerUpdateTest::failedChildFetchKeepsParentNew()
{
    WorkerFixture f("worker-update-account-11");
    f.settings.setValue("get_map_stashes", true);

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    QVERIFY(tab_a);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-11");
        QVERIFY(store.stashes().saveStashList({*tab_a}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    const std::vector<poe::StashTab> list_with_new_map = stashList(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("mapstash01", "Maps", 1, {}, "MapStash")});
    const poe::StashTab map_parent_body = stashOf(
        stashJson("mapstash01",
                  "Maps",
                  1,
                  QStringList{"p1"},
                  "MapStash",
                  {},
                  {stashJson("mapchild01", "Tier 1", 0, {}, "MapStash", "mapstash01")}));

    // Update 1: the list reveals new Map stash M; its parent fetch lands,
    // but the child fetch it queues dies.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverStashList(list_with_new_map);
    // Tab A and the new Map stash M launch together.
    QCOMPARE(f.callCount(), size_t(3));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("mapstash01"));
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    // M's parent reply discovers one child, launched as its own batch.
    f.deliverStash("mapstash01", map_parent_body);
    QCOMPARE(f.callCount(), size_t(4));
    QVERIFY(f.hasPendingStash("mapstash01", "mapchild01"));
    f.failStash("mapstash01", "mapchild01");
    QCOMPARE(f.refresh_count, 1);

    // Update 2 selects only tab A again: the parent is still incomplete,
    // so it is refetched and its child fetch retried.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.callCount(), size_t(5));
    f.deliverStashList(list_with_new_map);
    QCOMPARE(f.callCount(), size_t(7));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("mapstash01"));
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    f.deliverStash("mapstash01", map_parent_body);
    QCOMPARE(f.callCount(), size_t(8));
    QVERIFY(f.hasPendingStash("mapstash01", "mapchild01"));
    f.deliverChild(
        "mapstash01",
        "mapchild01",
        stashOf(stashJson("mapchild01", "Tier 1", 0, QStringList{"m1"}, "MapStash", "mapstash01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m1", "p1"}));
}

// The restart shape of the same gap: a cached Map parent whose reply
// recorded children must stay "new" at startup if a child row is missing,
// so the next content update refetches it.
void WorkerUpdateTest::cachedParentWithMissingChildRowStaysNew()
{
    WorkerFixture f("worker-update-account-12");
    f.settings.setValue("get_map_stashes", true);

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    const auto map_parent = json::readStash(
        stashJson("mapstash01",
                  "Maps",
                  1,
                  QStringList{"p1"},
                  "MapStash",
                  {},
                  {stashJson("mapchild01", "Tier 1", 0, {}, "MapStash", "mapstash01")}));
    QVERIFY(tab_a && map_parent);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-12");
        QVERIFY(store.stashes().saveStashList({*tab_a, *map_parent}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*map_parent, kRealm, kLeague));
        // The child row is missing: its fetch never landed before restart.
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "p1"}));

    // Refresh only tab A: the incomplete parent must be fetched anyway.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverStashList(stashList(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("mapstash01", "Maps", 1, {}, "MapStash")}));
    // Tab A and the incomplete parent launch together.
    QCOMPARE(f.callCount(), size_t(3));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("mapstash01"));
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    f.deliverStash(
        "mapstash01",
        stashOf(stashJson("mapstash01",
                          "Maps",
                          1,
                          QStringList{"p1"},
                          "MapStash",
                          {},
                          {stashJson("mapchild01", "Tier 1", 0, {}, "MapStash", "mapstash01")})));
    QCOMPARE(f.callCount(), size_t(4));
    QVERIFY(f.hasPendingStash("mapstash01", "mapchild01"));
    f.deliverChild(
        "mapstash01",
        "mapchild01",
        stashOf(stashJson("mapchild01", "Tier 1", 0, QStringList{"m1"}, "MapStash", "mapstash01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m1", "p1"}));
}

// Starting a child-fetch cycle must evict the parent from contents-known:
// an already-known parent whose reply introduces a new child that then
// fails would otherwise stay known, and the new child would never be
// retried (F55, review round 2).
void WorkerUpdateTest::knownParentWithNewFailedChildIsRetried()
{
    WorkerFixture f("worker-update-account-13");
    f.settings.setValue("get_map_stashes", true);

    // Cached state: tab A and a fully fetched Map stash whose saved reply
    // records child C1, with C1's row present — the parent starts known.
    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    const auto map_parent = json::readStash(
        stashJson("mapstash01",
                  "Maps",
                  1,
                  QStringList{"p1"},
                  "MapStash",
                  {},
                  {stashJson("mapchild01", "Tier 1", 0, {}, "MapStash", "mapstash01")}));
    const auto map_child = json::readStash(
        stashJson("mapchild01", "Tier 1", 0, QStringList{"m1"}, "MapStash", "mapstash01"));
    QVERIFY(tab_a && map_parent && map_child);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-13");
        QVERIFY(store.stashes().saveStashList({*tab_a, *map_parent}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*map_parent, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*map_child, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m1", "p1"}));

    const std::vector<poe::StashTab> stash_list = stashList(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("mapstash01", "Maps", 1, {}, "MapStash")});
    const poe::StashTab parent_with_new_child = stashOf(
        stashJson("mapstash01",
                  "Maps",
                  1,
                  QStringList{"p1"},
                  "MapStash",
                  {},
                  {stashJson("mapchild01", "Tier 1", 0, {}, "MapStash", "mapstash01"),
                   stashJson("mapchild02", "Tier 2", 1, {}, "MapStash", "mapstash01")}));

    // Update 1 selects the known parent. Its reply introduces child C2;
    // C1's refetch lands, C2's fetch dies.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*map_parent)});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverStashList(stash_list);
    // Only the selected Map parent is fetched; tab A is known and unselected.
    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingStash("mapstash01"));
    f.deliverStash("mapstash01", parent_with_new_child);
    // Both children — C1's refetch and the newly introduced C2 — launch
    // together as one child batch.
    QCOMPARE(f.callCount(), size_t(4));
    QVERIFY(f.hasPendingStash("mapstash01", "mapchild01"));
    QVERIFY(f.hasPendingStash("mapstash01", "mapchild02"));
    f.deliverChild(
        "mapstash01",
        "mapchild01",
        stashOf(stashJson("mapchild01", "Tier 1", 0, QStringList{"m1"}, "MapStash", "mapstash01")));
    // C2 dies; C1 already landed, so nothing is stranded.
    f.failStash("mapstash01", "mapchild02");
    QCOMPARE(f.refresh_count, 1);

    // Update 2 selects only tab A: the parent is incomplete again, so it
    // is refetched and the new child retried.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.callCount(), size_t(5));
    f.deliverStashList(stash_list);
    // Tab A and the incomplete parent launch together.
    QCOMPARE(f.callCount(), size_t(7));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("mapstash01"));
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    f.deliverStash("mapstash01", parent_with_new_child);
    // Both children launch together.
    QCOMPARE(f.callCount(), size_t(9));
    QVERIFY(f.hasPendingStash("mapstash01", "mapchild01"));
    QVERIFY(f.hasPendingStash("mapstash01", "mapchild02"));
    f.deliverChild(
        "mapstash01",
        "mapchild01",
        stashOf(stashJson("mapchild01", "Tier 1", 0, QStringList{"m1"}, "MapStash", "mapstash01")));
    f.deliverChild(
        "mapstash01",
        "mapchild02",
        stashOf(stashJson("mapchild02", "Tier 2", 1, QStringList{"m2"}, "MapStash", "mapstash01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m1", "m2", "p1"}));
}

// End-to-end F53: with the persistence wiring in place, a fresh top-level
// list deletes the rows of server-deleted tabs (a surviving Map child row
// is untouched — it is never in any list), and a Map parent's reply
// replaces its child rows.
void WorkerUpdateTest::datastoreFollowsServerDeletions()
{
    WorkerFixture f("worker-update-account-9");
    f.settings.setValue("get_map_stashes", true);

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    const auto map_parent = json::readStash(
        stashJson("mapstash01", "Maps", 2, QStringList{"p1"}, "MapStash"));
    const auto map_child = json::readStash(
        stashJson("mapchild01", "Tier 1", 0, QStringList{"m1"}, "MapStash", "mapstash01"));
    QVERIFY(tab_a && tab_b && map_parent && map_child);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-9");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b, *map_parent}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*map_parent, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*map_child, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "b1", "m1", "p1"}));

    UserStore store(QDir(f.dataDir()), "worker-update-account-9");
    connectPersistence(f.worker.get(), store);

    // Server-side, Tab B was deleted; the fresh list omits it.
    f.worker->Update(TabSelection::All);
    QCOMPARE(f.callCount(), size_t(2)); // both lists
    f.deliverStashList(stashList(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("mapstash01", "Maps", 1, {}, "MapStash")}));

    // Tab A and the Map parent launch as one stash content batch; the character
    // list is still outstanding.
    QCOMPARE(f.callCount(), size_t(4));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("mapstash01"));
    // Tab B's row is gone at list receipt; the Map child row survives.
    QCOMPARE(storedStashIds(store), QStringList({"mapchild01", "mapstash01", "stashaaaa1"}));

    f.deliverCharacterList({});
    QCOMPARE(f.callCount(), size_t(4)); // the empty character list adds no content
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));

    // The Map parent's reply lists a different child: the old child row is
    // replaced in the datastore and the new child launches.
    f.deliverStash(
        "mapstash01",
        stashOf(stashJson("mapstash01",
                          "Maps",
                          1,
                          QStringList{"p1"},
                          "MapStash",
                          {},
                          {stashJson("mapchild02", "Tier 2", 0, {}, "MapStash", "mapstash01")})));
    QCOMPARE(storedStashIds(store), QStringList({"mapstash01", "stashaaaa1"}));

    QCOMPARE(f.callCount(), size_t(5));
    QVERIFY(f.hasPendingStash("mapstash01", "mapchild02"));
    f.deliverChild(
        "mapstash01",
        "mapchild02",
        stashOf(stashJson("mapchild02", "Tier 2", 0, QStringList{"m2"}, "MapStash", "mapstash01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(storedStashIds(store), QStringList({"mapchild02", "mapstash01", "stashaaaa1"}));
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m2", "p1"}));
}

// End-to-end F53 for characters: a fresh character list deletes the rows
// of server-deleted characters.
void WorkerUpdateTest::datastoreFollowsCharacterDeletions()
{
    WorkerFixture f("worker-update-account-10");

    const auto char_1 = json::readCharacter(
        characterJson("charid0001", "CharOne", QStringList{"c1-item"}));
    const auto char_2 = json::readCharacter(
        characterJson("charid0002", "CharTwo", QStringList{"c2-item"}));
    QVERIFY(char_1 && char_2);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-10");
        QVERIFY(store.characters().saveCharacterList({*char_1, *char_2}));
        QVERIFY(store.characters().saveCharacter(*char_1));
        QVERIFY(store.characters().saveCharacter(*char_2));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_items.size(), size_t(2));

    UserStore store(QDir(f.dataDir()), "worker-update-account-10");
    connectPersistence(f.worker.get(), store);

    // Server-side, CharTwo was deleted; the fresh list omits it.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*char_1, 0)});
    QCOMPARE(f.callCount(), size_t(1));
    f.deliverCharacterList(characterList({characterJson("charid0001", "CharOne")}));

    QStringList ids;
    for (const auto &character : store.characters().getCharacterList(kRealm)) {
        ids.append(character.id);
    }
    QCOMPARE(ids, QStringList({"charid0001"}));

    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingCharacter("CharOne"));
    f.deliverCharacter("CharOne",
                       characterOf(characterJson("charid0001", "CharOne", QStringList{"c1-item"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"c1-item"}));
}

// The straggler-settle path: stopped calls are settled Canceled in bulk,
// exactly once, while live calls are untouched and remain deliverable. The
// finders exclude stopped stragglers, so an active update never delivers to
// one. Driven directly against the fake because the 5A worker cannot stop a
// token.
void WorkerUpdateTest::fakeSettlesStoppedStragglersCanceledExactlyOnce()
{
    NetworkManager network;
    RateLimiter limiter(network);
    FakePoeApiClient api(limiter);

    std::stop_source dead_a;
    std::stop_source dead_bob;
    std::stop_source live;
    dead_a.request_stop();
    dead_bob.request_stop();

    // Two stopped stragglers from an aborted update and one live call from the
    // next. The returned futures are retained so the settlement outcome — not
    // just the counts — is pinned.
    auto fa = api.getStash(kRealm, kLeague, "A", {}, dead_a.get_token());
    auto fbob = api.getCharacter(kRealm, "Bob", dead_bob.get_token());
    auto fc = api.getStash(kRealm, kLeague, "C", {}, live.get_token());

    QCOMPARE(api.callCount(), size_t(3));
    QCOMPARE(api.pendingCount(), size_t(3));
    QCOMPARE(api.stoppedStragglerCount(), size_t(2));

    // Only the live call is deliverable; the stopped stragglers are not.
    QVERIFY(api.hasPendingStash("C"));
    QVERIFY(!api.hasPendingStash("A"));
    QVERIFY(!api.hasPendingCharacter("Bob"));

    // Nothing is settled yet — every future is still outstanding.
    QVERIFY(!fa.isFinished());
    QVERIFY(!fbob.isFinished());
    QVERIFY(!fc.isFinished());

    // Settling the stragglers Canceled leaves only the live call pending.
    QCOMPARE(api.settleStoppedStragglers(), size_t(2));
    QCOMPARE(api.pendingCount(), size_t(1));
    QCOMPARE(api.stoppedStragglerCount(), size_t(0));

    // Each straggler finished with exactly one Canceled result — the outcome,
    // not merely a count. A Network settlement here would fail the test.
    QVERIFY(fa.isFinished());
    QCOMPARE(fa.resultCount(), 1);
    QVERIFY(!fa.result().has_value());
    QCOMPARE(fa.result().error().kind, RateLimit::FetchError::Kind::Canceled);
    QVERIFY(fbob.isFinished());
    QCOMPARE(fbob.resultCount(), 1);
    QVERIFY(!fbob.result().has_value());
    QCOMPARE(fbob.result().error().kind, RateLimit::FetchError::Kind::Canceled);

    // The live call was untouched by the sweep and is still outstanding.
    QVERIFY(!fc.isFinished());

    // A second sweep finds nothing to settle — no double-settle qFatal.
    QCOMPARE(api.settleStoppedStragglers(), size_t(0));

    // The live call is still deliverable and settles to a value on demand.
    QVERIFY(api.hasPendingStash("C"));
    api.resolveStash(api.pendingStash("C"), stashOf(stashJson("C", "Tab C", 0, QStringList{"c1"})));
    QCOMPARE(api.pendingCount(), size_t(0));
    QVERIFY(fc.isFinished());
    QCOMPARE(fc.resultCount(), 1);
    QVERIFY(fc.result().has_value());
}

// A stopped old-update straggler and a live new-update call may share a domain
// identity — a legitimate cross-update overlap that must NOT trip the F49
// duplicate tripwire. The finder resolves to the single live call; the
// straggler settles Canceled independently.
void WorkerUpdateTest::fakeTreatsCrossUpdateIdentityOverlapAsLive()
{
    NetworkManager network;
    RateLimiter limiter(network);
    FakePoeApiClient api(limiter);

    std::stop_source dead;
    std::stop_source live;
    dead.request_stop();

    // Both fetch stash "X": the first is an aborted update's straggler, the
    // second the new update's live request. Futures retained to pin outcomes.
    auto f_straggler = api.getStash(kRealm, kLeague, "X", {}, dead.get_token());
    auto f_live = api.getStash(kRealm, kLeague, "X", {}, live.get_token());
    QCOMPARE(api.pendingCount(), size_t(2));

    // Exactly one *deliverable* "X" exists, so the finder does not qFatal as an
    // F49 duplicate — it selects the live call, and the straggler stays put.
    QVERIFY(api.hasPendingStash("X"));
    api.resolveStash(api.pendingStash("X"), stashOf(stashJson("X", "Tab X", 0, QStringList{"x1"})));

    // The live call — and only it — finished with a value; the straggler is
    // still outstanding, not collateral of the delivery.
    QVERIFY(f_live.isFinished());
    QVERIFY(f_live.result().has_value());
    QVERIFY(!f_straggler.isFinished());

    // The straggler is still owed a settlement; the sweep clears it Canceled.
    QCOMPARE(api.pendingCount(), size_t(1));
    QCOMPARE(api.stoppedStragglerCount(), size_t(1));
    QCOMPARE(api.settleStoppedStragglers(), size_t(1));
    QCOMPARE(api.pendingCount(), size_t(0));
    QVERIFY(f_straggler.isFinished());
    QCOMPARE(f_straggler.resultCount(), 1);
    QVERIFY(!f_straggler.result().has_value());
    QCOMPARE(f_straggler.result().error().kind, RateLimit::FetchError::Kind::Canceled);
}

// W-TOKEN: every list/content call in one update carries the same non-default
// stop_token; a terminal failure stops that token; the next update carries a
// distinct, unstopped one. This is what makes the token the update's identity
// (the post-await check discards a straggler whose token was stopped). 5B keeps
// the generation guard alongside — W-IDENTITY (5D) proves the token replaces it.
void WorkerUpdateTest::everyCallInAnUpdateSharesOneTokenAndTheNextUpdateGetsAFreshOne()
{
    WorkerFixture f("wtoken");
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    const auto s1 = stashJson("stok1", "Tab S1", 0);

    f.worker->Update(TabSelection::All);
    // Both lists are issued up front under one update token that can be stopped
    // (non-default) and has not been stopped yet. Addressed by identity.
    QCOMPARE(f.callCount(), size_t(2));
    const std::stop_token token = f.api.tokenForStashList(kRealm, kLeague);
    QVERIFY(token.stop_possible());
    QVERIFY(!token.stop_requested());
    QVERIFY(f.api.tokenForCharacterList(kRealm) == token); // the character list shares it

    f.deliverStashList(stashList({s1}));
    QCOMPARE(f.callCount(), size_t(3)); // + content fetch for stok1
    // The content fetch shares the same update token — the same object identity,
    // not merely an equal value.
    QVERIFY(f.api.tokenForStash("stok1") == token);
    f.deliverCharacterList({});
    QCOMPARE(f.callCount(), size_t(3)); // the empty character list adds no content

    // A terminal failure stops that shared token (first-failure stop).
    QVERIFY(f.hasPendingStash("stok1"));
    f.failStash("stok1");
    QVERIFY(token.stop_requested());
    QCOMPARE(f.refresh_count, 1);

    // The next update gets a fresh source: its token can still be stopped, has
    // not been, and is a different token from the stopped one — and both its
    // lists share that new token.
    f.worker->Update(TabSelection::All);
    QCOMPARE(f.callCount(), size_t(5));
    const std::stop_token token2 = f.api.tokenForStashList(kRealm, kLeague);
    QVERIFY(token2.stop_possible());
    QVERIFY(!token2.stop_requested());
    QVERIFY(token2 != token);
    QVERIFY(f.api.tokenForCharacterList(kRealm) == token2);

    // Run the second update to a clean terminal state so teardown finds nothing
    // pending.
    f.deliverStashList(stashList({s1}));
    f.deliverCharacterList({});
    f.deliverStash("stok1", stashOf(stashJson("stok1", "Tab S1", 0, QStringList{"i1"})));
    QCOMPARE(f.refresh_count, 2);
}

// W-INIT (success): a first list whose future is already finished runs its
// continuation INLINE during the launch loop (S1-6). It must not finalize early
// or corrupt the counters that were initialized before the launch — the update
// proceeds and completes normally.
void WorkerUpdateTest::readyFirstListRunsInlineWithoutCorruptingCounters()
{
    WorkerFixture f("winit-ok");
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    // Arm the stash list to come back already-finished with one tab.
    f.api.preresolveStashList(stashList({stashJson("sready1", "Tab R1", 0)}));

    f.worker->Update(TabSelection::All);
    // The ready stash list ran inline during launch: it processed its tab and
    // launched that tab's content batch immediately (D6, not held behind the
    // character list), then RunUpdate issued the character list. The needed
    // counter was initialized before that inline content launch, so nothing
    // finalized early or corrupted the run.
    QCOMPARE(f.callCount(), size_t(3)); // stash list + sready1 content + character list
    QVERIFY(f.hasPendingCharacterList());
    QVERIFY(f.hasPendingStash("sready1"));
    QCOMPARE(f.refresh_count, 1);

    f.deliverCharacterList({});
    f.deliverStash("sready1", stashOf(stashJson("sready1", "Tab R1", 0, QStringList{"r1"})));

    // The inline-ready first list did not corrupt the run: exactly one fresh
    // refresh with the expected item.
    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"r1"}));
}

// W-INIT (error): a first list whose future is already finished with a FAILURE
// runs inline during launch and aborts the update cleanly — no early emit. The
// character list is still issued (D6: every required list is launched even when
// an earlier ready failure has already stopped the token), so it comes back a
// stopped straggler that settles Canceled and mutates nothing.
void WorkerUpdateTest::readyFirstListErrorAbortsLeavingTheSecondListAStraggler()
{
    WorkerFixture f("winit-err");
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    f.api.prerejectStashList(RateLimit::FetchError::Kind::Network);

    f.worker->Update(TabSelection::All);
    // Both lists were launched. The stash list failed inline and stopped the
    // shared token before the update went idle; the character list still went
    // out and now carries that stopped token — a straggler, not deliverable. No
    // content, no emit.
    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(!f.hasPendingCharacterList());
    QCOMPARE(f.api.stoppedStragglerCount(), size_t(1));
    QCOMPARE(f.refresh_count, 1);

    // Settling the straggler Canceled mutates nothing and leaves no emit; the
    // worker is already idle.
    QCOMPARE(f.settleStragglers(), size_t(1));
    QCOMPARE(f.refresh_count, 1);
    QCOMPARE(f.api.pendingCount(), size_t(0));

    // The worker returned to idle: a fresh update is accepted and completes,
    // proving the inline failure did not wedge anything.
    const auto s1 = stashJson("sready1", "Tab R1", 0);
    f.worker->Update(TabSelection::All);
    f.deliverStashList(stashList({s1}));
    f.deliverCharacterList({});
    f.deliverStash("sready1", stashOf(stashJson("sready1", "Tab R1", 0, QStringList{"r1"})));
    QCOMPARE(f.refresh_count, 2);
}

// W-THROW: an exceptional facade future (an IR4 boundary violation) is caught by
// the per-fetch task's root catch-all, which turns it into an ordinary failure:
// the completion is counted, the first-failure stop fires, and the update aborts
// to idle instead of wedging on a counter that would never move.
void WorkerUpdateTest::exceptionalFetchIsCaughtCountedAndAbortsTheUpdate()
{
    WorkerFixture f("wthrow");
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    const auto s1 = stashJson("sthrow1", "Tab T1", 0);

    f.worker->Update(TabSelection::All);
    f.deliverStashList(stashList({s1}));
    f.deliverCharacterList({});
    QCOMPARE(f.callCount(), size_t(3));
    QVERIFY(f.hasPendingStash("sthrow1"));
    const std::stop_token token = f.api.tokenForStash("sthrow1"); // captured while pending

    // The content fetch's future rethrows at co_await.
    f.throwStash("sthrow1");

    // No new refresh (the update aborted), and the shared token was stopped by
    // the first-failure path the catch-all fed into.
    QCOMPARE(f.refresh_count, 1);
    QVERIFY(token.stop_requested());

    // Not wedged: the worker is idle again and a fresh update completes.
    f.worker->Update(TabSelection::All);
    f.deliverStashList(stashList({s1}));
    f.deliverCharacterList({});
    f.deliverStash("sthrow1", stashOf(stashJson("sthrow1", "Tab T1", 0, QStringList{"t1"})));
    QCOMPARE(f.refresh_count, 2);
}

// W-THROW (handler body, stopped-but-active): an exception from inside a fetch
// continuation is contained by the per-fetch task's WHOLE-body catch-all, which
// forces a terminal AbortUpdate(). The fault fires from a failure handler AFTER
// it has already stopped the update token but BEFORE it records its completion
// flags — so the catch-all must decide ownership by generation/state, not token
// state: a token-based guard would misread this active update as a stale
// straggler, skip the abort, and wedge. This single pin proves the catch
// contains the throw, a stopped-but-active update still aborts, AbortUpdate()
// forces list+counter completion, and recovery reaches Idle. Uses the injected
// fault hook — never Qt's undefined slot-throwing behavior.
void WorkerUpdateTest::handlerExceptionAbortsToIdleInsteadOfWedging()
{
    WorkerFixture f("whandlerthrow");
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    const auto s1 = stashJson("shthrow1", "Tab H1", 0);

    // Drive to the content stage with the hook unarmed (so RunUpdate does not
    // consume it), then arm it: the content failure's handler stops the token
    // and hits the fault site before setting its completion flags.
    f.worker->Update(TabSelection::All);
    f.deliverStashList(stashList({s1}));
    f.deliverCharacterList({});
    QVERIFY(f.hasPendingStash("shthrow1"));

    f.worker->SetFaultHook([] { throw std::runtime_error("failure handler exploded"); });
    f.failStash("shthrow1");

    // Contained and forced terminal: no refresh, no leaked handle, no wedge.
    QCOMPARE(f.refresh_count, 1);
    QCOMPARE(f.worker->OutstandingFetchTasksForTest(), size_t(0));

    // Recovery reaches Idle: a fresh update is accepted and completes.
    f.worker->Update(TabSelection::All);
    f.deliverStashList(stashList({s1}));
    f.deliverCharacterList({});
    f.deliverStash("shthrow1", stashOf(stashJson("shthrow1", "Tab H1", 0, QStringList{"h1"})));
    QCOMPARE(f.refresh_count, 2);
}

// W-THROW (root orchestration): the update root task's own body has a catch-all
// distinct from the per-fetch tasks' (verification §3). A throw in orchestration
// itself — before any list is launched — is contained there and drives the same
// terminal AbortUpdate(), rather than escaping into the caller (ItemsManager /
// the UI). Recovery proves the worker reached Idle.
void WorkerUpdateTest::rootOrchestrationExceptionAbortsToIdle()
{
    WorkerFixture f("wrootthrow");
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    // Armed before Update(): RunUpdate() hits the root fault site at the top of
    // its body, before launching any list.
    f.worker->SetFaultHook([] { throw std::runtime_error("orchestration exploded"); });
    f.worker->Update(TabSelection::All);

    // Aborted before any request was issued, and it did not escape or wedge.
    QCOMPARE(f.callCount(), size_t(0));
    QCOMPARE(f.refresh_count, 1);
    QCOMPARE(f.worker->OutstandingFetchTasksForTest(), size_t(0));

    // Recovery reaches Idle.
    const auto s1 = stashJson("srthrow1", "Tab R1", 0);
    f.worker->Update(TabSelection::All);
    f.deliverStashList(stashList({s1}));
    f.deliverCharacterList({});
    f.deliverStash("srthrow1", stashOf(stashJson("srthrow1", "Tab R1", 0, QStringList{"r1"})));
    QCOMPARE(f.refresh_count, 2);
}

// W-SWEEP: each per-fetch completion schedules a deferred sweep that runs on a
// later event-loop turn (via a queued invocation — outside the completing
// coroutine) and destroys only finished handles. Across a full update every
// launched per-fetch handle is reclaimed and none is left live.
void WorkerUpdateTest::completionsScheduleADeferredSweepThatReclaimsFinishedHandles()
{
    WorkerFixture f("wsweep");
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    // The observer starts clean: the initial parse launches no fetches.
    QCOMPARE(f.sweep.scheduled, 0);
    QCOMPARE(f.sweep.executed, 0);

    const auto s1 = stashJson("ssweep1", "Tab W1", 0);

    f.worker->Update(TabSelection::All);

    // The stash-list completion launches its content batch and schedules one
    // deferred sweep. That sweep destroys the finished stash-list handle but
    // PRESERVES the two suspended handles it left in flight — the still-pending
    // character list and the just-launched content fetch. Only ready handles
    // are reclaimed; destroying a suspended one would detach, not stop, it.
    f.deliverStashList(stashList({s1}));
    QCOMPARE(f.sweep.executed, 1);
    QCOMPARE(f.sweep.destroyed, 1);
    QCOMPARE(f.sweep.live_after, size_t(2)); // character list + content, both suspended
    QCOMPARE(f.worker->OutstandingFetchTasksForTest(), size_t(2));

    // The character-list completion schedules another sweep (a later completion
    // always schedules its own), which reclaims the character-list handle and
    // leaves the suspended content handle live.
    f.deliverCharacterList({});
    QCOMPARE(f.sweep.executed, 2);
    QCOMPARE(f.sweep.destroyed, 2);
    QCOMPARE(f.sweep.live_after, size_t(1));

    // The final content completion sweeps the last handle; nothing is left live.
    f.deliverStash("ssweep1", stashOf(stashJson("ssweep1", "Tab W1", 0, QStringList{"w1"})));
    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(f.sweep.scheduled, 3);
    QCOMPARE(f.sweep.executed, 3);
    QCOMPARE(f.sweep.destroyed, 3);
    QCOMPARE(f.sweep.live_after, size_t(0));
    QCOMPARE(f.worker->OutstandingFetchTasksForTest(), size_t(0));
}

// W-F56 headline (verification §3): the batch worker submits both lists and each
// lane's complete content batch without one-at-a-time serialization. One scenario
// covers W-F56-LISTS (both lists up front), W-F56-CONTENT (each list's whole
// content batch; character work in flight while stash work is outstanding),
// W-F56-CHILDREN (folder children join the list batch; reply-discovered Map
// children launch as one batch), and W-F56-ORDER (deliberately non-sorted
// identities keep source traversal order within each lane, with no global lane
// order asserted). It FAILS if SubmitNextItemRequest-style serialization returns:
// a serial worker would leave only one content fetch in flight at a time, so the
// simultaneous-pending and lane-order assertions below could not all hold.
void WorkerUpdateTest::stagedBatchesRunConcurrentlyPreservingLaneOrder()
{
    WorkerFixture f("wf56");
    f.settings.setValue("get_map_stashes", true);
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    // A stash list in deliberately non-sorted order: s_zzz1 (sorts LAST) is
    // listed first, a folder with two nested children sits in the middle, and a
    // Map stash is last. A serial or sorted worker would not launch these in
    // this order.
    const std::vector<poe::StashTab> stashes = stashList(
        {stashJson("s_zzz1", "Zzz Tab", 5),
         stashJson("s_aaa1", "Aaa Tab", 2),
         stashJson("fold01",
                   "My Folder",
                   0,
                   {},
                   "Folder",
                   {},
                   {stashJson("fchild1", "Child One", 0, {}, "PremiumStash", "fold01"),
                    stashJson("fchild2", "Child Two", 1, {}, "PremiumStash", "fold01")}),
         stashJson("map01", "Maps", 9, {}, "MapStash")});
    const std::vector<poe::Character> characters = characterList(
        {characterJson("zedid00001", "Zed"), characterJson("annid00001", "Ann")});

    // A full update issues both lists at once.
    f.worker->Update(TabSelection::All);
    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingStashList());
    QVERIFY(f.hasPendingCharacterList());

    // The stash list's whole content batch goes out at once: the two plain tabs,
    // the folder, its two recursively discovered children (which join this list
    // batch), and the Map parent — six fetches, none held behind another.
    f.deliverStashList(stashes);
    QCOMPARE(f.callCount(), size_t(8));
    QVERIFY(f.hasPendingStash("s_zzz1"));
    QVERIFY(f.hasPendingStash("s_aaa1"));
    QVERIFY(f.hasPendingStash("fold01"));
    QVERIFY(f.hasPendingStash("fchild1"));
    QVERIFY(f.hasPendingStash("fchild2"));
    QVERIFY(f.hasPendingStash("map01"));

    // W-F56-ORDER: within the stash lane the launch order is the list's source
    // traversal order (folder children immediately after their folder), NOT the
    // ids' sorted order — s_zzz1 sorts last but launched first. Addressed by the
    // lane's own submission order, never a global call index.
    QCOMPARE(f.api.stashFetchOrder(),
             QStringList({"s_zzz1", "s_aaa1", "fold01", "fchild1", "fchild2", "map01"}));

    // The character list's content batch launches from its own handler — while
    // every stash fetch above is still outstanding. This is the cross-lane
    // concurrency F56 is about: character work is not queued behind stash work.
    f.deliverCharacterList(characters);
    QCOMPARE(f.callCount(), size_t(10));
    QVERIFY(f.hasPendingCharacter("Zed"));
    QVERIFY(f.hasPendingCharacter("Ann"));
    QVERIFY(f.hasPendingStash("s_zzz1")); // stash lane still outstanding meanwhile
    QVERIFY(f.hasPendingStash("map01"));
    // Character lane keeps its own source order (Zed before Ann, though Ann sorts
    // first). No order is asserted BETWEEN the lanes.
    QCOMPARE(f.api.characterFetchOrder(), QStringList({"Zed", "Ann"}));

    // Deliver the whole population in a deliberately scrambled order — arbitrary
    // completion order is supported.
    f.deliverStash("s_aaa1", stashOf(stashJson("s_aaa1", "Aaa Tab", 2, QStringList{"a1"})));
    f.deliverCharacter("Ann", characterOf(characterJson("annid00001", "Ann", QStringList{"ann1"})));
    f.deliverStash("fchild2", stashOf(stashJson("fchild2", "Child Two", 1, QStringList{"fc2"})));

    // The Map parent's reply discovers two children; they launch as one complete
    // child batch (W-F56-CHILDREN), not one at a time.
    f.deliverStash("map01",
                   stashOf(stashJson("map01",
                                     "Maps",
                                     9,
                                     QStringList{"mp1"},
                                     "MapStash",
                                     {},
                                     {stashJson("mc1", "Tier 1", 0, {}, "MapStash", "map01"),
                                      stashJson("mc2", "Tier 2", 1, {}, "MapStash", "map01")})));
    QCOMPARE(f.callCount(), size_t(12));
    QVERIFY(f.hasPendingStash("map01", "mc1"));
    QVERIFY(f.hasPendingStash("map01", "mc2"));

    f.deliverStash("s_zzz1", stashOf(stashJson("s_zzz1", "Zzz Tab", 5, QStringList{"z1"})));
    f.deliverStash("fold01", stashOf(stashJson("fold01", "My Folder", 0, {}, "Folder")));
    f.deliverCharacter("Zed", characterOf(characterJson("zedid00001", "Zed", QStringList{"zed1"})));
    f.deliverStash("fchild1", stashOf(stashJson("fchild1", "Child One", 0, QStringList{"fc1"})));
    f.deliverChild("map01",
                   "mc2",
                   stashOf(stashJson("mc2", "Tier 2", 1, QStringList{"m2"}, "MapStash", "map01")));
    // Still one child outstanding: the update has not finished.
    QCOMPARE(f.refresh_count, 1);
    f.deliverChild("map01",
                   "mc1",
                   stashOf(stashJson("mc1", "Tier 1", 0, QStringList{"m1"}, "MapStash", "map01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items),
             QStringList({"a1", "ann1", "fc1", "fc2", "m1", "m2", "mp1", "z1", "zed1"}));
}

// W-ER1 (verification §3): two worker fetches are in flight under one shared
// token; one fails terminally, which stops the token and returns the worker to
// idle immediately, leaving the sibling as a straggler. When the sibling settles
// Canceled its consumer resumes, sees the stopped token, and returns without
// running the handler — it mutates no state, emits nothing, causes no second
// terminal transition, and adds no failure that could corrupt the next update.
void WorkerUpdateTest::stoppedSiblingResumesCanceledAndMutatesNothing()
{
    WorkerFixture f("wer1");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    QVERIFY(tab_a && tab_b);
    {
        UserStore store(QDir(f.dataDir()), "wer1");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "b1"}));

    const std::vector<poe::StashTab> fresh = stashList(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("stashbbbb1", "Tab B", 1)});

    f.worker->Update(TabSelection::All);
    QCOMPARE(f.callCount(), size_t(2));
    f.deliverStashList(fresh);
    // Both tab contents are in flight at once under the same update token.
    QCOMPARE(f.callCount(), size_t(4));
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("stashbbbb1"));
    f.deliverCharacterList({});
    const std::stop_token token = f.api.tokenForStash("stashaaaa1");
    QVERIFY(token.stop_possible());
    QVERIFY(f.api.tokenForStash("stashbbbb1") == token); // the sibling shares the token

    // Tab A's fetch fails: first-failure stops the shared token and the update
    // goes idle at once, leaving tab B's fetch in flight as a straggler.
    f.failStash("stashaaaa1");
    QVERIFY(token.stop_requested());
    QCOMPARE(f.refresh_count, 1);
    QCOMPARE(f.api.stoppedStragglerCount(), size_t(1));

    const QStringList before = sortedItemIds(f.last_items);

    // The sibling settles Canceled and resumes: no emit, no state change, no
    // second terminal transition.
    QCOMPARE(f.settleStragglers(), size_t(1));
    QCOMPARE(f.refresh_count, 1);
    QCOMPARE(sortedItemIds(f.last_items), before);

    // The straggler carried no failure or corruption into the next update, which
    // completes cleanly.
    f.worker->Update(TabSelection::All);
    f.deliverStashList(fresh);
    f.deliverCharacterList({});
    f.deliverStash("stashaaaa1",
                   stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1-new"})));
    f.deliverStash("stashbbbb1",
                   stashOf(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1-new"})));
    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1-new", "b1-new"}));
}

// W-ER1 (list lane): the character list succeeds and launches its content while
// the stash list is still outstanding; then the stash list fails. First-failure
// must terminate the update immediately (D6), snapping BOTH lane counters — not
// only the list flags — so it does not wedge waiting on the character content it
// just abandoned. Without the content-counter snap in the list-failure branch,
// CheckUpdateFinished would never see items_received and the worker would hang in
// Updating, refusing the next update.
void WorkerUpdateTest::listFailureWithSiblingContentInFlightTerminatesCleanly()
{
    WorkerFixture f("wer1-list");

    const auto char_1 = json::readCharacter(
        characterJson("charid0001", "CharOne", QStringList{"c1"}));
    QVERIFY(char_1);
    {
        UserStore store(QDir(f.dataDir()), "wer1-list");
        QVERIFY(store.characters().saveCharacterList({*char_1}));
        QVERIFY(store.characters().saveCharacter(*char_1));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    f.worker->Update(TabSelection::All);
    QCOMPARE(f.callCount(), size_t(2));
    // The character list arrives first and launches its content while the stash
    // list is still in flight.
    f.deliverCharacterList(characterList({characterJson("charid0001", "CharOne")}));
    QCOMPARE(f.callCount(), size_t(3));
    QVERIFY(f.hasPendingCharacter("CharOne"));
    QVERIFY(f.hasPendingStashList());

    // Now the stash list fails. The update must return to idle at once, leaving
    // the character content as a stopped straggler — not wedge on it.
    f.failStashList();
    QCOMPARE(f.refresh_count, 1);
    QCOMPARE(f.api.stoppedStragglerCount(), size_t(1)); // the character content
    QCOMPARE(f.settleStragglers(), size_t(1));

    // Not wedged: a fresh update is accepted and completes.
    f.worker->Update(TabSelection::All);
    QCOMPARE(f.callCount(), size_t(5)); // + both lists of the recovery update
    f.deliverStashList({});
    f.deliverCharacterList(characterList({characterJson("charid0001", "CharOne")}));
    f.deliverCharacter("CharOne",
                       characterOf(characterJson("charid0001", "CharOne", QStringList{"c1-new"})));
    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"c1-new"}));
}

// W-INIT (content batch): both content fetches of a stash batch come back
// already-finished and run inline during the launch loop. Because the whole
// batch's needed counter is initialized before the first launch, the first
// inline completion cannot finalize the update on its own — only after BOTH
// complete does it finish, with both tabs' items. The character list is
// delivered first so both lists are already done when the ready batch launches,
// which is exactly the ordering under which an incrementally initialized counter
// would finalize early after only one tab.
void WorkerUpdateTest::readyContentBatchInitializesCountsBeforeInlineCompletion()
{
    WorkerFixture f("winit-content");
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    f.api.preresolveStash(stashOf(stashJson("sc_a", "Tab A", 0, QStringList{"a1"})));
    f.api.preresolveStash(stashOf(stashJson("sc_b", "Tab B", 1, QStringList{"b1"})));

    f.worker->Update(TabSelection::All);
    QCOMPARE(f.callCount(), size_t(2));
    f.deliverCharacterList({}); // both lists done before the ready batch launches
    f.deliverStashList(stashList({stashJson("sc_a", "Tab A", 0), stashJson("sc_b", "Tab B", 1)}));

    // Both ready fetches ran inline; the update finished exactly once, with both.
    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "b1"}));
}

// W-INIT (content batch, inline error): the first fetch of a stash batch comes
// back already-failed. The inline failure stops the shared token and returns the
// worker to idle, but the whole initialized batch must still be launched — the
// second fetch goes out carrying the stopped token and becomes a straggler. A
// launch loop that bailed on the inline failure would never issue it.
void WorkerUpdateTest::readyContentErrorStillLaunchesTheRestOfTheBatch()
{
    WorkerFixture f("winit-content-err");
    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    f.api.prerejectStash(RateLimit::FetchError::Kind::Network);

    f.worker->Update(TabSelection::All);
    f.deliverCharacterList({});
    f.deliverStashList(stashList({stashJson("sc_a", "Tab A", 0), stashJson("sc_b", "Tab B", 1)}));

    // Both content fetches were issued even though the first failed inline: sc_a
    // is settled (ready-error), sc_b is a stopped straggler.
    QCOMPARE(f.callCount(), size_t(4));
    QCOMPARE(f.refresh_count, 1);
    QVERIFY(!f.hasPendingStash("sc_a"));
    QCOMPARE(f.api.stoppedStragglerCount(), size_t(1));

    QCOMPARE(f.settleStragglers(), size_t(1));
    QCOMPARE(f.refresh_count, 1);
    QCOMPARE(f.api.pendingCount(), size_t(0));

    // Recovery: the inline failure did not wedge the worker.
    f.worker->Update(TabSelection::All);
    f.deliverStashList(stashList({stashJson("sc_a", "Tab A", 0), stashJson("sc_b", "Tab B", 1)}));
    f.deliverCharacterList({});
    f.deliverStash("sc_a", stashOf(stashJson("sc_a", "Tab A", 0, QStringList{"a1"})));
    f.deliverStash("sc_b", stashOf(stashJson("sc_b", "Tab B", 1, QStringList{"b1"})));
    QCOMPARE(f.refresh_count, 2);
}

// W-INIT (child batch): a Map parent's reply discovers two children and one of
// them comes back already-finished, running inline during the child batch launch.
// Because m_pending_children and the child count are set before the launch, the
// inline child completion neither finalizes the update early nor marks the parent
// contents-known before its last child lands — the update finishes only after the
// second child arrives, with both children's items.
void WorkerUpdateTest::readyChildCompletesInlineWithoutCorruptingParentBookkeeping()
{
    WorkerFixture f("winit-child");
    f.settings.setValue("get_map_stashes", true);

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    QVERIFY(tab_a);
    {
        UserStore store(QDir(f.dataDir()), "winit-child");
        QVERIFY(store.stashes().saveStashList({*tab_a}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    f.worker->Update(TabSelection::All);
    f.deliverStashList(stashList(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("map01", "Maps", 1, {}, "MapStash")}));
    f.deliverCharacterList({});
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));

    // Arm the first child to come back already-finished, then deliver the parent:
    // its reply launches the [mc1, mc2] child batch and mc1 completes inline.
    f.api.preresolveStash(
        stashOf(stashJson("mc1", "Tier 1", 0, QStringList{"m1"}, "MapStash", "map01")));
    f.deliverStash("map01",
                   stashOf(stashJson("map01",
                                     "Maps",
                                     1,
                                     QStringList{"mp1"},
                                     "MapStash",
                                     {},
                                     {stashJson("mc1", "Tier 1", 0, {}, "MapStash", "map01"),
                                      stashJson("mc2", "Tier 2", 1, {}, "MapStash", "map01")})));

    // mc1 ran inline, but the update is NOT finished: mc2 is still outstanding.
    QCOMPARE(f.refresh_count, 1);
    QVERIFY(!f.hasPendingStash("map01", "mc1")); // ready, settled inline
    QVERIFY(f.hasPendingStash("map01", "mc2"));

    f.deliverChild("map01",
                   "mc2",
                   stashOf(stashJson("mc2", "Tier 2", 1, QStringList{"m2"}, "MapStash", "map01")));

    // Only now does the update finish, with the parent's and both children's items.
    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m1", "m2", "mp1"}));
}

// W-INIT (child batch, inline error): the FIRST Map child comes back
// already-failed during the child batch launch. Like the content-batch error
// case, the whole initialized child stage must still be launched — the second
// child goes out carrying the stopped token — and the parent must stay
// retryable (its child cycle never completed), with exactly one failure
// transition.
void WorkerUpdateTest::readyChildErrorStillLaunchesTheRestAndParentStaysRetryable()
{
    WorkerFixture f("winit-child-err");
    f.settings.setValue("get_map_stashes", true);

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    QVERIFY(tab_a);
    {
        UserStore store(QDir(f.dataDir()), "winit-child-err");
        QVERIFY(store.stashes().saveStashList({*tab_a}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    const std::vector<poe::StashTab> stash_list = stashList(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("map01", "Maps", 1, {}, "MapStash")});
    const poe::StashTab parent_body = stashOf(
        stashJson("map01",
                  "Maps",
                  1,
                  QStringList{"mp1"},
                  "MapStash",
                  {},
                  {stashJson("mc1", "Tier 1", 0, {}, "MapStash", "map01"),
                   stashJson("mc2", "Tier 2", 1, {}, "MapStash", "map01")}));

    f.worker->Update(TabSelection::All);
    f.deliverStashList(stash_list);
    f.deliverCharacterList({});
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));

    const size_t mark = f.status_updates.size();
    // Arm the first child to fail already-finished, then deliver the parent: its
    // reply launches the [mc1, mc2] child batch and mc1 fails inline.
    f.api.prerejectStash(RateLimit::FetchError::Kind::Network);
    f.deliverStash("map01", parent_body);

    // The whole child stage still launched: mc1 is settled (ready-error), mc2 was
    // submitted with the stopped token and is a straggler. One failure transition.
    QVERIFY(!f.hasPendingStash("map01", "mc1"));
    QCOMPARE(f.api.stoppedStragglerCount(), size_t(1)); // mc2
    QCOMPARE(f.refresh_count, 1);
    QCOMPARE(f.readyTransitionsSince(mark), 1);
    QCOMPARE(f.settleStragglers(), size_t(1));

    // The parent's child cycle never completed, so it stays retryable: a
    // follow-up update refetches it and both children now succeed.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    f.deliverStashList(stash_list);
    QVERIFY(f.hasPendingStash("stashaaaa1"));
    QVERIFY(f.hasPendingStash("map01"));
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    f.deliverStash("map01", parent_body);
    QVERIFY(f.hasPendingStash("map01", "mc1"));
    QVERIFY(f.hasPendingStash("map01", "mc2"));
    f.deliverChild("map01",
                   "mc1",
                   stashOf(stashJson("mc1", "Tier 1", 0, QStringList{"m1"}, "MapStash", "map01")));
    f.deliverChild("map01",
                   "mc2",
                   stashOf(stashJson("mc2", "Tier 2", 1, QStringList{"m2"}, "MapStash", "map01")));
    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m1", "m2", "mp1"}));
}

// P-STATUS / P-FAILURE: a mid-batch failure must not run progress backward. With
// a three-tab content batch (needed=3) reporting 0/3 then 1/3, the first failure
// takes the direct terminal path — it does NOT count the failed fetch as received
// or snap `needed` down to fake CheckUpdateFinished's success equality — so the
// reported counters stay monotonic, and the failed update emits exactly one
// terminal Ready transition.
void WorkerUpdateTest::failureKeepsProgressMonotonicWithOneTerminalTransition()
{
    WorkerFixture f("wmono");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    const auto tab_c = json::readStash(stashJson("stashcccc1", "Tab C", 2, QStringList{"c1"}));
    QVERIFY(tab_a && tab_b && tab_c);
    {
        UserStore store(QDir(f.dataDir()), "wmono");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b, *tab_c}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_c, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    const std::vector<poe::StashTab> fresh = stashList({stashJson("stashaaaa1", "Tab A", 0),
                                                        stashJson("stashbbbb1", "Tab B", 1),
                                                        stashJson("stashcccc1", "Tab C", 2)});

    const size_t mark = f.status_updates.size();
    f.worker->Update(TabSelection::All);
    f.deliverStashList(fresh); // needed = 3, the whole batch launches
    f.deliverCharacterList({});
    f.deliverStash("stashaaaa1", stashOf(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    // Tab B fails while tab C is still in flight — the first failure with the
    // batch total (3) far above the received count (1).
    f.failStash("stashbbbb1");

    // Progress never ran backward across the update's status updates: neither
    // received nor needed decreased, and needed never dropped below received.
    size_t prev_received = 0;
    size_t prev_needed = 0;
    for (size_t i = mark; i < f.status_updates.size(); ++i) {
        const auto &c = f.status_updates[i].counters;
        QVERIFY(c.stashes_received >= prev_received);
        QVERIFY(c.stashes_needed >= prev_needed);
        QVERIFY(c.stashes_needed >= c.stashes_received);
        prev_received = c.stashes_received;
        prev_needed = c.stashes_needed;
    }
    // The batch total was reported at least once and never shrank to fake equality.
    QVERIFY(prev_needed >= 3);
    // Exactly one terminal Ready transition for the failed update (P-FAILURE).
    QCOMPARE(f.readyTransitionsSince(mark), 1);
    QCOMPARE(f.refresh_count, 1);

    // Tab C was still in flight when B failed; settle it Canceled.
    QCOMPARE(f.settleStragglers(), size_t(1));
}

// P-SELECTION (TabsOnly): a TabsOnly update requests BOTH lists (like All) but
// fetches no tab or character content — the lists refresh metadata only. Under
// batching this means each list handler's content batch is empty, and the update
// finishes on the two list receipts alone.
void WorkerUpdateTest::tabsOnlyRequestsBothListsButNoContent()
{
    WorkerFixture f("tabsonly");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Old Name", 0, QStringList{"a1"}));
    const auto char_1 = json::readCharacter(
        characterJson("charid0001", "CharOne", QStringList{"c1"}));
    QVERIFY(tab_a && char_1);
    {
        UserStore store(QDir(f.dataDir()), "tabsonly");
        QVERIFY(store.stashes().saveStashList({*tab_a}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.characters().saveCharacterList({*char_1}));
        QVERIFY(store.characters().saveCharacter(*char_1));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    f.worker->Update(TabSelection::TabsOnly);
    // Both lists are requested, exactly as a full update.
    QCOMPARE(f.callCount(), size_t(2));
    QVERIFY(f.hasPendingStashList());
    QVERIFY(f.hasPendingCharacterList());

    // Each list refreshes metadata (tab A renamed) but launches no content.
    f.deliverStashList(stashList({stashJson("stashaaaa1", "New Name", 0)}));
    f.deliverCharacterList(characterList({characterJson("charid0001", "CharOne")}));

    // No content fetch was ever issued, and the update finished on the lists
    // alone with the fresh metadata applied.
    QCOMPARE(f.callCount(), size_t(2));
    QCOMPARE(f.refresh_count, 2);
    const auto renamed = std::find_if(f.last_tabs.cbegin(),
                                      f.last_tabs.cend(),
                                      [](const ItemLocation &tab) {
                                          return tab.id() == "stashaaaa1";
                                      });
    QVERIFY(renamed != f.last_tabs.cend());
    QCOMPARE(renamed->tab_label(), QString("New Name"));
    // The cached items survived untouched (no content refetch).
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "c1"}));
}

QTEST_GUILESS_MAIN(WorkerUpdateTest)

#include "tst_workerupdate.moc"
