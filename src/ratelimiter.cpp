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

#include "ratelimiter.h"

#include <QEventLoop>
#include <QNetworkAccessManager>
#include <QNetworkReply>

#include <array>
#include <memory>

#include "boost/bind/bind.hpp"
#include "QsLog.h"

#include "fatalerror.h"
#include "network_info.h"
#include "oauthmanager.h"
#include "ratelimitmanager.h"

constexpr int UPDATE_INTERVAL_MSEC = 1000;

// Create a list of all the attributes a QNetworkRequest or QNetwork reply can have,
// since there is no other way to iterate over this list:
// https://doc.qt.io/qt-6/qnetworkrequest.html#Attribute-enum (as of July 29, 2004)
constexpr std::array ATTRIBUTES = {
    std::make_pair(QNetworkRequest::HttpStatusCodeAttribute, "HttpStatusCodeAttribute"), //	0	Replies only, type: QMetaType::Int(no default) Indicates the HTTP status code received from the HTTP server(like 200, 304, 404, 401, etc.).If the connection was not HTTP - based, this attribute will not be present.
    std::make_pair(QNetworkRequest::HttpReasonPhraseAttribute, "HttpReasonPhraseAttribute"), //	1	Replies only, type : QMetaType::QByteArray(no default) Indicates the HTTP reason phrase as received from the HTTP server(like "Ok", "Found", "Not Found", "Access Denied", etc.) This is the human - readable representation of the status code(see above).If the connection was not HTTP - based, this attribute will not be present.
    std::make_pair(QNetworkRequest::RedirectionTargetAttribute, "RedirectionTargetAttribute"), //	2	Replies only, type : QMetaType::QUrl(no default) If present, it indicates that the server is redirecting the request to a different URL.The Network Access API does follow redirections by default, unless QNetworkRequest::ManualRedirectPolicy is used.Additionally, if QNetworkRequest::UserVerifiedRedirectPolicy is used, then this attribute will be set if the redirect was not followed.The returned URL might be relative.Use QUrl::resolved() to create an absolute URL out of it.
    std::make_pair(QNetworkRequest::ConnectionEncryptedAttribute, "ConnectionEncryptedAttribute"), //	3	Replies only, type : QMetaType::Bool(default: false) Indicates whether the data was obtained through an encrypted(secure) connection.
    std::make_pair(QNetworkRequest::CacheLoadControlAttribute, "CacheLoadControlAttribute"), //	4	Requests only, type : QMetaType::Int(default: QNetworkRequest::PreferNetwork) Controls how the cache should be accessed.The possible values are those of QNetworkRequest::CacheLoadControl.Note that the default QNetworkAccessManager implementation does not support caching.However, this attribute may be used by certain backends to modify their requests(for example, for caching proxies).
    std::make_pair(QNetworkRequest::CacheSaveControlAttribute, "CacheSaveControlAttribute"), //	5	Requests only, type : QMetaType::Bool(default: true) Controls if the data obtained should be saved to cache for future uses.If the value is false, the data obtained will not be automatically cached.If true, data may be cached, provided it is cacheable(what is cacheable depends on the protocol being used).
    std::make_pair(QNetworkRequest::SourceIsFromCacheAttribute, "SourceIsFromCacheAttribute"), //	6	Replies only, type : QMetaType::Bool(default: false) Indicates whether the data was obtained from cache or not.
    std::make_pair(QNetworkRequest::DoNotBufferUploadDataAttribute, "DoNotBufferUploadDataAttribute"), //	7	Requests only, type : QMetaType::Bool(default: false) Indicates whether the QNetworkAccessManager code is allowed to buffer the upload data, e.g.when doing a HTTP POST.When using this flag with sequential upload data, the ContentLengthHeader header must be set.
    std::make_pair(QNetworkRequest::HttpPipeliningAllowedAttribute, "HttpPipeliningAllowedAttribute"), //	8	Requests only, type : QMetaType::Bool(default: false) Indicates whether the QNetworkAccessManager code is allowed to use HTTP pipelining with this request.
    std::make_pair(QNetworkRequest::HttpPipeliningWasUsedAttribute, "HttpPipeliningWasUsedAttribute"), //	9	Replies only, type : QMetaType::Bool Indicates whether the HTTP pipelining was used for receiving this reply.
    std::make_pair(QNetworkRequest::CustomVerbAttribute, "CustomVerbAttribute"), //	10	Requests only, type : QMetaType::QByteArray Holds the value for the custom HTTP verb to send(destined for usage of other verbs than GET, POST, PUT and DELETE).This verb is set when calling QNetworkAccessManager::sendCustomRequest().
    std::make_pair(QNetworkRequest::CookieLoadControlAttribute, "CookieLoadControlAttribute"), //	11	Requests only, type : QMetaType::Int(default: QNetworkRequest::Automatic) Indicates whether to send 'Cookie' headers in the request.This attribute is set to false by Qt WebKit when creating a cross - origin XMLHttpRequest where withCredentials has not been set explicitly to true by the Javascript that created the request.See here for more information. (This value was introduced in 4.7.)
    std::make_pair(QNetworkRequest::CookieSaveControlAttribute, "CookieSaveControlAttribute"), //	13	Requests only, type : QMetaType::Int(default: QNetworkRequest::Automatic) Indicates whether to save 'Cookie' headers received from the server in reply to the request.This attribute is set to false by Qt WebKit when creating a cross - origin XMLHttpRequest where withCredentials has not been set explicitly to true by the Javascript that created the request.See here for more information. (This value was introduced in 4.7.)
    std::make_pair(QNetworkRequest::AuthenticationReuseAttribute, "AuthenticationReuseAttribute"), //	12	Requests only, type : QMetaType::Int(default: QNetworkRequest::Automatic) Indicates whether to use cached authorization credentials in the request, if available.If this is set to QNetworkRequest::Manual and the authentication mechanism is 'Basic' or 'Digest', Qt will not send an 'Authorization' HTTP header with any cached credentials it may have for the request's URL. This attribute is set to QNetworkRequest::Manual by Qt WebKit when creating a cross-origin XMLHttpRequest where withCredentials has not been set explicitly to true by the Javascript that created the request. See here for more information. (This value was introduced in 4.7.)
    std::make_pair(QNetworkRequest::BackgroundRequestAttribute, "BackgroundRequestAttribute"), //	17	Type : QMetaType::Bool(default: false) Indicates that this is a background transfer, rather than a user initiated transfer.Depending on the platform, background transfers may be subject to different policies.
    std::make_pair(QNetworkRequest::Http2AllowedAttribute, "Http2AllowedAttribute"), //	19	Requests only, type : QMetaType::Bool(default: true) Indicates whether the QNetworkAccessManager code is allowed to use HTTP / 2 with this request.This applies to SSL requests or 'cleartext' HTTP / 2 if Http2CleartextAllowedAttribute is set.
    std::make_pair(QNetworkRequest::Http2WasUsedAttribute, "Http2WasUsedAttribute"), //	20	Replies only, type : QMetaType::Bool(default: false) Indicates whether HTTP / 2 was used for receiving this reply. (This value was introduced in 5.9.)
    std::make_pair(QNetworkRequest::EmitAllUploadProgressSignalsAttribute, "EmitAllUploadProgressSignalsAttribute"), //	18	Requests only, type : QMetaType::Bool(default: false) Indicates whether all upload signals should be emitted.By default, the uploadProgress signal is emitted only in 100 millisecond intervals. (This value was introduced in 5.5.)
    std::make_pair(QNetworkRequest::OriginalContentLengthAttribute, "OriginalContentLengthAttribute"), //	21	Replies only, type QMetaType::Int Holds the original content - length attribute before being invalidated and removed from the header when the data is compressed and the request was marked to be decompressed automatically. (This value was introduced in 5.9.)
    std::make_pair(QNetworkRequest::RedirectPolicyAttribute, "RedirectPolicyAttribute"), //	22	Requests only, type : QMetaType::Int, should be one of the QNetworkRequest::RedirectPolicy values(default: NoLessSafeRedirectPolicy). (This value was introduced in 5.9.)
    std::make_pair(QNetworkRequest::Http2DirectAttribute, "Http2DirectAttribute"), //	23	Requests only, type : QMetaType::Bool(default: false) If set, this attribute will force QNetworkAccessManager to use HTTP / 2 protocol without initial HTTP / 2 protocol negotiation.Use of this attribute implies prior knowledge that a particular server supports HTTP / 2. The attribute works with SSL or with 'cleartext' HTTP / 2 if Http2CleartextAllowedAttribute is set.If a server turns out to not support HTTP / 2, when HTTP / 2 direct was specified, QNetworkAccessManager gives up, without attempting to fall back to HTTP / 1.1.If both Http2AllowedAttribute and Http2DirectAttribute are set, Http2DirectAttribute takes priority. (This value was introduced in 5.11.)
    std::make_pair(QNetworkRequest::AutoDeleteReplyOnFinishAttribute, "AutoDeleteReplyOnFinishAttribute"), //	25	Requests only, type : QMetaType::Bool(default: false) If set, this attribute will make QNetworkAccessManager delete the QNetworkReply after having emitted "finished". (This value was introduced in 5.14.)
    std::make_pair(QNetworkRequest::ConnectionCacheExpiryTimeoutSecondsAttribute, "ConnectionCacheExpiryTimeoutSecondsAttribute"), //	26	Requests only, type : QMetaType::Int To set when the TCP connections to a server(HTTP1 and HTTP2) should be closed after the last pending request had been processed. (This value was introduced in 6.3.)
    std::make_pair(QNetworkRequest::Http2CleartextAllowedAttribute, "Http2CleartextAllowedAttribute"), //	27	Requests only, type : QMetaType::Bool(default: false) If set, this attribute will tell QNetworkAccessManager to attempt an upgrade to HTTP / 2 over cleartext(also known as h2c).Until Qt 7 the default value for this attribute can be overridden to true by setting the QT_NETWORK_H2C_ALLOWED environment variable.This attribute is ignored if the Http2AllowedAttribute is not set. (This value was introduced in 6.3.)
    std::make_pair(QNetworkRequest::UseCredentialsAttribute, "UseCredentialsAttribute"), //	28	Requests only, type : QMetaType::Bool(default: false) Indicates if the underlying XMLHttpRequest cross - site Access - Control requests should be made using credentials.Has no effect on same - origin requests.This only affects the WebAssembly platform. (This value was introduced in 6.5.)
    std::make_pair(QNetworkRequest::User, "User"), //	1000	Special type.Additional information can be passed in QVariants with types ranging from User to UserMax.The default implementation of Network Access will ignore any request attributes in this range and it will not produce any attributes in this range in replies.The range is reserved for extensions of QNetworkAccessManager.
    std::make_pair(QNetworkRequest::UserMax, "UserMax"), //	32767
};

RateLimiter::RateLimiter(
    QNetworkAccessManager& network_manager,
    OAuthManager& oauth_manager,
    POE_API mode)
    : network_manager_(network_manager)
    , oauth_manager_(oauth_manager)
    , mode_(mode)
{
    QLOG_TRACE() << "RateLimiter::RateLimiter() entered";
    update_timer_.setSingleShot(false);
    update_timer_.setInterval(UPDATE_INTERVAL_MSEC);
    connect(&update_timer_, &QTimer::timeout, this, &RateLimiter::SendStatusUpdate);
}

RateLimiter::~RateLimiter() {}

RateLimit::RateLimitedReply* RateLimiter::Submit(
    const QString& endpoint,
    QNetworkRequest network_request)
{
    QLOG_TRACE() << "RateLimiter::Submit() entered";
    QLOG_TRACE() << "RateLimiter::Submit() endpoint =" << endpoint;
    QLOG_TRACE() << "RateLimiter::Submit() network_request =" << network_request.url().toString();

    // Make sure the user agent is set according to GGG's guidance.
    network_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);

    // Create a new rate limited reply that we can return to the calling function.
    auto* reply = new RateLimit::RateLimitedReply();

    // Look for a rate limit manager for this endpoint.
    auto it = manager_by_endpoint_.find(endpoint);
    if (it != manager_by_endpoint_.end()) {

        // This endpoint is handled by an existing policy manager.
        RateLimitManager& manager = it->second;
        QLOG_DEBUG() << manager.policy().name() << "is handling" << endpoint;
        manager.QueueRequest(endpoint, network_request, reply);

    } else {

        // This is a new endpoint, so it's possible we need a new policy
        // manager, or that this endpoint should be managed by another
        // manager that has already been created, because the same rate limit
        // policy can apply to multiple managers.
        QLOG_DEBUG() << "Unknown endpoint encountered:" << endpoint;
        SetupEndpoint(endpoint, network_request, reply);

    };
    return reply;
}

void RateLimiter::SetupEndpoint(
    const QString& endpoint,
    QNetworkRequest network_request,
    RateLimit::RateLimitedReply* reply)
{
    QLOG_TRACE() << "RateLimiter::SetupEndpoint() entered";

    // Use a HEAD request to determine the policy status for a new endpoint.
    QLOG_DEBUG() << "Sending a HEAD for endpoint:" << endpoint;

    // Make sure the network request get an OAuth bearer token if necessary.
    if (mode_ == POE_API::OAUTH) {
        QLOG_TRACE() << "RateLimiter::SetupEndpoint() calling setAuthorization()";
        oauth_manager_.setAuthorization(network_request);
    };

    // Make the head request.
    QLOG_TRACE() << "RateLimiter::SetupEndpoint() sending a HEAD request for" << endpoint;
    QNetworkReply* network_reply = network_manager_.head(network_request);

    // Cause a fatal error if there was a network error.
    connect(network_reply, &QNetworkReply::errorOccurred, this,
        [=]() {
            QLOG_ERROR() << "RateLimiter::SetupEndpoint() network error in HEAD reply for" << endpoint;
            FatalError(QString("Network error %1 in HEAD reply for '%2': %3").arg(
                QString::number(network_reply->error()),
                endpoint,
                network_reply->errorString()));
        });

    // Cause a fatal error if there were any SSL errors.
    connect(network_reply, &QNetworkReply::sslErrors, this,
        [=](const QList<QSslError>& errors) {
            QLOG_ERROR() << "RateLimiter::SetupEndpoint() SSL error in HEAD reply for endpoint:" << endpoint;
            QStringList messages;
            for (const auto& error : errors) {
                messages.append(error.errorString());
            };
            FatalError(QString("SSL error(s) in HEAD reply for '%1': %2").arg(
                endpoint,
                messages.join(", ")));
        });

    // WARNING: it is important to wait for this head request to finish before proceeding,
    // because otherwise acquisiton may end up flooding the network with a series of HEAD
    // requests, which has gotten users blocked before by Cloudflare, which is a problem
    // GGG may not have control over.
    //
    // Another solution to this problem would be to allow requests to queue here instead,
    // but that would be a lot more complex.
    QEventLoop loop;
    connect(network_reply, &QNetworkReply::finished, &loop, &QEventLoop::quit);
    loop.exec();

    QLOG_TRACE() << "RateLimiter::SetupEndpoint() received a HEAD reply for" << endpoint;
    ProcessHeadResponse(endpoint, network_request, reply, network_reply);
}

void RateLimiter::ProcessHeadResponse(
    const QString& endpoint,
    QNetworkRequest network_request,
    RateLimit::RateLimitedReply* reply,
    QNetworkReply* network_reply)
{
    QLOG_TRACE() << "RateLimiter::ProcessHeadResponse() entered";
    QLOG_TRACE() << "RateLimiter::ProcessHeadResponse() endpoint =" << endpoint;
    QLOG_TRACE() << "RateLimiter::ProcessHeadResponse() url =" << network_request.url().toString();

    // Make sure the network reply is a valid pointer before using it.
    if (network_reply == nullptr) {
        QLOG_ERROR() << "The HEAD reply was null";
        FatalError(QString("The HEAD reply was null"));
    };

    // Check for network errors.
    if (network_reply->error() != QNetworkReply::NoError) {
        QLOG_ERROR() << "The HEAD reply had a network error";
        LogSetupReply(network_request, network_reply);
        FatalError(QString("Network error %1 in HEAD reply for '%2': %3").arg(
            QString::number(network_reply->error()),
            endpoint,
            network_reply->errorString()));
    };

    // Check for other HTTP errors.
    const int response_code = RateLimit::ParseStatus(network_reply);
    const bool response_failed = (response_code < 200) || (response_code > 299);
    if (response_failed) {
        QLOG_ERROR() << "The HEAD request failed";
        LogSetupReply(network_request, network_reply);
        FatalError(QString("HTTP status %1 in HEAD reply for '%2'").arg(
            QString::number(response_code),
            endpoint));
    };

    // All endpoints should be rate limited.
    if (!network_reply->hasRawHeader("X-Rate-Limit-Policy")) {
        QLOG_ERROR() << "The HEAD response did not contain a rate limit policy for endpoint:" << endpoint;
        LogSetupReply(network_request, network_reply);
        FatalError(QString("he HEAD response did not contain a rate limit policy for endpoint: '%1'").arg(endpoint));
    };

    // Get or create the manager for this policy.
    const QString policy_name = network_reply->rawHeader("X-Rate-Limit-Policy");
    RateLimitManager& manager = GetManager(endpoint, policy_name);

    // Update the policy manager and queue the request.
    manager.Update(network_reply);
    manager.QueueRequest(endpoint, network_request, reply);

    // Emit a status update for anyone listening.
    SendStatusUpdate();
}

void RateLimiter::LogSetupReply(const QNetworkRequest& request, const QNetworkReply* reply) {

    // Log the request headers.
    for (const auto& name : request.rawHeaderList()) {
        const bool is_authorization = (0 == name.compare("Authorization", Qt::CaseInsensitive));
        QByteArray value = request.rawHeader(name);
        if (is_authorization) {
            // Mask the OAuth bearer token so it's not written to the log.
            value.fill('*');
            value += " (The OAuth token has been masked for security)";
        };
        QLOG_INFO() << "RateLimiter::SetupEndpoint() HEAD request header" << name << "=" << value;
    };

    // Log the request attributes.
    for (const auto& pair : ATTRIBUTES) {
        const QNetworkRequest::Attribute& code = pair.first;
        const char* name = pair.second;
        const QVariant value = request.attribute(code);
        if (value.isValid()) {
            QLOG_INFO() << "RateLimiter::SetupEndpoint() HEAD request attribute" << name << "=" << value.toString();
        };
    };

    // Log the reply headers.
    for (const auto& header : reply->rawHeaderPairs()) {
        const auto& name = header.first;
        const auto& value = header.second;
        QLOG_INFO() << "RateLimiter::SetupEndpoint() HEAD reply header" << name << "=" << value;
    };

    // Log the reply attributes.
    for (const auto& pair : ATTRIBUTES) {
        const QNetworkRequest::Attribute& code = pair.first;
        const char* name = pair.second;
        const QVariant value = reply->attribute(code);
        if (value.isValid()) {
            QLOG_INFO() << "RateLimiter::SetupEndpoint() HEAD reply attribute" << name << "=" << value.toString();
        };
    };
}

RateLimitManager& RateLimiter::GetManager(
    const QString& endpoint,
    const QString& policy_name)
{
    QLOG_TRACE() << "RateLimiter::GetManager() entered";
    QLOG_TRACE() << "RateLimiter::GetManager() endpoint = " << endpoint;
    QLOG_TRACE() << "RateLimiter::GetManager() policy_name = " << policy_name;

    auto it = manager_by_policy_.find(policy_name);
    if (it == manager_by_policy_.end()) {
        // Create a new policy manager.
        QLOG_DEBUG() << "Creating rate limit policy" << policy_name << "for" << endpoint;
        auto sender = boost::bind(&RateLimiter::SendRequest, this, boost::placeholders::_1);
        auto mgr = std::make_unique<RateLimitManager>(sender);
        auto& manager = *managers_.emplace_back(std::move(mgr));
        connect(&manager, &RateLimitManager::PolicyUpdated, this, &RateLimiter::OnPolicyUpdated);
        connect(&manager, &RateLimitManager::QueueUpdated, this, &RateLimiter::OnQueueUpdated);
        connect(&manager, &RateLimitManager::Paused, this, &RateLimiter::OnManagerPaused);
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
}

void RateLimiter::OnPolicyUpdated(const RateLimit::Policy& policy)
{
    QLOG_TRACE() << "RateLimiter::OnPolicyUpdated() entered";
    emit PolicyUpdate(policy);
}

void RateLimiter::OnQueueUpdated(const QString& policy_name, int queued_requests) {
    QLOG_TRACE() << "RateLimiter::OnQueueUpdated() entered";
    emit QueueUpdate(policy_name, queued_requests);
}

void RateLimiter::OnManagerPaused(const QString& policy_name, const QDateTime& until) {
    QLOG_TRACE() << "RateLimiter::OnManagerPaused() entered";
    QLOG_TRACE() << "RateLimiter::OnManagerPaused()"
        << "pausing until" << until.toString()
        << "for" << policy_name;
    pauses_.emplace(until, policy_name);
    update_timer_.start();
}

void RateLimiter::SendStatusUpdate()
{
    QLOG_TRACE() << "RateLimiter::SendStatusUpdate() entered";

    // Get rid of any pauses that finished in the past.
    const QDateTime now = QDateTime::currentDateTime();
    while (!pauses_.empty() && (pauses_.begin()->first < now)) {
        pauses_.erase(pauses_.begin());
    };

    if (pauses_.empty()) {
        QLOG_TRACE() << "RateLimiter::SendStatusUpdate() stopping status updates";
        update_timer_.stop();
    } else {
        const auto& pause = *pauses_.begin();
        const QDateTime& pause_end = pause.first;
        const QString policy_name = pause.second;
        emit Paused(now.secsTo(pause_end), policy_name);
    };
}
