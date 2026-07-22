// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest>

#include <QDateTime>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QPointer>
#include <QSettings>

#include <chrono>
#include <expected>
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

    // I-CANCEL-* : a worker update aborts (first-failure from a sibling lane)
    // while the victim lane is suspended at each pump checkpoint; the chain
    // completes Canceled and the worker discards the straggler (§6).
    void i_cancel_pacing();
    void i_cancel_gate();
    void i_cancel_flight();

    // I-SHUT-* : worker-then-hub destruction while the pump is suspended at each
    // checkpoint; nothing resumes after destruction (§6).
    void i_shut_pacing();
    void i_shut_gate();
    void i_shut_flight();
    void i_shut_retry();

    // I-LEAK-BOUND : at shutdown only the detached frame leaks; every owner and
    // reply outside its closure is destroyed (§6).
    void i_leak_bound();

    // I-RETENTION : across a long full-chain update the completed
    // payload/future population does not accumulate; the deferred sweep releases
    // completed frames (§6).
    void i_retention();
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

    // A minimal stash-tab JSON object (metadata/items default). Ids match the
    // datastore seed so the worker reconciles and selects them.
    QByteArray stashTabJson(const QString &id, int index)
    {
        return QString(R"({"id":"%1","name":"Tab %2","type":"PremiumStash","index":%3})")
            .arg(id, QString::number(index + 1), QString::number(index))
            .toUtf8();
    }

    // A StashListWrapper body listing the given ids in order.
    QByteArray stashListBody(const QStringList &ids)
    {
        QByteArrayList tabs;
        for (int i = 0; i < ids.size(); ++i) {
            tabs.append(stashTabJson(ids.at(i), i));
        }
        return "{\"stashes\":[" + tabs.join(",") + "]}";
    }

    // A StashWrapper body for one tab (empty items — the retention pin cares
    // about frame/future lifetime, not payload contents).
    QByteArray stashContentBody(const QString &id, int index)
    {
        QByteArray tab = stashTabJson(id, index);
        tab.chop(1); // drop the closing brace to append items
        return "{\"stash\":" + tab + ",\"items\":[]}}";
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

    // A pass-through recording layer over the REAL facade (used only by
    // I-CANCEL-FLIGHT). It delegates to PoeApiClient for the actual request
    // building and parsing, then chains one recording continuation onto the
    // returned future so the test can observe the OUTCOME the worker received —
    // success payload vs a FetchError kind. This is what lets the flight pin
    // distinguish "the pump delivered Canceled" from "the pump delivered success
    // and the worker's own post-await token gate discarded it" (the two are
    // otherwise byte-identical at the worker). A second continuation on the
    // facade's returned future is safe: it has none of its own yet, and the
    // worker awaits THIS future via a QCoroFutureWatcher, a separate mechanism.
    class RecordingApiClient : public PoeApiClient
    {
    public:
        using PoeApiClient::PoeApiClient;

        struct Outcome
        {
            bool settled = false;
            bool success = false;
            RateLimit::FetchError::Kind kind{};
        };

        Outcome stash_list;
        Outcome character_list;

        Result<poe::StashListWrapper> listStashes(const QString &realm,
                                                  const QString &league,
                                                  std::stop_token token = {}) override
        {
            return record(PoeApiClient::listStashes(realm, league, token), stash_list);
        }

        Result<poe::CharacterListWrapper> listCharacters(const QString &realm,
                                                         std::stop_token token = {}) override
        {
            return record(PoeApiClient::listCharacters(realm, token), character_list);
        }

    private:
        template<typename T>
        Result<T> record(Result<T> future, Outcome &slot)
        {
            return future.then([&slot](const std::expected<T, RateLimit::FetchError> &value) {
                slot.settled = true;
                slot.success = value.has_value();
                slot.kind = value ? RateLimit::FetchError::Kind{} : value.error().kind;
                return value;
            });
        }
    };

    // The real worker -> real facade -> real hub chain, with a FakeScheduler and
    // a FakeNetworkManager at the edges. Every layer is a value or a unique_ptr
    // so a scenario can destroy the consumers before the hub and the fake
    // network last, matching UserSession ownership (§6 step 3).
    struct Rig
    {
        // tap=true swaps the plain facade for the recording layer and turns on
        // the hub's network capture, so a scenario can prove both the delivered
        // outcome (via the tap) and that a reply landed and was recorded (via
        // the capture file) — the two halves I-CANCEL-FLIGHT needs.
        explicit Rig(const QString &account, bool tap = false)
            : settings(bm.tempDir.filePath("settings.ini"), QSettings::IniFormat)
            , account_name(account)
        {
            settings.setValue("realm", kRealm);
            settings.setValue("league", kLeague);
            settings.setValue("account", account);
            settings.sync();

            network = std::make_unique<FakeNetworkManager>();
            limiter = std::make_unique<RateLimiter>(*network, &scheduler);
            if (tap) {
                auto recording = std::make_unique<RecordingApiClient>(*limiter);
                recorder = recording.get();
                api = std::move(recording);
                // EnableCapture must precede the first submission (hub contract).
                limiter->EnableCapture(capturePath());
            } else {
                api = std::make_unique<PoeApiClient>(*limiter);
            }
            worker = std::make_unique<ItemsManagerWorker>(settings, *bm.manager, *api);
            WorkerTestAccess(*worker).setSweepObserver(&sweep);

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
                             [this](ProgramState state, const QString &message) {
                                 ++status_updates;
                                 if (state == ProgramState::Ready) {
                                     ++ready_transitions;
                                     last_ready_message = message;
                                 }
                             });
            // A processed (not discarded) list emits this; a canceled straggler
            // must NOT, which is how the cancel scenarios prove discard.
            QObject::connect(worker.get(),
                             &ItemsManagerWorker::stashListReceived,
                             worker.get(),
                             [this](const std::vector<poe::StashTab> &,
                                    const QString &,
                                    const QString &) { ++stash_list_received; });
        }

        WorkerTestAccess access() const { return WorkerTestAccess(*worker); }

        QString dataDir() const { return bm.tempDir.filePath("data"); }

        QString capturePath() const { return bm.tempDir.filePath("capture.jsonl"); }

        // True if the hub's capture recorded a landed reply for a URL matching
        // `url_contains` with the given HTTP status. A reply is captured in
        // RecordLandedReply — BEFORE the post-land cancellation checkpoint — so a
        // record here proves the reply landed and was recorded even though its
        // caller was later completed Canceled (the "recorded" half of §6
        // I-CANCEL-FLIGHT). Capture is flushed per line, so it is readable while
        // the limiter is still alive.
        bool captureHasReply(const QString &url_contains, int status) const
        {
            QFile file(capturePath());
            if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
                return false;
            }
            while (!file.atEnd()) {
                const QJsonObject record = QJsonDocument::fromJson(file.readLine()).object();
                if (record.value("kind").toString() == "reply"
                    && record.value("url").toString().contains(url_contains)
                    && record.value("status").toInt() == status) {
                    return true;
                }
            }
            return false;
        }

        // Seed N stash tabs into the datastore so a Selected update of them needs
        // exactly the stash list — one endpoint. The Selected list-need logic
        // keys off already-known tabs (m_tabs), so an unseeded selection would
        // fetch nothing. Returns the locations to pass to Update().
        std::vector<ItemLocation> seedStashTabs(int n)
        {
            std::vector<poe::StashTab> stashes;
            std::vector<ItemLocation> locations;
            for (int i = 0; i < n; ++i) {
                poe::StashTab stash;
                stash.id = QString("stash%1").arg(i + 1, 5, 10, QChar('0'));
                stash.name = QString("Integration Tab %1").arg(i + 1);
                stash.type = "PremiumStash";
                stash.index = static_cast<unsigned>(i);
                stashes.push_back(stash);
                locations.push_back(ItemLocation(stash));
            }
            UserStore store(QDir(dataDir()), account_name);
            if (!store.stashes().saveStashList(stashes, kRealm, kLeague)) {
                qFatal("seedStashTabs: failed to persist the seed list");
            }
            for (const auto &stash : stashes) {
                if (!store.stashes().saveStash(stash, kRealm, kLeague)) {
                    qFatal("seedStashTabs: failed to persist a seed tab");
                }
            }
            return locations;
        }

        // Seed one stash tab, then run the worker to Idle: OnRePoEReady starts
        // the (real) parse thread, which reads the cache and emits the initial
        // ItemsRefreshed. Wait for that on the event loop, not the fake scheduler
        // (the parse thread is a real QThread unrelated to the hub's clock).
        // Stores the seeded tabs' locations for Update() to select.
        void seedAndReachIdle(int tab_count = 1)
        {
            selections = seedStashTabs(tab_count);
            selection = selections.front();
            worker->OnRePoEReady();
            QTRY_COMPARE_WITH_TIMEOUT(initial_refreshes, 1, 10000);
        }

        BuyoutManagerFixture bm;
        QSettings settings;
        QString account_name;
        ItemLocation selection;
        std::vector<ItemLocation> selections;
        FakeScheduler scheduler;
        // Declared before the worker so it outlives it: the worker holds a raw
        // pointer to it and writes sweep counts through it (verification §2).
        WorkerSweepObserver sweep;
        std::unique_ptr<FakeNetworkManager> network;
        std::unique_ptr<RateLimiter> limiter;
        std::unique_ptr<PoeApiClient> api;
        // Non-owning alias to `api` when the recording tap is enabled (null
        // otherwise), so a scenario can read the outcomes the worker received.
        RecordingApiClient *recorder = nullptr;
        std::unique_ptr<ItemsManagerWorker> worker;

        int initial_refreshes = 0;
        int update_refreshes = 0;
        int status_updates = 0;
        int ready_transitions = 0;
        int stash_list_received = 0;
        QString last_ready_message;
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

namespace {

    QString urlOf(const FakeNetworkManager::Sent &sent) { return sent.request.url().toString(); }

    // The index of the single sent request matching an operation and a URL
    // fragment (the facade's list URLs are .../stash/<league> and
    // .../character), or -1. Distinguishes the two lanes without assuming a
    // dispatch order.
    int findSent(Rig &rig, QNetworkAccessManager::Operation op, const QString &url_contains)
    {
        for (int i = 0; i < rig.network->count(); ++i) {
            if (rig.network->sent(i).op == op && urlOf(rig.network->sent(i)).contains(url_contains)) {
                return i;
            }
        }
        return -1;
    }

    // A TabsOnly update needs both lists (two endpoints); the stash list is
    // submitted first, so its HEAD probe is sent(0). Drive to there.
    void beginTabsOnly(Rig &rig)
    {
        rig.worker->Update(TabSelection::TabsOnly);
        settle(rig.scheduler);
        QCOMPARE(rig.network->count(), 1);
        QCOMPARE(rig.network->sent(0).op, QNetworkAccessManager::HeadOperation);
        QVERIFY(urlOf(rig.network->sent(0)).contains("/stash/"));
    }

    // Establish the stash-list endpoint from its HEAD. When `saturate`, the
    // policy comes back at its limit so the list request's pacing deadline is a
    // full period out — it stays in the pacing sleep while the sibling fails.
    void establishStashList(Rig &rig, bool saturate)
    {
        const QByteArray limit = saturate ? "1:60:60" : "10:60:60";
        const QByteArray state = saturate ? "1:60:0" : "0:60:0";
        rig.network->sent(0).reply->finish(policyHeaders(limit, state), 200);
        drainEvents();
    }

    // Past the gate spacing floor the stash HEAD charged (t=250), the exclusive
    // character HEAD dispatches as sent(1). The stash-list GET cannot dispatch
    // while a HEAD is pending (writer preference), so it is parked at pacing
    // (saturated) or the gate (not) — exactly where the abort must find it.
    void dispatchCharHead(Rig &rig)
    {
        advanceAndSettle(rig.scheduler, kGateSpacing);
        QCOMPARE(rig.network->count(), 2);
        QCOMPARE(rig.network->sent(1).op, QNetworkAccessManager::HeadOperation);
        QVERIFY(urlOf(rig.network->sent(1)).contains("/character"));
    }

    // The shared cancel proof for the pacing/gate variants: fail the character
    // HEAD (a setup failure) so the worker's first-failure aborts the update and
    // stops the shared token, then settle. The victim stash-list request — parked
    // at its checkpoint — must wake Canceled without ever sending its GET, and
    // the worker must discard it (no stashListReceived, no refresh) and finish on
    // exactly one terminal failure.
    void abortViaCharHeadAndProve(Rig &rig)
    {
        const int ready_before = rig.ready_transitions;
        rig.network->sent(1).reply->finish(policyHeaders(), 403, QNetworkReply::ContentAccessDenied);
        settle(rig.scheduler);

        // The stash GET was never sent (only the two HEADs), and the victim was
        // discarded, not processed.
        QCOMPARE(rig.network->count(), 2);
        QCOMPARE(rig.stash_list_received, 0);
        QCOMPARE(rig.update_refreshes, 0);
        // Exactly one terminal failure transition.
        QCOMPARE(rig.ready_transitions, ready_before + 1);
        QVERIFY(rig.last_ready_message.contains("failed"));
        // The whole chain settled: both list coroutines resumed and the sweep
        // reclaimed them — nothing left suspended.
        QCOMPARE(rig.access().outstandingFetchTasks(), size_t(0));
    }

} // namespace

void WorkerIntegrationTest::i_cancel_pacing()
{
    Rig rig("integration-cancel-pacing");
    rig.seedAndReachIdle();
    beginTabsOnly(rig);
    establishStashList(rig, /*saturate=*/true); // stash list parks in pacing sleep
    dispatchCharHead(rig);
    abortViaCharHeadAndProve(rig);
}

void WorkerIntegrationTest::i_cancel_gate()
{
    Rig rig("integration-cancel-gate");
    rig.seedAndReachIdle();
    beginTabsOnly(rig);
    establishStashList(rig, /*saturate=*/false); // stash list clears pacing, waits at the gate
    dispatchCharHead(rig);
    abortViaCharHeadAndProve(rig);
}

void WorkerIntegrationTest::i_cancel_flight()
{
    // tap=true: record the outcome the worker received and turn on capture, so
    // this pin can prove BOTH halves the worker-visible state cannot (the
    // worker's own token gate discards a straggler regardless of what the pump
    // delivered, so "no refresh / one failure" alone can't tell pump-Canceled
    // from pump-success).
    Rig rig("integration-cancel-flight", /*tap=*/true);
    rig.seedAndReachIdle();
    beginTabsOnly(rig);

    // Establish both endpoints so both list GETs can be in flight at once. The
    // character HEAD must fully establish here — a still-pending HEAD would block
    // the stash GET by writer preference. The two endpoints get DISTINCT policy
    // names so the hub routes them to separate managers: same-policy endpoints
    // share one serial pump, which would queue the char GET behind the in-flight
    // stash GET instead of running them concurrently under the gate's cap of 2.
    establishStashList(rig, /*saturate=*/false);
    dispatchCharHead(rig);
    rig.network->sent(1).reply->finish(policyHeaders("10:60:60", "0:60:0", "character-policy"), 200);
    drainEvents();

    // Advance the clock a gate-spacing step at a time until both list GETs have
    // dispatched. One big jump would not work: the gate stamps each send's
    // spacing floor at the coroutine's RESUME instant (the current fake time),
    // so the two sends must be paced out across separate advances, exactly as
    // the hub's own tests do.
    for (int step = 0; step < 12 && findSent(rig, QNetworkAccessManager::GetOperation, "/character") < 0;
         ++step) {
        advanceAndSettle(rig.scheduler, kGateSpacing);
    }
    const int stash_get = findSent(rig, QNetworkAccessManager::GetOperation, "/stash/");
    const int char_get = findSent(rig, QNetworkAccessManager::GetOperation, "/character");
    QVERIFY2(stash_get >= 0, "the stash-list GET must be in flight");
    QVERIFY2(char_get >= 0, "the character-list GET must be in flight");

    QPointer<InFlightReply> victim = rig.network->sent(stash_get).reply;
    QVERIFY(!victim.isNull());
    const int ready_before = rig.ready_transitions;

    // The sibling fails in flight -> worker aborts -> shared token stops while
    // the victim is still in flight.
    rig.network->sent(char_get).reply->finish(policyHeaders(), 500, QNetworkReply::InternalServerError);
    settle(rig.scheduler);

    // Cancellation never aborts the in-flight reply (D2/R6-1): the victim is
    // still alive and unfinished, waiting to land.
    QVERIFY2(!victim.isNull(), "the in-flight reply must not be aborted by cancellation");
    QVERIFY(!victim->isFinished());

    // The reply lands and is recorded; the post-land checkpoint sees the stopped
    // token and completes it Canceled. The worker discards it.
    rig.network->sent(stash_get).reply->finish(policyHeaders("10:60:60", "1:60:0"),
                                               200,
                                               QNetworkReply::NoError,
                                               R"({"stashes":[]})");
    settle(rig.scheduler);

    QCOMPARE(rig.stash_list_received, 0);
    QCOMPARE(rig.update_refreshes, 0);
    QCOMPARE(rig.ready_transitions, ready_before + 1);
    QVERIFY(rig.last_ready_message.contains("failed"));
    QCOMPARE(rig.access().outstandingFetchTasks(), size_t(0));

    // The two halves the worker-visible state cannot show, isolating the pump's
    // post-land cancellation checkpoint (ratelimitmanager.cpp:421):
    // (1) the pump delivered the victim Canceled — NOT a success the worker's own
    //     token gate then discarded (that would leave the assertions above
    //     identical). A mutation that skipped the post-land token check would make
    //     this a successful StashListWrapper and fail here.
    QVERIFY2(rig.recorder->stash_list.settled, "the victim lane must have completed");
    QVERIFY2(!rig.recorder->stash_list.success,
             "the pump must deliver a failure, not a success the worker discarded");
    QCOMPARE(rig.recorder->stash_list.kind, RateLimit::FetchError::Kind::Canceled);
    // (2) the reply LANDED and was RECORDED before that cancellation (checkpoint
    //     placement — after RecordLandedReply, not before): the capture holds a
    //     200 reply record for the victim URL even though its caller got Canceled.
    //     A mutation moving cancellation before the record would fail this.
    QVERIFY2(rig.captureHasReply("/stash/Standard", 200),
             "the victim reply must be recorded as landed despite being canceled");
}

void WorkerIntegrationTest::i_leak_bound()
{
    Rig rig("integration-leak-bound");
    rig.seedAndReachIdle();

    // Owners that must be destroyed at shutdown, not leaked. Both are QObjects,
    // so a QPointer nulls when the object is destroyed.
    QPointer<ItemsManagerWorker> worker_obj = rig.worker.get();
    QPointer<RateLimiter> limiter_obj = rig.limiter.get();

    // Drive the list request into flight (one suspended per-fetch frame).
    rig.worker->Update(TabSelection::Selected, {rig.selection});
    settle(rig.scheduler);
    QCOMPARE(rig.network->count(), 1);
    rig.network->sent(0).reply->finish(policyHeaders(), 200);
    drainEvents();
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    advanceAndSettle(rig.scheduler, kGateSpacing - std::chrono::milliseconds(kNormalBufferMsec));
    const int list_get = findSent(rig, QNetworkAccessManager::GetOperation, "/stash/");
    QVERIFY(list_get >= 0);
    QPointer<InFlightReply> reply = rig.network->sent(list_get).reply;
    QVERIFY(!reply.isNull());
    QCOMPARE(rig.access().outstandingFetchTasks(), size_t(1));

    // Shutdown in UserSession order. The suspended frame detaches and leaks by
    // accepted design (§6) — but nothing OUTSIDE its closure may: the worker and
    // hub owners are destroyed as their unique_ptrs reset, and the in-flight
    // reply (owned by the QNAM parent, not the frame) is freed when the network
    // dies. No processEvents() runs after destruction.
    rig.worker.reset();
    QVERIFY2(worker_obj.isNull(), "the worker owner must be destroyed, not leaked");
    rig.api.reset();
    rig.limiter.reset();
    QVERIFY2(limiter_obj.isNull(), "the hub owner must be destroyed, not leaked");
    QVERIFY2(!reply.isNull(), "the in-flight reply outlives the hub (R6-1)");
    rig.network.reset();
    QVERIFY2(reply.isNull(),
             "the reply, outside the detached frame's closure, must be freed by its QNAM parent");
}

void WorkerIntegrationTest::i_retention()
{
    Rig rig("integration-retention");
    constexpr int kTabs = 6;
    rig.seedAndReachIdle(kTabs);

    QStringList ids;
    for (const auto &loc : rig.selections) {
        ids.append(loc.id());
    }

    // A Selected update of all seeded tabs: the stash list, then a content batch
    // of kTabs stash fetches.
    rig.worker->Update(TabSelection::Selected, rig.selections);
    settle(rig.scheduler);
    QCOMPARE(rig.network->count(), 1);
    QCOMPARE(rig.network->sent(0).op, QNetworkAccessManager::HeadOperation);

    // Establish the list endpoint and return the whole list.
    rig.network->sent(0).reply->finish(policyHeaders(), 200);
    drainEvents();
    int list_get = -1;
    for (int step = 0; step < 12 && list_get < 0; ++step) {
        advanceAndSettle(rig.scheduler, kGateSpacing);
        list_get = findSent(rig, QNetworkAccessManager::GetOperation, "/stash/Standard");
        // The content GETs also match "/stash/"; pin the list GET to the bare
        // league URL so a content GET is never mistaken for it.
        if (list_get >= 0 && urlOf(rig.network->sent(list_get)) != "https://api.pathofexile.com/stash/Standard") {
            list_get = -1;
        }
    }
    QVERIFY(list_get >= 0);
    rig.network->sent(list_get).reply->finish(policyHeaders("10:60:60", "1:60:0"),
                                              200,
                                              QNetworkReply::NoError,
                                              stashListBody(ids));
    drainEvents();

    // Drive the whole content batch to completion, finishing each content HEAD
    // (endpoint setup) and content GET as it dispatches. Track the peak number
    // of outstanding per-fetch handles: the batch launches all at once, so the
    // peak is the launch set — it must NOT grow per completion, and the deferred
    // sweep must drain it back to zero as replies land.
    size_t peak = 0;
    for (int step = 0; step < 200 && rig.update_refreshes == 0; ++step) {
        peak = std::max(peak, rig.access().outstandingFetchTasks());
        for (int i = 0; i < rig.network->count(); ++i) {
            const FakeNetworkManager::Sent &s = rig.network->sent(i);
            if (s.reply.isNull() || s.reply->isFinished()) {
                continue;
            }
            const QString last = urlOf(s).section('/', -1);
            if (s.op == QNetworkAccessManager::HeadOperation) {
                s.reply->finish(policyHeaders(), 200); // establish the content endpoint
            } else if (last.startsWith("stash")) {
                s.reply->finish(policyHeaders("10:60:60", "1:60:0"),
                                200,
                                QNetworkReply::NoError,
                                stashContentBody(last, 0));
            }
        }
        drainEvents();
        advanceAndSettle(rig.scheduler, kGateSpacing);
    }

    // The update succeeded exactly once...
    QCOMPARE(rig.update_refreshes, 1);
    // ...and every per-fetch frame was reclaimed by the deferred sweep — nothing
    // accumulated for the update's lifetime.
    QCOMPARE(rig.access().outstandingFetchTasks(), size_t(0));
    QCOMPARE(rig.sweep.live_after, size_t(0));
    // One stash-list handle plus kTabs content handles, all destroyed.
    QVERIFY2(rig.sweep.destroyed >= 1 + kTabs, "the sweep must reclaim every completed handle");
    // The population never grew beyond the launched set: 1 list + kTabs content.
    // (A small allowance covers a not-yet-swept completion coexisting with the
    // launch set; the point is it is bounded, not linear in completions.)
    QVERIFY2(peak <= size_t(1 + kTabs + 1), "the outstanding-handle population must stay bounded");
}

QTEST_GUILESS_MAIN(WorkerIntegrationTest)

#include "tst_workerintegration.moc"
