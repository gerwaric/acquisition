/*
    Copyright (C) 2014-2024 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "ratelimitmanager.h"

#include <QApplication>
#include <QErrorMessage>
#include <QMessageBox>
#include <QNetworkAccessManager>
#include <QNetworkReply>

#include <QsLog/QsLog.h>

#include "util/fatalerror.h"
#include "util/oauthmanager.h"

#include "ratelimit.h"
#include "ratelimitpolicy.h"
#include "ratelimitedreply.h"
#include "ratelimitedrequest.h"
#include "ratelimiter.h"

// This HTTP status code means there was a rate limit violation.
constexpr int VIOLATION_STATUS = 429;

// A delay added to every send to avoid flooding the server.
constexpr int NORMAL_BUFFER_MSEC = 100;

// Minium time between sends for any given policy.
constexpr int MINIMUM_INTERVAL_MSEC = 250;

// GGG has stated that when they are keeping track of request times,
// they have a timing resolution, which they called a "bucket".
// 
// This explained some otherwise mysterious rate violations that I
// was seeing very intermittently. Unless there's a away to find out
// where those timing buckets begin and end precisely, all we can do
// is use the bucket size as a minimum delay.
//
// GGG has also stated that this bucket resolution may be different
// for different policies, but the one I had been asking them about
// was 5.0 seconds. They also noted that this number is currently
// not documented or exposed to api users in any way.
//
// So until GGG provides more information, I'm applying a minimum
// 5.2 second delay when we are at or over the limit.
//
// In the future hopefully there will be a way to use the right
// number for each policy.
constexpr int TIMING_BUCKET_MSEC = 5200;

// Create a new rate limit manager based on an existing policy.
RateLimitManager::RateLimitManager(SendFcn sender)
    : m_sender(sender)
    , m_policy(nullptr)
{
    QLOG_TRACE() << "RateLimitManager::RateLimitManager() entered";
    // Setup the active request timer to call SendRequest each time it's done.
    m_activation_timer.setSingleShot(true);
    connect(&m_activation_timer, &QTimer::timeout, this, &RateLimitManager::SendRequest);
}

RateLimitManager::~RateLimitManager() {
}

const RateLimitPolicy& RateLimitManager::policy() {
    if (!m_policy) {
        FatalError("The rate limit manager's policy is null!");
    };
    return *m_policy;
}

// Send the active request immediately.
void RateLimitManager::SendRequest() {

    QLOG_TRACE() << "RateLimitManager::SendRequest() entered";
    if (!m_policy) {
        QLOG_ERROR() << "The rate limit manager attempted to send a request without a policy.";
        return;
    };

    if (!m_active_request) {
        QLOG_ERROR() << "The rate limit manager attempted to send a request with no request to send.";
        return;
    };

    auto& request = *m_active_request;
    QLOG_TRACE() << m_policy->name()
        << "sending request" << request.id
        << "to" << request.endpoint
        << "via" << request.network_request.url().toString();

    if (!m_sender) {
        QLOG_ERROR() << "Rate limit manager cannot send requests.";
        return;
    };
    QNetworkReply* reply = m_sender(request.network_request);
    connect(reply, &QNetworkReply::finished, this, &RateLimitManager::ReceiveReply);
};

// Called when the active request's reply is finished.
void RateLimitManager::ReceiveReply()
{
    QLOG_TRACE() << "RateLimitManager::ReceiveReply() entered";
    QNetworkReply* reply = qobject_cast<QNetworkReply*>(sender());

    if (!m_policy) {
        QLOG_ERROR() << "The rate limit manager cannot recieve a reply when the policy is null.";
        return;
    };

    if (!m_active_request) {
        QLOG_ERROR() << "The rate limit manager received a reply without an active request.";
        return;
    };

    // Make sure the reply has a rate-limit header.
    if (!reply->hasRawHeader("X-Rate-Limit-Policy")) {
        QLOG_ERROR() << "Received a reply for" << m_policy->name() << "without rate limit headers.";
        return;
    };

    const QDateTime reply_time = RateLimit::ParseDate(reply).toLocalTime();
    const int reply_status = RateLimit::ParseStatus(reply);
    QLOG_TRACE() << "RateLimitManager::ReceiveReply()"
        << m_policy->name()
        << "received reply for request" << m_active_request->id
        << "with status" << reply_status;

    // Save the reply time.
    QLOG_TRACE() << "RateLimitManager::ReceiveReply()"
        << m_policy->name()
        << "adding to history:" << reply_time.toString();
    m_history.push_front(reply_time);

    // Now examine the new policy and update ourselves accordingly.
    Update(reply);

    bool violation_detected = false;

    if (reply->error() == QNetworkReply::NoError) {

        // Check for errors.
        if (m_policy->status() >= RateLimitPolicy::Status::VIOLATION) {
            QLOG_ERROR() << "Reply did not have an error, but the rate limit policy shows a violation occured.";
            violation_detected = true;
        };
        if (reply_status == VIOLATION_STATUS) {
            QLOG_ERROR() << "Reply did not have an error, but the HTTP status indicates a rate limit violation.";
            violation_detected = true;
        };

        // Since the request finished successfully, signal complete()
        // so anyone listening can handle the reply.
        if (m_active_request->reply) {
            QLOG_TRACE() << "RateLimiteManager::ReceiveReply() about to emit 'complete' signal";
            emit m_active_request->reply->complete(reply);
        } else {
            QLOG_ERROR() << "Cannot complete the rate limited request because the reply is null.";
        };

        m_active_request = nullptr;

        // Activate the next queued reqeust.
        ActivateRequest();

    } else {

        reply->deleteLater();

        if (reply_status == VIOLATION_STATUS) {
            if (!reply->hasRawHeader("Retry-After")) {
                QLOG_ERROR() << "HTTP status indicates a rate limit violation, but 'Retry-After' is missing";
            };
            if (m_policy->status() != RateLimitPolicy::Status::VIOLATION) {
                QLOG_ERROR() << "HTTP status indicates a rate limit violation, but was not flagged in the policy update";
            };
            violation_detected = true;
        };

        if (reply->hasRawHeader("Retry-After")) {

            // There was a rate limit violation.
            violation_detected = true;
            const int retry_sec = reply->rawHeader("Retry-After").toInt();
            const int retry_msec = (1000 * retry_sec) + TIMING_BUCKET_MSEC;
            QLOG_ERROR() << "Rate limit VIOLATION for policy"
                << m_policy->name()
                << "(retrying after" << (retry_msec / 1000) << "seconds)";
            m_activation_timer.setInterval(retry_msec);
            m_activation_timer.start();

        } else {

            // Some other HTTP error was encountered.
            QLOG_ERROR() << "policy manager for" << m_policy->name()
                << "request" << m_active_request->id
                << "reply status was " << reply_status
                << "and error was" << reply->error();

        };

        m_active_request->reply = nullptr;
    };

    if (violation_detected) {
        emit Violation(m_policy->name());
    };
}

void RateLimitManager::Update(QNetworkReply* reply) {

    QLOG_TRACE() << "RateLimitManager::Update() entered";

    // Get the rate limit policy from this reply.
    QLOG_TRACE() << "RateLimitManager::Update() parsing policy";
    auto new_policy = std::make_unique<RateLimitPolicy>(reply);

    // If there was an existing policy, compare them.
    if (m_policy) {
        QLOG_TRACE() << "RateLimitManager::Update()"
            << m_policy->name() << "checking update against existing policy";
        m_policy->Check(*new_policy);
    };

    // Update the rate limit policy.
    m_policy = std::move(new_policy);

    // Grow the history capacity if needed.
    const size_t capacity = m_history.capacity();
    const size_t max_hits = m_policy->maximum_hits();
    if (capacity < max_hits) {
        QLOG_DEBUG() << m_policy->name()
            << "increasing history capacity"
            << "from" << capacity
            << "to" << max_hits;
        m_history.set_capacity(max_hits);
    };

    emit PolicyUpdated(policy());
}

// If the rate limit manager is busy, the request will be queued.
// Otherwise, the request will be sent immediately, making the
// manager busy and causing subsequent requests to be queued.
void RateLimitManager::QueueRequest(
    const QString& endpoint,
    const QNetworkRequest& network_request,
    RateLimitedReply* reply)
{
    QLOG_TRACE() << "RateLimitManager::QueueRequest() entered";
    auto request = std::make_unique<RateLimitedRequest>(endpoint, network_request, reply);
    m_queued_requests.push_back(std::move(request));
    if (m_active_request) {
        emit QueueUpdated(m_policy->name(), static_cast<int>(m_queued_requests.size()));
    } else {
        ActivateRequest();
    };
}

// Send the active request at the next time it will be safe to do so
// without violating the rate limit policy.
void RateLimitManager::ActivateRequest() {

    QLOG_TRACE() << "RateLimitManager::ActivateRequest() entered";
    if (!m_policy) {
        QLOG_ERROR() << "Cannot activate a request because the policy is null.";
        return;
    };
    if (m_active_request) {
        QLOG_DEBUG() << "Cannot activate a request because a request is already active.";
        return;
    };
    if (m_queued_requests.empty()) {
        QLOG_DEBUG() << "Cannot active a request because the queue is empty.";
        return;
    };

    m_active_request = std::move(m_queued_requests.front());
    m_queued_requests.pop_front();
    emit QueueUpdated(m_policy->name(), static_cast<int>(m_queued_requests.size()));

    const QDateTime now = QDateTime::currentDateTime();

    QDateTime next_send = m_policy->GetNextSafeSend(m_history);

    if (next_send.isValid() == false) {
        QLOG_ERROR() << "Cannot activate a request because the next send is invalid";
        return;
    };

    QLOG_TRACE() << "RateLimitManager::ActivateRequest()"
        << m_policy->name()
        << "next_send before adjustment is" << next_send.toString()
        << "(in" << now.secsTo(next_send) << "seconds)";

    if (m_policy->status() >= RateLimitPolicy::Status::BORDERLINE) {
        next_send = next_send.addMSecs(TIMING_BUCKET_MSEC);
        QLOG_DEBUG() << QString("Rate limit policy '%1' is BORDERLINE, added %2 msecs to send at %3").arg(
            m_policy->name(), QString::number(TIMING_BUCKET_MSEC), next_send.toString());
    } else {
        QLOG_TRACE() << "RateLimitManager::ActivateRequest()"
            << m_policy->name() << "is NOT borderline,"
            << "adding" << QString::number(NORMAL_BUFFER_MSEC) << "msec to next send";
        next_send = next_send.addMSecs(NORMAL_BUFFER_MSEC);
    };

    static QDateTime last_send;

    if (last_send.isValid()) {
        if (last_send.msecsTo(next_send) < MINIMUM_INTERVAL_MSEC) {
            QLOG_TRACE() << "RateLimitManager::ActivateRequest()"
                << "adding" << QString::number(MINIMUM_INTERVAL_MSEC)
                << "to next send";
            next_send = last_send.addMSecs(MINIMUM_INTERVAL_MSEC);
        };
    };

    int delay = QDateTime::currentDateTime().msecsTo(next_send);
    if (delay < 0) {
        delay = 0;
    };

    QLOG_TRACE() << "RateLimitManager::ActivateRequest()"
        << "waiting" << delay << "msecs to send request" << m_active_request->id
        << "at" << next_send.toLocalTime().toString();
    m_activation_timer.setInterval(delay);
    m_activation_timer.start();
    if (delay > 0) {
        emit Paused(m_policy->name(), next_send);
    };
}
