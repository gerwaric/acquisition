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

#pragma once

#include <QDateTime>
#include <QNetworkRequest>
#include <QObject>
#include <QTimer>

#include <deque>
#include <list>

#include "ratelimit.h"

class QNetworkReply;

class RateLimitManager : public QObject {
	Q_OBJECT

public:
	RateLimitManager(QObject* parent = nullptr);

	// Move a request into to this manager's queue.
	void QueueRequest(const QString& endpoint, const QNetworkRequest request, RateLimit::RateLimitedReply* reply);

	void Update(QNetworkReply* reply);

	const RateLimit::Policy& policy() const { return *policy_; };

	const QDateTime& next_send() const { return next_send_; };

	bool isActive() const { return (active_request_ != nullptr); };

signals:
	// Emitted when a network request is ready to go.
	void RequestReady(RateLimitManager* manager, QNetworkRequest request);

	// Emitted when the underlying policy has been updated.
	void PolicyUpdated(const RateLimit::Policy& policy);

public slots:
	// Called when a reply has been received. Checks for errors. Updates the
	// rate limit policy if one was received. Puts the response in the
	// dispatch queue for callbacks. Checks to see if another request is
	// waiting to be activated.
	void ReceiveReply();

private slots:

	// Sends the currently active request and connects it to ReceiveReply().
	void SendRequest();

private:

	struct RateLimitedRequest;

	// Called right after active_request is loaded with a new request. This
	// will determine when that request can be sent and setup the active
	// request timer to send that request after a delay.
	void ActivateRequest();

	// Resends the active request after a delay due to a violation.
	//void ResendAfterViolation();

	// Used to send requests after a delay.
	QTimer activation_timer_;

	// When a reply is recieved and the policy state has been updated or a 
	// rate violation has been detected, the next possible send time is calculated
	// and stored here.
	QDateTime next_send_;

	// Store the time of the last send for this policy, just so we can have an
	// extra check to make sure we don't flood GGG with requests.
	QDateTime last_send_;

	// Keep a unique_ptr to the policy associated with this manager,
	// which will be updated whenever a reply with the X-Rate-Limit-Policy
	// header is received.
	std::unique_ptr<RateLimit::Policy> policy_;

	// The active request
	std::unique_ptr<RateLimitedRequest> active_request_;

	// Requests that are waiting to be activated.
	std::deque<std::unique_ptr<RateLimitedRequest>> queued_requests_;

	// We use a history of the received reply times so that we can calculate
	// when the next safe send time will be. This allows us to calculate the
	// least delay necessary to stay compliant.
	//
	// A circular buffer is used because it's fast to access, and the number
	// of items we have to store only changes when a rate limit policy
	// changes, which should not happen regularly, but we handle that case, too.
	RateLimit::RequestHistory history_;
};