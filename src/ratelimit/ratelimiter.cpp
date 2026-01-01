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

#include "ratelimiter.h"

#include <QEventLoop>
#include <QNetworkReply>

#include "ratelimitedreply.h"
#include "ratelimitmanager.h"
#include "ratelimitpolicy.h"
#include "util/fatalerror.h"
#include "util/networkmanager.h"
#include "util/oauthmanager.h"
#include "util/spdlog_qt.h"

static_assert(ACQUISITION_USE_SPDLOG);

constexpr int UPDATE_INTERVAL_MSEC = 1000;

RateLimiter::RateLimiter(NetworkManager &network_manager)
    : m_network_manager(network_manager)
{
    spdlog::trace("RateLimiter::RateLimiter() entered");
    m_update_timer.setSingleShot(false);
    m_update_timer.setInterval(UPDATE_INTERVAL_MSEC);
    connect(&m_update_timer, &QTimer::timeout, this, &RateLimiter::SendStatusUpdate);
}

RateLimiter::~RateLimiter() {}

RateLimitedReply *RateLimiter::Submit(const QString &endpoint, QNetworkRequest network_request)
{
    // Create a new rate limited reply that we can return to the calling function.
    auto *reply = new RateLimitedReply();

    // Look for a rate limit manager for this endpoint.
    auto it = m_manager_by_endpoint.find(endpoint);
    if (it != m_manager_by_endpoint.end()) {
        // This endpoint is handled by an existing policy manager.
        RateLimitManager &manager = *it->second;
        spdlog::trace("Rate Limit policy {} is handling '{}': {}",
                      manager.policy().name(),
                      endpoint,
                      network_request.url().toString());
        manager.QueueRequest(endpoint, network_request, reply);

    } else {
        // This is a new endpoint, so it's possible we need a new policy
        // manager, or that this endpoint should be managed by another
        // manager that has already been created, because the same rate limit
        // policy can apply to multiple managers.
        spdlog::debug("New endpoint encountered: '{}': {}",
                      endpoint,
                      network_request.url().toString());
        SetupEndpoint(endpoint, network_request, reply);
    }
    return reply;
}

void RateLimiter::SetupEndpoint(const QString &endpoint,
                                QNetworkRequest network_request,
                                RateLimitedReply *reply)
{
    spdlog::trace("RateLimiter::SetupEndpoint() entered");

    // Use a HEAD request to determine the policy status for a new endpoint.
    spdlog::debug("Sending a HEAD for endpoint: {}", endpoint);

    // Make the head request.
    spdlog::trace("RateLimiter::SetupEndpoint() sending a HEAD request for {}", endpoint);
    QNetworkReply *network_reply = m_network_manager.head(network_request);

    // Cause a fatal error if there was a network error.
    connect(network_reply, &QNetworkReply::errorOccurred, this, [=]() {
        const auto error_code = network_reply->error();
        if ((error_code >= 200) && (error_code <= 299)) {
            spdlog::debug("RateLimit::SetupEndpoint() HEAD reply status is {}", error_code);
            return;
        }
        const QString error_value = QString::number(error_code);
        const QString error_string = network_reply->errorString();
        spdlog::error("RateLimiter::SetupEndpoint() network error in HEAD reply for {}", endpoint);
        FatalError(QString("Network error %1 in HEAD reply for '%2': %3")
                       .arg(error_value, endpoint, error_string));
    });

    // Cause a fatal error if there were any SSL errors.
    connect(network_reply, &QNetworkReply::sslErrors, this, [=](const QList<QSslError> &errors) {
        spdlog::error("RateLimiter::SetupEndpoint() SSL error in HEAD reply for endpoint: {}",
                      endpoint);
        QStringList messages;
        for (const auto &error : errors) {
            messages.append(error.errorString());
        }
        FatalError(
            QString("SSL error(s) in HEAD reply for '%1': %2").arg(endpoint, messages.join(", ")));
    });

    // WARNING: it is important to wait for this head request to finish before proceeding,
    // because otherwise acquisition may end up flooding the network with a series of HEAD
    // requests, which has gotten users blocked before by Cloudflare, which is a problem
    // GGG may not have control over.
    //
    // Another solution to this problem would be to allow requests to queue here instead,
    // but that would be a lot more complex.
    QEventLoop loop;
    connect(network_reply, &QNetworkReply::finished, &loop, &QEventLoop::quit);
    loop.exec();

    spdlog::trace("RateLimiter::SetupEndpoint() received a HEAD reply for {}", endpoint);
    ProcessHeadResponse(endpoint, network_request, reply, network_reply);
}

void RateLimiter::ProcessHeadResponse(const QString &endpoint,
                                      QNetworkRequest network_request,
                                      RateLimitedReply *reply,
                                      QNetworkReply *network_reply)
{
    spdlog::trace("RateLimiter::ProcessHeadResponse() endpoint='{}', url='{}'",
                  endpoint,
                  network_request.url().toString());

    // Make sure the network reply is a valid pointer before using it.
    if (network_reply == nullptr) {
        spdlog::error("The HEAD reply was null.");
        FatalError(QString("The HEAD reply was null"));
    }

    // Check for network errors.
    const auto error_code = network_reply->error();
    if (error_code != QNetworkReply::NoError) {
        if ((error_code >= 200) && (error_code <= 299)) {
            spdlog::debug("The HEAD reply has status {}", error_code);
        } else {
            spdlog::error("The HEAD reply had a network error.");
            LogSetupReply(network_request, network_reply);
            FatalError(QString("Network error %1 in HEAD reply for '%2': %3")
                           .arg(QString::number(network_reply->error()),
                                endpoint,
                                network_reply->errorString()));
        }
    }

    // Check for other HTTP errors.
    const int response_code = RateLimit::ParseStatus(network_reply);
    const bool response_failed = (response_code < 200) || (response_code > 299);
    if (response_failed) {
        spdlog::error("The HEAD request failed");
        LogSetupReply(network_request, network_reply);
        FatalError(QString("HTTP status %1 in HEAD reply for '%2'")
                       .arg(QString::number(response_code), endpoint));
    }

    // All endpoints should be rate limited.
    if (!network_reply->hasRawHeader("X-Rate-Limit-Policy")) {
        spdlog::error("The HEAD response did not contain a rate limit policy for endpoint: {}",
                      endpoint);
        LogSetupReply(network_request, network_reply);
        FatalError(
            QString("he HEAD response did not contain a rate limit policy for endpoint: '%1'")
                .arg(endpoint));
    }

    // Extract the policy name.
    const QString policy_name = network_reply->rawHeader("X-Rate-Limit-Policy");

    // Log the headers.
    QStringList lines;
    lines.reserve(network_reply->rawHeaderList().size() + 2);
    lines.append(QString("<HEAD_RESPONSE_HEADERS policy_name='%1'>").arg(policy_name));
    const auto raw_headers = network_reply->rawHeaderList();
    for (const auto &name : raw_headers) {
        if (QString::fromUtf8(name).startsWith("X-Rate-Limit", Qt::CaseInsensitive)) {
            lines.append(QString("%1 = '%2'").arg(name, network_reply->rawHeader(name)));
        }
    }
    lines.append("</HEAD_RESPONSE_HEADERS>");
    spdlog::debug("HEAD response received for {}:\n{}", policy_name, lines.join("\n"));

    // Create the rate limit manager.
    RateLimitManager &manager = GetManager(endpoint, policy_name);

    // Update the policy manager and queue the request.
    manager.Update(network_reply);
    manager.QueueRequest(endpoint, network_request, reply);

    // Emit a status update for anyone listening.
    SendStatusUpdate();
}

void RateLimiter::LogSetupReply(const QNetworkRequest &request, const QNetworkReply *reply)
{
    NetworkManager::logRequest(request);
    NetworkManager::logReply(reply);
}

RateLimitManager &RateLimiter::GetManager(const QString &endpoint, const QString &policy_name)
{
    spdlog::trace("RateLimiter::GetManager() entered");
    spdlog::trace("RateLimiter::GetManager() endpoint = {}", endpoint);
    spdlog::trace("RateLimiter::GetManager() policy_name = {}", policy_name);

    auto it = m_manager_by_policy.find(policy_name);
    if (it == m_manager_by_policy.end()) {
        // Create a new policy manager.
        spdlog::debug("Creating rate limit policy {} for {}", policy_name, endpoint);
        auto sender = std::bind_front(&RateLimiter::SendRequest, this);
        auto mgr = std::make_unique<RateLimitManager>(sender);
        auto &manager = m_managers.emplace_back(std::move(mgr));
        connect(manager.get(),
                &RateLimitManager::PolicyUpdated,
                this,
                &RateLimiter::OnPolicyUpdated);
        connect(manager.get(), &RateLimitManager::QueueUpdated, this, &RateLimiter::OnQueueUpdated);
        connect(manager.get(), &RateLimitManager::Paused, this, &RateLimiter::OnManagerPaused);
        connect(manager.get(), &RateLimitManager::Violation, this, &RateLimiter::OnViolation);
        m_manager_by_policy[policy_name] = manager.get();
        m_manager_by_endpoint[endpoint] = manager.get();
        return *manager;
    } else {
        // Use an existing policy manager.
        spdlog::debug("Using an existing rate limit policy {} for {}", policy_name, endpoint);
        RateLimitManager *manager = it->second;
        m_manager_by_endpoint[endpoint] = manager;
        return *manager;
    }
}

QNetworkReply *RateLimiter::SendRequest(const QNetworkRequest &request)
{
    return m_network_manager.get(request);
}

void RateLimiter::OnUpdateRequested()
{
    spdlog::trace("RateLimiter::OnUpdateRequested() entered");
    for (const auto &manager : m_managers) {
        emit PolicyUpdate(manager->policy());
    }
}

void RateLimiter::OnPolicyUpdated(const RateLimitPolicy &policy)
{
    spdlog::trace("RateLimiter::OnPolicyUpdated() entered");
    emit PolicyUpdate(policy);
}

void RateLimiter::OnQueueUpdated(const QString &policy_name, int queued_requests)
{
    emit QueueUpdate(policy_name, queued_requests);
}

void RateLimiter::OnManagerPaused(const QString &policy_name, const QDateTime &until)
{
    spdlog::trace("RateLimiter::OnManagerPaused() pausing until {} for {}",
                  until.toString(),
                  policy_name);
    m_pauses[until] = policy_name;
    m_update_timer.start();
}

void RateLimiter::OnViolation(const QString &policy_name)
{
    ++m_violation_count;
    spdlog::error(
        "RateLimiter: {} was violated. So far {} rate limit violations have been detected.",
        policy_name,
        m_violation_count);
}

void RateLimiter::SendStatusUpdate()
{
    // Get rid of any pauses that finished in the past.
    const QDateTime now = QDateTime::currentDateTime();
    while (!m_pauses.empty() && (m_pauses.begin()->first < now)) {
        m_pauses.erase(m_pauses.begin());
    }

    if (m_pauses.empty()) {
        spdlog::trace("RateLimiter::SendStatusUpdate() stopping status updates");
        m_update_timer.stop();
    } else {
        const auto &pause = *m_pauses.begin();
        const QDateTime &pause_end = pause.first;
        const QString policy_name = pause.second;
        emit Paused(now.secsTo(pause_end), policy_name);
    }
}
