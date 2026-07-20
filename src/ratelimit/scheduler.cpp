// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "ratelimit/scheduler.h"

#include <QTimer>

#include <algorithm>

namespace RateLimit {

    Scheduler::~Scheduler() = default;

    TimerScheduler::TimerScheduler()
    {
        m_clock.start();
    }

    std::chrono::milliseconds TimerScheduler::Now() const
    {
        return std::chrono::milliseconds(m_clock.elapsed());
    }

    void TimerScheduler::CallAt(std::chrono::milliseconds when,
                                QObject *context,
                                std::function<void()> callback)
    {
        using namespace std::chrono_literals;
        const auto delay = std::max(when - Now(), 0ms);
        if (context) {
            QTimer::singleShot(delay, Qt::PreciseTimer, context, std::move(callback));
        } else {
            QTimer::singleShot(delay, Qt::PreciseTimer, std::move(callback));
        }
    }

} // namespace RateLimit
