// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#pragma once

#include <QCoroTask>

#include <chrono>
#include <deque>
#include <memory>
#include <optional>

#include <QDateTime>
#include <QNetworkRequest>
#include <QObject>
#include <QString>

#include "ratelimit/ratelimit.h"

class QNetworkReply;

class NetworkCapture;
class RateLimitedReply;
struct RateLimitedRequest;
class RateLimitPolicy;

namespace RateLimit {
    class Gate;
    class Scheduler;
} // namespace RateLimit

// Manages a single rate limit policy, which may apply to multiple endpoints.
//
// The policy pump (network-redesign spec, D3): a plain deque of entries and
// an on-demand drain coroutine — one linear loop (take, pace, gate, send,
// maybe retry, deliver) instead of the old timer-and-signal state machine.
// The 429 retry is a loop iteration: invisible to the caller, who sees
// exactly one final completion (the F57 wedge is unconstructible).
//
// Phase-3 boundary note: completion is still the RateLimitedReply shape —
// the caller receives the final QNetworkReply, errored or not, and owns it.
// Entries carry no stop token yet (nothing can cancel them until the
// QFuture boundary lands in phase 4), so every wait passes an explicit
// never-stopped token.
class RateLimitManager : public QObject
{
    Q_OBJECT

public:
    // This is the signature of the function used to send requests.
    using SendFcn = std::function<QNetworkReply *(QNetworkRequest &)>;

    RateLimitManager(SendFcn sender,
                     RateLimit::Scheduler &scheduler,
                     RateLimit::Gate &gate,
                     NetworkCapture *capture = nullptr);
    ~RateLimitManager();

    // Move a request into this manager's queue. The drain starts (or is
    // joined) if a policy is installed; entries queued before the first
    // Update() wait for it.
    void QueueRequest(const QString &endpoint,
                      const QNetworkRequest &request,
                      RateLimitedReply *reply);

    // Install or update the policy from a reply's rate-limit headers.
    // Total-parse gated (D8): headers that fail the grammar update nothing,
    // and a Full set whose policy name does not match the installed
    // policy's is refused loudly — a pump's policy name never changes after
    // creation (D4). Definition changes under the same name are adopted
    // (logged by Check()).
    void Update(QNetworkReply *reply);

    const RateLimitPolicy &policy();

    // Hold the next send until no earlier than the given time — the D4
    // HEAD-429 case: the hub establishes the pump but its first send waits
    // out Retry-After + pad + buffer. Sends pace normally afterwards.
    void HoldUntil(const QDateTime &until);

    // Milliseconds until the next scheduled send: the remaining pacing or
    // retry sleep, measured on the injected scheduler; -1 when no send is
    // scheduled (idle, waiting at the gate, or in flight).
    int msecToNextSend() const;

signals:
    // Emitted when the underlying policy has been updated.
    void PolicyUpdated(const RateLimitPolicy &policy);

    // Emitted when a request has been added to the queue;
    void QueueUpdated(const QString policy_name, int queued_requests);

    // Emitted when a network request has to wait to be sent.
    void Paused(const QString &policy_name, const QDateTime &until);

    // Emitted when a rate limit violation has been detected.
    void Violation(const QString &policy_name);

private:
    // The drain coroutine: processes entries until the deque is empty,
    // then returns. Started by QueueRequest (or Update, for entries queued
    // before the policy arrived) when none is running. Its whole body is
    // wrapped in a catch-all — no exception ever escapes a root coroutine
    // (IR4/R5-1): an escape completes nothing, fails the queue, and puts
    // the pump in a terminal failed state, loudly.
    QCoro::Task<> Drain();

    // One entry, start to final outcome: pacing sleep, then up to
    // MAX_ATTEMPTS gate-acquire/send/classify rounds per the D3 attempt
    // table.
    QCoro::Task<> ProcessEntry(RateLimitedRequest &entry);

    // Per-send bookkeeping, always first (D3): every landed reply records
    // its history event and capture record, then is parse-attempted for a
    // policy update (observation precedes classification, R6-3/D8).
    void RecordLandedReply(RateLimitedRequest &entry, QNetworkReply *reply);

    // Emit Paused and remember the deadline msecToNextSend() reports.
    void AnnouncePause(const QDateTime &until, std::chrono::milliseconds deadline);

    // Used to print log messages about rate limit violations.
    void LogPolicyHistory();

    // Function handle used to send network requests.
    const SendFcn m_sender;

    // Injected clock/timer and the hub's layer-1 gate (D5). Every send
    // acquires the gate; pacing and retry sleeps run on the scheduler.
    RateLimit::Scheduler &m_scheduler;
    RateLimit::Gate &m_gate;

    // Optional research instrument (see docs/design/network-ground-truth.md);
    // owned by the RateLimiter, null when capture is disabled.
    NetworkCapture *const m_capture;

    // Keep a unique_ptr to the policy associated with this manager,
    // which will be updated whenever a reply with valid rate limit
    // headers and a matching policy name is received.
    std::unique_ptr<RateLimitPolicy> m_policy;

    // Requests that are waiting to be processed by the drain.
    std::deque<std::unique_ptr<RateLimitedRequest>> m_queue;

    // The drain task handle (owned member, never fire-and-forget). Only
    // replaced while no drain is running — replacing it then destroys a
    // completed frame, never detaches a live one (S1-1).
    QCoro::Task<> m_drain_task;
    bool m_draining = false;

    // Set when the drain's catch-all trips: the pump failed terminally and
    // loudly; later submissions are refused (no restart — a restart on
    // throw risks a tight crash loop; the next session starts clean).
    bool m_failed = false;

    // The deadline of the pacing or retry sleep currently in progress, on
    // the scheduler's clock; empty when no send is scheduled.
    std::optional<std::chrono::milliseconds> m_next_send_deadline;

    // Sends are held until this time when valid (HoldUntil).
    QDateTime m_earliest_send;

    // We use a history of the received reply times so that we can calculate
    // when the next safe send time will be. This allows us to calculate the
    // least delay necessary to stay compliant.
    std::deque<RateLimit::Event> m_history;
    size_t m_history_size{0};
};
