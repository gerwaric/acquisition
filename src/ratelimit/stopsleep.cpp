// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "ratelimit/stopsleep.h"

#include <QCoroFuture>

#include <QFuture>
#include <QPromise>

#include <atomic>
#include <memory>

#include "ratelimit/scheduler.h"

namespace {

    // Shared between the deadline callback and the stop callback: whichever
    // settles first wins, the other is a no-op. The shared_ptr keeps the
    // state alive for a deadline callback that fires after a stop already
    // won (the scheduler has no cancellation — see Scheduler::CallAt).
    struct SleepState
    {
        QPromise<RateLimit::SleepResult> promise;
        std::atomic_flag settled = ATOMIC_FLAG_INIT;

        void Settle(RateLimit::SleepResult result)
        {
            if (!settled.test_and_set()) {
                promise.addResult(result);
                promise.finish();
            }
        }
    };

} // namespace

namespace RateLimit {

    QCoro::Task<SleepResult> SleepUntil(Scheduler &scheduler,
                                        std::chrono::milliseconds deadline,
                                        std::stop_token token)
    {
        // D2 checkpoint shape: a pre-stopped wait never suspends.
        if (token.stop_requested()) {
            co_return SleepResult::Stopped;
        }

        auto state = std::make_shared<SleepState>();
        state->promise.start();
        QFuture<SleepResult> future = state->promise.future();

        // Null context: the shared state guards the callback's lifetime.
        scheduler.CallAt(deadline, nullptr, [state] { state->Settle(SleepResult::Deadline); });

        // Settling the promise here is safe on the request_stop() stack: the
        // only consumer is the QCoro QFuture awaiter below, which resumes
        // through the event loop (QFutureWatcher, S1-5) — the queued-resume
        // contract (R4-2) rides on that, so request_stop() returns before
        // the waiter resumes.
        std::stop_callback wake(token, [state] { state->Settle(SleepResult::Stopped); });

        co_return co_await future;
    }

} // namespace RateLimit
