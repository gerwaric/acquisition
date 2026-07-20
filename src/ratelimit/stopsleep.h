// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QCoroTask>

#include <chrono>
#include <stop_token>

namespace RateLimit {

    class Scheduler;

    enum class SleepResult { Deadline, Stopped };

    // The stop-interruptible sleep (network-redesign spec, D2/D3): QCoro's
    // sleeps take no stop token, so the pump's pacing and retry sleeps use
    // this primitive instead.
    //
    // Contract (D2, R4-2, R5-2; spike-verified S1-5):
    //  - One wait takes exactly one token — the entry's (R5-2: there is no
    //    second, hub-level token anywhere).
    //  - Completes SleepResult::Deadline when the scheduler reaches the
    //    deadline, or SleepResult::Stopped when the token is stopped —
    //    whichever settles first wins; the loser is a no-op.
    //  - Wake-on-stop is a queued resume: request_stop() returns before the
    //    waiter resumes. std::stop_callback runs synchronously inside
    //    request_stop(), so the callback only completes a QPromise; the
    //    QCoro QFuture awaiter delivers the resumption through the event
    //    loop (QFutureWatcher), never on the requesting stack.
    //  - A wait on an already-stopped token returns Stopped without
    //    suspending — the checkpoint shape from D2.
    QCoro::Task<SleepResult> SleepUntil(Scheduler &scheduler,
                                        std::chrono::milliseconds deadline,
                                        std::stop_token token);

} // namespace RateLimit
