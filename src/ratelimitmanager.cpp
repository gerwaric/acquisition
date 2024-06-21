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

#include <QNetworkReply>

#include "QsLog.h"

#include "ratelimit.h"

// This HTTP status code means there was a rate limit violation.
constexpr int VIOLATION_STATUS = 429;

// A delay added to make sure we don't get a violation.
constexpr int NORMAL_BUFFER_MSEC = 250;
constexpr int BORDERLINE_BUFFER_MSEC = 1500;

// Minium time between sends for any given policy.
constexpr int MINIMUM_INTERVAL_MSEC = 500;

// When there is a violation, add this much time to how long we
// wait just to make sure we don't trigger another violation.
constexpr int VIOLATION_BUFFER_MSEC = 1000;

// Total number of rate-limited requests that have been created.
unsigned long RateLimitManager::RateLimitedRequest::request_count = 0;

// Create a new rate limit manager based on an existing policy.
RateLimitManager::RateLimitManager(QObject* parent) :
	QObject(parent),
	next_send_(QDateTime::currentDateTime().toLocalTime()),
	last_send_(QDateTime()),
	policy_(nullptr)
{
	// Setup the active request timer to call SendRequest each time it's done.
	activation_timer_.setSingleShot(true);
	connect(&activation_timer_, &QTimer::timeout, this, &RateLimitManager::SendRequest);
}

// Send the active request immediately.
void RateLimitManager::SendRequest() {

	if (!active_request_) {
		QLOG_DEBUG() << "RateLimit::RateLimitManager: no active request to send";
		return;
	};

	const auto& request = *active_request_;
	QLOG_TRACE() << policy_->name()
		<< "sending request" << request.id
		<< "to" << request.endpoint
		<< "via" << request.network_request.url().toString();

	// Finally, send the request and note the time.
	last_send_ = QDateTime::currentDateTime().toLocalTime();
	emit RequestReady(this, request.network_request);
};

// Called when the active request's reply is finished.
void RateLimitManager::ReceiveReply()
{
	QNetworkReply* reply = qobject_cast<QNetworkReply*>(sender());

	// Make sure the reply has a rate-limit header.
	if (!reply->hasRawHeader("X-Rate-Limit-Policy")) {
		QLOG_ERROR() << "Received a reply for" << policy_->name() << "without rate limit headers.";
		return;
	};

	const QDateTime reply_time = RateLimit::ParseDate(reply).toLocalTime();
	const int reply_status = RateLimit::ParseStatus(reply);
	QLOG_TRACE() << policy_->name()
		<< "received reply for request" << active_request_->id
		<< "with status" << reply_status;

	// Save the reply time.
	history_.push_front(reply_time);

	// Now examine the new policy and update ourselves accordingly.
	Update(reply);

	if (reply->error() == QNetworkReply::NoError) {

		if (policy_->status() >= RateLimit::PolicyStatus::VIOLATION) {
			QLOG_ERROR() << "Reply did not have an error, but the rate limit policy shows a violation occured.";
		};
		if (reply_status == VIOLATION_STATUS) {
			QLOG_ERROR() << "Reply did not have an error, but the HTTP status indicates a rate limit violation.";
		};

		auto* submit = active_request_->reply;
		active_request_ = nullptr;

		emit submit->complete(reply);

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
			next_send_ = reply_time.addMSecs(retry_msec);
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

	// Get the rate limit policy from this reply.
	auto new_policy = std::make_unique<RateLimit::Policy>(reply);

	// If there was an existing policy, compare them.
	if (policy_) {
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

	const QDateTime time = policy_->GetNextSafeSend(history_);
	if (next_send_ < time) {
		// Update this manager's send time only if it's later
		// than the manager thinks we need to wait.
		QLOG_TRACE() << "Updating next send:"
			<< "from" << next_send_.toString()
			<< "to" << time.toString();
		next_send_ = time;
	};

	emit PolicyUpdated(policy());
}

// If the rate limit manager is busy, the request will be queued.
// Otherwise, the request will be sent immediately, making the
// manager busy and causing subsequent requests to be queued.
void RateLimitManager::QueueRequest(const QString & endpoint, const QNetworkRequest network_request, RateLimit::RateLimitedReply* reply) {
	auto request = std::make_unique<RateLimitedRequest>(endpoint, network_request, reply);
	queued_requests_.push_back(std::move(request));
	if (!active_request_) {
		ActivateRequest();
	};
}

// Send the active request at the next time it will be safe to do so
// without violating the rate limit policy.
void RateLimitManager::ActivateRequest() {

	if (active_request_) {
		QLOG_DEBUG() << "RateLimit::RateLimitManager::ActivateRequest: a request is already active";
		return;
	};

	if (queued_requests_.empty()) {
		QLOG_DEBUG() << "RateLimit::RateLimitManager::ActivateReuqest: no requests queued";
		return;
	};

	active_request_ = std::move(queued_requests_.front());
	queued_requests_.pop_front();

	if (next_send_.isValid() == false) {
		QLOG_ERROR() << "RateLimit::RateLimitManager::ActivateRequest(): next_send is invalid";
	};

	QDateTime send = next_send_;

	if (policy_->status() >= RateLimit::PolicyStatus::BORDERLINE) {
		send = send.addMSecs(BORDERLINE_BUFFER_MSEC);
	} else {
		send = send.addMSecs(NORMAL_BUFFER_MSEC);
	};

	if (last_send_.isValid()) {
		if (last_send_.msecsTo(send) < MINIMUM_INTERVAL_MSEC) {
			send = last_send_.addMSecs(MINIMUM_INTERVAL_MSEC);
		};
	};
	
	int delay = QDateTime::currentDateTime().msecsTo(send);
	if (delay < 0) {
		delay = 0;
	};

	QLOG_TRACE() << policy_->name() << "waiting" << (delay / 1000)
		<< "seconds to send request" << active_request_->id
		<< "at" << next_send_.toLocalTime().toString();

	activation_timer_.setInterval(delay);
	activation_timer_.start();
}
