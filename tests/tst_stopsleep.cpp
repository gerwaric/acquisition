// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest>

#include <QCoroTask>

#include <optional>
#include <stop_token>

#include "fakescheduler.h"
#include "ratelimit/scheduler.h"
#include "ratelimit/stopsleep.h"

// Standalone tests for the stop-interruptible sleep (network-redesign spec,
// "Testing plan" item 4): one token per wait; wakes on stop; completes on
// deadline; resumes via the event loop — request_stop() returns before the
// waiter resumes (R4-2). The injected FakeScheduler makes every timing
// assertion exact: the tests never sleep and no coarse-timer slack applies.

// moc-lexer note (see tst_workerupdate.cpp): declare the Q_OBJECT class
// before any helpers containing string literals with '//' in them.
class StopSleepTest : public QObject
{
    Q_OBJECT

private slots:
    void deadlineCompletesExactlyAtDeadline();
    void stopWakesPromptlyWithQueuedResume();
    void preStoppedTokenNeverSuspends();
    void pastDeadlineCompletesThroughTheEventLoop();
    void stopAfterDeadlineChangesNothing();
    void twoWaitsOnOneTokenBothWake();
    void timerSchedulerSmoke();
};

namespace {

    using namespace std::chrono_literals;

    struct Probe
    {
        bool done = false;
        std::optional<RateLimit::SleepResult> result;
    };

    QCoro::Task<> Run(Probe &probe,
                      RateLimit::Scheduler &scheduler,
                      std::chrono::milliseconds deadline,
                      std::stop_token token)
    {
        probe.result = co_await RateLimit::SleepUntil(scheduler, deadline, token);
        probe.done = true;
    }

    // Deliver queued coroutine resumptions (the QFutureWatcher hop). A fixed
    // pass count keeps this deterministic — no wall-clock dependence; each
    // pass delivers the events the previous one posted.
    void drainEvents()
    {
        for (int i = 0; i < 20; ++i) {
            QCoreApplication::processEvents();
        }
    }

} // namespace

void StopSleepTest::deadlineCompletesExactlyAtDeadline()
{
    FakeScheduler scheduler;
    std::stop_source stop;
    Probe probe;
    auto task = Run(probe, scheduler, 5000ms, stop.get_token());

    drainEvents();
    QVERIFY(!probe.done);

    // One millisecond short of the deadline: still asleep. The injected
    // clock is exact — no ±6% bound needed.
    scheduler.AdvanceBy(4999ms);
    drainEvents();
    QVERIFY(!probe.done);

    scheduler.AdvanceBy(1ms);
    drainEvents();
    QVERIFY(probe.done);
    QCOMPARE(*probe.result, RateLimit::SleepResult::Deadline);
}

void StopSleepTest::stopWakesPromptlyWithQueuedResume()
{
    FakeScheduler scheduler;
    std::stop_source stop;
    Probe probe;
    auto task = Run(probe, scheduler, 3600000ms, stop.get_token());

    drainEvents();
    QVERIFY(!probe.done);

    // R4-2 pin: request_stop() returns before the waiter resumes — the
    // stop_callback only settles a QPromise, and the resumption is
    // delivered through the event loop.
    stop.request_stop();
    QVERIFY(!probe.done);

    drainEvents();
    QVERIFY(probe.done);
    QCOMPARE(*probe.result, RateLimit::SleepResult::Stopped);

    // The clock never moved: the waiter woke without the hour elapsing and
    // the test never slept.
    QCOMPARE(scheduler.Now(), 0ms);
}

void StopSleepTest::preStoppedTokenNeverSuspends()
{
    FakeScheduler scheduler;
    std::stop_source stop;
    stop.request_stop();

    Probe probe;
    auto task = Run(probe, scheduler, 3600000ms, stop.get_token());

    // D2 checkpoint shape: completed synchronously, no suspension, no event
    // processing, and nothing was ever scheduled.
    QVERIFY(probe.done);
    QCOMPARE(*probe.result, RateLimit::SleepResult::Stopped);
    QCOMPARE(scheduler.PendingCount(), 0);
}

void StopSleepTest::pastDeadlineCompletesThroughTheEventLoop()
{
    FakeScheduler scheduler;
    scheduler.AdvanceBy(100ms);

    std::stop_source stop;
    Probe probe;
    auto task = Run(probe, scheduler, 50ms, stop.get_token());

    // Delivery is never synchronous within CallAt, and the fake delivers
    // only from inside an advance.
    QVERIFY(!probe.done);
    drainEvents();
    QVERIFY(!probe.done);

    scheduler.AdvanceBy(0ms);
    drainEvents();
    QVERIFY(probe.done);
    QCOMPARE(*probe.result, RateLimit::SleepResult::Deadline);
}

void StopSleepTest::stopAfterDeadlineChangesNothing()
{
    FakeScheduler scheduler;
    std::stop_source stop;
    Probe probe;
    auto task = Run(probe, scheduler, 100ms, stop.get_token());

    scheduler.AdvanceBy(100ms);
    drainEvents();
    QVERIFY(probe.done);
    QCOMPARE(*probe.result, RateLimit::SleepResult::Deadline);

    // First settle wins; a later stop is a no-op (the wait's stop_callback
    // was deregistered when the frame completed).
    stop.request_stop();
    drainEvents();
    QCOMPARE(*probe.result, RateLimit::SleepResult::Deadline);
}

void StopSleepTest::twoWaitsOnOneTokenBothWake()
{
    // One token per wait (R5-2) means each wait registers its own callback;
    // one stop_source waking every wait on its token is the D2 abort shape
    // (all of an update's entries share the update token).
    FakeScheduler scheduler;
    std::stop_source stop;
    Probe first;
    Probe second;
    auto task1 = Run(first, scheduler, 3600000ms, stop.get_token());
    auto task2 = Run(second, scheduler, 7200000ms, stop.get_token());

    drainEvents();
    QVERIFY(!first.done);
    QVERIFY(!second.done);

    stop.request_stop();
    QVERIFY(!first.done);
    QVERIFY(!second.done);

    drainEvents();
    QVERIFY(first.done);
    QVERIFY(second.done);
    QCOMPARE(*first.result, RateLimit::SleepResult::Stopped);
    QCOMPARE(*second.result, RateLimit::SleepResult::Stopped);
}

void StopSleepTest::timerSchedulerSmoke()
{
    // The production scheduler is real-timer code, so this is a smoke test
    // only — condition-driven waits, no timing bounds (the ±6% lesson from
    // tst_ratelimitmanager.cpp).
    RateLimit::TimerScheduler scheduler;

    // A destroyed context drops the callback.
    bool dropped_fired = false;
    {
        QObject context;
        scheduler.CallAt(scheduler.Now() + 5ms, &context, [&] { dropped_fired = true; });
    }
    QTest::qWait(30);
    QVERIFY(!dropped_fired);

    // A live context fires.
    bool fired = false;
    QObject context;
    scheduler.CallAt(scheduler.Now() + 5ms, &context, [&] { fired = true; });
    QTRY_VERIFY(fired);

    // A null context fires (self-guarding callbacks).
    bool null_context_fired = false;
    scheduler.CallAt(scheduler.Now(), nullptr, [&] { null_context_fired = true; });
    QTRY_VERIFY(null_context_fired);

    // The sleep composes with the production scheduler.
    std::stop_source stop;
    Probe probe;
    auto task = Run(probe, scheduler, scheduler.Now() + 10ms, stop.get_token());
    QTRY_VERIFY(probe.done);
    QCOMPARE(*probe.result, RateLimit::SleepResult::Deadline);
}

QTEST_GUILESS_MAIN(StopSleepTest)

#include "tst_stopsleep.moc"
