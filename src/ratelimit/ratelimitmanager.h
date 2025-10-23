/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include <deque>

#include <QDateTime>
#include <QNetworkRequest>
#include <QObject>
#include <QString>
#include <QTimer>

#include "network_info.h"
#include "ratelimit.h"

class QNetworkAccessManager;
class QNetworkReply;

class RateLimitedReply;
struct RateLimitedRequest;
class RateLimitPolicy;

// Manages a single rate limit policy, which may apply to multiple endpoints.
class RateLimitManager : public QObject
{
    Q_OBJECT

public:
    // This is the signature of the function used to send requests.
    using SendFcn = std::function<QNetworkReply *(QNetworkRequest &)>;

    RateLimitManager(SendFcn sender);
    ~RateLimitManager();

    // Move a request into to this manager's queue.
    void QueueRequest(const QString &endpoint,
                      const QNetworkRequest &request,
                      RateLimitedReply *reply);

    void Update(QNetworkReply *reply);

    const RateLimitPolicy &policy();

    int msecToNextSend() const { return m_activation_timer.remainingTime(); };

signals:
    // Emitted when a network request is ready to go.
    void RequestReady(RateLimitManager *manager, QNetworkRequest request, POE_API mode);

    // Emitted when the underlying policy has been updated.
    void PolicyUpdated(const RateLimitPolicy &policy);

    // Emitted when a request has been added to the queue;
    void QueueUpdated(const QString policy_name, int queued_requests);

    // Emitted when a network request has to wait to be sent.
    void Paused(const QString &policy_name, const QDateTime &until);

    // Emitted when a rate limit violation has been detected.
    void Violation(const QString &policy_name);

public slots:

    // Called whent the timer runs out to sends the active request and
    // connects the network reply it to ReceiveReply().
    void SendRequest();

    // Called when a reply has been received. Checks for errors. Updates the
    // rate limit policy if one was received. Puts the response in the
    // dispatch queue for callbacks. Checks to see if another request is
    // waiting to be activated.
    void ReceiveReply();

private:
    // Function handle used to send network reqeusts.
    const SendFcn m_sender;

    // Used to print log messages about rate limit violations.
    void LogPolicyHistory();

    // Called right after active_request is loaded with a new request. This
    // will determine when that request can be sent and setup the active
    // request timer to send that request after a delay.
    void ActivateRequest();

    // Used to send requests after a delay.
    QTimer m_activation_timer;

    // Keep a unique_ptr to the policy associated with this manager,
    // which will be updated whenever a reply with the X-Rate-Limit-Policy
    // header is received.
    std::unique_ptr<RateLimitPolicy> m_policy;

    // The active request
    std::unique_ptr<RateLimitedRequest> m_active_request;

    // Requests that are waiting to be activated.
    std::deque<std::unique_ptr<RateLimitedRequest>> m_queued_requests;

    // We use a history of the received reply times so that we can calculate
    // when the next safe send time will be. This allows us to calculate the
    // least delay necessary to stay compliant.
    //
    // A circular buffer is used because it's fast to access, and the number
    // of items we have to store only changes when a rate limit policy
    // changes, which should not happen regularly, but we handle that case, too.
    std::deque<RateLimit::Event> m_history;
    unsigned m_history_size{0};
};
