// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QCoroTask>

#include <QObject>
#include <QPointer>

#include <chrono>
#include <deque>
#include <memory>
#include <optional>
#include <stop_token>

namespace RateLimit {

    class Scheduler;

    // The gate (network-redesign spec, D5): the single layer-1 object every
    // send the pumps and the setup path dispatch must acquire. Internal to
    // the hub — it has no other callers. Properties, each pinned standalone
    // in tst_gate.cpp:
    //
    //  - In-flight cap: IN_FLIGHT_CAP ordinary permits (P-B's stated global
    //    burst bound).
    //  - HEAD exclusive, with writer preference (IR6): a HEAD permit takes
    //    the whole gate, and once a HEAD is waiting no new ordinary permits
    //    are issued — a busy pump cannot starve endpoint setup.
    //  - FIFO among ordinary waiters (R7): permits grant in arrival order;
    //    the HEAD writer preference is the only queue-jump.
    //  - Minimum inter-send spacing: MIN_SEND_SPACING between sends,
    //    across everything the gate sees (F58's intent, done deliberately).
    //    Grant and dispatch are separate moments — the grant settles a
    //    promise, but the winner dispatches only after its queued resume,
    //    which a busy main thread can delay — so the floor is measured
    //    from a dispatch stamp, not from grant time: the Permit stamps the
    //    gate when it is constructed (at waiter resume, in the same event-
    //    loop iteration as the dispatch that follows), and a granted-but-
    //    unstamped permit defers every further grant until its stamp or
    //    release arrives. One conservative consequence: an entry that
    //    acquires, sees its stop, and releases without sending still
    //    charges one spacing interval (over-spacing, the harmless
    //    direction). Phase 3's permit-dispatch API can refine the stamp
    //    point.
    //  - Permit span (IR6/R4-1): a permit is held from acquisition until the
    //    holder releases it — the pump releases at reply-finish, so a permit
    //    never survives into a retry sleep. Release is explicit or RAII.
    //
    // Waits are stop-interruptible with the same queued-resume contract as
    // the sleep primitive (D2/R4-2, S1-5): request_stop() returns before the
    // waiter resumes, and a stopped wait yields an invalid Permit without
    // ever holding a slot. Timing runs on the injected Scheduler, so the
    // tests are deterministic and never sleep ("Testing plan" item 4).
    //
    // Main-thread only, like every limiter object. Destroying the gate while
    // waiters are suspended is only valid after the event loop has stopped
    // (shutdown-by-destruction — nothing delivers resumptions there, S1-2).
    class Gate : public QObject
    {
        Q_OBJECT

    public:
        // Both tunable; revisit with capture data (D5).
        static constexpr int IN_FLIGHT_CAP = 2;
        static constexpr std::chrono::milliseconds MIN_SEND_SPACING{250};

        explicit Gate(Scheduler &scheduler, QObject *parent = nullptr);

        // Move-only RAII permit. A default-constructed (or stopped-wait)
        // Permit is invalid; releasing is idempotent and the destructor
        // releases whatever is still held.
        class Permit
        {
        public:
            Permit() = default;
            Permit(Permit &&other) noexcept;
            Permit &operator=(Permit &&other) noexcept;
            Permit(const Permit &) = delete;
            Permit &operator=(const Permit &) = delete;
            ~Permit();

            bool valid() const { return m_held; }
            void Release();

        private:
            friend class Gate;
            Permit(Gate *gate, bool head);

            QPointer<Gate> m_gate;
            bool m_head = false;
            bool m_held = false;
        };

        // Wait for an ordinary permit. Returns an invalid Permit if the
        // token stops first; a pre-stopped token never enqueues. The token
        // is deliberately not defaulted: every pump wait carries its
        // entry's token (D2) — a wait that is genuinely non-cancelable
        // passes {} explicitly.
        QCoro::Task<Permit> Acquire(std::stop_token token);

        // Wait for the exclusive HEAD permit (writer preference over
        // ordinary waiters; FIFO among HEADs). The setup path's probe
        // proceeds even when every parked entry cancels (D4), so its wait
        // passes {} explicitly.
        QCoro::Task<Permit> AcquireHead(std::stop_token token);

    private:
        struct WaiterState;

        QCoro::Task<Permit> AcquireImpl(bool head, std::stop_token token);
        void GrantPass();
        void ScheduleGrantPass(std::chrono::milliseconds when);
        void RecordDispatch();
        void ReleasePermit(bool head);

        Scheduler &m_scheduler;
        std::deque<std::shared_ptr<WaiterState>> m_ordinary;
        std::deque<std::shared_ptr<WaiterState>> m_heads;
        int m_active = 0;
        bool m_head_active = false;
        // A grant whose Permit has not yet stamped its dispatch time; no
        // further grant is decided while one is outstanding.
        bool m_grant_pending = false;
        std::optional<std::chrono::milliseconds> m_last_send;
        std::optional<std::chrono::milliseconds> m_scheduled_pass;
    };

} // namespace RateLimit
