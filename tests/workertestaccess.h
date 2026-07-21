// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <cstddef>
#include <functional>

#include "itemsmanagerworker.h"

// The one test-only seam into ItemsManagerWorker (network-redesign phase 5,
// verification §2): a friend accessor that bundles every observation and
// injection the worker suite needs — the injected read-only sweep observer and
// one-shot fault hook, and read-only snapshots of the outstanding task handles
// and the progress counters — instead of standing *ForTest / Set* methods on the
// production API.
//
// The worker header only forward-declares this as a friend; it is DEFINED here,
// in a test-only header, so production code cannot reach the worker's private
// state through it. Included by the worker suite (and, later, the phase-5
// integration runner). The worker in turn has no test-only public API.
class WorkerTestAccess
{
public:
    // A snapshot of the completion counters, read on each StatusUpdate to pin
    // that reported progress is monotonic (P-STATUS). Lives here, not on the
    // worker, so the production type carries no test data structs.
    struct ProgressForTest
    {
        size_t stashes_received;
        size_t stashes_needed;
        size_t characters_received;
        size_t characters_needed;
    };

    explicit WorkerTestAccess(ItemsManagerWorker &worker)
        : m_worker(worker)
    {}

    // Inject a read-only sweep observer (W-SWEEP). The worker writes counts as it
    // schedules/runs sweeps; null in production leaves behavior unchanged.
    void setSweepObserver(WorkerSweepObserver *observer) { m_worker.m_sweep_observer = observer; }

    // Arm a one-shot fault hook (verification §3, W-THROW). It fires once at the
    // next fault site reached — the root orchestration body (RunUpdate) or a
    // failure handler right after it stops the token, before it finishes its
    // failure bookkeeping — so a test can prove each catch-all contains a throw
    // and drives a terminal AbortUpdate() without relying on Qt's undefined
    // slot-throwing behavior. Null in production.
    void setFaultHook(std::function<void()> hook) { m_worker.m_fault_hook = std::move(hook); }

    // How many per-fetch task handles the worker still holds. This reaches zero
    // only after every awaited future — including any stopped old-update
    // straggler — has settled and the deferred sweep has drained, not merely when
    // an update reaches its terminal state. Ordinary fixtures assert zero at
    // teardown (verification §2).
    size_t outstandingFetchTasks() const { return m_worker.m_fetch_tasks.size(); }

    ProgressForTest progressCounters() const
    {
        return {m_worker.m_stashes_received,
                m_worker.m_stashes_needed,
                m_worker.m_characters_received,
                m_worker.m_characters_needed};
    }

private:
    ItemsManagerWorker &m_worker;
};
