// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest>

#include <QDateTime>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QPointer>

#include <chrono>

#include "fakenetworkmanager.h"
#include "fakescheduler.h"
#include "ratelimit/gate.h"
#include "ratelimit/ratelimit.h"
#include "ratelimit/ratelimitedreply.h"
#include "ratelimit/ratelimiter.h"

// Hub-level setup pins (network-redesign spec, D4 and "Testing plan"
// item 3): Unknown -> Probing -> Established, the async HEAD behind the
// gate's exclusive permit, parked-entry forwarding, and every setup-failure
// flavor failing parked entries cleanly under the cooldown — where the old
// hub blocked in a nested event loop and killed the app with FatalError.
// The injected FakeScheduler drives all timing; nothing here sleeps.
//
// Phase-3 boundary note: failure "kinds" are expressed as errored synthetic
// replies (error code, HTTP status, message) — phase 4 maps these onto
// FetchError values. Cancellation pins wait for stop tokens (phase 4/5).

// moc-lexer note (see tst_workerupdate.cpp): declare the Q_OBJECT class
// before any helpers containing string literals with '//' in them.
class RateLimiterTest : public QObject
{
    Q_OBJECT

private slots:
    void headEstablishesEndpointAndForwardsParkedFifo();
    void headTransportFailureFailsParkedAndCoolsDown();
    void headHttpErrorFailsParked();
    void headUnparseable2xxFailsParked();
    void cooldownExpiryAllowsReprobe();
    void headFull429WithValidRetryAfterEstablishesWithHold();
    void nonFull429CooldownHonorsRetryAfter();
    void establishedEndpointSticksThroughRequestFailures();
    void concurrentSetupsSerializeHeadsAtGate();
};

namespace {

    using namespace std::chrono_literals;

    constexpr auto kGateSpacing = RateLimit::Gate::MIN_SEND_SPACING;
    constexpr int kNormalBufferMsec = 100;

    QByteArray rfcDateNow()
    {
        return QDateTime::currentDateTimeUtc().toString(Qt::RFC2822Date).toUtf8();
    }

    QList<QNetworkReply::RawHeaderPair> policyHeaders(const QByteArray &limit,
                                                      const QByteArray &state,
                                                      const QByteArray &policy_name
                                                      = "test-request-limit")
    {
        return {
            {"X-Rate-Limit-Policy", policy_name},
            {"X-Rate-Limit-Rules", "Ip"},
            {"X-Rate-Limit-Ip", limit},
            {"X-Rate-Limit-Ip-State", state},
            {"Date", rfcDateNow()},
        };
    }

    QNetworkRequest request(const QString &leaf)
    {
        return QNetworkRequest(QUrl("https://api.example.test/" + leaf));
    }

    // Caller-side observations for one Submit(): completions, the surfaced
    // reply's status/error, and whether the hub has destroyed the
    // RateLimitedReply (it does, synchronously after completing — F59).
    struct Submission
    {
        QPointer<RateLimitedReply> reply;
        int completions = 0;
        int last_status = 0;
        QNetworkReply::NetworkError last_error = QNetworkReply::NoError;

        void attach(RateLimitedReply *r)
        {
            reply = r;
            QObject::connect(r, &RateLimitedReply::complete, [this](QNetworkReply *network_reply) {
                ++completions;
                last_status = RateLimit::ParseStatus(network_reply);
                last_error = network_reply->error();
                network_reply->deleteLater();
            });
        }
    };

    void drainEvents()
    {
        for (int i = 0; i < 20; ++i) {
            QCoreApplication::processEvents();
            QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        }
    }

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

    // The network outlives the limiter (declared first).
    struct Rig
    {
        FakeScheduler scheduler;
        FakeNetworkManager network;
        RateLimiter limiter{network, &scheduler};
    };

} // namespace

void RateLimiterTest::headEstablishesEndpointAndForwardsParkedFifo()
{
    Rig rig;

    // Two submissions before the endpoint is known: the first fires the
    // probe, the second parks behind it. Nothing blocks and no completion
    // is synchronous — though the HEAD itself may go out during Submit
    // when the gate's fast path grants without suspending (S1-6).
    Submission s1;
    Submission s2;
    s1.attach(rig.limiter.Submit("ep", request("one")));
    s2.attach(rig.limiter.Submit("ep", request("two")));
    settle(rig.scheduler);
    QCOMPARE(rig.network.count(), 1);
    QCOMPARE(rig.network.sent(0).op, QNetworkAccessManager::HeadOperation);
    QCOMPARE(s1.completions, 0);
    QCOMPARE(s2.completions, 0);

    // The probe lands Full: the endpoint establishes and the parked
    // entries forward in submission order into the pump.
    rig.network.sent(0).reply->finish(policyHeaders("10:60:60", "0:60:0"), 200);
    drainEvents();
    QCOMPARE(rig.network.count(), 1);

    // First send: pacing ends at t=100, but the HEAD's dispatch at t=0
    // charged the gate's spacing floor, so it goes at t=250.
    advanceAndSettle(rig.scheduler, std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.network.count(), 1);
    advanceAndSettle(rig.scheduler, kGateSpacing - std::chrono::milliseconds(kNormalBufferMsec));
    QCOMPARE(rig.network.count(), 2);
    QCOMPARE(rig.network.sent(1).op, QNetworkAccessManager::GetOperation);
    QCOMPARE(rig.network.sent(1).request.url(), request("one").url());

    rig.network.sent(1).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(s1.completions, 1);
    QCOMPARE(s1.last_status, 200);

    // The second parked entry follows behind the spacing floor.
    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.network.count(), 3);
    QCOMPARE(rig.network.sent(2).request.url(), request("two").url());
    rig.network.sent(2).reply->finish(policyHeaders("10:60:60", "2:60:0"), 200);
    drainEvents();
    QCOMPARE(s2.completions, 1);
}

void RateLimiterTest::headTransportFailureFailsParkedAndCoolsDown()
{
    Rig rig;

    Submission s1;
    Submission s2;
    s1.attach(rig.limiter.Submit("ep", request("one")));
    s2.attach(rig.limiter.Submit("ep", request("two")));
    settle(rig.scheduler);
    QCOMPARE(rig.network.count(), 1);

    // The probe dies in transport: every parked entry completes with an
    // errored reply carrying the failure, and the caller handles are
    // destroyed after completion (the pump's contract, mirrored).
    rig.network.sent(0).reply->finish({}, 0, QNetworkReply::ConnectionRefusedError);
    drainEvents();
    QCOMPARE(s1.completions, 1);
    QCOMPARE(s1.last_error, QNetworkReply::ConnectionRefusedError);
    QVERIFY(s1.reply.isNull());
    QCOMPARE(s2.completions, 1);
    QCOMPARE(s2.last_error, QNetworkReply::ConnectionRefusedError);

    // Inside the cooldown a resubmission fails fast with the prior
    // failure's shape — no probe is sent — and the completion arrives
    // through the event loop, never synchronously inside Submit.
    Submission s3;
    s3.attach(rig.limiter.Submit("ep", request("three")));
    QCOMPARE(s3.completions, 0);
    drainEvents();
    QCOMPARE(s3.completions, 1);
    QCOMPARE(s3.last_error, QNetworkReply::ConnectionRefusedError);
    QCOMPARE(rig.network.count(), 1);
}

void RateLimiterTest::headHttpErrorFailsParked()
{
    Rig rig;

    Submission s1;
    s1.attach(rig.limiter.Submit("ep", request("one")));
    settle(rig.scheduler);
    QCOMPARE(rig.network.count(), 1);

    rig.network.sent(0).reply->finish({}, 500, QNetworkReply::InternalServerError);
    drainEvents();
    QCOMPARE(s1.completions, 1);
    QCOMPARE(s1.last_status, 500);
    QVERIFY(s1.last_error != QNetworkReply::NoError);
}

void RateLimiterTest::headUnparseable2xxFailsParked()
{
    Rig rig;

    Submission s1;
    s1.attach(rig.limiter.Submit("ep", request("one")));
    settle(rig.scheduler);
    QCOMPARE(rig.network.count(), 1);

    // A clean 200 with no usable rate-limit headers: the endpoint must
    // never run unpaced, so this is a setup failure (Protocol flavor), not
    // a degrade-and-proceed.
    rig.network.sent(0).reply->finish({{"Content-Type", "text/html"}}, 200);
    drainEvents();
    QCOMPARE(s1.completions, 1);
    QCOMPARE(s1.last_error, QNetworkReply::ProtocolFailure);

    // And no pump exists for the endpoint: a resubmission inside the
    // cooldown fails fast without any network traffic.
    Submission s2;
    s2.attach(rig.limiter.Submit("ep", request("two")));
    drainEvents();
    QCOMPARE(s2.completions, 1);
    QCOMPARE(rig.network.count(), 1);
}

void RateLimiterTest::cooldownExpiryAllowsReprobe()
{
    Rig rig;

    Submission s1;
    s1.attach(rig.limiter.Submit("ep", request("one")));
    settle(rig.scheduler);
    rig.network.sent(0).reply->finish({}, 0, QNetworkReply::ConnectionRefusedError);
    drainEvents();
    QCOMPARE(s1.completions, 1);
    QCOMPARE(rig.network.headCount(), 1);

    // Once SETUP_RETRY_COOLDOWN passes, a submission probes again.
    advanceAndSettle(rig.scheduler, 60s);
    Submission s2;
    s2.attach(rig.limiter.Submit("ep", request("two")));
    settle(rig.scheduler);
    QCOMPARE(rig.network.headCount(), 2);

    // And this probe can succeed: the failure was contained, not sticky.
    rig.network.sent(1).reply->finish(policyHeaders("10:60:60", "0:60:0"), 200);
    drainEvents();
    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.network.count(), 3);
    rig.network.sent(2).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(s2.completions, 1);
    QCOMPARE(s2.last_status, 200);
}

void RateLimiterTest::headFull429WithValidRetryAfterEstablishesWithHold()
{
    Rig rig;

    Submission s1;
    s1.attach(rig.limiter.Submit("ep", request("one")));
    settle(rig.scheduler);
    QCOMPARE(rig.network.count(), 1);

    // A Full HEAD 429 with a valid Retry-After still teaches the topology
    // (N16/N24): the pump establishes, and its first send is held past
    // Retry-After + pad + buffer = 120+60+1 = 181s. The OK state isolates
    // the hold from policy pacing. The HEAD consumed no send attempt —
    // the request completes on its first real send.
    auto headers = policyHeaders("10:60:60", "0:60:0");
    headers.append(QNetworkReply::RawHeaderPair{"Retry-After", "120"});
    rig.network.sent(0).reply->finish(headers, 429, QNetworkReply::UnknownContentError);
    drainEvents();
    QCOMPARE(s1.completions, 0); // established, not failed

    advanceAndSettle(rig.scheduler, 100s);
    QCOMPARE(rig.network.count(), 1); // still holding
    advanceAndSettle(rig.scheduler, 85s);
    QCOMPARE(rig.network.count(), 2);
    QCOMPARE(rig.network.sent(1).op, QNetworkAccessManager::GetOperation);
    rig.network.sent(1).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(s1.completions, 1);
    QCOMPARE(s1.last_status, 200);
}

void RateLimiterTest::nonFull429CooldownHonorsRetryAfter()
{
    Rig rig;

    Submission s1;
    s1.attach(rig.limiter.Submit("ep", request("one")));
    settle(rig.scheduler);

    // A 429 without Full headers is a setup failure whose cooldown honors
    // the validly-present Retry-After: 120s instead of the 60s floor.
    rig.network.sent(0).reply->finish({{"Retry-After", "120"}},
                                      429,
                                      QNetworkReply::UnknownContentError);
    drainEvents();
    QCOMPARE(s1.completions, 1);
    QCOMPARE(s1.last_status, 429);

    // 70s in: still cooling down, no probe.
    advanceAndSettle(rig.scheduler, 70s);
    Submission s2;
    s2.attach(rig.limiter.Submit("ep", request("two")));
    drainEvents();
    QCOMPARE(s2.completions, 1);
    QCOMPARE(rig.network.headCount(), 1);

    // 125s in: the window has passed; a new probe goes out.
    advanceAndSettle(rig.scheduler, 55s);
    Submission s3;
    s3.attach(rig.limiter.Submit("ep", request("three")));
    settle(rig.scheduler);
    QCOMPARE(rig.network.headCount(), 2);
}

void RateLimiterTest::establishedEndpointSticksThroughRequestFailures()
{
    Rig rig;

    Submission s1;
    s1.attach(rig.limiter.Submit("ep", request("one")));
    settle(rig.scheduler);
    rig.network.sent(0).reply->finish(policyHeaders("10:60:60", "0:60:0"), 200);
    drainEvents();
    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.network.count(), 2);

    // The request itself dies (a timeout-shaped transport failure). The
    // endpoint stays Established: valid topology is never discarded and no
    // HEAD is re-burned (N16).
    rig.network.sent(1).reply->finish({}, 0, QNetworkReply::TimeoutError);
    drainEvents();
    QCOMPARE(s1.completions, 1);
    QCOMPARE(s1.last_error, QNetworkReply::TimeoutError);

    Submission s2;
    s2.attach(rig.limiter.Submit("ep", request("two")));
    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.network.headCount(), 1); // no re-probe
    QCOMPARE(rig.network.count(), 3);
    QCOMPARE(rig.network.sent(2).op, QNetworkAccessManager::GetOperation);
    rig.network.sent(2).reply->finish(policyHeaders("10:60:60", "1:60:0"), 200);
    drainEvents();
    QCOMPARE(s2.completions, 1);
}

void RateLimiterTest::concurrentSetupsSerializeHeadsAtGate()
{
    Rig rig;

    // Two unknown endpoints at once: their probes serialize at the gate's
    // exclusive HEAD permit — one HEAD in flight, ever (the F5 standing
    // constraint, now a property of the gate).
    Submission s1;
    Submission s2;
    s1.attach(rig.limiter.Submit("ep-one", request("one")));
    s2.attach(rig.limiter.Submit("ep-two", request("two")));
    settle(rig.scheduler);
    QCOMPARE(rig.network.count(), 1);
    QCOMPARE(rig.network.sent(0).op, QNetworkAccessManager::HeadOperation);

    // While the second HEAD waits, writer preference also holds back the
    // first pump's sends — no ordinary permit is issued past a waiting
    // HEAD.
    rig.network.sent(0).reply->finish(policyHeaders("10:60:60", "0:60:0", "policy-one"), 200);
    drainEvents();
    settle(rig.scheduler);
    QCOMPARE(rig.network.count(), 1);

    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.network.count(), 2);
    QCOMPARE(rig.network.sent(1).op, QNetworkAccessManager::HeadOperation);

    rig.network.sent(1).reply->finish(policyHeaders("10:60:60", "0:60:0", "policy-two"), 200);
    drainEvents();

    // With both endpoints established, the parked GETs drain in arrival
    // order behind the spacing floor.
    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.network.count(), 3);
    QCOMPARE(rig.network.sent(2).request.url(), request("one").url());
    advanceAndSettle(rig.scheduler, kGateSpacing);
    QCOMPARE(rig.network.count(), 4);
    QCOMPARE(rig.network.sent(3).request.url(), request("two").url());

    rig.network.sent(2).reply->finish(policyHeaders("10:60:60", "1:60:0", "policy-one"), 200);
    rig.network.sent(3).reply->finish(policyHeaders("10:60:60", "1:60:0", "policy-two"), 200);
    drainEvents();
    QCOMPARE(s1.completions, 1);
    QCOMPARE(s2.completions, 1);
}

QTEST_GUILESS_MAIN(RateLimiterTest)

#include "tst_ratelimiter.moc"
