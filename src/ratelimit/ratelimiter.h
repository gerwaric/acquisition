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

#include <QNetworkRequest>
#include <QObject>
#include <QString>
#include <QTimer>

#include <list>
#include <map>
#include <memory>

#include "network_info.h"

class QNetworkAccessManager;
class QNetworkReply;

class OAuthManager;
class RateLimitedReply;
class RateLimitManager;
class RateLimitPolicy;

class RateLimiter : public QObject {
    Q_OBJECT

public:
    // Create a rate limiter.
    RateLimiter(
        QNetworkAccessManager& network_manager,
        OAuthManager& oauth_manager,
        POE_API mode);

    ~RateLimiter();

    // Submit a request-callback pair to the rate limiter. The caller is responsible
    // for freeing the RateLimitedReply object with deleteLater() when the completed()
    // signal has been emitted.
    RateLimitedReply* Submit(
        const QString& endpoint,
        QNetworkRequest network_request);

public slots:
    // Used by the GUI to request a manual refresh.
    void OnUpdateRequested();

signals:
    // Emitted when one of the policy managers has signalled a policy update.
    void PolicyUpdate(const RateLimitPolicy& policy);

    // Emitted when a request has been added to a queue.
    void QueueUpdate(const QString& policy_name, int queued_requests);

    // Signal sent to the UI so the user can see what's going on.
    void Paused(int seconds, const QString& policy_name);

private slots:

    void SendStatusUpdate();

    // Received from individual policy managers.
    void OnPolicyUpdated(const RateLimitPolicy& policy);

    // Received from individual policy managers.
    void OnQueueUpdated(const QString& policy_name, int queued_requests);

    // Received from individual policy managers.
    void OnManagerPaused(const QString& policy_name, const QDateTime& until);

    // Recieved from indivual policy managers.
    void OnViolation(const QString& policy_name);

private:

    // Process the first request for an endpoint we haven't encountered before.
    void SetupEndpoint(
        const QString& endpoint,
        QNetworkRequest network_request,
        RateLimitedReply* reply);

    // Process the first request for an endpoint we haven't encountered before.
    void ProcessHeadResponse(
        const QString& endpoint,
        QNetworkRequest network_request,
        RateLimitedReply* reply,
        QNetworkReply* network_reply);

    // Log extra details about the HEAD request and replies
    void LogSetupReply(const QNetworkRequest& request, const QNetworkReply* reply);

    // Get or create the rate limit policy manager for the given endpoint.
    RateLimitManager& GetManager(
        const QString& endpoint,
        const QString& policy_name);

    // This function is passed to individual managers via a bound
    // function so they can send network requests without having
    // to know anything about OAuth.
    QNetworkReply* SendRequest(QNetworkRequest network_request);

    // Reference to the Application's network access manager.
    QNetworkAccessManager& m_network_manager;

    // Reference to the Application's OAuth manager.
    OAuthManager& m_oauth_manager;

    POE_API m_mode;

    QTimer m_update_timer;

    std::map<QDateTime, QString> m_pauses;

    std::list<std::unique_ptr<RateLimitManager>> m_managers;
    std::map<const QString, RateLimitManager*> m_manager_by_policy;
    std::map<const QString, RateLimitManager*> m_manager_by_endpoint;

    unsigned int m_violation_count{ 0 };

};
