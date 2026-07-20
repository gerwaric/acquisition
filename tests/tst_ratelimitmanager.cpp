// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest>

#include <QDateTime>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QPointer>
#include <QSignalSpy>
#include <QTemporaryDir>

#include <chrono>
#include <stdexcept>

#include "fakenetwork.h"
#include "fakescheduler.h"
#include "fakesender.h"
#include "ratelimit/gate.h"
#include "ratelimit/networkcapture.h"
#include "ratelimit/ratelimit.h"
#include "ratelimit/ratelimitedreply.h"
#include "ratelimit/ratelimitmanager.h"
#include "ratelimit/ratelimitpolicy.h"

// Phase-3 harness for the network redesign (docs/design/network-redesign.md,
// "Testing plan" item 2): the coroutine pump against the same fake sender as
// the phase-1 harness, now with the injected FakeScheduler and the gate, so
// every timing assertion is exact on the fake clock — no coarse-timer slack,
// no real sleeping. Pacing deadlines are still derived from the wall clock
// (GetNextSafeSend is untouched policy arithmetic), so bounds that involve a
// wall-to-fake conversion allow a small slop in the harmless direction.
//
// Deliberately flipped phase-1 pins, per the spec:
//  - F57: a 429 retry no longer destroys the caller's reply; the caller sees
//    exactly one final completion.
//  - F58: the dead 1s spacing is deleted; the gate's MIN_SEND_SPACING floor
//    is the deliberate replacement, pinned exactly below.
//  - Name mismatch: a Full reply naming a different policy leaves the
//    installed policy byte-for-byte un-updated AND completes the request
//    FetchError{Protocol} (D8/IR1 — both halves, as of phase 4a).
//  - Queue-before-install: Update() now starts the drain for held entries
//    instead of leaving them stuck until the next QueueRequest.
//  - F59: resolved by construction (phase 4a). The boundary is
//    QFuture<FetchOutcome>, so no reply object is handed out at all — Qt's
//    shared state is the single owner, and the pump releases every reply it
//    creates, final ones included.
//  - The terminal failed state completes its queued entries Internal instead
//    of dropping them: no caller waits on a promise that never settles.
// Kept pins: terminal 429s (missing or unacceptable Retry-After) and error
// surfacing, both now as typed FetchError values.

// moc-lexer note (see tst_workerupdate.cpp): declare the Q_OBJECT class
// before any helpers containing string literals with '//' in them.
class RateLimitManagerTest : public QObject
{
    Q_OBJECT

private slots:
    void updateInstallsPolicyFromHeadReply();
    void updateAdoptsChangedPolicyDefinition();
    void nameMismatchLeavesPolicyUntouched();
    void queueBeforePolicyInstallDrainsOnInstall();
    void okPolicySendsAfterNormalBuffer();
    void queueDrainsFifoWithGateSpacing();
    void saturatedPolicyWaitsPeriodPlusBucket_data();
    void saturatedPolicyWaitsPeriodPlusBucket();
    void nonRetryableHttpErrorSurfacesReplyAndAdvances();
    void headerlessNetworkFailureSurfacesReplyAndAdvances();
    void successWithViolationStateEmitsViolation();
    void violation429WithoutRetryAfterSurfacesReplyAndPauses();
    void retry429CompletesCallerExactlyOnce();
    void retryAfterPadDominatesWhenLarger();
    void retriesExhaustedCompleteImmediately();
    void unacceptableRetryAfterIsTerminal_data();
    void unacceptableRetryAfterIsTerminal();
    void drainRestartsAfterQueueEmpties();
    void holdInstalledWhileWaitingAtGateIsHonored();
    void observationUpdatesPolicyOnErrorReplies();
    void retrySleepIsPermitFree();
    void senderExceptionFailsPumpTerminally();
};

namespace {

    using namespace std::chrono_literals;

    // Mirrors NORMAL_BUFFER_MSEC in ratelimitmanager.cpp: the pad added to
    // every send while the policy is below BORDERLINE.
    constexpr int kNormalBufferMsec = 100;

    constexpr auto kGateSpacing = RateLimit::Gate::MIN_SEND_SPACING;

    constexpr const char *kPolicyName = "test-request-limit";
    constexpr const char *kEndpoint = "Test Endpoint";

    QByteArray rfcDateNow()
    {
        return QDateTime::currentDateTimeUtc().toString(Qt::RFC2822Date).toUtf8();
    }

    // Synthetic headers for a single-rule ("Ip") policy — the shape GGG
    // actually serves (see the captured example in tst_networkcapture.cpp).
    QList<QNetworkReply::RawHeaderPair> policyHeaders(const QByteArray &limit,
                                                      const QByteArray &state,
                                                      const QByteArray &policy_name = kPolicyName)
    {
        return {
            {"X-Rate-Limit-Policy", policy_name},
            {"X-Rate-Limit-Rules", "Ip"},
            {"X-Rate-Limit-Ip", limit},
            {"X-Rate-Limit-Ip-State", state},
            {"Date", rfcDateNow()},
        };
    }

    // Install a policy the way the hub's HEAD setup does: parse it out of a
    // finished (HEAD-style) reply and hand it to Update().
    void installPolicy(RateLimitManager &manager, const QByteArray &limit, const QByteArray &state)
    {
        FakeNetworkReply head(QNetworkRequest(QUrl("https://api.example.test/head")),
                              "",
                              QNetworkReply::NoError,
                              nullptr,
                              policyHeaders(limit, state),
                              200);
        manager.Update(&head);
    }

    // A caller awaiting a QFuture<FetchOutcome> (D1), plus the observations
    // the pins need: how many times the promise settled (exactly once, ever
    // — the F57 wedge is unconstructible), what it settled to, and the
    // caller's own stop_source so the cancellation pins can abort it.
    //
    // The continuation carries no context object and runs on whatever stack
    // finishes the promise — synchronously for a future that is already
    // finished (S1-6), which the fail-fast paths rely on.
    struct Caller
    {
        std::stop_source source;
        QFuture<RateLimit::FetchOutcome> future;

        int completions = 0;
        bool succeeded = false;
        QByteArray body;
        std::optional<RateLimit::FetchError> error;

        std::stop_token token() { return source.get_token(); }
        void abort() { source.request_stop(); }

        void attach(QFuture<RateLimit::FetchOutcome> f)
        {
            future = f;
            future.then([this](const RateLimit::FetchOutcome &outcome) {
                ++completions;
                succeeded = outcome.has_value();
                if (outcome) {
                    body = *outcome;
                    error.reset();
                } else {
                    body.clear();
                    error = outcome.error();
                }
            });
        }

        // Convenience accessors so the pins read close to the shape they had
        // when the boundary was a QNetworkReply.
        std::optional<RateLimit::FetchError::Kind> kind() const
        {
            return error ? std::optional(error->kind) : std::nullopt;
        }
        int last_status() const { return error ? error->http_status : (succeeded ? 200 : 0); }
        QNetworkReply::NetworkError last_error() const
        {
            return error ? error->network_error : QNetworkReply::NoError;
        }
    };

    QNetworkRequest request(const QString &leaf)
    {
        return QNetworkRequest(QUrl("https://api.example.test/" + leaf));
    }

    // Deliver queued coroutine resumptions (the QFutureWatcher hop) and
    // deferred deletions (deleteLater posted outside any event loop is not
    // delivered by nested processEvents alone). A fixed pass count keeps
    // this deterministic — no wall-clock dependence.
    void drainEvents()
    {
        for (int i = 0; i < 20; ++i) {
            QCoreApplication::processEvents();
            QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        }
    }

    // Interleave due-callback delivery with resumption delivery until a
    // chain (sleep fires -> drain resumes -> gate schedules -> grant fires
    // -> send) has fully played out at the current fake instant.
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

    // The standard fixture: one pump on a fake clock behind a fresh gate.
    // The sender outlives the manager (declared first).
    struct Rig
    {
        FakeScheduler scheduler;
        RateLimit::Gate gate{scheduler};
        FakeSender sender;
        RateLimitManager manager{sender.fcn(), scheduler, gate};
    };

    // A probe-shaped exclusive HEAD occupant for the gate, dispatched like
    // the hub's setup path.
    struct GateBlocker
    {
        bool done = false;
        RateLimit::Gate::Permit permit;
    };

    QCoro::Task<> takeHead(GateBlocker &blocker, RateLimit::Gate &gate)
    {
        blocker.permit = co_await gate.AcquireHead(std::stop_token{});
        blocker.permit.MarkDispatched();
        blocker.done = true;
    }

} // namespace

void RateLimitManagerTest::updateInstallsPolicyFromHeadReply()
{
    Rig rig;

    int policy_updates = 0;
    connect(&rig.manager, &RateLimitManager::PolicyUpdated, [&](const RateLimitPolicy &) {
        ++policy_updates;
    });

    installPolicy(rig.manager, "30:60:60", "0:60:0");

    QCOMPARE(policy_updates, 1);
    QCOMPARE(rig.manager.policy().name(), QString(kPolicyName));
    QCOMPARE(rig.manager.policy().status(), RateLimit::Status::OK);
    QCOMPARE(rig.manager.policy().maximum_hits(), 30);
    // Installing a policy alone never sends anything.
    QCOMPARE(rig.sender.count(), 0);
}

void RateLimitManagerTest::updateAdoptsChangedPolicyDefinition()
{
    Rig rig;

    int policy_updates = 0;
    connect(&rig.manager, &RateLimitManager::PolicyUpdated, [&](const RateLimitPolicy &) {
        ++policy_updates;
    });

    installPolicy(rig.manager, "2:60:60", "0:60:0");
    // A changed definition under the SAME name (limit 2 -> 5) is logged as
    // a mismatch by Check() but adopted: D8 requires dynamic limit changes
    // to update pacing state.
    installPolicy(rig.manager, "5:60:60", "0:60:0");

    QCOMPARE(policy_updates, 2);
    QCOMPARE(rig.manager.policy().maximum_hits(), 5);
}

void RateLimitManagerTest::nameMismatchLeavesPolicyUntouched()
{
    // The flipped phase-1 pin (D8/IR1): a steady-state reply carrying a
    // DIFFERENT policy name is refused loudly — the policy stays
    // byte-for-byte un-updated and no PolicyUpdated fires. The request
    // still completes normally in phase 3: this boundary can only express
    // failure as an errored reply, and this reply is a clean 200 — phase 4
    // turns the completion into FetchError{Protocol}.
    Rig rig;
    installPolicy(rig.manager, "10:60:60", "0:60:0");

    int policy_updates = 0;
    connect(&rig.manager, &RateLimitManager::PolicyUpdated, [&](const RateLimitPolicy &) {
        ++policy_updates;
    });

    Caller c1;
    c1.attach(rig.manager.QueueRequest(kEndpoint, request("one"), c1.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    rig.sender.sent(0).reply->finish(policyHeaders("20:60:60", "1:60:0", "renamed-request-limit"),
                                     200);
    drainEvents();
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status(), 200);
    QCOMPARE(rig.manager.policy().name(), QString(kPolicyName));
    QCOMPARE(rig.manager.policy().maximum_hits(), 10);
    QCOMPARE(policy_updates, 0);
}

void RateLimitManagerTest::queueBeforePolicyInstallDrainsOnInstall()
{
    // Flipped phase-1 pin: entries queued before any policy is installed
    // are held (the drain needs pacing state), but Update() now starts the
    // drain for them — the old code left them stuck until the NEXT
    // QueueRequest arrived.
    Rig rig;

    Caller first;
    Caller second;
    first.attach(rig.manager.QueueRequest(kEndpoint, request("one"), first.token()));
    second.attach(rig.manager.QueueRequest(kEndpoint, request("two"), second.token()));
    QCOMPARE(rig.manager.msecToNextSend(), -1);
    settle(rig.scheduler);
    QCOMPARE(rig.sender.count(), 0);

    installPolicy(rig.manager, "10:60:60", "0:60:0");
    settle(rig.scheduler);
    QCOMPARE(rig.sender.count(), 0); // the normal buffer still applies

    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);
    // FIFO: the first held request goes out first.
    QCOMPARE(rig.sender.sent(0).request.url(), request("one").url());

    rig.sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(first.completions, 1);

    // The second held request follows, behind the gate's spacing floor
    // (measured from the first dispatch on the fake clock).
    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.sender.count(), 2);
    QCOMPARE(rig.sender.sent(1).request.url(), request("two").url());
    rig.sender.sent(1).reply->finish(policyHeaders("10:60:60", "2:60:0"), 200);
    drainEvents();
    QCOMPARE(second.completions, 1);
}

void RateLimitManagerTest::okPolicySendsAfterNormalBuffer()
{
    Rig rig;
    installPolicy(rig.manager, "10:60:60", "0:60:0");

    QSignalSpy paused_spy(&rig.manager, &RateLimitManager::Paused);

    Caller caller;
    const QDateTime before = QDateTime::currentDateTime();
    caller.attach(rig.manager.QueueRequest(kEndpoint, request("one"), caller.token()));

    // Nothing is sent synchronously: with an OK policy the send is
    // scheduled NORMAL_BUFFER_MSEC out, and the wait is announced.
    QCOMPARE(rig.sender.count(), 0);
    const int wait = rig.manager.msecToNextSend();
    QVERIFY2(wait > 0 && wait <= kNormalBufferMsec, qPrintable(QString::number(wait)));
    QCOMPARE(paused_spy.count(), 1);
    QCOMPARE(paused_spy.at(0).at(0).toString(), QString(kPolicyName));
    const qint64 pause = before.msecsTo(paused_spy.at(0).at(1).toDateTime());
    QVERIFY2(pause >= kNormalBufferMsec, qPrintable(QString::number(pause)));
    QVERIFY2(pause <= kNormalBufferMsec + 1000, qPrintable(QString::number(pause)));

    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);
    // In flight: no send is scheduled.
    QCOMPARE(rig.manager.msecToNextSend(), -1);

    rig.sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(caller.completions, 1);
    QCOMPARE(caller.last_status(), 200);
    QCOMPARE(caller.last_error(), QNetworkReply::NoError);
    // F59 is resolved by construction: there is no caller-owned object to
    // destroy. The promise settled exactly once, and the shared state is the
    // single owner of the outcome.
    QVERIFY(caller.future.isFinished());
    QVERIFY(caller.succeeded);
}

void RateLimitManagerTest::queueDrainsFifoWithGateSpacing()
{
    // The F58 flip: consecutive sends are spaced by the gate's
    // MIN_SEND_SPACING floor, measured from the previous dispatch stamp on
    // the fake clock — deliberately, at the right scope, instead of the
    // dead per-manager 1s interval the old code never applied.
    Rig rig;
    installPolicy(rig.manager, "10:60:60", "0:60:0");

    QSignalSpy queue_spy(&rig.manager, &RateLimitManager::QueueUpdated);

    Caller c1;
    Caller c2;
    c1.attach(rig.manager.QueueRequest(kEndpoint, request("one"), c1.token()));
    c2.attach(rig.manager.QueueRequest(kEndpoint, request("two"), c2.token()));

    // The drain picks up the first request immediately (0 left waiting);
    // the second joins the queue behind it (1 waiting).
    QCOMPARE(queue_spy.count(), 2);
    QCOMPARE(queue_spy.at(0).at(1).toInt(), 0);
    QCOMPARE(queue_spy.at(1).at(1).toInt(), 1);

    // First dispatch at t=100 on the fake clock.
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);
    QCOMPARE(rig.sender.sent(0).request.url(), request("one").url());
    rig.sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(c1.completions, 1);

    // The second request's pacing sleep ends at ~t=200, but the gate holds
    // it to exactly t=100+MIN_SEND_SPACING=350.
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);
    advanceAndSettle(rig.scheduler,
                     kGateSpacing - std::chrono::milliseconds(kNormalBufferMsec) - 1ms);
    QCOMPARE(rig.sender.count(), 1);
    advanceAndSettle(rig.scheduler, 1ms);
    QCOMPARE(rig.sender.count(), 2);
    QCOMPARE(rig.sender.sent(1).request.url(), request("two").url());
    rig.sender.sent(1).reply->finish(policyHeaders("10:60:60", "2:60:0"), 200);
    drainEvents();
    QCOMPARE(c2.completions, 1);
}

void RateLimitManagerTest::saturatedPolicyWaitsPeriodPlusBucket_data()
{
    QTest::addColumn<QByteArray>("limit");
    QTest::addColumn<QByteArray>("ok_state");
    QTest::addColumn<QByteArray>("full_state");
    QTest::addColumn<int>("expected_wait_msec");

    // The wait is period + timing bucket + 1s buffer, measured from the
    // newest relevant history event (ratelimitpolicy.cpp). Periods at or
    // below the 75s cutoff use the 5s initial bucket; longer periods use
    // the 60s sustained bucket.
    QTest::newRow("short-period") << QByteArray("1:60:60") << QByteArray("0:60:0")
                                  << QByteArray("1:60:0") << 66000;
    QTest::newRow("long-period") << QByteArray("1:80:60") << QByteArray("0:80:0")
                                 << QByteArray("1:80:0") << 141000;
}

void RateLimitManagerTest::saturatedPolicyWaitsPeriodPlusBucket()
{
    QFETCH(QByteArray, limit);
    QFETCH(QByteArray, ok_state);
    QFETCH(QByteArray, full_state);
    QFETCH(int, expected_wait_msec);

    Rig rig;
    installPolicy(rig.manager, limit, ok_state);

    Caller c1;
    c1.attach(rig.manager.QueueRequest(kEndpoint, request("one"), c1.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    const QDateTime before_finish = QDateTime::currentDateTime();
    rig.sender.sent(0).reply->finish(policyHeaders(limit, full_state), 200);
    drainEvents();
    QCOMPARE(c1.completions, 1);
    QCOMPARE(rig.manager.policy().status(), RateLimit::Status::BORDERLINE);

    // The saturated policy schedules the next request a full
    // period-plus-bucket out, announced via Paused, and sends nothing in
    // the meantime. The deadline is derived from wall-clock history events,
    // so the fake-clock wait allows a small wall-elapsed slop downward.
    QSignalSpy paused_spy(&rig.manager, &RateLimitManager::Paused);
    Caller c2;
    c2.attach(rig.manager.QueueRequest(kEndpoint, request("two"), c2.token()));

    QCOMPARE(rig.sender.count(), 1);
    const int wait = rig.manager.msecToNextSend();
    QVERIFY2(wait >= expected_wait_msec - 1000, qPrintable(QString::number(wait)));
    QVERIFY2(wait <= expected_wait_msec, qPrintable(QString::number(wait)));

    QCOMPARE(paused_spy.count(), 1);
    const qint64 pause = before_finish.msecsTo(paused_spy.at(0).at(1).toDateTime());
    QVERIFY2(pause >= expected_wait_msec - 1000, qPrintable(QString::number(pause)));
    QVERIFY2(pause <= expected_wait_msec + 1000, qPrintable(QString::number(pause)));

    advanceAndSettle(rig.scheduler, 250ms);
    QCOMPARE(rig.sender.count(), 1);
    QCOMPARE(c2.completions, 0);
}

void RateLimitManagerTest::nonRetryableHttpErrorSurfacesReplyAndAdvances()
{
    Rig rig;
    installPolicy(rig.manager, "10:60:60", "0:60:0");

    QSignalSpy violation_spy(&rig.manager, &RateLimitManager::Violation);

    Caller c1;
    Caller c2;
    c1.attach(rig.manager.QueueRequest(kEndpoint, request("one"), c1.token()));
    c2.attach(rig.manager.QueueRequest(kEndpoint, request("two"), c2.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    // A non-429 HTTP error is not retried: the errored reply is surfaced to
    // the caller as a completion and the queue advances.
    rig.sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"),
                                     500,
                                     QNetworkReply::InternalServerError);
    drainEvents();
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status(), 500);
    QCOMPARE(c1.last_error(), QNetworkReply::InternalServerError);
    QCOMPARE(c1.kind(), RateLimit::FetchError::Kind::Http);
    QCOMPARE(violation_spy.count(), 0);

    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.sender.count(), 2);
    rig.sender.sent(1).reply->finish(policyHeaders("10:60:60", "2:60:0"), 200);
    drainEvents();
    QCOMPARE(c2.completions, 1);
}

void RateLimitManagerTest::headerlessNetworkFailureSurfacesReplyAndAdvances()
{
    Rig rig;
    installPolicy(rig.manager, "10:60:60", "0:60:0");

    Caller c1;
    Caller c2;
    c1.attach(rig.manager.QueueRequest(kEndpoint, request("one"), c1.token()));
    c2.attach(rig.manager.QueueRequest(kEndpoint, request("two"), c2.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    // A network-level failure produces a reply without rate-limit headers.
    // It is surfaced to the caller as a failed completion and the queue
    // advances; the headers fail the total parse, so the policy is left
    // alone (the exchange still records its history event — D3).
    rig.sender.sent(0).reply->finish({}, 0, QNetworkReply::ConnectionRefusedError);
    drainEvents();
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status(), 0);
    QCOMPARE(c1.last_error(), QNetworkReply::ConnectionRefusedError);
    QCOMPARE(c1.kind(), RateLimit::FetchError::Kind::Network);
    QCOMPARE(rig.manager.policy().status(), RateLimit::Status::OK);

    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.sender.count(), 2);
    rig.sender.sent(1).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(c2.completions, 1);
}

void RateLimitManagerTest::successWithViolationStateEmitsViolation()
{
    Rig rig;
    installPolicy(rig.manager, "2:60:60", "0:60:0");

    QSignalSpy violation_spy(&rig.manager, &RateLimitManager::Violation);

    Caller c1;
    c1.attach(rig.manager.QueueRequest(kEndpoint, request("one"), c1.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    // A 200 whose headers report state above the limit is still completed
    // normally, but the violation is detected and signalled.
    rig.sender.sent(0).reply->finish(policyHeaders("2:60:60", "3:60:60"), 200);
    drainEvents();
    QCOMPARE(violation_spy.count(), 1);
    QCOMPARE(violation_spy.at(0).at(0).toString(), QString(kPolicyName));
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status(), 200);
}

void RateLimitManagerTest::violation429WithoutRetryAfterSurfacesReplyAndPauses()
{
    Rig rig;
    installPolicy(rig.manager, "2:60:60", "0:60:0");

    QSignalSpy violation_spy(&rig.manager, &RateLimitManager::Violation);

    Caller c1;
    Caller c2;
    c1.attach(rig.manager.QueueRequest(kEndpoint, request("one"), c1.token()));
    c2.attach(rig.manager.QueueRequest(kEndpoint, request("two"), c2.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    // A 429 with no Retry-After is NOT retried: it is a terminal 429 (D3)
    // and the reply is surfaced to the caller. The violation is recorded
    // regardless.
    rig.sender.sent(0).reply->finish(policyHeaders("2:60:60", "3:60:60"),
                                     429,
                                     QNetworkReply::UnknownContentError);
    drainEvents();
    QCOMPARE(violation_spy.count(), 1);
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status(), 429);
    QCOMPARE(c1.kind(), RateLimit::FetchError::Kind::RateLimited);
    // A terminal 429 with no acceptable Retry-After says so in the value.
    QCOMPARE(c1.error->retry_after_acceptable, false);
    QCOMPARE(c1.error->attempts, 1);

    // The violated policy then paces the next request a full
    // period-plus-bucket out (60 + 5 + 1 seconds) instead of sending it.
    QCOMPARE(rig.sender.count(), 1);
    const int wait = rig.manager.msecToNextSend();
    QVERIFY2(wait >= 65000, qPrintable(QString::number(wait)));
    QVERIFY2(wait <= 66000, qPrintable(QString::number(wait)));
    advanceAndSettle(rig.scheduler, 200ms);
    QCOMPARE(rig.sender.count(), 1);
    QCOMPARE(c2.completions, 0);
}

void RateLimitManagerTest::retry429CompletesCallerExactlyOnce()
{
    // The F57 flip: the retry is a drain-loop iteration, invisible to the
    // caller — no destroyed reply, no dropped completion, exactly one final
    // completion when the retried send succeeds. The wedge is gone.
    Rig rig;
    installPolicy(rig.manager, "2:60:60", "0:60:0");

    QSignalSpy violation_spy(&rig.manager, &RateLimitManager::Violation);

    Caller victim;
    victim.attach(rig.manager.QueueRequest(kEndpoint, request("one"), victim.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    auto retry_headers = policyHeaders("2:60:60", "3:60:60");
    retry_headers.append(QNetworkReply::RawHeaderPair{"Retry-After", "1"});
    rig.sender.sent(0).reply->finish(retry_headers, 429, QNetworkReply::UnknownContentError);
    drainEvents();

    // Nothing has completed yet: the retry is invisible to the caller.
    QVERIFY(!victim.future.isFinished());
    QCOMPARE(victim.completions, 0);
    QCOMPARE(violation_spy.count(), 1);

    // The retry deadline is max(now + RetryAfter + pad + buffer,
    // GetNextSafeSend): here the violated policy's 66s dominates the
    // 1+60+1=62s Retry-After formula.
    const int wait = rig.manager.msecToNextSend();
    QVERIFY2(wait >= 65000, qPrintable(QString::number(wait)));
    QVERIFY2(wait <= 66000, qPrintable(QString::number(wait)));

    advanceAndSettle(rig.scheduler, 66s);
    QCOMPARE(rig.sender.count(), 2);
    // The same request is resent, and the intermediate 429 reply has been
    // released by the pump's dispatch-time owner (exactly once — the
    // fake sender's QPointer nulls).
    QCOMPARE(rig.sender.sent(1).request.url(), request("one").url());
    QVERIFY(rig.sender.sent(0).reply.isNull());

    // The retried request succeeds: the caller sees exactly one final
    // completion, carrying bytes rather than a reply. The FINAL reply is now
    // released by the pump's dispatch-time owner too (the flipped half of
    // F59): no reply object escapes the limiter any more, so there is
    // nothing for a caller to own, leak, or destroy at the wrong moment.
    rig.sender.sent(1).reply->finish(policyHeaders("2:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(victim.completions, 1);
    QCOMPARE(victim.last_status(), 200);
    QVERIFY(victim.succeeded);
    QVERIFY(rig.sender.sent(1).reply.isNull());
}

void RateLimitManagerTest::retryAfterPadDominatesWhenLarger()
{
    // The N19 pad pin: when Retry-After + RETRY_BUCKET_PAD + buffer exceeds
    // GetNextSafeSend, the retry honors the padded server deadline —
    // RA=10 gives 10+60+1=71s against the policy's 66s.
    Rig rig;
    installPolicy(rig.manager, "2:60:60", "0:60:0");

    Caller victim;
    victim.attach(rig.manager.QueueRequest(kEndpoint, request("one"), victim.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    auto retry_headers = policyHeaders("2:60:60", "3:60:60");
    retry_headers.append(QNetworkReply::RawHeaderPair{"Retry-After", "10"});
    rig.sender.sent(0).reply->finish(retry_headers, 429, QNetworkReply::UnknownContentError);
    drainEvents();

    QCOMPARE(victim.completions, 0);
    const int wait = rig.manager.msecToNextSend();
    QVERIFY2(wait >= 70000, qPrintable(QString::number(wait)));
    QVERIFY2(wait <= 71000, qPrintable(QString::number(wait)));

    // Not resent before the padded deadline.
    advanceAndSettle(rig.scheduler, 66s);
    QCOMPARE(rig.sender.count(), 1);
    advanceAndSettle(rig.scheduler, 5s);
    QCOMPARE(rig.sender.count(), 2);
}

void RateLimitManagerTest::retriesExhaustedCompleteImmediately()
{
    // MAX_ATTEMPTS is 3 total sends: after the third 429 the entry
    // completes immediately with the final 429 — never a sleep on an
    // exhausted attempt (D3). Retry-After 0 is valid (the pad dominates).
    Rig rig;
    installPolicy(rig.manager, "2:60:60", "0:60:0");

    Caller victim;
    victim.attach(rig.manager.QueueRequest(kEndpoint, request("one"), victim.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));

    auto retry_headers = policyHeaders("2:60:60", "3:60:60");
    retry_headers.append(QNetworkReply::RawHeaderPair{"Retry-After", "0"});

    for (int attempt = 0; attempt < 3; ++attempt) {
        QCOMPARE(rig.sender.count(), attempt + 1);
        rig.sender.sent(attempt).reply->finish(retry_headers,
                                               429,
                                               QNetworkReply::UnknownContentError);
        drainEvents();
        if (attempt < 2) {
            QCOMPARE(victim.completions, 0);
            advanceAndSettle(rig.scheduler, 70s);
        }
    }

    // Terminal, immediately: one completion carrying the final 429, no
    // resend scheduled.
    QCOMPARE(victim.completions, 1);
    QCOMPARE(victim.last_status(), 429);
    QCOMPARE(victim.kind(), RateLimit::FetchError::Kind::RateLimited);
    // Retries were exhausted, not refused: an acceptable Retry-After was
    // present on every attempt.
    QCOMPARE(victim.error->retry_after_acceptable, true);
    QCOMPARE(victim.error->attempts, 3);
    QCOMPARE(rig.sender.count(), 3);
    QCOMPARE(rig.manager.msecToNextSend(), -1);
}

void RateLimitManagerTest::unacceptableRetryAfterIsTerminal_data()
{
    // The R7 grammar/product-policy vectors: negative and non-numeric fail
    // the grammar; above-the-cap is declined as product policy. All are
    // terminal 429s — the violation is recorded, no retry is scheduled.
    QTest::addColumn<QByteArray>("retry_after");

    QTest::newRow("negative") << QByteArray("-5");
    QTest::newRow("non-numeric") << QByteArray("soon");
    QTest::newRow("above-cap") << QByteArray("901");
}

void RateLimitManagerTest::unacceptableRetryAfterIsTerminal()
{
    QFETCH(QByteArray, retry_after);

    Rig rig;
    installPolicy(rig.manager, "2:60:60", "0:60:0");

    QSignalSpy violation_spy(&rig.manager, &RateLimitManager::Violation);

    Caller victim;
    victim.attach(rig.manager.QueueRequest(kEndpoint, request("one"), victim.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    auto headers = policyHeaders("2:60:60", "3:60:60");
    headers.append(QNetworkReply::RawHeaderPair{"Retry-After", retry_after});
    rig.sender.sent(0).reply->finish(headers, 429, QNetworkReply::UnknownContentError);
    drainEvents();

    QCOMPARE(victim.completions, 1);
    QCOMPARE(victim.last_status(), 429);
    QCOMPARE(violation_spy.count(), 1);
    QCOMPARE(rig.manager.msecToNextSend(), -1);
    QCOMPARE(rig.sender.count(), 1);
}

void RateLimitManagerTest::drainRestartsAfterQueueEmpties()
{
    // Drain lifecycle: the drain exits when the deque empties; a later
    // submission starts a new one (submit-while-running joins are pinned by
    // the FIFO test above).
    Rig rig;
    installPolicy(rig.manager, "10:60:60", "0:60:0");

    Caller c1;
    c1.attach(rig.manager.QueueRequest(kEndpoint, request("one"), c1.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);
    rig.sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(c1.completions, 1);

    Caller c2;
    c2.attach(rig.manager.QueueRequest(kEndpoint, request("two"), c2.token()));
    QVERIFY(rig.manager.msecToNextSend() > 0);
    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.sender.count(), 2);
    rig.sender.sent(1).reply->finish(policyHeaders("10:60:60", "2:60:0"), 200);
    drainEvents();
    QCOMPARE(c2.completions, 1);
}

void RateLimitManagerTest::holdInstalledWhileWaitingAtGateIsHonored()
{
    // The review-#2 pin: a hold installed while an entry is already
    // suspended in the gate wait (a HEAD-429 probe joining this pump —
    // the shared-policy case, D4) must still be honored. The entry
    // releases its permit undispatched, sleeps out the hold, and
    // reacquires. The capture rides along to pin that the hold becomes
    // the recorded scheduling decision.
    QTemporaryDir capture_dir;
    QVERIFY(capture_dir.isValid());
    const QString capture_path = capture_dir.filePath("capture.jsonl");

    FakeScheduler scheduler;
    RateLimit::Gate gate{scheduler};
    FakeSender sender;
    NetworkCapture capture{capture_path};
    RateLimitManager manager{sender.fcn(), scheduler, gate, &capture};
    installPolicy(manager, "10:60:60", "0:60:0");

    // An exclusive HEAD holds the whole gate, dispatched at t=0.
    GateBlocker head;
    auto head_task = takeHead(head, gate);
    QVERIFY(head.done && head.permit.valid());

    // The entry paces normally, then parks in Acquire() behind the HEAD.
    Caller caller;
    caller.attach(manager.QueueRequest(kEndpoint, request("one"), caller.token()));
    advanceAndSettle(scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(sender.count(), 0);

    // The probe 429s and joins this pump: a 180s hold lands while the
    // entry is still suspended at the gate; then the HEAD releases.
    manager.HoldUntil(scheduler.Now() + 180s);
    head.permit.Release();

    // The entry is granted at the spacing floor (t=250, from the HEAD's
    // dispatch stamp) but must not send: it gives the permit back
    // undispatched and sleeps to the hold deadline — installed at t=100,
    // so exactly 179750ms remain when queried at t=350.
    advanceAndSettle(scheduler, kGateSpacing);
    QCOMPARE(sender.count(), 0);
    QCOMPARE(manager.msecToNextSend(), 179750);

    advanceAndSettle(scheduler, 180s);
    QCOMPARE(sender.count(), 1);
    sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(caller.completions, 1);
    QCOMPARE(caller.last_status(), 200);

    // The capture recorded the hold as the entry's scheduling decision:
    // 'scheduled' must be the hold deadline, not the stale pre-hold
    // pacing time. Fake time swallowed the 180s wait while wall time
    // barely moved, so the honest record puts 'scheduled' far AFTER
    // 'sent' (both wall-clock) — the stale value would sit within
    // milliseconds of it.
    QFile capture_file(capture_path);
    QVERIFY(capture_file.open(QIODevice::ReadOnly));
    const QList<QByteArray> lines = capture_file.readAll().split('\n');
    QCOMPARE(lines.size(), 2); // one record plus the trailing newline
    const QJsonObject record = QJsonDocument::fromJson(lines[0]).object();
    QCOMPARE(record["kind"].toString(), QString("reply"));
    const QDateTime scheduled = QDateTime::fromString(record["scheduled"].toString(),
                                                      Qt::ISODateWithMs);
    const QDateTime sent = QDateTime::fromString(record["sent"].toString(), Qt::ISODateWithMs);
    QVERIFY(scheduled.isValid() && sent.isValid());
    const qint64 announced_hold_msec = sent.msecsTo(scheduled);
    QVERIFY2(announced_hold_msec > 170000, qPrintable(QString::number(announced_hold_msec)));
    QVERIFY2(announced_hold_msec <= 180000, qPrintable(QString::number(announced_hold_msec)));
}

void RateLimitManagerTest::observationUpdatesPolicyOnErrorReplies()
{
    // Observation precedes classification (R6-3/D8): a 500 and a
    // 2xx-plus-transport-error carrying Full matching headers update the
    // policy while completing their status/network-driven outcomes.
    Rig rig;
    installPolicy(rig.manager, "10:60:60", "0:60:0");

    Caller c1;
    Caller c2;
    c1.attach(rig.manager.QueueRequest(kEndpoint, request("one"), c1.token()));
    c2.attach(rig.manager.QueueRequest(kEndpoint, request("two"), c2.token()));
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.sender.count(), 1);

    // A 500 with a changed limit definition: the error is surfaced, the
    // policy is updated anyway.
    rig.sender.sent(0).reply->finish(policyHeaders("20:60:60", "1:60:0"),
                                     500,
                                     QNetworkReply::InternalServerError);
    drainEvents();
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status(), 500);
    QCOMPARE(rig.manager.policy().maximum_hits(), 20);

    // A 2xx whose transport failed mid-body (Qt reports both a status and
    // an error): completes as a failure, still updates the policy.
    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.sender.count(), 2);
    rig.sender.sent(1).reply->finish(policyHeaders("25:60:60", "2:60:0"),
                                     200,
                                     QNetworkReply::RemoteHostClosedError);
    drainEvents();
    QCOMPARE(c2.completions, 1);
    QCOMPARE(c2.last_error(), QNetworkReply::RemoteHostClosedError);
    QCOMPARE(rig.manager.policy().maximum_hits(), 25);
}

void RateLimitManagerTest::retrySleepIsPermitFree()
{
    // R4-1: a permit is released at reply-finish and never held across a
    // retry sleep. With the gate's cap at 2, two pumps parked in retry
    // sleeps would otherwise hold both permits — the third pump's send
    // proves the slots were freed.
    FakeScheduler scheduler;
    RateLimit::Gate gate{scheduler};
    FakeSender sender_a;
    FakeSender sender_b;
    FakeSender sender_c;
    RateLimitManager manager_a{sender_a.fcn(), scheduler, gate};
    RateLimitManager manager_b{sender_b.fcn(), scheduler, gate};
    RateLimitManager manager_c{sender_c.fcn(), scheduler, gate};
    installPolicy(manager_a, "2:60:60", "0:60:0");
    installPolicy(manager_b, "2:60:60", "0:60:0");
    installPolicy(manager_c, "2:60:60", "0:60:0");

    Caller ca;
    Caller cb;
    ca.attach(manager_a.QueueRequest(kEndpoint, request("a"), ca.token()));
    cb.attach(manager_b.QueueRequest(kEndpoint, request("b"), cb.token()));
    advanceAndSettle(scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    advanceAndSettle(scheduler, kGateSpacing);
    QCOMPARE(sender_a.count(), 1);
    QCOMPARE(sender_b.count(), 1);

    // Both pumps take a retryable 429 and enter ~66s retry sleeps.
    auto retry_headers = policyHeaders("2:60:60", "3:60:60");
    retry_headers.append(QNetworkReply::RawHeaderPair{"Retry-After", "1"});
    sender_a.sent(0).reply->finish(retry_headers, 429, QNetworkReply::UnknownContentError);
    sender_b.sent(0).reply->finish(retry_headers, 429, QNetworkReply::UnknownContentError);
    drainEvents();
    QVERIFY(manager_a.msecToNextSend() > 60000);
    QVERIFY(manager_b.msecToNextSend() > 60000);

    // While both sleep, the third pump acquires the gate and sends.
    Caller cc;
    cc.attach(manager_c.QueueRequest(kEndpoint, request("c"), cc.token()));
    advanceAndSettle(scheduler, kGateSpacing);
    QCOMPARE(sender_c.count(), 1);
    QVERIFY(manager_a.msecToNextSend() > 0);
    QVERIFY(manager_b.msecToNextSend() > 0);
}

void RateLimitManagerTest::senderExceptionFailsPumpTerminally()
{
    // The exception pins (IR4/R5-1): an exception escaping into the drain
    // is contained — the pump fails terminally and loudly, every queued
    // entry is completed Internal, and later submissions are refused the
    // same way. No crash, no restart, and no caller left waiting forever.
    FakeScheduler scheduler;
    RateLimit::Gate gate{scheduler};
    RateLimitManager manager{[](QNetworkRequest &) -> QNetworkReply * {
                                 throw std::runtime_error("sender exploded");
                             },
                             scheduler,
                             gate};
    installPolicy(manager, "10:60:60", "0:60:0");

    Caller victim;
    Caller queued;
    victim.attach(manager.QueueRequest(kEndpoint, request("one"), victim.token()));
    queued.attach(manager.QueueRequest(kEndpoint, request("two"), queued.token()));
    advanceAndSettle(scheduler, std::chrono::milliseconds(kNormalBufferMsec));

    // The active entry and everything queued behind it are completed
    // Internal rather than dropped: the terminal state fails callers, it
    // does not strand them on a promise that never settles. The active
    // entry's completion comes from the drain's scoped completion guard.
    QCOMPARE(victim.completions, 1);
    QCOMPARE(victim.kind(), RateLimit::FetchError::Kind::Internal);
    QCOMPARE(queued.completions, 1);
    QCOMPARE(queued.kind(), RateLimit::FetchError::Kind::Internal);
    QCOMPARE(manager.msecToNextSend(), -1);

    // Later submissions are refused — but cleanly and immediately: one
    // Internal completion, no send attempt, and nothing scheduled that could
    // complete it a second time later.
    Caller after;
    after.attach(manager.QueueRequest(kEndpoint, request("after"), after.token()));
    settle(scheduler);
    QCOMPARE(after.completions, 1);
    QCOMPARE(after.kind(), RateLimit::FetchError::Kind::Internal);
    advanceAndSettle(scheduler, 1s);
    QCOMPARE(after.completions, 1);
}

QTEST_GUILESS_MAIN(RateLimitManagerTest)

#include "tst_ratelimitmanager.moc"
