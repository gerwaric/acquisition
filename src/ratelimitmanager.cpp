/*
    Copyright 2023 Gerwaric

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

#include "boost/function.hpp"
#include "QsLog.h"

#include "network_info.h"
#include "oauthmanager.h"
#include "ratelimit.h"
#include "ratelimiter.h"

// This HTTP status code means there was a rate limit violation.
constexpr int VIOLATION_STATUS = 429;

// A delay added to make sure we don't get a violation.
constexpr int NORMAL_BUFFER_MSEC = 250;
constexpr int BORDERLINE_BUFFER_MSEC = 2000;

// Minium time between sends for any given policy.
constexpr int MINIMUM_INTERVAL_MSEC = 500;

// When there is a violation, add this much time to how long we
// wait just to make sure we don't trigger another violation.
constexpr int VIOLATION_BUFFER_MSEC = 2000;

// Total number of rate-limited requests that have been created.
unsigned long RateLimitedRequest::request_count = 0;

// Create a new rate limit manager based on an existing policy.
RateLimitManager::RateLimitManager(QObject* parent, SendFcn sender) :
    QObject(parent),
    sender_(sender),
    policy_(nullptr)
{
    QLOG_TRACE() << "RateLimitManager::RateLimitManager() entered";
    // Setup the active request timer to call SendRequest each time it's done.
    activation_timer_.setSingleShot(true);
    connect(&activation_timer_, &QTimer::timeout, this, &RateLimitManager::SendRequest);
}

const RateLimit::Policy& RateLimitManager::policy() {
    if (!policy_) {
        const QString message = "The rate limit manager's policy is null.";
        QLOG_FATAL() << message;
        QMessageBox::critical(nullptr,
            "Acquisition Fatal Error - Rate Limit Manager",
            message,
            QMessageBox::StandardButton::Abort,
            QMessageBox::StandardButton::Abort);
    };
    return *policy_;
}

// Send the active request immediately.
void RateLimitManager::SendRequest() {

    QLOG_TRACE() << "RateLimitManager::SendRequest() entered";
    if (!policy_) {
        QLOG_ERROR() << "The rate limit manager attempted to send a request without a policy.";
        return;
    };

    if (!active_request_) {
        QLOG_ERROR() << "The rate limit manager attempted to send a request with no request to send.";
        return;
    };

    auto& request = *active_request_;
    QLOG_TRACE() << policy_->name()
        << "sending request" << request.id
        << "to" << request.endpoint
        << "via" << request.network_request.url().toString();

    if (!sender_) {
        QLOG_ERROR() << "Rate limit manager cannot send requests.";
        return;
    };
    QNetworkReply* reply = sender_(request.network_request);
    connect(reply, &QNetworkReply::finished, this, &RateLimitManager::ReceiveReply);
};

// Called when the active request's reply is finished.
void RateLimitManager::ReceiveReply()
{
    QLOG_TRACE() << "RateLimitManager::ReceiveReply() entered";
    QNetworkReply* reply = qobject_cast<QNetworkReply*>(sender());

    if (!policy_) {
        QLOG_ERROR() << "The rate limit manager cannot recieve a reply when the policy is null.";
        return;
    };

    if (!active_request_) {
        QLOG_ERROR() << "The rate limit manager received a reply without an active request.";
        return;
    };

    // Make sure the reply has a rate-limit header.
    if (!reply->hasRawHeader("X-Rate-Limit-Policy")) {
        QLOG_ERROR() << "Received a reply for" << policy_->name() << "without rate limit headers.";
        return;
    };

    const QDateTime reply_time = RateLimit::ParseDate(reply).toLocalTime();
    const int reply_status = RateLimit::ParseStatus(reply);
    QLOG_TRACE() << "RateLimitManager::ReceiveReply()"
        << policy_->name()
        << "received reply for request" << active_request_->id
        << "with status" << reply_status;

    // Save the reply time.
    QLOG_TRACE() << "RateLimitManager::ReceiveReply()"
        << policy_->name()
        << "adding to history:" << reply_time.toString();
    history_.push_front(reply_time);

    // Now examine the new policy and update ourselves accordingly.
    Update(reply);

    if (reply->error() == QNetworkReply::NoError) {

        // Check for errors.
        if (policy_->status() >= RateLimit::PolicyStatus::VIOLATION) {
            QLOG_ERROR() << "Reply did not have an error, but the rate limit policy shows a violation occured.";
        };
        if (reply_status == VIOLATION_STATUS) {
            QLOG_ERROR() << "Reply did not have an error, but the HTTP status indicates a rate limit violation.";
        };

        // Since the request finished successfully, signal complete()
        // so anyone listening can handle the reply.
        if (active_request_->reply) {
            QLOG_TRACE() << "RateLimiteManager::ReceiveReply() about to emit 'complete' signal";
            emit active_request_->reply->complete(reply);
        } else {
            QLOG_ERROR() << "Cannot complete the rate limited request because the reply is null.";
        };

        active_request_ = nullptr;

        // Activate the next queued reqeust.
        ActivateRequest();

    } else {

        reply->deleteLater();

        if (reply_status == VIOLATION_STATUS) {
            if (!reply->hasRawHeader("Retry-After")) {
                QLOG_ERROR() << "HTTP status indicates a rate limit violation, but 'Retry-After' is missing";
            };
            if (policy_->status() != RateLimit::PolicyStatus::VIOLATION) {
                QLOG_ERROR() << "HTTP status indicates a rate limit violation, but was not flagged in the policy update";
            };
        };

        if (reply->hasRawHeader("Retry-After")) {

            // There was a rate limit violation.
            const int retry_sec = reply->rawHeader("Retry-After").toInt();
            const int retry_msec = (1000 * retry_sec) + VIOLATION_BUFFER_MSEC;
            QLOG_ERROR() << "Rate limit VIOLATION for policy"
                << policy_->name()
                << "(retrying after" << (retry_msec / 1000) << "seconds)";
            activation_timer_.setInterval(retry_msec);
            activation_timer_.start();

        } else {

            // Some other HTTP error was encountered.
            QLOG_ERROR() << "policy manager for" << policy_->name()
                << "request" << active_request_->id
                << "reply status was " << reply_status
                << "and error was" << reply->error();

        };

        active_request_->reply = nullptr;
    };
}

void RateLimitManager::Update(QNetworkReply* reply) {

    QLOG_TRACE() << "RateLimitManager::Update() entered";

    // Get the rate limit policy from this reply.
    QLOG_TRACE() << "RateLimitManager::Update() parsing policy";
    auto new_policy = std::make_unique<RateLimit::Policy>(reply);

    // If there was an existing policy, compare them.
    if (policy_) {
        QLOG_TRACE() << "RateLimitManager::Update()"
            << policy_->name() << "checking update against existing policy";
        policy_->Check(*new_policy);
    };

    // Update the rate limit policy.
    policy_ = std::move(new_policy);

    // Grow the history capacity if needed.
    const size_t capacity = history_.capacity();
    const size_t max_hits = policy_->maximum_hits();
    if (capacity < max_hits) {
        QLOG_DEBUG() << policy_->name()
            << "increasing history capacity"
            << "from" << capacity
            << "to" << max_hits;
        history_.set_capacity(max_hits);
    };

    emit PolicyUpdated(policy());
}

// If the rate limit manager is busy, the request will be queued.
// Otherwise, the request will be sent immediately, making the
// manager busy and causing subsequent requests to be queued.
void RateLimitManager::QueueRequest(
    const QString& endpoint,
    const QNetworkRequest network_request,
    RateLimit::RateLimitedReply* reply)
{
    QLOG_TRACE() << "RateLimitManager::QueueRequest() entered";
    auto request = std::make_unique<RateLimitedRequest>(endpoint, network_request, reply);
    queued_requests_.push_back(std::move(request));
    if (active_request_) {
        emit QueueUpdated(policy_->name(), queued_requests_.size());
    } else {
        ActivateRequest();
    };
}

// Send the active request at the next time it will be safe to do so
// without violating the rate limit policy.
void RateLimitManager::ActivateRequest() {

    QLOG_TRACE() << "RateLimitManager::ActivateRequest() entered";
    if (!policy_) {
        QLOG_ERROR() << "Cannot activate a request because the policy is null.";
        return;
    };
    if (active_request_) {
        QLOG_DEBUG() << "Cannot activate a request because a request is already active.";
        return;
    };
    if (queued_requests_.empty()) {
        QLOG_DEBUG() << "Cannot active a request because the queue is empty.";
        return;
    };

    active_request_ = std::move(queued_requests_.front());
    queued_requests_.pop_front();
    emit QueueUpdated(policy_->name(), queued_requests_.size());

    const QDateTime now = QDateTime::currentDateTime();

    QDateTime next_send = policy_->GetNextSafeSend(history_);

    if (next_send.isValid() == false) {
        QLOG_ERROR() << "Cannot activate a request because the next send is invalid";
        return;
    };

    QLOG_TRACE() << "RateLimitManager::ActivateRequest()"
        << policy_->name()
        << "next_send before adjustment is" << next_send.toString()
        << "(in" << now.secsTo(next_send) << "seconds)";
 
    if (policy_->status() >= RateLimit::PolicyStatus::BORDERLINE) {
        QLOG_TRACE() << "RateLimitManager::ActivateRequest()"
            << policy_->name() << "is BORDERLINE,"
            << "adding" << QString::number(BORDERLINE_BUFFER_MSEC) << "msec to next send";
        next_send = next_send.addMSecs(BORDERLINE_BUFFER_MSEC);
    } else {
        QLOG_TRACE() << "RateLimitManager::ActivateRequest()"
            << policy_->name() << "is NOT borderline,"
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
        << "waiting" << delay << "msecs to send request" << active_request_->id
        << "at" << next_send.toLocalTime().toString();
    activation_timer_.setInterval(delay);
    activation_timer_.start();
    if (delay > 0) {
        emit Paused(policy_->name(), next_send);
    };
}
