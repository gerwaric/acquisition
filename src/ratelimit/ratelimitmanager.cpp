// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#include "ratelimitmanager.h"

#include <QApplication>
#include <QErrorMessage>
#include <QMessageBox>
#include <QNetworkReply>

#include "ratelimit.h"
#include "ratelimitedreply.h"
#include "ratelimitedrequest.h"
#include "ratelimiter.h"
#include "ratelimitpolicy.h"
#include "util/fatalerror.h"
#include "util/spdlog_qt.h"
#include "util/util.h"

static_assert(ACQUISITION_USE_SPDLOG);

// For debugging rate limit violations, keep around more history than should be needed
constexpr int HISTORY_BUFFER = 5;

// This HTTP status code means there was a rate limit violation.
constexpr int VIOLATION_STATUS = 429;

// A delay added to every send to avoid flooding the server.
constexpr int NORMAL_BUFFER_MSEC = 100;

// Minium time between sends for any given policy.
constexpr int MINIMUM_INTERVAL_MSEC = 1000;

// Maximum time we expect a request to take. This is used to detect
// issues like timezones and clock errors.
constexpr int MAXIMUM_API_RESPONSE_SEC = 60;

// This is another parameter used to check the system clock.
constexpr int MAXIMUM_EARLY_ARRIVAL_SEC = 30;

// Create a new rate limit manager based on an existing policy.
RateLimitManager::RateLimitManager(SendFcn sender)
    : m_sender(sender)
    , m_policy(nullptr)
{
    spdlog::trace("RateLimitManager::RateLimitManager() entered");
    // Setup the active request timer to call SendRequest each time it's done.
    m_activation_timer.setSingleShot(true);
    connect(&m_activation_timer, &QTimer::timeout, this, &RateLimitManager::SendRequest);
}

RateLimitManager::~RateLimitManager() {}

const RateLimitPolicy &RateLimitManager::policy()
{
    if (!m_policy) {
        FatalError("The rate limit manager's policy is null!");
    }
    return *m_policy;
}

// Send the active request immediately.
void RateLimitManager::SendRequest()
{
    spdlog::trace("RateLimitManager::SendRequest() entered");
    if (!m_policy) {
        spdlog::error("The rate limit manager attempted to send a request without a policy.");
        return;
    }

    if (!m_active_request) {
        spdlog::error(
            "The rate limit manager attempted to send a request with no request to send.");
        return;
    }

    auto &request = *m_active_request;
    spdlog::trace("{} sending request {} to {} via {}",
                  m_policy->name(),
                  request.id,
                  request.endpoint,
                  request.network_request.url().toString());

    if (!m_sender) {
        spdlog::error("Rate limit manager cannot send requests.");
        return;
    }
    m_active_request->send_time = QDateTime::currentDateTime().toLocalTime();
    QNetworkReply *reply = m_sender(request.network_request);
    connect(reply, &QNetworkReply::finished, this, &RateLimitManager::ReceiveReply);
}

// Called when the active request's reply is finished.
void RateLimitManager::ReceiveReply()
{
    spdlog::trace("RateLimitManager::ReceiveReply() entered");
    QNetworkReply *reply = qobject_cast<QNetworkReply *>(sender());

    const QDateTime now = QDateTime::currentDateTime().toLocalTime();

    if (!m_policy) {
        spdlog::error("The rate limit manager cannot recieve a reply when the policy is null.");
        return;
    }

    if (!m_active_request) {
        spdlog::error("The rate limit manager received a reply without an active request.");
        return;
    }

    // Make sure the reply has a rate-limit header.
    if (!reply->hasRawHeader("X-Rate-Limit-Policy")) {
        spdlog::error("The rate limit manager received a reply for {} without rate limit headers.",
                      m_policy->name());
        return;
    }

    // Add this reply to the history.
    RateLimit::Event event;
    event.request_id = m_active_request->id;
    event.request_url = m_active_request->network_request.url().toString();
    event.request_time = m_active_request->send_time;
    event.received_time = now;
    event.reply_time = RateLimit::ParseDate(reply).toLocalTime();
    event.reply_status = RateLimit::ParseStatus(reply);

    m_history.push_front(event);
    if (m_history.size() > m_history_size) {
        m_history.pop_back();
    }

    const int response_sec = event.request_time.secsTo(event.reply_time);
    if (response_sec > MAXIMUM_API_RESPONSE_SEC) {
        spdlog::error("WARNING: The system clock may be wrong: an API call seems to have taken too "
                      "long: {} seconds."
                      " This may lead to API rate limit violations.",
                      response_sec);
    } else if (response_sec < -MAXIMUM_EARLY_ARRIVAL_SEC) {
        spdlog::error("WARNING: The system clock may be wrong: an API call seems to have been "
                      "answered {}s before it was made."
                      " This may lead to API rate limit violations.",
                      -response_sec);
    }

    spdlog::trace("RateLimitManager {} received reply for request {} with status {}",
                  m_policy->name(),
                  event.request_id,
                  event.reply_status);

    // Now examine the new policy and update ourselves accordingly.
    Update(reply);

    bool violation_detected = false;

    if (reply->error() == QNetworkReply::NoError) {
        // Check for errors.
        if (m_policy->status() >= RateLimit::Status::VIOLATION) {
            spdlog::error("Reply did not have an error, but the rate limit policy shows a "
                          "violation occured.");
            violation_detected = true;
        }
        if (event.reply_status == VIOLATION_STATUS) {
            spdlog::error("Reply did not have an error, but the HTTP status indicates a rate limit "
                          "violation.");
            violation_detected = true;
        }

        // Since the request finished successfully, signal complete()
        // so anyone listening can handle the reply.
        if (m_active_request->reply) {
            spdlog::trace("RateLimiteManager::ReceiveReply() about to emit 'complete' signal");
            emit m_active_request->reply->complete(reply);
        } else {
            spdlog::error("Cannot complete the rate limited request because the reply is null: {} "
                          "request {}: {}",
                          m_policy->name(),
                          m_active_request->id,
                          m_active_request->network_request.url().toString());
        }

        m_active_request = nullptr;

        // Activate the next queued reqeust.
        ActivateRequest();

    } else {
        reply->deleteLater();

        if (event.reply_status == VIOLATION_STATUS) {
            if (!reply->hasRawHeader("Retry-After")) {
                spdlog::error(
                    "HTTP status indicates a rate limit violation, but 'Retry-After' is missing");
            }
            if (m_policy->status() != RateLimit::Status::VIOLATION) {
                spdlog::error("HTTP status indicates a rate limit violation, but was not flagged "
                              "in the policy update");
            }
            violation_detected = true;
        }

        if (reply->hasRawHeader("Retry-After")) {
            // There was a rate limit violation.
            violation_detected = true;
            const int retry_sec = reply->rawHeader("Retry-After").toInt();
            const int retry_msec = (1000 * retry_sec);
            spdlog::error("Rate limit VIOLATION for policy {} (retrying after {} seconds)",
                          m_policy->name(),
                          (retry_msec / 1000));
            m_activation_timer.setInterval(retry_msec);
            m_activation_timer.start();

        } else {
            // Some other HTTP error was encountered.
            spdlog::error("policy manager for {} request {} reply status was {} and error was {}",
                          m_policy->name(),
                          m_active_request->id,
                          event.reply_status,
                          reply->error());
        }
        m_active_request->reply = nullptr;
    }

    if (violation_detected) {
        spdlog::error("Rate limit violation detected for policy '{}':\n{}",
                      m_policy->name(),
                      m_policy->GetBorderlineReport());
        LogPolicyHistory();
        emit Violation(m_policy->name());
    } else {
        // For now, let's print the borderline report for trace debugging.
        if (m_policy->status() == RateLimit::Status::BORDERLINE) {
            spdlog::warn("Rate limit policy '{}' is BORDERLINE and the next safe send is at {}",
                         m_policy->name(),
                         m_policy->GetNextSafeSend(m_history).toString());
            if (spdlog::should_log(spdlog::level::trace)) {
                spdlog::trace("Rate limit borderline report for policy '{}':\n{}",
                              m_policy->name(),
                              m_policy->GetBorderlineReport());
            }
        }
    }
}

void RateLimitManager::Update(QNetworkReply *reply)
{
    spdlog::trace("RateLimitManager::Update() entered");

    // Get the rate limit policy from this reply.
    spdlog::trace("RateLimitManager::Update() parsing policy");
    auto new_policy = std::make_unique<RateLimitPolicy>(reply);

    // If there was an existing policy, compare them.
    if (m_policy) {
        spdlog::trace("RateLimitManager::Update() {} checking update against existing policy",
                      m_policy->name());
        if (!m_policy->Check(*new_policy)) {
            spdlog::error("Rate Limit Policy: the updated policy is mismatched:\nCurrent "
                          "Policy:\n{}\nNew Policy:\n{}",
                          m_policy->GetPolicyReport(),
                          new_policy->GetPolicyReport());
        }
    }

    // Update the rate limit policy.
    m_policy = std::move(new_policy);

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
}

// If the rate limit manager is busy, the request will be queued.
// Otherwise, the request will be sent immediately, making the
// manager busy and causing subsequent requests to be queued.
void RateLimitManager::QueueRequest(const QString &endpoint,
                                    const QNetworkRequest &network_request,
                                    RateLimitedReply *reply)
{
    auto request = std::make_unique<RateLimitedRequest>(endpoint, network_request, reply);
    m_queued_requests.push_back(std::move(request));
    if (m_active_request) {
        emit QueueUpdated(m_policy->name(), static_cast<int>(m_queued_requests.size()));
    } else {
        ActivateRequest();
    }
}

// Send the active request at the next time it will be safe to do so
// without violating the rate limit policy.
void RateLimitManager::ActivateRequest()
{
    spdlog::trace("RateLimitManager::ActivateRequest() entered");
    if (!m_policy) {
        spdlog::error("Cannot activate a request because the policy is null.");
        return;
    }
    if (m_active_request) {
        spdlog::debug("Cannot activate a request because a request is already active.");
        return;
    }
    if (m_queued_requests.empty()) {
        spdlog::debug("Cannot active a request because the queue is empty.");
        return;
    }

    m_active_request = std::move(m_queued_requests.front());
    m_queued_requests.pop_front();
    emit QueueUpdated(m_policy->name(), static_cast<int>(m_queued_requests.size()));

    const QDateTime now = QDateTime::currentDateTime();

    QDateTime next_send = m_policy->GetNextSafeSend(m_history);

    if (next_send.isValid() == false) {
        spdlog::error("Cannot activate a request because the next send is invalid");
        return;
    }

    spdlog::trace(
        "RateLimitManager::ActivateRequest() {} next_send before adjustment is {} (in {} seconds)",
        m_policy->name(),
        next_send.toString(),
        now.secsTo(next_send));

    if (m_policy->status() < RateLimit::Status::BORDERLINE) {
        spdlog::trace("RateLimitManager::ActivateRequest() {} is NOT b,orderline, adding {} msecs "
                      "to next send",
                      m_policy->name(),
                      NORMAL_BUFFER_MSEC);
        next_send = next_send.addMSecs(NORMAL_BUFFER_MSEC);
    }

    static QDateTime last_send;

    if (last_send.isValid()) {
        if (last_send.msecsTo(next_send) < MINIMUM_INTERVAL_MSEC) {
            spdlog::trace("RateLimitManager::ActivateRequest() adding {} to next send",
                          MINIMUM_INTERVAL_MSEC);
            next_send = last_send.addMSecs(MINIMUM_INTERVAL_MSEC);
        }
    }

    int delay = QDateTime::currentDateTime().msecsTo(next_send);
    if (delay < 0) {
        delay = 0;
    }

    spdlog::trace(
        "RateLimitManager::ActivateRequest() {} waiting {} msecs to send request {} at {}",
        m_policy->name(),
        delay,
        m_active_request->id,
        next_send.toLocalTime().toString());
    m_activation_timer.setInterval(delay);
    m_activation_timer.start();
    if (delay > 0) {
        emit Paused(m_policy->name(), next_send);
    }
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
