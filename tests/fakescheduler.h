// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QPointer>
#include <QtGlobal>

#include <chrono>
#include <cstddef>
#include <functional>
#include <utility>
#include <vector>

#include "ratelimit/scheduler.h"

// Deterministic scheduler for the primitive tests (network-redesign spec,
// "Testing plan" item 4): time only moves when the test advances it, so the
// tests never sleep and no coarse-timer slack ever applies. Callbacks are
// delivered only from inside AdvanceTo/AdvanceBy — never from CallAt — in
// (deadline, scheduling order); a callback scheduled by a firing callback
// fires in the same advance if it is due. Note that delivering a callback is
// not the same as resuming a coroutine: resumptions still ride the real
// event loop (QFutureWatcher), which tests drain separately.
class FakeScheduler : public RateLimit::Scheduler
{
public:
    std::chrono::milliseconds Now() const override { return m_now; }

    void CallAt(std::chrono::milliseconds when,
                QObject *context,
                std::function<void()> callback) override
    {
        m_pending.push_back(Pending{when,
                                    m_next_seq++,
                                    context != nullptr,
                                    QPointer<QObject>(context),
                                    std::move(callback)});
    }

    void AdvanceBy(std::chrono::milliseconds delta) { AdvanceTo(m_now + delta); }

    void AdvanceTo(std::chrono::milliseconds t)
    {
        // Now() is a monotonic clock (Scheduler contract): time never moves
        // backward, and a negative AdvanceBy() is a test bug. Unconditional
        // on purpose — Q_ASSERT compiles out under QT_NO_DEBUG, and CI
        // builds Release, which is precisely where an unnoticed backward
        // clock would silently weaken these timing tests.
        if (t < m_now) {
            qFatal("FakeScheduler: time moved backward, from %lldms to %lldms",
                   static_cast<long long>(m_now.count()),
                   static_cast<long long>(t.count()));
        }
        while (true) {
            // The earliest (when, seq) pending callback due at or before t.
            // Re-scanned every iteration because a fired callback may have
            // scheduled more work.
            size_t best = m_pending.size();
            for (size_t i = 0; i < m_pending.size(); ++i) {
                if (m_pending[i].when > t) {
                    continue;
                }
                if (best == m_pending.size()
                    || std::pair(m_pending[i].when, m_pending[i].seq)
                           < std::pair(m_pending[best].when, m_pending[best].seq)) {
                    best = i;
                }
            }
            if (best == m_pending.size()) {
                break;
            }
            Pending fired = std::move(m_pending[best]);
            m_pending.erase(m_pending.begin() + static_cast<ptrdiff_t>(best));
            // The callback observes its own fire time (never earlier than a
            // past-due deadline's scheduling time already passed).
            m_now = std::max(m_now, fired.when);
            if (!fired.has_context || fired.context) {
                fired.callback();
            }
        }
        m_now = t;
    }

    int PendingCount() const { return static_cast<int>(m_pending.size()); }

private:
    struct Pending
    {
        std::chrono::milliseconds when;
        quint64 seq;
        bool has_context;
        QPointer<QObject> context;
        std::function<void()> callback;
    };

    std::chrono::milliseconds m_now{0};
    quint64 m_next_seq{0};
    std::vector<Pending> m_pending;
};
