// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest>

#include <QDateTime>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QPointer>
#include <QSignalSpy>

#include "fakenetwork.h"
#include "fakesender.h"
#include "ratelimit/ratelimit.h"
#include "ratelimit/ratelimitedreply.h"
#include "ratelimit/ratelimitmanager.h"
#include "ratelimit/ratelimitpolicy.h"

// Phase-1 harness for the network redesign (docs/design/network-redesign.md,
// "Testing plan" item 1): pins the CURRENT behavior of RateLimitManager
// against a fake sender serving synthetic X-Rate-Limit-* headers. This layer
// had zero coverage — FakeRateLimiter overrides Submit() and bypasses the
// managers entirely — which is exactly how F57 shipped.
//
// Several pins document known-broken behavior on purpose, most notably the
// F57 wedge (a 429 retry destroys the caller's RateLimitedReply, so the
// retried completion is dropped) and the F58 dead spacing code. The
// coroutine pump (phase 3) must pass this harness minus those pins, flipped
// deliberately — not silently.

// moc-lexer note (see tst_workerupdate.cpp): declare the Q_OBJECT class
// before any helpers containing string literals with '//' in them.
class RateLimitManagerTest : public QObject
{
    Q_OBJECT

private slots:
    void updateInstallsPolicyFromHeadReply();
    void updateReplacesMismatchedPolicy();
    void queueBeforePolicyInstallHoldsRequests();
    void okPolicySendsAfterNormalBuffer();
    void queueDrainsFifoWithoutMinimumIntervalSpacing();
    void saturatedPolicyWaitsPeriodPlusBucket_data();
    void saturatedPolicyWaitsPeriodPlusBucket();
    void nonRetryableHttpErrorSurfacesReplyAndAdvances();
    void headerlessNetworkFailureSurfacesReplyAndAdvances();
    void successWithViolationStateEmitsViolation();
    void violation429WithoutRetryAfterSurfacesReplyAndPauses();
    void f57RetryDestroysCallerReplyAndDropsCompletion();
};

namespace {

    // Mirrors NORMAL_BUFFER_MSEC in ratelimitmanager.cpp: the pad added to
    // every send while the policy is below BORDERLINE.
    constexpr int kNormalBufferMsec = 100;

    constexpr const char *kPolicyName = "test-request-limit";
    constexpr const char *kEndpoint = "Test Endpoint";

    QByteArray rfcDateNow()
    {
        return QDateTime::currentDateTimeUtc().toString(Qt::RFC2822Date).toUtf8();
    }

    // Synthetic headers for a single-rule ("Ip") policy — the shape GGG
    // actually serves (see the captured example in tst_networkcapture.cpp).
    QList<QNetworkReply::RawHeaderPair> policyHeaders(const QByteArray &limit,
                                                      const QByteArray &state)
    {
        return {
            {"X-Rate-Limit-Policy", kPolicyName},
            {"X-Rate-Limit-Rules", "Ip"},
            {"X-Rate-Limit-Ip", limit},
            {"X-Rate-Limit-Ip-State", state},
            {"Date", rfcDateNow()},
        };
    }

    // Install a policy the way RateLimiter::ProcessHeadResponse does: parse
    // it out of a finished (HEAD-style) reply and hand it to Update().
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

    // A caller-side RateLimitedReply plus the observations the pins need:
    // completion count, the surfaced reply's status and error, and whether
    // the manager has destroyed the object yet (F59: the manager owns it via
    // unique_ptr and deletes it synchronously right after emitting complete).
    struct Caller
    {
        QPointer<RateLimitedReply> reply;
        int completions = 0;
        int last_status = 0;
        QNetworkReply::NetworkError last_error = QNetworkReply::NoError;

        Caller()
            : reply(new RateLimitedReply)
        {
            QObject::connect(reply, &RateLimitedReply::complete, [this](QNetworkReply *r) {
                ++completions;
                last_status = RateLimit::ParseStatus(r);
                last_error = r->error();
            });
        }
    };

    QNetworkRequest request(const QString &leaf)
    {
        return QNetworkRequest(QUrl("https://api.example.test/" + leaf));
    }

} // namespace

void RateLimitManagerTest::updateInstallsPolicyFromHeadReply()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());

    int policy_updates = 0;
    connect(&manager, &RateLimitManager::PolicyUpdated, [&](const RateLimitPolicy &) {
        ++policy_updates;
    });

    installPolicy(manager, "30:60:60", "0:60:0");

    QCOMPARE(policy_updates, 1);
    QCOMPARE(manager.policy().name(), QString(kPolicyName));
    QCOMPARE(manager.policy().status(), RateLimit::Status::OK);
    QCOMPARE(manager.policy().maximum_hits(), 30);
    // Installing a policy alone never sends anything.
    QCOMPARE(sender.count(), 0);
}

void RateLimitManagerTest::updateReplacesMismatchedPolicy()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());

    int policy_updates = 0;
    connect(&manager, &RateLimitManager::PolicyUpdated, [&](const RateLimitPolicy &) {
        ++policy_updates;
    });

    installPolicy(manager, "2:60:60", "0:60:0");
    // A mismatched update (limit changed 2 -> 5) is logged but silently
    // replaces the policy. (The pump's IR1 contract differs: a mismatched
    // steady-state reply must leave the policy un-updated.)
    installPolicy(manager, "5:60:60", "0:60:0");

    QCOMPARE(policy_updates, 2);
    QCOMPARE(manager.policy().maximum_hits(), 5);
}

void RateLimitManagerTest::queueBeforePolicyInstallHoldsRequests()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());

    // Queueing before any policy is installed holds the request: activation
    // bails on the null policy and the timer is never started. Update()
    // does not re-activate either — the request stays held until the next
    // QueueRequest arrives. This is the Update-before-Queue contract that
    // RateLimiter::ProcessHeadResponse maintains.
    Caller first;
    manager.QueueRequest(kEndpoint, request("one"), first.reply);
    QCOMPARE(manager.msecToNextSend(), -1);
    QTest::qWait(150);
    QCOMPARE(sender.count(), 0);

    installPolicy(manager, "10:60:60", "0:60:0");
    QTest::qWait(150);
    QCOMPARE(sender.count(), 0);

    Caller second;
    manager.QueueRequest(kEndpoint, request("two"), second.reply);
    QTRY_COMPARE(sender.count(), 1);
    // FIFO: the held request goes out first.
    QCOMPARE(sender.sent(0).request.url(), request("one").url());

    sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    QCOMPARE(first.completions, 1);

    QTRY_COMPARE(sender.count(), 2);
    QCOMPARE(sender.sent(1).request.url(), request("two").url());
    sender.sent(1).reply->finish(policyHeaders("10:60:60", "2:60:0"), 200);
    QCOMPARE(second.completions, 1);
}

void RateLimitManagerTest::okPolicySendsAfterNormalBuffer()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());
    installPolicy(manager, "10:60:60", "0:60:0");

    QSignalSpy paused_spy(&manager, &RateLimitManager::Paused);

    Caller caller;
    const QDateTime before = QDateTime::currentDateTime();
    QElapsedTimer elapsed;
    elapsed.start();
    manager.QueueRequest(kEndpoint, request("one"), caller.reply);

    // Nothing is sent synchronously: with an OK policy the send is
    // scheduled NORMAL_BUFFER_MSEC out, and the wait is announced.
    QCOMPARE(sender.count(), 0);
    QVERIFY(manager.msecToNextSend() >= 0);
    QVERIFY2(manager.msecToNextSend() <= kNormalBufferMsec + 50,
             qPrintable(QString::number(manager.msecToNextSend())));
    QCOMPARE(paused_spy.count(), 1);
    QCOMPARE(paused_spy.at(0).at(0).toString(), QString(kPolicyName));
    const qint64 pause = before.msecsTo(paused_spy.at(0).at(1).toDateTime());
    QVERIFY2(pause >= kNormalBufferMsec, qPrintable(QString::number(pause)));
    QVERIFY2(pause <= kNormalBufferMsec + 1000, qPrintable(QString::number(pause)));

    QTRY_COMPARE(sender.count(), 1);
    // Coarse timers may fire up to ~5% early.
    QVERIFY2(elapsed.elapsed() >= 90, qPrintable(QString::number(elapsed.elapsed())));

    sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    QCOMPARE(caller.completions, 1);
    QCOMPARE(caller.last_status, 200);
    QCOMPARE(caller.last_error, QNetworkReply::NoError);
    // F59 pin: the manager destroyed the caller's reply object synchronously
    // right after emitting complete.
    QVERIFY(caller.reply.isNull());
}

void RateLimitManagerTest::queueDrainsFifoWithoutMinimumIntervalSpacing()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());
    installPolicy(manager, "10:60:60", "0:60:0");

    QSignalSpy queue_spy(&manager, &RateLimitManager::QueueUpdated);

    Caller c1;
    Caller c2;
    manager.QueueRequest(kEndpoint, request("one"), c1.reply);
    manager.QueueRequest(kEndpoint, request("two"), c2.reply);

    // First queue activates immediately (0 left waiting); the second joins
    // the queue behind the active request (1 waiting).
    QCOMPARE(queue_spy.count(), 2);
    QCOMPARE(queue_spy.at(0).at(1).toInt(), 0);
    QCOMPARE(queue_spy.at(1).at(1).toInt(), 1);

    QTRY_COMPARE(sender.count(), 1);
    QCOMPARE(sender.sent(0).request.url(), request("one").url());
    sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    QCOMPARE(c1.completions, 1);

    // F58 pin: MINIMUM_INTERVAL_MSEC (1s) spacing is dead code — last_send
    // is never assigned — so the next send is scheduled only
    // NORMAL_BUFFER_MSEC out, not a full second.
    const int wait = manager.msecToNextSend();
    QVERIFY2(wait >= 0 && wait <= kNormalBufferMsec + 50, qPrintable(QString::number(wait)));

    QTRY_COMPARE(sender.count(), 2);
    QCOMPARE(sender.sent(1).request.url(), request("two").url());
    sender.sent(1).reply->finish(policyHeaders("10:60:60", "2:60:0"), 200);
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

    FakeSender sender;
    RateLimitManager manager(sender.fcn());
    installPolicy(manager, limit, ok_state);

    Caller c1;
    manager.QueueRequest(kEndpoint, request("one"), c1.reply);
    QTRY_COMPARE(sender.count(), 1);

    const QDateTime before_finish = QDateTime::currentDateTime();
    sender.sent(0).reply->finish(policyHeaders(limit, full_state), 200);
    QCOMPARE(c1.completions, 1);
    QCOMPARE(manager.policy().status(), RateLimit::Status::BORDERLINE);

    // The saturated policy schedules the next request a full
    // period-plus-bucket out, announced via Paused, and sends nothing in
    // the meantime.
    QSignalSpy paused_spy(&manager, &RateLimitManager::Paused);
    Caller c2;
    manager.QueueRequest(kEndpoint, request("two"), c2.reply);

    // msecToNextSend reads QTimer::remainingTime, and coarse timers adjust
    // their deadline by up to 5% — so the bound is ±6%, not exact.
    QCOMPARE(sender.count(), 1);
    const int wait = manager.msecToNextSend();
    QVERIFY2(wait >= (expected_wait_msec * 94) / 100, qPrintable(QString::number(wait)));
    QVERIFY2(wait <= (expected_wait_msec * 106) / 100, qPrintable(QString::number(wait)));

    QCOMPARE(paused_spy.count(), 1);
    const qint64 pause = before_finish.msecsTo(paused_spy.at(0).at(1).toDateTime());
    QVERIFY2(pause >= expected_wait_msec - 100, qPrintable(QString::number(pause)));
    QVERIFY2(pause <= expected_wait_msec + 2000, qPrintable(QString::number(pause)));

    QTest::qWait(250);
    QCOMPARE(sender.count(), 1);
    QCOMPARE(c2.completions, 0);
}

void RateLimitManagerTest::nonRetryableHttpErrorSurfacesReplyAndAdvances()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());
    installPolicy(manager, "10:60:60", "0:60:0");

    QSignalSpy violation_spy(&manager, &RateLimitManager::Violation);

    Caller c1;
    Caller c2;
    manager.QueueRequest(kEndpoint, request("one"), c1.reply);
    manager.QueueRequest(kEndpoint, request("two"), c2.reply);
    QTRY_COMPARE(sender.count(), 1);

    // A non-429 HTTP error is not retried: the errored reply is surfaced to
    // the caller as a completion and the queue advances.
    sender.sent(0).reply->finish(policyHeaders("10:60:60", "1:60:0"),
                                 500,
                                 QNetworkReply::InternalServerError);
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status, 500);
    QCOMPARE(c1.last_error, QNetworkReply::InternalServerError);
    QVERIFY(c1.reply.isNull());
    QCOMPARE(violation_spy.count(), 0);

    QTRY_COMPARE(sender.count(), 2);
    sender.sent(1).reply->finish(policyHeaders("10:60:60", "2:60:0"), 200);
    QCOMPARE(c2.completions, 1);
}

void RateLimitManagerTest::headerlessNetworkFailureSurfacesReplyAndAdvances()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());
    installPolicy(manager, "10:60:60", "0:60:0");

    Caller c1;
    Caller c2;
    manager.QueueRequest(kEndpoint, request("one"), c1.reply);
    manager.QueueRequest(kEndpoint, request("two"), c2.reply);
    QTRY_COMPARE(sender.count(), 1);

    // A network-level failure produces a reply without rate-limit headers.
    // It is surfaced to the caller as a failed completion and the queue
    // advances; the policy is left alone (no Update, no history event).
    sender.sent(0).reply->finish({}, 0, QNetworkReply::ConnectionRefusedError);
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status, 0);
    QCOMPARE(c1.last_error, QNetworkReply::ConnectionRefusedError);
    QVERIFY(c1.reply.isNull());
    QCOMPARE(manager.policy().status(), RateLimit::Status::OK);

    QTRY_COMPARE(sender.count(), 2);
    sender.sent(1).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    QCOMPARE(c2.completions, 1);
}

void RateLimitManagerTest::successWithViolationStateEmitsViolation()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());
    installPolicy(manager, "2:60:60", "0:60:0");

    QSignalSpy violation_spy(&manager, &RateLimitManager::Violation);

    Caller c1;
    manager.QueueRequest(kEndpoint, request("one"), c1.reply);
    QTRY_COMPARE(sender.count(), 1);

    // A 200 whose headers report state above the limit is still completed
    // normally, but the violation is detected and signalled.
    sender.sent(0).reply->finish(policyHeaders("2:60:60", "3:60:60"), 200);
    QCOMPARE(violation_spy.count(), 1);
    QCOMPARE(violation_spy.at(0).at(0).toString(), QString(kPolicyName));
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status, 200);
}

void RateLimitManagerTest::violation429WithoutRetryAfterSurfacesReplyAndPauses()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());
    installPolicy(manager, "2:60:60", "0:60:0");

    QSignalSpy violation_spy(&manager, &RateLimitManager::Violation);

    Caller c1;
    Caller c2;
    manager.QueueRequest(kEndpoint, request("one"), c1.reply);
    manager.QueueRequest(kEndpoint, request("two"), c2.reply);
    QTRY_COMPARE(sender.count(), 1);

    // A 429 with no Retry-After is NOT retried: it falls through to the
    // non-retryable branch and the 429 reply is surfaced to the caller.
    // (The pump keeps this shape: missing Retry-After is terminal.)
    sender.sent(0).reply->finish(policyHeaders("2:60:60", "3:60:60"),
                                 429,
                                 QNetworkReply::UnknownContentError);
    QCOMPARE(violation_spy.count(), 1);
    QCOMPARE(c1.completions, 1);
    QCOMPARE(c1.last_status, 429);
    QVERIFY(c1.reply.isNull());

    // The violated policy then paces the next request a full
    // period-plus-bucket out (60 + 5 + 1 seconds) instead of sending it.
    // Bounds are ±6% because coarse timers adjust their deadline up to 5%.
    QCOMPARE(sender.count(), 1);
    const int wait = manager.msecToNextSend();
    QVERIFY2(wait >= 62000, qPrintable(QString::number(wait)));
    QVERIFY2(wait <= 70000, qPrintable(QString::number(wait)));
    QTest::qWait(200);
    QCOMPARE(sender.count(), 1);
    QCOMPARE(c2.completions, 0);
}

void RateLimitManagerTest::f57RetryDestroysCallerReplyAndDropsCompletion()
{
    FakeSender sender;
    RateLimitManager manager(sender.fcn());
    installPolicy(manager, "2:60:60", "0:60:0");

    QSignalSpy violation_spy(&manager, &RateLimitManager::Violation);

    Caller victim;
    manager.QueueRequest(kEndpoint, request("one"), victim.reply);
    QTRY_COMPARE(sender.count(), 1);

    auto retry_headers = policyHeaders("2:60:60", "3:60:60");
    retry_headers.append(QNetworkReply::RawHeaderPair{"Retry-After", "1"});

    QElapsedTimer since_429;
    since_429.start();
    sender.sent(0).reply->finish(retry_headers, 429, QNetworkReply::UnknownContentError);

    // F57, pinned as it currently fails: the Retry-After branch nulls
    // m_active_request->reply, destroying the caller's RateLimitedReply on
    // the spot. Nothing has been completed.
    QVERIFY(victim.reply.isNull());
    QCOMPARE(victim.completions, 0);
    QCOMPARE(violation_spy.count(), 1);

    // Retry-After is honored: the resend is scheduled ~1000ms out (with a
    // ±6% allowance for coarse-timer deadline adjustment).
    const int wait = manager.msecToNextSend();
    QVERIFY2(wait > 850 && wait <= 1060, qPrintable(QString::number(wait)));

    QTRY_COMPARE(sender.count(), 2);
    QVERIFY2(since_429.elapsed() >= 900, qPrintable(QString::number(since_429.elapsed())));
    // The same request is resent.
    QCOMPARE(sender.sent(1).request.url(), request("one").url());

    // The retried request succeeds — but the caller's handle is gone, so
    // the completion is dropped on the floor. This is the wedge: the
    // worker's counters never reconcile and the update never finishes.
    // (Failing-behavior pin; the phase-3 pump must flip it so the caller
    // sees exactly one final completion.)
    sender.sent(1).reply->finish(policyHeaders("2:60:60", "1:60:0"), 200);
    QCOMPARE(victim.completions, 0);

    // The manager itself is not wedged — only the caller is. A later
    // request still flows normally.
    Caller after;
    manager.QueueRequest(kEndpoint, request("after"), after.reply);
    QTRY_COMPARE(sender.count(), 3);
    sender.sent(2).reply->finish(policyHeaders("2:60:60", "2:60:0"), 200);
    QCOMPARE(after.completions, 1);
}

QTEST_GUILESS_MAIN(RateLimitManagerTest)

#include "tst_ratelimitmanager.moc"
