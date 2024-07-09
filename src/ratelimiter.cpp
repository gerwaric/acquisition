/*
    Copyright 2023 Gerwaric

    This file is part of Acquisition.

    Acquisition is free software : you can redistribute it and /or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.If not, see < http://www.gnu.org/licenses/>.
*/

#include "ratelimiter.h"

#include <QMutex>
#include <QMutexLocker>
#include <QNetworkAccessManager>
#include <QNetworkReply>

#include <memory>

#include "boost/bind.hpp"
#include "boost/function.hpp"
#include "QsLog.h"

#include "ratelimit.h"
#include "ratelimitmanager.h"
#include "network_info.h"
#include "oauthmanager.h"

constexpr int UPDATE_INTERVAL_MSEC = 1000;

RateLimiter::RateLimiter(QObject* parent,
    QNetworkAccessManager& network_manager,
    OAuthManager& oauth_manager,
    POE_API mode)
    :
    QObject(parent),
    network_manager_(network_manager),
    oauth_manager_(oauth_manager),
    mode_(mode)
{
    QLOG_TRACE() << "RateLimiter::RateLimiter() entered";
    update_timer_.setSingleShot(false);
    update_timer_.setInterval(UPDATE_INTERVAL_MSEC);
    connect(&update_timer_, &QTimer::timeout, this, &RateLimiter::SendStatusUpdate);
}

RateLimit::RateLimitedReply* RateLimiter::Submit(
    const QString& endpoint,
    QNetworkRequest network_request)
{
    QLOG_TRACE() << "RateLimiter::Submit() entered";
    QLOG_TRACE() << "RateLimiter::Submit() endpoint =" << endpoint;
    QLOG_TRACE() << "RateLimiter::Submit() network_request =" << network_request.url().toString();

    // Make sure the user agent is set according to GGG's guidance.
    network_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);

    auto* reply = new RateLimit::RateLimitedReply();

    auto it = manager_by_endpoint_.find(endpoint);
    if (it != manager_by_endpoint_.end()) {

        // This endpoint is handled by an existing policy manager.
        RateLimitManager& manager = it->second;
        QLOG_DEBUG() << manager.policy().name() << "is handling" << endpoint;
        manager.QueueRequest(endpoint, network_request, reply);

    } else {

        // Use a HEAD request to determine the policy status for a new endpoint.
        if (mode_ == POE_API::OAUTH) {
            oauth_manager_.setAuthorization(network_request);
        };
        QNetworkReply* network_reply = network_manager_.head(network_request);
        connect(network_reply, &QNetworkReply::finished, this,
            [=]() {
                SetupEndpoint(endpoint, network_request, reply, network_reply);
            });

    };
    return reply;
}

void RateLimiter::SetupEndpoint(
    const QString& endpoint,
    QNetworkRequest network_request,
    RateLimit::RateLimitedReply* reply,
    QNetworkReply* network_reply)
{
    QLOG_TRACE() << "RateLimiter::SetupEndpoint() entered";
    QLOG_TRACE() << "RateLimiter::SetupEndpoint() endpoint =" << endpoint;
    QLOG_TRACE() << "RateLimiter::SetupEndpoint() network_request =" << network_request.url().toString();

    // All endpoints should be rate limited.
    if (!network_reply->hasRawHeader("X-Rate-Limit-Policy")) {
        QLOG_TRACE() << "RateLimiter:SetupEndpoint(): invalid HEAD reply without a rate limit policy";
        for (const auto& item : network_request.rawHeaderList()) {
            QLOG_TRACE() << "RateLimiter:SetupEndpoint() request header" << item << "=" << network_request.rawHeader(item);
        };
        for (const auto& pair : network_reply->rawHeaderPairs()) {
            QLOG_TRACE() << "RateLimiter::SetupEndpoint() repy header" << pair.first << "=" << pair.second;
        };
        QString url = network_reply->request().url().toString();
        QLOG_ERROR() << "The endpoint is apparently not rate-limited:" << endpoint << "(" + url + ")";
        return;
    };

    const QString policy_name = network_reply->rawHeader("X-Rate-Limit-Policy");

    RateLimitManager& manager = GetManager(endpoint, policy_name);
    manager.Update(network_reply);
    manager.QueueRequest(endpoint, network_request, reply);
    SendStatusUpdate();
}

RateLimitManager& RateLimiter::GetManager(
    const QString& endpoint,
    const QString& policy_name)
{
    QLOG_TRACE() << "RateLimiter::GetManager() entered";
    QLOG_TRACE() << "RateLimiter::GetManager() endpoint = " << endpoint;
    QLOG_TRACE() << "RateLimiter::GetManager() policy_name = " << policy_name;

    // Make sure this function is thread-safe, since it modifies out managers.
    static QMutex mutex;
    QMutexLocker locker(&mutex);

    auto it = manager_by_policy_.find(policy_name);
    if (it == manager_by_policy_.end()) {
        // Create a new policy manager.
        QLOG_DEBUG() << "Creating rate limit policy" << policy_name << "for" << endpoint;
        auto sender = boost::bind(&RateLimiter::SendRequest, this, _1);
        auto mgr = std::make_unique<RateLimitManager>(this, sender);
        auto& manager = *managers_.emplace_back(std::move(mgr));
        connect(&manager, &RateLimitManager::PolicyUpdated, this, &RateLimiter::OnPolicyUpdated);
        manager_by_policy_.emplace(policy_name, manager);
        manager_by_endpoint_.emplace(endpoint, manager);
        return manager;
    } else {
        // Use an existing policy manager.
        QLOG_DEBUG() << "Using an existing rate limit policy" << policy_name << "for" << endpoint;
        RateLimitManager& manager = it->second;
        manager_by_endpoint_.emplace(endpoint, manager);
        return manager;
    };
}

QNetworkReply* RateLimiter::SendRequest(QNetworkRequest request) {
    if (mode_ == POE_API::OAUTH) {
        oauth_manager_.setAuthorization(request);
    };
    return network_manager_.get(request);
}

void RateLimiter::OnUpdateRequested()
{
    QLOG_TRACE() << "RateLimiter::OnUpdateRequested() entered";
    for (const auto& manager : managers_) {
        emit PolicyUpdate(manager->policy());
    };
    SendStatusUpdate();
}

void RateLimiter::OnPolicyUpdated(const RateLimit::Policy& policy)
{
    QLOG_TRACE() << "RateLimiter::OnPolicyUpdated() entered";
    emit PolicyUpdate(policy);
    SendStatusUpdate();
}

void RateLimiter::SendStatusUpdate()
{
    QLOG_TRACE() << "RateLimiter::SendStatusUpdate() entered";
    QDateTime next_send;
    QString limiting_policy;
    for (auto& manager : managers_) {
        if (!manager) {
            QLOG_ERROR() << "Cannot send a status update: the rate limit manager is invalid.";
            continue;
        };
        if (!manager->isActive()) {
            continue;
        };
        // need to handle if there is one pause policy and one not pause.
        if (next_send.isValid() && (next_send <= manager->next_send())) {
            continue;
        };
        next_send = manager->next_send();
        limiting_policy = manager->policy().name();
    };

    int pause;
    if (!next_send.isValid()) {
        pause = 0;
    } else {
        pause = QDateTime::currentDateTime().secsTo(next_send);
        if (pause < 0) {
            pause = 0;
        };
    };

    if (pause > 0) {
        if (!update_timer_.isActive()) {
            QLOG_TRACE() << "RateLimiter::SendStatusUpdate() starting status updates (" << limiting_policy << ")";
            update_timer_.start();
        };
    } else {
        if (update_timer_.isActive()) {
            QLOG_TRACE() << "RateLimiter::SendStatusUpdate() stopping status updates";
            update_timer_.stop();
        };
    };

    QLOG_TRACE() << "RateLimiter::SendStatusUpdate() rateLimiter is PAUSED" << pause << "for" << limiting_policy;
    emit Paused(pause, limiting_policy);
}
