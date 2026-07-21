// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest>

#include <QDateTime>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QPointer>
#include <QSettings>

#include <chrono>
#include <memory>

#include "datastore/stashrepo.h"
#include "datastore/userstore.h"
#include "fakenetworkmanager.h"
#include "fakescheduler.h"
#include "fakesender.h" // InFlightReply
#include "itemsmanagerworker.h"
#include "poe/poeapiclient.h"
#include "ratelimit/gate.h"
#include "ratelimit/ratelimiter.h"
#include "testfixtures.h"
#include "workertestaccess.h"

// Phase-5 full-chain integration, shutdown, and retention (network-redesign
// verification §6). Every other worker test fakes the facade one level above
// the hub; these drive the REAL worker -> real PoeApiClient -> real RateLimiter
// hub, with only a FakeScheduler and a FakeNetworkManager at the edges. That is
// the only seam that can exercise cross-layer cancellation at each pump
// checkpoint and the post-event-loop destruction contract the facade fake
// cannot reach (D2, "Shutdown and task ownership," testing-plan item 6).
//
// Isolation is the whole point (§6): each shutdown proof intentionally leaves
// detached QCoro frames and depends on NO event-loop iteration after owner
// destruction, so a shared Qt Test process that continued to a later method
// would violate the premise. Each scenario is its own private slot, and CMake
// registers ONE CTest per slot (COMMAND tst_workerintegration <slot>), so
// exactly one destructive scenario runs per process (QTEST_GUILESS_MAIN
// forwards the slot-name argument to QTest::qExec).
//
// A single established endpoint reaches all three pump suspension points just by
// how far the scheduler is advanced: the list request paces until t=100ms, then
// waits at the gate for the 250ms spacing floor the HEAD's t=0 dispatch charged,
// then dispatches and sits in flight until its reply is finished. No production
// timing is asserted here — those constants are pinned in tst_ratelimiter and
// tst_ratelimitmanager; this file uses them only to park the chain determinist_
// ally.

// moc-lexer note (see tst_workerupdate.cpp / tst_ratelimiter.cpp): the Q_OBJECT
// class must be declared before any helper holding a string literal with '//'
// in it (the request URLs below), or moc loses the class.
class WorkerIntegrationTest : public QObject
{
    Q_OBJECT

private slots:
    // A wiring sanity check, not an §6 pin: a list fetch flows worker -> facade
    // -> hub -> fake network and back, proving the full chain is assembled right
    // before the destructive scenarios lean on it.
    void fullChainStashListSucceeds();

    // I-SHUT-* : worker-then-hub destruction while the pump is suspended at each
    // checkpoint; nothing resumes after destruction (§6).
    void i_shut_pacing();
    void i_shut_gate();
    void i_shut_flight();
    void i_shut_retry();
};

namespace {

    using namespace std::chrono_literals;

    constexpr const char *kRealm = "pc";
    constexpr const char *kLeague = "Standard";

    constexpr auto kGateSpacing = RateLimit::Gate::MIN_SEND_SPACING; // 250ms
    constexpr int kNormalBufferMsec = 100;

    QByteArray rfcDateNow()
    {
        return QDateTime::currentDateTimeUtc().toString(Qt::RFC2822Date).toUtf8();
    }

    // The rate-limit headers that establish a single-rule policy from a HEAD
    // probe (identical shape to tst_ratelimiter's helper). limit/state are the
    // "hits:period:restriction" triples the parser reads.
    QList<QNetworkReply::RawHeaderPair> policyHeaders(
        const QByteArray &limit = "10:60:60",
        const QByteArray &state = "0:60:0",
        const QByteArray &policy_name = "test-request-limit")
    {
        return {
            {"X-Rate-Limit-Policy", policy_name},
            {"X-Rate-Limit-Rules", "Ip"},
            {"X-Rate-Limit-Ip", limit},
            {"X-Rate-Limit-Ip-State", state},
            {"Date", rfcDateNow()},
        };
    }

    void drainEvents()
    {
        for (int i = 0; i < 20; ++i) {
            QCoreApplication::processEvents();
            QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        }
    }

    // Interleave queued coroutine resumptions with scheduler callbacks at the
    // current fake instant: a resumed frame may schedule more work due now, so
    // AdvanceBy(0) then drain, repeatedly (the tst_ratelimiter idiom).
    void settle(FakeScheduler &scheduler)
    {
        for (int i = 0; i < 10; ++i) {
            drainEvents();
            scheduler.AdvanceBy(0ms);
        }
        drainEvents();
    }

    void advanceAndSettle(FakeScheduler &scheduler, std::chrono::milliseconds delta)
    {
        scheduler.AdvanceBy(delta);
        settle(scheduler);
    }

    // The real worker -> real facade -> real hub chain, with a FakeScheduler and
    // a FakeNetworkManager at the edges. Every layer is a value or a unique_ptr
    // so a scenario can destroy the consumers before the hub and the fake
    // network last, matching UserSession ownership (§6 step 3).
    struct Rig
    {
        explicit Rig(const QString &account)
            : settings(bm.tempDir.filePath("settings.ini"), QSettings::IniFormat)
            , account_name(account)
        {
            settings.setValue("realm", kRealm);
            settings.setValue("league", kLeague);
            settings.setValue("account", account);
            settings.sync();

            network = std::make_unique<FakeNetworkManager>();
            limiter = std::make_unique<RateLimiter>(*network, &scheduler);
            api = std::make_unique<PoeApiClient>(*limiter);
            worker = std::make_unique<ItemsManagerWorker>(settings, *bm.manager, *api);

            QObject::connect(worker.get(),
                             &ItemsManagerWorker::ItemsRefreshed,
                             worker.get(),
                             [this](const Items &items,
                                    const std::vector<ItemLocation> &,
                                    bool initial) {
                                 if (initial) {
                                     ++initial_refreshes;
                                 } else {
                                     ++update_refreshes;
                                 }
                                 last_item_count = items.size();
                             });
            QObject::connect(worker.get(),
                             &ItemsManagerWorker::StatusUpdate,
                             worker.get(),
                             [this](ProgramState state, const QString &) {
                                 ++status_updates;
                                 if (state == ProgramState::Ready) {
                                     ++ready_transitions;
                                 }
                             });
        }

        WorkerTestAccess access() const { return WorkerTestAccess(*worker); }

        QString dataDir() const { return bm.tempDir.filePath("data"); }

        // Seed one stash tab into the datastore so a Selected update of it needs
        // exactly the stash list — one endpoint. The Selected list-need logic
        // keys off already-known tabs (m_tabs), so an unseeded selection would
        // fetch nothing. Returns the location to pass to Update().
        ItemLocation seedOneStashTab()
        {
            poe::StashTab stash;
            stash.id = "stash00001";
            stash.name = "Integration Tab";
            stash.type = "PremiumStash";
            stash.index = 0;

            UserStore store(QDir(dataDir()), account_name);
            if (!store.stashes().saveStashList({stash}, kRealm, kLeague)
                || !store.stashes().saveStash(stash, kRealm, kLeague)) {
                qFatal("seedOneStashTab: failed to persist the seed tab");
            }
            return ItemLocation(stash);
        }

        // Seed one stash tab, then run the worker to Idle: OnRePoEReady starts
        // the (real) parse thread, which reads the cache and emits the initial
        // ItemsRefreshed. Wait for that on the event loop, not the fake scheduler
        // (the parse thread is a real QThread unrelated to the hub's clock).
        // Stores the seeded tab's location for Update() to select.
        void seedAndReachIdle()
        {
            selection = seedOneStashTab();
            worker->OnRePoEReady();
            QTRY_COMPARE_WITH_TIMEOUT(initial_refreshes, 1, 10000);
        }

        BuyoutManagerFixture bm;
        QSettings settings;
        QString account_name;
        ItemLocation selection;
        FakeScheduler scheduler;
        std::unique_ptr<FakeNetworkManager> network;
        std::unique_ptr<RateLimiter> limiter;
        std::unique_ptr<PoeApiClient> api;
        std::unique_ptr<ItemsManagerWorker> worker;

        int initial_refreshes = 0;
        int update_refreshes = 0;
        int status_updates = 0;
        int ready_transitions = 0;
        size_t last_item_count = 0;
    };

} // namespace

void WorkerIntegrationTest::fullChainStashListSucceeds()
{
    Rig rig("integration-sanity");
    rig.seedAndReachIdle();

    // A Selected update of one stash tab needs the stash list only.
    rig.worker->Update(TabSelection::Selected, {rig.selection});
    settle(rig.scheduler);

    // The hub parked the list request and fired a HEAD probe for its endpoint.
    QCOMPARE(rig.network->count(), 1);
    QCOMPARE(rig.network->sent(0).op, QNetworkAccessManager::HeadOperation);

    // Establish the endpoint: the parked list request forwards into the pump.
    rig.network->sent(0).reply->finish(policyHeaders(), 200);
    drainEvents();

    // Pace (t=100) and clear the gate spacing floor the HEAD charged (t=250).
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    advanceAndSettle(rig.scheduler, kGateSpacing - std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.network->count(), 2);
    QCOMPARE(rig.network->sent(1).op, QNetworkAccessManager::GetOperation);

    // The list comes back empty: the update needs no content and finishes.
    rig.network->sent(1).reply->finish(policyHeaders("10:60:60", "1:60:0"),
                                       200,
                                       QNetworkReply::NoError,
                                       R"({"stashes":[]})");
    drainEvents();
    settle(rig.scheduler);

    QCOMPARE(rig.update_refreshes, 1);
    QCOMPARE(rig.access().outstandingFetchTasks(), size_t(0));
}

namespace {

    // Drive a fresh Selected update to the named pump suspension and return with
    // the chain parked there. After this returns, exactly the HEAD (and, for
    // flight/retry, the list GET) has been sent; the list request's coroutine is
    // suspended at the checkpoint.
    enum class Checkpoint { Pacing, Gate, Flight, Retry };

    void driveToCheckpoint(Rig &rig, Checkpoint where)
    {
        rig.worker->Update(TabSelection::Selected, {rig.selection});
        settle(rig.scheduler);
        QCOMPARE(rig.network->count(), 1);
        QCOMPARE(rig.network->sent(0).op, QNetworkAccessManager::HeadOperation);

        // Establish the endpoint; the list request enters the pump and parks in
        // the pacing sleep (deadline t=100).
        rig.network->sent(0).reply->finish(policyHeaders(), 200);
        drainEvents();
        QCOMPARE(rig.network->count(), 1); // still only the HEAD; the GET is paced

        if (where == Checkpoint::Pacing) {
            return;
        }

        // Past pacing (t=100) but before the gate spacing floor (t=250): the
        // request is now suspended at the gate.
        advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
        QCOMPARE(rig.network->count(), 1);
        if (where == Checkpoint::Gate) {
            return;
        }

        // Past the spacing floor: the GET dispatches and sits in flight.
        advanceAndSettle(rig.scheduler,
                         kGateSpacing - std::chrono::milliseconds(kNormalBufferMsec));
        QCOMPARE(rig.network->count(), 2);
        QCOMPARE(rig.network->sent(1).op, QNetworkAccessManager::GetOperation);
        if (where == Checkpoint::Flight) {
            return;
        }

        // Retry: land a 429 with a valid Retry-After so the pump enters the
        // retry sleep instead of completing.
        QList<QNetworkReply::RawHeaderPair> headers = policyHeaders("10:60:60", "1:60:0");
        headers.append(QNetworkReply::RawHeaderPair{"Retry-After", "5"});
        rig.network->sent(1).reply->finish(headers, 429, QNetworkReply::UnknownContentError);
        drainEvents();
        // The retry sleep is now armed; no further send until it elapses.
        QCOMPARE(rig.network->count(), 2);
    }

    // The shared shutdown proof: with the chain parked at `where`, capture the
    // in-flight reply (if any), then destroy consumers before the hub and the
    // fake network last (UserSession order, §6 step 3). Assert nothing resumed
    // after destruction and that owners/replies outside the detached-frame
    // closure were freed. NO processEvents() runs after destruction (§6 step 5).
    void proveShutdown(Rig &rig, Checkpoint where)
    {
        driveToCheckpoint(rig, where);

        // The update is genuinely suspended mid-fetch: exactly one per-fetch
        // coroutine is live (the list request, parked at the checkpoint) and the
        // update has emitted no refresh. Without this the scenario could be
        // "proving" a chain that already completed. (ready_transitions is not
        // asserted absolutely here: the initial cache parse emits its own Ready
        // status, so it is only meaningful as a delta across destruction below.)
        QCOMPARE(rig.access().outstandingFetchTasks(), size_t(1));
        QCOMPARE(rig.update_refreshes, 0);

        // Baselines no post-destruction resumption may cross: a resumed
        // coroutine would drive the worker to a terminal Ready or a refresh.
        const int refreshes_before = rig.update_refreshes;
        const int ready_before = rig.ready_transitions;

        // The in-flight reply (flight/retry) is owned by the fake QNAM, not by
        // the detached frame: it must be freed when the network is destroyed.
        QPointer<InFlightReply> reply;
        if (where == Checkpoint::Flight || where == Checkpoint::Retry) {
            reply = rig.network->sent(1).reply;
            QVERIFY(!reply.isNull());
        }

        // Step 1: consumers die first. The worker detaches its suspended per-
        // fetch frame (S1-1); nothing resumes it, because no loop iteration runs
        // after this point.
        rig.worker.reset();
        rig.api.reset();

        // Step 2: the hub dies. Its pumps' drain frames detach; a still-in-flight
        // reply is NOT aborted and NOT freed here (R6-1) — its QNAM parent frees
        // it next.
        rig.limiter.reset();
        if (where == Checkpoint::Flight || where == Checkpoint::Retry) {
            QVERIFY2(!reply.isNull(), "the in-flight reply must outlive the hub (R6-1)");
        }

        // Step 3: the fake network parent dies last, freeing the reply.
        rig.network.reset();
        if (where == Checkpoint::Flight || where == Checkpoint::Retry) {
            QVERIFY2(reply.isNull(), "the QNAM parent must free the in-flight reply at shutdown");
        }

        // Nothing resumed across any destruction: no terminal transition, no
        // refresh. (Recorders are lambdas bound to the worker, already gone; the
        // counters captured above are the sentinel.)
        QCOMPARE(rig.update_refreshes, refreshes_before);
        QCOMPARE(rig.ready_transitions, ready_before);
    }

} // namespace

void WorkerIntegrationTest::i_shut_pacing()
{
    Rig rig("integration-shut-pacing");
    rig.seedAndReachIdle();
    proveShutdown(rig, Checkpoint::Pacing);
}

void WorkerIntegrationTest::i_shut_gate()
{
    Rig rig("integration-shut-gate");
    rig.seedAndReachIdle();
    proveShutdown(rig, Checkpoint::Gate);
}

void WorkerIntegrationTest::i_shut_flight()
{
    Rig rig("integration-shut-flight");
    rig.seedAndReachIdle();
    proveShutdown(rig, Checkpoint::Flight);
}

void WorkerIntegrationTest::i_shut_retry()
{
    Rig rig("integration-shut-retry");
    rig.seedAndReachIdle();
    proveShutdown(rig, Checkpoint::Retry);
}

QTEST_GUILESS_MAIN(WorkerIntegrationTest)

#include "tst_workerintegration.moc"
