/*
	Copyright 2023 Tom Holz

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

#include "boost/format.hpp"

#include <QApplication>
#include <QNetworkCookie>
#include <QNetworkCookieJar>
#include <QNetworkRequest>
#include <QUrl>

#include "QsLog.h"

#include "network_info.h"
#include "ratelimit.h"

using namespace RateLimit;

void RateLimit::Error(const QString& message)
{
    QMetaObject::invokeMethod(qApp, "FatalError", Qt::QueuedConnection, Q_ARG(QString, message));
}

//=========================================================================================
// Local function declarations
//=========================================================================================

static QByteArray GetHeader(QNetworkReply* const reply, const QByteArray& name);
static QByteArray GetRateLimitPolicy(QNetworkReply* const reply);
static QByteArrayList GetHeaderList(QNetworkReply* const reply, const QByteArray& name, const char delim);
static QByteArrayList GetRateLimitRules(QNetworkReply* const reply);
static QByteArrayList GetRateLimit(QNetworkReply* const reply, const QByteArray& rule);
static QByteArrayList GetRateLimitState(QNetworkReply* const reply, const QByteArray& rule);
static QDateTime GetDate(QNetworkReply* const reply);
static int GetStatus(QNetworkReply* const reply);

static QString GetEndpoint(const QUrl& url);

static void Dispatch(std::unique_ptr<RateLimitedRequest> request);

//=========================================================================================
// INTERAL CLASSES AND FUNCTIONS
//=========================================================================================

RuleItemData::RuleItemData() :
    hits(-1),
    period(-1),
    restriction(-1) {};

RuleItemData::RuleItemData(const QByteArray& header_fragment) :
    hits(-1),
    period(-1),
    restriction(-1)
{
    const QByteArrayList parts = header_fragment.split(':');
    hits = parts[0].toInt();
    period = parts[1].toInt();
    restriction = parts[2].toInt();
}

RuleItemData::operator QString() const {
    return QString("%1:%2:%3").arg(
        QString::number(hits),
        QString::number(period),
        QString::number(restriction));
}

RuleItem::operator QString() const {
    return QString("%1/%2:%3:%4").arg(
        QString::number(state.hits),
        QString::number(limit.hits),
        QString::number(limit.period),
        QString::number(limit.restriction));
}

PolicyRule::PolicyRule() {};

PolicyRule::PolicyRule(const QByteArray& rule_name, QNetworkReply* const reply) :
    name(QString(rule_name)),
    items({})
{
    const QByteArrayList limit_fragments = GetRateLimit(reply, rule_name);
    const QByteArrayList state_fragments = GetRateLimitState(reply, rule_name);
    const int item_count = limit_fragments.size();
    items = std::vector<RuleItem>(item_count);
    for (int j = 0; j < item_count; ++j) {
        items[j].limit = RuleItemData(limit_fragments[j]);
        items[j].state = RuleItemData(state_fragments[j]);
    };
}

PolicyRule::operator QString() const {
    QStringList list;
    for (const RuleItem& element : items) {
        list.push_back(QString(element));
    };
    return QString("%1: %2").arg(name, list.join(", "));
}

Policy::Policy(const QString& name) :
    name(name),
    status(PolicyStatus::UNKNOWN),
    max_period(0) {};

Policy::Policy(QNetworkReply* const reply) :
    name(GetRateLimitPolicy(reply)),
    status(PolicyStatus::UNKNOWN),
    max_period(0)
{
	const QByteArrayList rule_names = GetRateLimitRules(reply);
	const int rule_count = rule_names.size();

	// Allocate a new vector of rules for this policy.
	rules = std::vector<PolicyRule>(rule_count);

	// Iterate over all the rule names expected.
	for (int i = 0; i < rule_count; ++i) {

        // Parse the next rule.
        rules[i] = PolicyRule(rule_names[i], reply);

        // Update the maximum period.
        for (auto& item : rules[i].items) {
            if (max_period < item.limit.period) {
                max_period = item.limit.period;
            };
        };
	};
    // Check the status of the rate limit.
    CheckStatus();
}

void Policy::CheckStatus() {
    status = PolicyStatus::UNKNOWN;
    for (auto& rule : rules) {
        for (auto& item : rule.items) {
            const RuleItemData& limit = item.limit;
            const RuleItemData& state = item.state;
            if (item.limit.period != item.state.period) {
                status = PolicyStatus::INVALID;
            };
            if (status != PolicyStatus::INVALID) {
                if (state.hits > limit.hits) {
                    status = PolicyStatus::VIOLATION;
                }
                else if (status != PolicyStatus::VIOLATION) {
                    if (state.hits >= (limit.hits - BORDERLINE_REQUEST_BUFFER)) {
                        status = PolicyStatus::BORDERLINE;
                    };
                };
            };
        };
    };
    if (status == PolicyStatus::UNKNOWN) {
        status = PolicyStatus::OK;
    };
}

//------------------------------------------------------------------------------
// Rate Limited Request
//------------------------------------------------------------------------------

// Total number of rate-limited requests that have been created.
unsigned long RateLimitedRequest::request_count = 0;

// Create a new rate-limited request.
RateLimitedRequest::RateLimitedRequest(QNetworkAccessManager& manager, const QNetworkRequest& request, const Callback callback) :
    id(++request_count),
    network_manager(manager),
    network_request(request),
    worker_callback(callback),
    endpoint(GetEndpoint(request.url())),
    network_reply(nullptr),
    reply_time(QDateTime()),
    reply_status(-1)
{
}

//------------------------------------------------------------------------------
// Policy Manager
//------------------------------------------------------------------------------

// Create a new rate limit manager based on an existing policy.
PolicyManager::PolicyManager(std::unique_ptr<Policy> policy_) :
    policy(std::move(policy_)),
    busy(false),
    next_send(QDateTime::currentDateTime()),
    last_send(QDateTime()),
    violation(false)
{
    // Setup the active request timer to call SendRequest each time it's done.
    active_request_timer.setSingleShot(true);
    QObject::connect(&active_request_timer, &QTimer::timeout, [=]() { SendRequest(); });

    // Check the policy for pre-existing violations, e.g. if Acquisition
    // has been recently restarted and we are still in time-out from a
    // prior rate limit violation.
    OnPolicyUpdate();
}

// Put a request in the dispatch queue so it's callback can be triggered.
void Dispatch(std::unique_ptr<RateLimitedRequest> request)
{
    // When a request has been successfully replied-to, then it's ready to be
    // dispatched. There's a special function for this because replies may come
    // back in a different order than they were submitted. This function
    // keeps track of which replies have been recieved and triggers callback 
    // in order, so the calling code doesn't have to worry about order.

    // Move finished requests into their own list so they can be reordered by
    // request id, which is how we guarantee that request callbacks will be
    // dispatched in order.
    static std::list<std::unique_ptr<RateLimitedRequest>> finished_requests = {};

    // Request id of the next request that should be sent back to the application
    static unsigned long next_request_to_send = 1;

	// First, insert this request into the queue of waiting
	// items so that the queue is always ordered based on
	// request id.
	if (finished_requests.empty()) {

		// The queue is empty.
		finished_requests.push_back(std::move(request));

	} else if (request->id > finished_requests.back()->id) {

		// The request belongs at the end.
		finished_requests.push_back(std::move(request));

	} else {

		// Find where in the queue this request fits.
		for (auto pos = finished_requests.begin(); pos != finished_requests.end(); ++pos) {
			if (request->id < pos->get()->id) {
				finished_requests.insert(pos, std::move(request));
				break;
			};
		};
	};

	// Second, check to see if we can send one or more
	// requests from the front of the queue.
	while (finished_requests.empty() == false) {

		// Stop if the next request isn't the one we are waiting for.
		if (next_request_to_send != finished_requests.front()->id) {
			break;
		};

		// Take this request off the front of the queue.
		std::unique_ptr<RateLimitedRequest> request = std::move(finished_requests.front());
		finished_requests.pop_front();
		++next_request_to_send;

		// Trigger the callback for this request now.
		request->worker_callback(request->network_reply);
		request->network_reply->deleteLater();
		request = nullptr;
	};
}

// Update this policy manager when the policy has changed. This means updating
// the number of reply times we keep around of the policy's limits have changed,
// and figuring out the next time a request can be sent without violating this
// policy.
void PolicyManager::OnPolicyUpdate()
{
    // Grow the history capacity if needed.
    if (known_reply_times.capacity() < policy->max_period) {
        QLOG_DEBUG() << policy->name
            << "increasing history capacity"
            << "from" << known_reply_times.capacity()
            << "to" << policy->max_period;
        known_reply_times.set_capacity(policy->max_period);
    };

    // Nothing to do if we are safe.
    if (policy->status == PolicyStatus::OK) {
        return;
    };

    // Need to know the current number of items in the reply
    // history so we don't try to read past them.
    const size_t history_size = known_reply_times.size();

	for (const PolicyRule& rule : policy->rules) {

		for (const RuleItem& item : rule.items) {

            const RuleItemData& limit = item.limit;
            const RuleItemData& state = item.state;

			const int& current_hits = state.hits;
			const int& maximum_hits = limit.hits;
			const int& period_tested = limit.period;

			// First, check to see if we are at (or past) the current
			// rate limit policy's maximum. If that's the case, we need
			// to update the next time it will be safe to send a request.
			//
			// For example, if a limitation allows up to 10 requests in a
			// 60 second period, then if there have already been 10 hits
			// against that limitation, we cannot make another until the
			// first of those 10 hits falls out of the 60 second period.
			//
			// This is why we store a history of reply times.
			//
			// However, it's possible we hit a rate limit policy on the
			// very first request the application makes. This can happen
			// if the application was just restarted after a prior rate
			// limit violation.
			//
			// Therefore, there have been 10 hits in the last 60 seconds
			// against the example policy, but the application only knows
			// about 4 of them, then the best we can do is go back to
			// the earliest of those 4 replies and add the restriction
			// to that request's timestamp.
			//
			// This means the only time a real rate limit violation
			// should occur is if the application's very first request
			// is restricted.

            if (current_hits > maximum_hits) {
                QLOG_ERROR() << "RATE LIMIT VIOLATION:" << policy->name << QString(rule);
                Error("rate limit violation");
            }

			if (current_hits >= (maximum_hits - BORDERLINE_REQUEST_BUFFER)) {
                QLOG_DEBUG() << "about to violate" << policy->name << QString(rule);

				// Determine how far back into the history we can look.
				size_t n = current_hits;
				if (n > history_size) {
					n = history_size;
				};

				// Start with the timestamp of the earliest known
				// reply relevant to this limitation.
				QDateTime starting_time;
				if (n < 1) {
					starting_time = QDateTime::currentDateTime();
				} else {
					starting_time = known_reply_times[n - 1];
				};

				// Calculate the next time it will be safe to send a request.
                const QDateTime next_safe_time = starting_time
                    .addSecs(period_tested)
                    .addMSecs(SAFETY_BUFFER_MSEC);

                if (next_safe_time.isValid() == false) {
                    QLOG_ERROR() << "error updating next safe time in OnPolicyUpdate:"
                        << "\n\tstarting time is" << starting_time.toString()
                        << "\n\tperiod_tested is" << period_tested
                        << "\n\tnext_safe_time is" << next_safe_time;
                    Error("next_safe_time is invalid");
                };

				// Update this manager's send time only if it's later
				// than the manager thinks we need to wait.
                QLOG_TRACE() << "Updating next send:"
                    << "\n\tstarting_time  is" << starting_time.toString()
                    << "\n\tnext_safe_time is" << next_safe_time.toString()
                    << "\n\tnext_send      is" << next_send.toString();
				if (next_safe_time > next_send) {
					next_send = next_safe_time;
				};
			};
		};
	};

}

// If the rate limit manager is busy, the request will be queued.
// Otherwise, the request will be sent immediately, making the
// manager busy and causing subsequent requests to be queued.
void PolicyManager::QueueRequest(std::unique_ptr<RateLimitedRequest> request) {
	if (busy) {
		QLOG_TRACE() << policy->name << "queuing request" << request->id;
		request_queue.push_back(std::move(request));
	} else {
		busy = true;
		active_request = std::move(request);
		ActivateRequest();
	};
}

// Send the active request at the next time it will be safe to do so
// without violating the rate limit policy.
void PolicyManager::ActivateRequest() {

    if (next_send.isValid() == false) {
        Error("next_send is invalid");
    };

    const QDateTime now = QDateTime::currentDateTime();

    int msec_delay = now.msecsTo(next_send);
    if (msec_delay < MINIMUM_ACTIVATION_DELAY_MSEC) {
        msec_delay = MINIMUM_ACTIVATION_DELAY_MSEC;
    };

    if (last_send.isValid()) {
        const QDateTime min_send = last_send.addMSecs(MINIMUM_INTERVAL_MSEC);
        const int min_delay = now.msecsTo(min_send);
        if (min_delay > msec_delay) {
            msec_delay = min_delay;
        };
    };

	// Need to wait and rerun this function when it's safe to send.
	QLOG_TRACE() << policy->name
		<< "waiting" << (msec_delay / 1000)
		<< "seconds to send active request" << active_request->id
		<< "at" << next_send.toLocalTime().toString();

    active_request_timer.setInterval(msec_delay);
    QMetaObject::invokeMethod(&active_request_timer, "start", Qt::QueuedConnection);
    if (msec_delay > 1000) {
        emit TimerStarted();
    };
}

// Send the active request immediately.
void PolicyManager::SendRequest() {
    
    if (active_request == nullptr) {
        QLOG_DEBUG() << "The active request is empty.";
        return;
    };

    if (violation == true) {
        QLOG_ERROR() << "A violation seems to be in effect. Cannot send requests.";
        return;
    };

    QLOG_TRACE() << policy->name
        << "sending request" << active_request->id
        << "to" << active_request->endpoint
        << "via" << active_request->network_request.url().toString();

    if (active_request->network_reply != nullptr) {
        QLOG_ERROR() << "The network reply for the active request is not empty";
        Error("active_request->network_reply is not null");
    };

    last_send = QDateTime::currentDateTime();
    QNetworkReply* reply = active_request->network_manager.get(active_request->network_request);
    active_request->network_reply = reply;
    QObject::connect(reply, &QNetworkReply::finished, [=]() { ReceiveReply(); });
};

// Called when the active request's network_reply is finished.
void PolicyManager::ReceiveReply() {

	QNetworkReply* reply = active_request->network_reply;
	active_request->reply_time = GetDate(reply);
	active_request->reply_status = GetStatus(reply);
    active_request->reply_cached = reply->attribute(QNetworkRequest::SourceIsFromCacheAttribute).toBool();
    
    QLOG_TRACE() << policy->name
		<< "received reply for request" << active_request->id
		<< "from cache" << active_request->reply_cached << "with status" << active_request->reply_status;

    if (reply->hasRawHeader("X-Rate-Limit-Policy")) {

        const QString reply_policy_name = reply->rawHeader("X-Rate-Limit-Policy");

        // Check the rate limit policy name from the header.
        if (policy->name != reply_policy_name) {
            QLOG_ERROR() << "policy manager for" << policy->name
                << "received header reply with" << reply_policy_name;
            Error("expected " + policy->name + " but received " + reply_policy_name);
        };

        if (active_request->reply_cached == false) {

            // Save the reply time if this was not a cached reply.
            known_reply_times.push_front(active_request->reply_time);

            // Read the updated policy limits and state from the network reply.
            policy = std::make_unique<Policy>(reply);

            // Now examine the new policy and update ourselves accordingly.
            OnPolicyUpdate();

            emit TimerStarted();
        };
    }
    else if (active_request->reply_cached == false) {
        if (policy->name != "<none>") {
            QLOG_ERROR() << "policy manager for" << policy->name
                << "received a reply without a rate limit policy";
            Error(policy->name + " received a reply without a rate limit policy");
        };
	};

	// Check for errors before dispatching the request
	if (active_request->reply_status == RATE_LIMIT_VIOLATION_STATUS) {

		// There was a rate limit violation.
		ResendAfterViolation();

	} else if (active_request->network_reply->error() != QNetworkReply::NoError) {

        if (active_request->reply_cached) {
            QLOG_ERROR() << "VIOLATION detected in a reponse with an error";
        };

		// Some other HTTP error was encountered.
		QLOG_ERROR() << "policy manager for" << policy->name
			<< "request" << active_request->id
			<< "reply status was " << active_request->reply_status
			<< "and error was" << reply->error();
        Error("Unexpected HTTP error");

	} else {

		// No errors or violations, so move this request to the dispatch queue.
		violation = false;
		Dispatch(std::move(active_request));
		if (request_queue.empty()) {
			busy = false;
		} else {
			// Stay busy and activate the next request in the queue.
			active_request = std::move(request_queue.front());
			request_queue.pop_front();
			ActivateRequest();
		};
	};
}

// A violation was detected, so we need to wait to resend the active request.
void PolicyManager::ResendAfterViolation()
{
	// Set the violation flag now. It will be unset when a reply is received that doesn't
	// indicat a violation.
	violation = true;

    if (active_request->reply_cached) {
        QLOG_ERROR() << "VIOLATION detected in a cached response.";
    };

	// Determine how long we need to wait.
	const int delay_sec = active_request->network_reply->rawHeader("Retry-After").toInt();
	const int delay_msec = (delay_sec * 1000) + EXTRA_RATE_VIOLATION_MSEC;
	QLOG_ERROR() << policy->name
		<< "RATE LIMIT VIOLATION on request" << active_request->id << "of" << delay_sec << "seconds";
    for (auto& header : active_request->network_reply->rawHeaderPairs()) {
        QLOG_DEBUG() << header.first << "=" << header.second;
    };

	// Update the time it will be safe to send again.
	next_send = active_request->reply_time.addMSecs(delay_msec);
    if (next_send.isValid() == false) {
        QLOG_DEBUG() << "policy manager for" << policy->name
            << "\n\tnext_send after violation is invalid:"
            << "\n\t" << "request id" << active_request->id
            << "\n\t" << "request endpoint" << active_request->endpoint
            << "\n\t" << "Retry-After" << active_request->network_reply->rawHeader("Retry-After")
            << "\n\t" << "reply time was" << active_request->reply_time.toString();
        Error("next_send is invalid");
    };

    Error("rate limit violation");

	// Reset this request before resending it, which means
	// letting QT know the assocated reply can be deleted.
	active_request->network_reply->deleteLater();
	active_request->network_reply = nullptr;
	active_request->reply_time = QDateTime();
	active_request->reply_status = -1;
	ActivateRequest();
}

bool PolicyManager::IsBusy() const {
    return (active_request != nullptr);
};

QString PolicyManager::GetCurrentStatus() const {

    const QString info = QString("%1 with %2 queued requests").arg(
        POLICY_STATE[policy->status],
        QString::number(request_queue.size()));

    const int delay = QDateTime::currentDateTime().secsTo(next_send);

    QStringList lines;
    lines.push_back(policy->name);
    const std::vector<PolicyRule>& rules = policy->rules;
    for (const auto& rule : rules) {
        lines.push_back(QString("( %1 )").arg(QString(rule)));
    };
    lines.push_back(info);
    switch (policy->status) {
    case PolicyStatus::OK:
        lines.push_back("Not rate limited.");
        break;
    case PolicyStatus::BORDERLINE:
        lines.push_back(QString("Paused for %1 seconds to avoid a violation.").arg(QString::number(delay)));
        break;
    case PolicyStatus::VIOLATION:
        lines.push_back(QString("Paused for %1 seconds due to VIOLATION.").arg(QString::number(delay)));
        break;
    default:
        break;
    };
    
    return lines.join("\n  ");
}

//=========================================================================================
// Local function definitions
//=========================================================================================

// Get a header field from an HTTP reply.
static QByteArray GetHeader(QNetworkReply* const reply, const QByteArray& name) {
    if (reply->hasRawHeader(name)) {
        return reply->rawHeader(name);
    }
    else {
        QLOG_ERROR() << "network reply is missing" << name;
        throw std::runtime_error("missing reply header");
    };
}

// Get a header field and split into a list.
static QByteArrayList GetHeaderList(QNetworkReply* const reply, const QByteArray& name, const char delim) {
    QByteArray value = GetHeader(reply, name);
    QByteArrayList items = value.split(delim);
    if (items.isEmpty() == true) {
        QLOG_ERROR() << "GetHeaderList():" << name << "is empty";
        throw std::runtime_error("expected header list is empty");
    };
    return items;
}

// Return the name of the policy from a network reply.
static QByteArray GetRateLimitPolicy(QNetworkReply* const reply) {
    return GetHeader(reply, "X-Rate-Limit-Policy");
}

// Return the name(s) of the rule(s) from a network reply.
static QByteArrayList GetRateLimitRules(QNetworkReply* const reply) {
    return GetHeaderList(reply, "X-Rate-Limit-Rules", ',');
}

// Return a list of one or more items that define a rule's limits.
static QByteArrayList GetRateLimit(QNetworkReply* const reply, const QByteArray& rule) {
    return GetHeaderList(reply, "X-Rate-Limit-" + rule, ',');
}

// Return a list of one or more items that define a rule's current state.
static QByteArrayList GetRateLimitState(QNetworkReply* const reply, const QByteArray& rule) {
    return GetHeaderList(reply, "X-Rate-Limit-" + rule + "-State", ',');
}

// Return the date from the HTTP reply headers.
static QDateTime GetDate(QNetworkReply* const reply) {
    QString timestamp = QString(GetHeader(reply, "Date"));
    const QDateTime date = QDateTime::fromString(timestamp, Qt::RFC2822Date);
    if (date.isValid() == false) {
        QLOG_ERROR() << "invalid date parsed from" << timestamp;
        RateLimit::Error("invalid date in http reply");
    };
    return date;
}

// Return the HTTP status from the reply headers.
static int GetStatus(QNetworkReply* const reply) {
    return reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
}

// Return the "endpoint" for a given URL.
static QString GetEndpoint(const QUrl& url) {
    // Strip everything except the scheme, host, and path.
    return url.toString(QUrl::RemoveUserInfo
        | QUrl::RemovePort
        | QUrl::RemoveQuery
        | QUrl::StripTrailingSlash);
}

//=========================================================================================
// Final public class
//=========================================================================================

RateLimiter::RateLimiter(QNetworkAccessManager& network_manager) :
    initialized(false),
    initial_manager(network_manager),
    initial_endpoints(KNOWN_ENDPOINTS.length()),
    initial_replies(KNOWN_ENDPOINTS.length()),
    initial_replies_received(0)
{
    // Start sending HEAD requests to construct the rate limit policy manager.
    SendInitRequest();

    // Creat the policy manager that will handle non-limited requests.
    default_manager = std::make_unique<PolicyManager>(std::make_unique<Policy>("<none>"));

    // Setup the status update timer.
    status_updater.setSingleShot(false);
    status_updater.setInterval(1000);
    QObject::connect(&status_updater, &QTimer::timeout, [=]() { DoStatusUpdate(); });
}

void RateLimiter::Submit(QNetworkAccessManager& network_manager, QNetworkRequest network_request, Callback request_callback)
{
    network_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);

    // Create a new rate-limited request.
    std::unique_ptr<RateLimitedRequest> request = std::make_unique<RateLimitedRequest>(
        network_manager, network_request, request_callback);

    // Dispatch or stage the request depending on if we are fully initialized.
    if (initialized) {
        DispatchRequest(std::move(request));
    }
    else {
        staged_requests.push_back(std::move(request));
    };
}

void RateLimiter::DispatchRequest(std::unique_ptr<RateLimitedRequest> request) {

    // See if any of the policy managers should handle this request.
    for (auto& manager : managers) {
        if (manager->endpoints.contains(request->endpoint)) {
            QLOG_DEBUG() << manager->policy->name << ":" << request->endpoint;
            manager->QueueRequest(std::move(request));
            return;
        };
    };

    // Fall back to using the default manager which does not apply any rate-limiting.
    // It's useful to use a policy manager here instead of writing custom code so that
    // we aren't duplicating code. We can also take advantage of the ordered dispatching
    // so that even non-limited requests will be returned in the order they were called.
    QLOG_DEBUG() << "No policy manager for:" << request->endpoint;
    default_manager->QueueRequest(std::move(request));
};

void RateLimiter::OnTimerStarted() {

    // Make sure the program status is being updated.
    if (status_updater.isActive() == false) {
        QLOG_DEBUG() << "Starting rate limit status updates";
        QMetaObject::invokeMethod(&status_updater, "start", Qt::QueuedConnection);
    };
}

void RateLimiter::SendInitRequest() {

    const int n = initial_replies_received;
    if (n >= KNOWN_ENDPOINTS.length()) {
        QLOG_ERROR() << "HEAD request" << n << "is invalid";
        return;
    };
    const QString endpoint = KNOWN_ENDPOINTS[n];
    QLOG_DEBUG() << "Sending HEAD request" << n << "to" << endpoint;
    QNetworkRequest request = QNetworkRequest(QUrl(endpoint));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    initial_endpoints[n] = request.url().toString();
    initial_replies[n] = initial_manager.head(request);
    connect(initial_replies[n], &QNetworkReply::finished, this, &RateLimiter::ReceiveInitReply);
}

void RateLimiter::ReceiveInitReply() {

    QLOG_DEBUG() << "Received HEAD reply.";
    if (++initial_replies_received < KNOWN_ENDPOINTS.length()) {
        SendInitRequest();
    }
    else {
        FinishInit();
    };
}

void RateLimiter::FinishInit() {

    QLOG_DEBUG() << "Finishing initialization.";
    QStringList policy_names;
    std::vector<std::unique_ptr<Policy>> policies;
    std::vector<QStringList> policy_endpoints;

    for (int n = 0; n < KNOWN_ENDPOINTS.length(); ++n) {
        QNetworkReply* reply = initial_replies.at(n);
        for (auto& pair : reply->rawHeaderPairs()) {
            QLOG_DEBUG() << "HEAD" << n << pair.first << "=" << pair.second;
        };
        if (reply->hasRawHeader("X-Rate-Limit-Policy")) {
            QString endpoint = initial_endpoints.at(n);
            std::unique_ptr<Policy> p = std::make_unique<Policy>(reply);
            int k;
            for (k = 0; k < policies.size(); ++k) {
                if (policies.at(k)->name == p->name) {
                    QLOG_DEBUG() << "Updating policy" << n << p->name << "with" << endpoint;
                    policies.at(k) = std::move(p);
                    policy_endpoints.at(k).push_back(endpoint);
                    break;
                };
            };
            if (k == policies.size()) {
                QLOG_DEBUG() << "Creating policy" << n << p->name << "for" << endpoint;
                policies.push_back(std::move(p));
                policy_endpoints.push_back(QStringList(endpoint));
            };
        };
    };

    for (int n = 0; n < policies.size(); ++n) {
        std::unique_ptr<Policy> p = std::move(policies.at(n));
        std::unique_ptr<PolicyManager> pm = std::make_unique<PolicyManager>(std::move(p));
        pm->endpoints = policy_endpoints[n];
        connect(pm.get(), &PolicyManager::TimerStarted, this, &RateLimiter::OnTimerStarted);
        QLOG_DEBUG() << "PolicyManager" << n << "created for" << pm->policy->name;
        for (int k = 0; k < pm->endpoints.length(); ++k) {
            QLOG_DEBUG() << "PolicyManager" << n << "endpoint" << k << "is" << pm->endpoints[k];
        };
        managers.push_back(std::move(pm));
    };

    initialized = true;

    QLOG_DEBUG() << "Dispatching" << staged_requests.size() << "staged requests.";
    for (auto& request : staged_requests) {
        DispatchRequest(std::move(request));
    };
}

void RateLimiter::DoStatusUpdate()
{
    bool keep_going = false;
    QStringList lines;
    for (auto& manager : managers) {
        lines.push_back(manager->GetCurrentStatus());
        lines.push_back("");
        if (manager->IsBusy()) {
            keep_going = true;
        };
    };
    if (keep_going == false) {
        QLOG_DEBUG() << "Stopping rate limit status updates";
        QMetaObject::invokeMethod(&status_updater, "stop", Qt::QueuedConnection);
    };
    emit StatusUpdate(lines.join('\n'));
}

