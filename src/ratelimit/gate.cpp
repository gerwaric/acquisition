// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "ratelimit/gate.h"

#include <QCoroFuture>

#include <QFuture>
#include <QPromise>

#include <algorithm>
#include <atomic>

#include "ratelimit/scheduler.h"

namespace RateLimit {

    // Shared between the grant pass and the wait's stop callback: whichever
    // settles first wins. Settling with true is a grant (the gate has
    // already counted the permit); settling with false is a stopped wait.
    struct Gate::WaiterState
    {
        QPromise<bool> promise;
        std::atomic_flag settled = ATOMIC_FLAG_INIT;

        bool TrySettle(bool granted)
        {
            if (settled.test_and_set()) {
                return false;
            }
            promise.addResult(granted);
            promise.finish();
            return true;
        }

        bool IsSettled() const { return settled.test(); }
    };

    Gate::Gate(Scheduler &scheduler, QObject *parent)
        : QObject(parent)
        , m_scheduler(scheduler)
    {}

    Gate::Permit::Permit(Gate *gate, bool head)
        : m_gate(gate)
        , m_head(head)
        , m_held(true)
    {}

    Gate::Permit::Permit(Permit &&other) noexcept
        : m_gate(std::move(other.m_gate))
        , m_head(other.m_head)
        , m_held(std::exchange(other.m_held, false))
    {}

    Gate::Permit &Gate::Permit::operator=(Permit &&other) noexcept
    {
        if (this != &other) {
            Release();
            m_gate = std::move(other.m_gate);
            m_head = other.m_head;
            m_held = std::exchange(other.m_held, false);
        }
        return *this;
    }

    Gate::Permit::~Permit()
    {
        Release();
    }

    void Gate::Permit::Release()
    {
        if (!m_held) {
            return;
        }
        m_held = false;
        if (m_gate) {
            m_gate->ReleasePermit(m_head);
        }
    }

    QCoro::Task<Gate::Permit> Gate::Acquire(std::stop_token token)
    {
        return AcquireImpl(false, std::move(token));
    }

    QCoro::Task<Gate::Permit> Gate::AcquireHead(std::stop_token token)
    {
        return AcquireImpl(true, std::move(token));
    }

    QCoro::Task<Gate::Permit> Gate::AcquireImpl(bool head, std::stop_token token)
    {
        // D2 checkpoint shape: a pre-stopped wait never enqueues.
        if (token.stop_requested()) {
            co_return Permit();
        }

        auto state = std::make_shared<WaiterState>();
        state->promise.start();
        QFuture<bool> future = state->promise.future();
        (head ? m_heads : m_ordinary).push_back(state);

        // Fast path: this may settle the promise synchronously, in which
        // case the await below never suspends (S1-6).
        GrantPass();

        // Wake-on-stop mirrors the sleep primitive (S1-5): settling the
        // promise is safe on the request_stop() stack because the only
        // consumer is the QFuture awaiter below, which resumes through the
        // event loop — request_stop() returns before the waiter resumes
        // (R4-2). The pruning pass is deferred through the scheduler so a
        // request_stop() that stops several waiters at once settles them
        // all before any grant decision is re-made.
        QPointer<Gate> guard(this);
        std::stop_callback wake(token, [state, guard] {
            if (state->TrySettle(false) && guard) {
                guard->ScheduleGrantPass(guard->m_scheduler.Now());
            }
        });

        const bool granted = co_await future;
        co_return granted ? Permit(this, head) : Permit();
    }

    void Gate::GrantPass()
    {
        const auto settled = [](const std::shared_ptr<WaiterState> &waiter) {
            return waiter->IsSettled();
        };
        while (true) {
            // Stopped waiters hold no queue position: they must not block
            // FIFO order or keep the writer preference engaged.
            std::erase_if(m_ordinary, settled);
            std::erase_if(m_heads, settled);

            std::shared_ptr<WaiterState> next;
            bool head = false;
            if (!m_heads.empty()) {
                // Writer preference (IR6): while a HEAD waits, no ordinary
                // permit is issued; the HEAD itself needs the gate empty.
                if (m_active > 0 || m_head_active) {
                    return;
                }
                next = m_heads.front();
                head = true;
            } else if (!m_ordinary.empty()) {
                if (m_head_active || m_active >= IN_FLIGHT_CAP) {
                    return;
                }
                next = m_ordinary.front();
            } else {
                return;
            }

            // Spacing floor: grants are what dispatch sends, so grants are
            // what the floor spaces — HEADs included.
            const auto now = m_scheduler.Now();
            if (m_last_grant) {
                const auto earliest = *m_last_grant + MIN_SEND_SPACING;
                if (now < earliest) {
                    ScheduleGrantPass(earliest);
                    return;
                }
            }

            (head ? m_heads : m_ordinary).pop_front();
            if (!next->TrySettle(true)) {
                continue; // lost to a stop that landed mid-pass
            }
            // Counted at grant time, not at resumption: the cap must bound
            // in-flight permits even while the winner's resume is still
            // queued.
            if (head) {
                m_head_active = true;
            } else {
                ++m_active;
            }
            m_last_grant = now;
        }
    }

    void Gate::ScheduleGrantPass(std::chrono::milliseconds when)
    {
        // A pass already scheduled at or before `when` covers this request.
        if (m_scheduled_pass && *m_scheduled_pass <= when) {
            return;
        }
        m_scheduled_pass = when;
        m_scheduler.CallAt(when, this, [this, when] {
            if (m_scheduled_pass && *m_scheduled_pass == when) {
                m_scheduled_pass.reset();
            }
            GrantPass();
        });
    }

    void Gate::ReleasePermit(bool head)
    {
        if (head) {
            m_head_active = false;
        } else {
            --m_active;
        }
        GrantPass();
    }

} // namespace RateLimit
