// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#pragma once

#include <list>
#include <map>
#include <memory>

#include <QNetworkRequest>
#include <QObject>
#include <QString>
#include <QTimer>

class QNetworkReply;

class NetworkManager;
class OAuthManager;
class RateLimitedReply;
class RateLimitManager;
class RateLimitPolicy;

class RateLimiter : public QObject
{
    Q_OBJECT

public:
    // Create a rate limiter.
    explicit RateLimiter(NetworkManager &network_manager);

    ~RateLimiter();

    // Submit a request-callback pair to the rate limiter. The caller is responsible
    // for freeing the RateLimitedReply object with deleteLater() when the completed()
    // signal has been emitted.
    RateLimitedReply *Submit(const QString &endpoint, QNetworkRequest network_request);

public slots:
    // Used by the GUI to request a manual refresh.
    void OnUpdateRequested();

signals:
    // Emitted when one of the policy managers has signalled a policy update.
    void PolicyUpdate(const RateLimitPolicy &policy);

    // Emitted when a request has been added to a queue.
    void QueueUpdate(const QString &policy_name, int queued_requests);

    // Signal sent to the UI so the user can see what's going on.
    void Paused(int seconds, const QString &policy_name);

private slots:

    void SendStatusUpdate();

    // Received from individual policy managers.
    void OnPolicyUpdated(const RateLimitPolicy &policy);

    // Received from individual policy managers.
    void OnQueueUpdated(const QString &policy_name, int queued_requests);

    // Received from individual policy managers.
    void OnManagerPaused(const QString &policy_name, const QDateTime &until);

    // Recieved from indivual policy managers.
    void OnViolation(const QString &policy_name);

private:
    // Process the first request for an endpoint we haven't encountered before.
    void SetupEndpoint(const QString &endpoint,
                       QNetworkRequest network_request,
                       RateLimitedReply *reply);

    // Process the first request for an endpoint we haven't encountered before.
    void ProcessHeadResponse(const QString &endpoint,
                             QNetworkRequest network_request,
                             RateLimitedReply *reply,
                             QNetworkReply *network_reply);

    // Log extra details about the HEAD request and replies
    void LogSetupReply(const QNetworkRequest &request, const QNetworkReply *reply);

    // Get or create the rate limit policy manager for the given endpoint.
    RateLimitManager &GetManager(const QString &endpoint, const QString &policy_name);

    // This function is passed to individual managers via a bound
    // function so they can send network requests without having
    // to know anything about OAuth.
    QNetworkReply *SendRequest(const QNetworkRequest &network_request);

    // Reference to the Application's network access manager.
    NetworkManager &m_network_manager;

    QTimer m_update_timer;

    std::map<QDateTime, QString> m_pauses;

    std::list<std::unique_ptr<RateLimitManager>> m_managers;
    std::map<const QString, RateLimitManager *> m_manager_by_policy;
    std::map<const QString, RateLimitManager *> m_manager_by_endpoint;

    unsigned int m_violation_count{0};
};
