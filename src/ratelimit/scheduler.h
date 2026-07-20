// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QElapsedTimer>

#include <chrono>
#include <functional>

class QObject;

namespace RateLimit {

    // Injected monotonic clock and one-shot callback scheduler for the
    // coroutine primitives (network-redesign spec, "Testing plan" item 4).
    // The stop-interruptible sleep and the gate take a Scheduler so their
    // timing is deterministic under test: the fake advances time manually
    // and the tests never sleep. The phase-3 pump reuses the same interface
    // for its pacing and retry deadlines.
    //
    // Instants are milliseconds on a monotonic clock with an arbitrary
    // per-scheduler origin; they are not wall-clock times.
    class Scheduler
    {
    public:
        virtual ~Scheduler();

        // The current instant. Never decreases.
        virtual std::chrono::milliseconds Now() const = 0;

        // Invoke callback once Now() >= when; a deadline at or before Now()
        // fires as soon as possible. Delivery is never synchronous within
        // CallAt. There is no cancellation: if context is non-null and has
        // been destroyed, the callback is dropped; a callback scheduled with
        // a null context must guard its own lifetime.
        virtual void CallAt(std::chrono::milliseconds when,
                            QObject *context,
                            std::function<void()> callback) = 0;
    };

    // The production scheduler: QElapsedTimer for Now() and precise
    // single-shot timers for CallAt. Precise on purpose — coarse timers may
    // fire up to 5% early, which would break exact pacing deadlines (the
    // ±6%-bounds gotcha in tst_ratelimitmanager.cpp).
    class TimerScheduler : public Scheduler
    {
    public:
        TimerScheduler();

        std::chrono::milliseconds Now() const override;
        void CallAt(std::chrono::milliseconds when,
                    QObject *context,
                    std::function<void()> callback) override;

    private:
        QElapsedTimer m_clock;
    };

} // namespace RateLimit
