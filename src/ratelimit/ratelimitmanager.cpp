// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#include "ratelimit/ratelimitmanager.h"

#include <QCoroNetworkReply>

#include <QNetworkReply>

#include <algorithm>

#include "ratelimit/gate.h"
#include "ratelimit/networkcapture.h"
#include "ratelimit/ratelimit.h"
#include "ratelimit/ratelimitedreply.h"
#include "ratelimit/ratelimitedrequest.h"
#include "ratelimit/ratelimitpolicy.h"
#include "ratelimit/scheduler.h"
#include "ratelimit/stopsleep.h"
#include "util/fatalerror.h"
#include "util/networkmanager.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/util.h"

// For debugging rate limit violations, keep around more history than should be needed
constexpr int HISTORY_BUFFER = 5;

// This HTTP status code means there was a rate limit violation.
constexpr int VIOLATION_STATUS = 429;

// A delay added to every send to avoid flooding the server.
constexpr int NORMAL_BUFFER_MSEC = 100;

// Total sends per entry: one original plus two retries (D3). Bounded so a
// systemically broken policy cannot hammer the API (N10).
constexpr int MAX_ATTEMPTS = 3;

// The retry deadline is max(now + Retry-After + pad + buffer,
// GetNextSafeSend(history)) — D3. The pad is unconditional: 60 s is the
// largest bucket tier GGG has named for any policy (N12), assumed to cover
// the legacy policy too; no policy inspection, so the retry path is immune
// to the initial-vs-sustained question (Q4). The buffer matches the timing
// bucket buffer in ratelimitpolicy.cpp.
constexpr int RETRY_BUCKET_PAD_SECS = 60;
constexpr int RETRY_BUFFER_SECS = 1;

// Maximum time we expect a request to take. This is used to detect
// issues like timezones and clock errors.
constexpr int MAXIMUM_API_RESPONSE_SEC = 60;

// This is another parameter used to check the system clock.
constexpr int MAXIMUM_EARLY_ARRIVAL_SEC = 30;

namespace {

    // The dispatch-time RAII reply owner (D3/R5-4): every attempt's reply is
    // released exactly once — by this guard for retried or abandoned
    // attempts, or by the caller after the final reply is handed over.
    struct DeferredDelete
    {
        void operator()(QNetworkReply *reply) const { reply->deleteLater(); }
    };
    using ReplyGuard = std::unique_ptr<QNetworkReply, DeferredDelete>;

} // namespace

RateLimitManager::RateLimitManager(SendFcn sender,
                                   RateLimit::Scheduler &scheduler,
                                   RateLimit::Gate &gate,
                                   NetworkCapture *capture)
    : m_sender(sender)
    , m_scheduler(scheduler)
    , m_gate(gate)
    , m_capture(capture)
    , m_policy(nullptr)
{
    spdlog::trace("RateLimitManager::RateLimitManager() entered");
}

RateLimitManager::~RateLimitManager() {}

void RateLimitManager::HoldUntil(const QDateTime &until)
{
    if (!m_earliest_send.isValid() || m_earliest_send < until) {
        m_earliest_send = until;
    }
}

const RateLimitPolicy &RateLimitManager::policy()
{
    if (!m_policy) {
        FatalError("The rate limit manager's policy is null!");
    }
    return *m_policy;
}

int RateLimitManager::msecToNextSend() const
{
    if (!m_next_send_deadline) {
        return -1;
    }
    const auto remaining = *m_next_send_deadline - m_scheduler.Now();
    return static_cast<int>(std::max<std::chrono::milliseconds::rep>(remaining.count(), 0));
}

void RateLimitManager::Update(QNetworkReply *reply)
{
    spdlog::trace("RateLimitManager::Update() entered");

    // Get the rate limit policy from this reply. The parse is total (D8):
    // a reply whose rate-limit headers fail the grammar updates nothing —
    // today's parser crashed on these inputs.
    auto parsed = RateLimitPolicy::Parse(reply);
    if (!parsed) {
        spdlog::error("Rate Limit Policy: not updating '{}': the reply's rate-limit headers "
                      "failed to parse: {}",
                      m_policy ? m_policy->name() : QString("<no policy>"),
                      parsed.error());
        return;
    }

    // If there was an existing policy, compare them.
    if (m_policy) {
        // A pump's policy name never changes after creation (D4/D8): a
        // mismatched name leaves the policy byte-for-byte un-updated.
        // Phase 4 surfaces the affected request as a Protocol error; until
        // then the loud log and the capture record are the detection.
        if (m_policy->name() != parsed->name()) {
            spdlog::error("Rate Limit Policy: REFUSING to update '{}' from a reply naming a "
                          "different policy '{}' — the policy is left un-updated (D8)",
                          m_policy->name(),
                          parsed->name());
            return;
        }
        // A changed definition under the same name is adopted: dynamic
        // limit changes must update pacing state. Check() logs the diff.
        if (!m_policy->Check(*parsed)) {
            spdlog::error("Rate Limit Policy: the updated policy is mismatched:\nCurrent "
                          "Policy:\n{}\nNew Policy:\n{}",
                          m_policy->GetPolicyReport(),
                          parsed->GetPolicyReport());
        }
    }

    // Update the rate limit policy.
    m_policy = std::make_unique<RateLimitPolicy>(std::move(*parsed));

    // Grow the history capacity if needed.
    const size_t max_hits = m_policy->maximum_hits();
    const size_t history_size = max_hits + HISTORY_BUFFER;
    if (history_size > m_history_size) {
        spdlog::debug("{}: increasing history size from {} events to {} events.",
                      m_policy->name(),
                      m_history_size,
                      history_size);
        m_history_size = history_size;
    }

    emit PolicyUpdated(policy());

    // Entries queued before the policy arrived can drain now.
    if (!m_draining && !m_failed && !m_queue.empty()) {
        m_draining = true;
        m_drain_task = Drain();
    }
}

void RateLimitManager::QueueRequest(const QString &endpoint,
                                    const QNetworkRequest &network_request,
                                    RateLimitedReply *reply)
{
    if (m_failed) {
        spdlog::error("The rate limit pump for '{}' is in the terminal failed state; refusing "
                      "the request for {}",
                      m_policy ? m_policy->name() : QString("<no policy>"),
                      endpoint);
        reply->deleteLater();
        return;
    }
    m_queue.push_back(std::make_unique<RateLimitedRequest>(endpoint, network_request, reply));
    if (m_draining) {
        emit QueueUpdated(m_policy->name(), static_cast<int>(m_queue.size()));
        return;
    }
    if (!m_policy) {
        // Held until Update() installs a policy: the drain needs pacing
        // state to run (the hub's Update-before-Queue contract).
        return;
    }
    m_draining = true;
    m_drain_task = Drain();
}

QCoro::Task<> RateLimitManager::Drain()
{
    // No exception ever escapes a root coroutine (IR4/R5-1): the catch-all
    // is what implements the terminal failed state.
    try {
        while (!m_queue.empty()) {
            auto entry = std::move(m_queue.front());
            m_queue.pop_front();
            emit QueueUpdated(m_policy->name(), static_cast<int>(m_queue.size()));
            co_await ProcessEntry(*entry);
        }
    } catch (const std::exception &e) {
        m_failed = true;
        spdlog::error("The rate limit pump for '{}' FAILED with an exception: {} — {} queued "
                      "requests dropped; later submissions will be refused",
                      m_policy ? m_policy->name() : QString("<no policy>"),
                      e.what(),
                      m_queue.size());
    } catch (...) {
        m_failed = true;
        spdlog::error("The rate limit pump for '{}' FAILED with an unknown exception — {} queued "
                      "requests dropped; later submissions will be refused",
                      m_policy ? m_policy->name() : QString("<no policy>"),
                      m_queue.size());
    }
    if (m_failed) {
        // Fail the remaining entries fast. Destroying the caller's
        // RateLimitedReply without a completion is the only failure this
        // boundary can express for an unsent request; phase 4's FetchError
        // makes these clean Internal errors.
        m_queue.clear();
        m_next_send_deadline.reset();
    }
    m_draining = false;
}

QCoro::Task<> RateLimitManager::ProcessEntry(RateLimitedRequest &entry)
{
    // Pacing sleep: the same GetNextSafeSend arithmetic as ever (N25/N26),
    // plus the normal buffer while the policy is below borderline, plus
    // any externally-imposed hold (the D4 HEAD-429 case).
    QDateTime next_send = m_policy->GetNextSafeSend(m_history);
    if (m_policy->status() < RateLimit::Status::BORDERLINE) {
        next_send = next_send.addMSecs(NORMAL_BUFFER_MSEC);
    }
    if (m_earliest_send.isValid() && next_send < m_earliest_send) {
        next_send = m_earliest_send;
    }
    entry.scheduled_time = next_send;
    const qint64 pacing_delay = QDateTime::currentDateTime().msecsTo(next_send);
    if (pacing_delay > 0) {
        spdlog::trace("{} waiting {} msecs to send request {}",
                      m_policy->name(),
                      pacing_delay,
                      entry.id);
        AnnouncePause(next_send, m_scheduler.Now() + std::chrono::milliseconds(pacing_delay));
        co_await RateLimit::SleepUntil(m_scheduler, *m_next_send_deadline, {});
        m_next_send_deadline.reset();
    }

    for (int attempt = 1; attempt <= MAX_ATTEMPTS; ++attempt) {
        // Every send acquires the gate (D5). Phase-3 entries carry no stop
        // token, so the wait is explicitly non-cancelable.
        auto permit = co_await m_gate.Acquire(std::stop_token{});

        entry.send_time = QDateTime::currentDateTime().toLocalTime();
        permit.MarkDispatched();
        ReplyGuard reply(m_sender(entry.network_request));
        if (!reply) {
            // A sender that cannot send is a wiring bug; contained as a
            // pump failure rather than a null dereference.
            throw std::runtime_error("the rate limit manager's sender returned no reply");
        }

        co_await reply.get();

        // The permit span ends at reply-finish (D5): a permit is never
        // held across a retry sleep (R4-1).
        permit.Release();

        // Bookkeeping first, then observation, then classification (D3).
        RecordLandedReply(entry, reply.get());

        const int status = RateLimit::ParseStatus(reply.get());
        if (status == VIOLATION_STATUS) {
            // Every 429 records its violation before any retry decision:
            // the server counted the exchange (N25).
            spdlog::error("Rate limit violation detected for policy '{}':\n{}",
                          m_policy->name(),
                          m_policy->GetBorderlineReport());
            LogPolicyHistory();
            emit Violation(m_policy->name());

            const auto retry_after = RateLimit::ParseRetryAfter(reply.get());
            if (retry_after && attempt < MAX_ATTEMPTS) {
                const QDateTime now = QDateTime::currentDateTime();
                QDateTime retry_at = now.addSecs(*retry_after + RETRY_BUCKET_PAD_SECS
                                                 + RETRY_BUFFER_SECS);
                const QDateTime safe_send = m_policy->GetNextSafeSend(m_history);
                if (retry_at < safe_send) {
                    retry_at = safe_send;
                }
                spdlog::error("Rate limit VIOLATION for policy {} (retrying at {}, attempt "
                              "{} of {})",
                              m_policy->name(),
                              retry_at.toString(),
                              attempt + 1,
                              MAX_ATTEMPTS);
                entry.scheduled_time = retry_at;
                const qint64 retry_delay = std::max<qint64>(now.msecsTo(retry_at), 0);
                AnnouncePause(retry_at, m_scheduler.Now() + std::chrono::milliseconds(retry_delay));
                co_await RateLimit::SleepUntil(m_scheduler, *m_next_send_deadline, {});
                m_next_send_deadline.reset();
                // The guard releases this attempt's 429 reply; the next
                // attempt re-acquires the gate like every send (ER7).
                continue;
            }
            // Terminal 429: retries exhausted, or no acceptable Retry-After
            // (missing, non-numeric, negative, or above the cap). Complete
            // immediately — never sleep on an exhausted attempt (D3).
            if (!retry_after) {
                spdlog::error("Rate limit violation for policy {} is not retryable: no "
                              "acceptable Retry-After",
                              m_policy->name());
            } else {
                spdlog::error("Rate limit violation for policy {}: retries exhausted after {} "
                              "attempts",
                              m_policy->name(),
                              attempt);
            }
        } else if (reply->error() == QNetworkReply::NoError
                   && m_policy->status() >= RateLimit::Status::VIOLATION) {
            // The reply looked fine but the headers report a violation.
            spdlog::error("Reply did not have an error, but the rate limit policy shows a "
                          "violation occured.");
            spdlog::error("Rate limit violation detected for policy '{}':\n{}",
                          m_policy->name(),
                          m_policy->GetBorderlineReport());
            LogPolicyHistory();
            emit Violation(m_policy->name());
        } else if (reply->error() != QNetworkReply::NoError) {
            // Non-429 failures are not retried: the errored reply is
            // surfaced to the caller, who counts the request as complete.
            spdlog::error("policy manager for {} request {} reply status was {} and error was {}",
                          m_policy->name(),
                          entry.id,
                          status,
                          reply->error());
            NetworkManager::logRequest(entry.network_request);
            NetworkManager::logReply(reply.get());
        }

        // Final outcome — success, non-retryable error, or terminal 429:
        // hand the reply to the caller, who owns it now (the pump is
        // stable and the permit long released — IR6).
        if (entry.reply) {
            QNetworkReply *raw = reply.release();
            emit entry.reply->complete(raw);
            // The caller's handle is destroyed synchronously after the
            // emission — the pinned F59 ownership behavior, unchanged
            // until the QFuture boundary replaces it in phase 4.
            entry.reply.reset();
        }
        co_return;
    }
}

void RateLimitManager::RecordLandedReply(RateLimitedRequest &entry, QNetworkReply *reply)
{
    const QDateTime now = QDateTime::currentDateTime().toLocalTime();

    // Capture before any validation, so degraded replies are recorded too.
    if (m_capture) {
        m_capture->RecordReply(m_policy->name(), entry, reply, now);
    }

    // Every landed reply records its history event (D3) — the server
    // counted the exchange whatever the client does next (N25).
    RateLimit::Event event;
    event.request_id = entry.id;
    event.request_url = entry.network_request.url().toString();
    event.request_time = entry.send_time;
    event.received_time = now;
    event.reply_time = RateLimit::ParseDate(reply).toLocalTime();
    event.reply_status = RateLimit::ParseStatus(reply);
    m_history.push_front(event);
    if (m_history.size() > m_history_size) {
        m_history.pop_back();
    }

    if (event.reply_time.isValid()) {
        const int response_sec = event.request_time.secsTo(event.reply_time);
        if (response_sec > MAXIMUM_API_RESPONSE_SEC) {
            spdlog::error("WARNING: The system clock may be wrong: an API call seems to have "
                          "taken too long: {} seconds."
                          " This may lead to API rate limit violations.",
                          response_sec);
        } else if (response_sec < -MAXIMUM_EARLY_ARRIVAL_SEC) {
            spdlog::error("WARNING: The system clock may be wrong: an API call seems to have "
                          "been answered {}s before it was made."
                          " This may lead to API rate limit violations.",
                          -response_sec);
        }
    }

    spdlog::trace("RateLimitManager {} received reply for request {} with status {}",
                  m_policy->name(),
                  event.request_id,
                  event.reply_status);

    // Observation follows bookkeeping and precedes classification
    // (R6-3/D8): the headers are parse-attempted on every landed reply,
    // whatever its status — a 429, a 500, and a 2xx-plus-transport-error
    // all keep the policy current. Update() applies the Full-and-matching
    // gate and logs failures loudly.
    Update(reply);

    if (m_policy->status() == RateLimit::Status::BORDERLINE) {
        spdlog::warn("Rate limit policy '{}' is BORDERLINE and the next safe send is at {}",
                     m_policy->name(),
                     m_policy->GetNextSafeSend(m_history).toString());
    }
}

void RateLimitManager::AnnouncePause(const QDateTime &until, std::chrono::milliseconds deadline)
{
    m_next_send_deadline = deadline;
    emit Paused(m_policy->name(), until);
}

void RateLimitManager::LogPolicyHistory()
{
    const QString status = Util::toString(m_policy->status());
    QStringList lines;
    lines.append("Rate Limit Policy details:");
    lines.append(
        QString("<RATE_LIMIT_POLICY policy_name='%1' status='%2'>").arg(m_policy->name(), status));
    for (const auto &rule : m_policy->rules()) {
        for (const auto &item : rule.items()) {
            lines.append(QString("%1:%2(%3s) = %4/%5")
                             .arg(m_policy->name(),
                                  rule.name(),
                                  QString::number(item.limit().period()),
                                  QString::number(item.state().hits()),
                                  QString::number(item.limit().hits())));
        }
    }
    for (size_t i = 0; i < m_history.size(); ++i) {
        const auto &item = m_history[i];
        lines.append(QString("#%1: request %2 sent %3, received %4, status %5: %6")
                         .arg(QString::number(i + 1),
                              QString::number(item.request_id),
                              item.request_time.toString("yyyy-MMM-dd HH:mm:ss.zzz"),
                              item.reply_time.toString("yyyy-MMM-dd HH:mm:ss.zzz"),
                              QString::number(item.reply_status),
                              item.request_url));
    }
    lines.append("</RATE_LIMIT_POLICY>");
    spdlog::error(lines.join("\n"));
}
