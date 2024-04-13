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

#include "ratelimit.h"

#include <QUrl>

#include "QsLog.h"

#include "application.h"
#include "network_info.h"
#include "oauth.h"
#include "util.h"

using namespace RateLimit;

//=========================================================================================
// Local function declarations
//=========================================================================================

static QByteArray ParseHeader(QNetworkReply* const reply, const QByteArray& name);
static QByteArray ParseRateLimitPolicy(QNetworkReply* const reply);
static QByteArrayList ParseHeaderList(QNetworkReply* const reply, const QByteArray& name, const char delim);
static QByteArrayList ParseRateLimitRules(QNetworkReply* const reply);
static QByteArrayList ParseRateLimit(QNetworkReply* const reply, const QByteArray& rule);
static QByteArrayList ParseRateLimitState(QNetworkReply* const reply, const QByteArray& rule);
static QDateTime ParseDate(QNetworkReply* const reply);
static int ParseStatus(QNetworkReply* const reply);

static QString GetEndpoint(const QUrl& url);

static void Dispatch(std::unique_ptr<RateLimitedRequest> request);

//=========================================================================================
// Classes to represent a rate limit policy
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
	const QByteArrayList limit_fragments = ParseRateLimit(reply, rule_name);
	const QByteArrayList state_fragments = ParseRateLimitState(reply, rule_name);
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

Policy::Policy() :
	name("<<EMPTY-POLICY>>"),
	empty(true),
	status(PolicyStatus::UNKNOWN),
	max_period(0) {};

Policy::Policy(QNetworkReply* const reply) :
	name(ParseRateLimitPolicy(reply)),
	empty(false),
	status(PolicyStatus::UNKNOWN),
	max_period(0)
{
	const QByteArrayList rule_names = ParseRateLimitRules(reply);
	const int rule_count = rule_names.size();

	// Allocate a new vector of rules for this policy.
	rules = std::vector<PolicyRule>(rule_count);

	// Iterate over all the rule names expected.
	for (int i = 0; i < rule_count; ++i) {

		// Parse the next rule.
		rules[i] = PolicyRule(rule_names[i], reply);

		// Update the maximum period.
		for (const auto& item : rules[i].items) {
			if (max_period < item.limit.period) {
				max_period = item.limit.period;
			};
		};
	};
	// Check the status of the rate limit.
	UpdateStatus();
}

void Policy::UpdateStatus() {
	for (const auto& rule : rules) {
		for (const auto& item : rule.items) {
			const RuleItemData& limit = item.limit;
			const RuleItemData& state = item.state;
			if (item.limit.period != item.state.period) {
				QLOG_ERROR() << "Ivalid rate limit state:" << name;
				status = PolicyStatus::INVALID;
			};
			if (status != PolicyStatus::INVALID) {
				if (state.hits > limit.hits) {
					QLOG_ERROR() << "Rate limit violation detected:" << name;
					status = PolicyStatus::VIOLATION;
				} else if (status != PolicyStatus::VIOLATION) {
					if (state.hits >= (limit.hits - BORDERLINE_REQUEST_BUFFER)) {
						QLOG_WARN() << "Rate limit is borderline:" << name;
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

//=========================================================================================
// Rate Limited Request
//=========================================================================================

// Total number of rate-limited requests that have been created.
unsigned long RateLimitedRequest::request_count = 0;

// Create a new rate-limited request.
RateLimitedRequest::RateLimitedRequest(const QNetworkRequest& request, const Callback callback) :
	id(++request_count),
	network_request(request),
	worker_callback(callback),
	endpoint(GetEndpoint(request.url())),
	network_reply(nullptr),
	reply_time(QDateTime()),
	reply_status(-1)
{
}

//=========================================================================================
// Policy Manager
//=========================================================================================

// Create a new rate limit manager based on an existing policy.
PolicyManager::PolicyManager(std::unique_ptr<Policy> policy_, QObject* parent) :
	QObject(parent),
	policy(std::move(policy_)),
	busy(false),
	next_send(QDateTime::currentDateTime()),
	last_send(QDateTime()),
	violation(false)
{
	// Setup the active request timer to call SendRequest each time it's done.
	active_request_timer.setSingleShot(true);
	connect(&active_request_timer, &QTimer::timeout, this, &PolicyManager::SendRequest);

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
			};

			if (current_hits >= (maximum_hits - BORDERLINE_REQUEST_BUFFER)) {
				QLOG_DEBUG() << "about to violate" << policy->name << QString(rule);

				// Determine how far back into the history we can look.
				const size_t n = (current_hits > history_size) ? history_size : current_hits;

				// Start with the timestamp of the earliest known
				// reply relevant to this limitation.
				const QDateTime starting_time = (n < 1)
					? QDateTime::currentDateTime()
					: known_reply_times[n - 1];

				// Calculate the next time it will be safe to send a request.
				const QDateTime next_safe_time = starting_time
					.addSecs(period_tested)
					.addMSecs(SAFETY_BUFFER_MSEC);

				if (next_safe_time.isValid() == false) {
					QLOG_ERROR() << "error updating next safe time in OnPolicyUpdate:"
						<< "\n\tstarting time is" << starting_time.toString()
						<< "\n\tperiod_tested is" << period_tested
						<< "\n\tnext_safe_time is" << next_safe_time;
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
		QLOG_ERROR() << "next_send is invalid";
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
		<< "seconds to send request" << active_request->id
		<< "at" << next_send.toLocalTime().toString();

	active_request_timer.setInterval(msec_delay);
	QMetaObject::invokeMethod(&active_request_timer, "start");
	if (msec_delay > 1000) {
		emit RateLimitingStarted();
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

	if (active_request->network_reply != nullptr) {
		QLOG_ERROR() << "The network reply for the active request is not empty";
		return;
	};

	QLOG_TRACE() << policy->name
		<< "sending request" << active_request->id
		<< "to" << active_request->endpoint
		<< "via" << active_request->network_request.url().toString();

	// Finally, send the request and note the time.
	last_send = QDateTime::currentDateTime();
	emit RequestReady(active_request->network_request);
};

// Called when the active request's reply is finished.
void PolicyManager::ReceiveReply() {

	QNetworkReply* reply = qobject_cast<QNetworkReply*>(sender());
	active_request->network_reply = reply;
	active_request->reply_time = ParseDate(reply);
	active_request->reply_status = ParseStatus(reply);

	QLOG_TRACE() << policy->name
		<< "received reply for request" << active_request->id
		<< "with status" << active_request->reply_status;

	if (reply->hasRawHeader("X-Rate-Limit-Policy")) {

		const QString reply_policy_name = reply->rawHeader("X-Rate-Limit-Policy");

		// Check the rate limit policy name from the header.
		if (policy->name != reply_policy_name) {
			QLOG_ERROR() << "policy manager for" << policy->name
				<< "received header reply with" << reply_policy_name;
		};

		// Save the reply time if this was not a cached reply.
		known_reply_times.push_front(active_request->reply_time);

		// Read the updated policy limits and state from the network reply.
		policy = std::make_unique<Policy>(reply);

		// Now examine the new policy and update ourselves accordingly.
		OnPolicyUpdate();

		// Alert listeners like the rate limiter
		emit ReplyReceived();

	} else {
		if (policy->empty == false) {
			QLOG_ERROR() << "policy manager for" << policy->name
				<< "received a reply without a rate limit policy";
		};
	};

	// Check for errors before dispatching the request
	if (active_request->reply_status == RATE_LIMIT_VIOLATION_STATUS) {

		// There was a rate limit violation.
		ResendAfterViolation();

	} else if (active_request->network_reply->error() != QNetworkReply::NoError) {

		// Some other HTTP error was encountered.
		QLOG_ERROR() << "policy manager for" << policy->name
			<< "request" << active_request->id
			<< "reply status was " << active_request->reply_status
			<< "and error was" << reply->error();

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

	// Determine how long we need to wait.
	const int delay_sec = active_request->network_reply->rawHeader("Retry-After").toInt();
	const int delay_msec = (delay_sec * 1000) + EXTRA_RATE_VIOLATION_MSEC;
	QLOG_ERROR() << policy->name
		<< "RATE LIMIT VIOLATION on request" << active_request->id << "of" << delay_sec << "seconds";
	for (const auto& header : active_request->network_reply->rawHeaderPairs()) {
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
	};

	// Reset this request before resending it, which means
	// letting QT know the assocated reply can be deleted.
	active_request->network_reply->deleteLater();
	active_request->network_reply = nullptr;
	active_request->reply_time = QDateTime();
	active_request->reply_status = -1;
	ActivateRequest();
}

int PolicyManager::GetPauseDuration() const {
	int delay = QDateTime::currentDateTime().secsTo(next_send);
	return (delay > 0) ? delay : 0;
}

QString PolicyManager::GetStatusMessage() const {

	const QString info = QString("%1 with %2 queued requests").arg(
		POLICY_STATE[policy->status],
		QString::number(request_queue.size()));

	const int delay = GetPauseDuration();

	QStringList lines;
	lines.push_back(policy->name);
	std::vector<PolicyRule>& rules = policy->rules;
	for (auto& rule : rules) {
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
// Local functions
//=========================================================================================

// Get a header field from an HTTP reply.
static QByteArray ParseHeader(QNetworkReply* const reply, const QByteArray& name) {
	if (reply->hasRawHeader(name)) {
		return reply->rawHeader(name);
	} else {
		QLOG_ERROR() << "GetHeader(): missing header:" << name;
		return QByteArray();
	};
}

// Get a header field and split into a list.
static QByteArrayList ParseHeaderList(QNetworkReply* const reply, const QByteArray& name, const char delim) {
	QByteArray value = ParseHeader(reply, name);
	QByteArrayList items = value.split(delim);
	if (items.isEmpty() == true) {
		QLOG_ERROR() << "GetHeaderList():" << name << "is empty";
	};
	return items;
}

// Return the name of the policy from a network reply.
static QByteArray ParseRateLimitPolicy(QNetworkReply* const reply) {
	return ParseHeader(reply, "X-Rate-Limit-Policy");
}

// Return the name(s) of the rule(s) from a network reply.
static QByteArrayList ParseRateLimitRules(QNetworkReply* const reply) {
	return ParseHeaderList(reply, "X-Rate-Limit-Rules", ',');
}

// Return a list of one or more items that define a rule's limits.
static QByteArrayList ParseRateLimit(QNetworkReply* const reply, const QByteArray& rule) {
	return ParseHeaderList(reply, "X-Rate-Limit-" + rule, ',');
}

// Return a list of one or more items that define a rule's current state.
static QByteArrayList ParseRateLimitState(QNetworkReply* const reply, const QByteArray& rule) {
	return ParseHeaderList(reply, "X-Rate-Limit-" + rule + "-State", ',');
}

// Return the date from the HTTP reply headers.
static QDateTime ParseDate(QNetworkReply* const reply) {
	QString timestamp = QString(Util::FixTimezone(ParseHeader(reply, "Date")));
	const QDateTime date = QDateTime::fromString(timestamp, Qt::RFC2822Date);
	if (date.isValid() == false) {
		QLOG_ERROR() << "invalid date parsed from" << timestamp;
	};
	return date;
}

// Return the HTTP status from the reply headers.
static int ParseStatus(QNetworkReply* const reply) {
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
// The application-facing Rate Limiter
//=========================================================================================

RateLimiter::RateLimiter(Application& app, QObject* parent) :
	QObject(parent),
	network_manager_(app.network_manager()),
	oauth_manager_(app.oauth_manager())
{
	// Creat the policy manager that will handle non-limited requests.
	default_manager = std::make_shared<PolicyManager>(std::make_unique<Policy>(), this);
	connect(default_manager.get(), &PolicyManager::RequestReady, this, &RateLimiter::SendRequest);

	// Setup the status update timer.
	status_updater.setSingleShot(false);
	status_updater.setInterval(1000);
	connect(&status_updater, &QTimer::timeout, this, &RateLimiter::DoStatusUpdate);
}

void RateLimiter::SendRequest(QNetworkRequest request) {
	PolicyManager* manager = qobject_cast<PolicyManager*>(sender());
	if (manager->policy->empty == false) {
		if (oauth_manager_.access_token() != nullptr) {
			oauth_manager_.addAuthorization(request);
		};
	};
	QNetworkReply* reply = network_manager_.get(request);
	connect(reply, &QNetworkReply::finished, manager, &PolicyManager::ReceiveReply);
}

void RateLimiter::Submit(QNetworkRequest network_request, Callback request_callback)
{
	// Make sure the user agent is set according to GGG's guidance.
	network_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);

	// We need the endpoint to see if there's a manager for this request.
	const QString endpoint = GetEndpoint(network_request.url());

	if (endpoint_mapping.count(endpoint) > 0) {

		// This endpoint is known to use an existing policy manager.
		std::unique_ptr<RateLimitedRequest> request = std::make_unique<RateLimitedRequest>(
			network_request, request_callback);
		PolicyManager& manager = *endpoint_mapping[endpoint];
		QLOG_DEBUG() << manager.policy->name << "is handling" << endpoint;
		manager.QueueRequest(std::move(request));

	} else {

		// This is a new endpoint.
		QNetworkReply* reply = network_manager_.head(network_request);
		connect(reply, &QNetworkReply::finished, this,
			[=]() { SetupEndpoint(network_request, request_callback, reply); });

	};
}

void RateLimiter::SetupEndpoint(QNetworkRequest network_request, Callback request_callback, QNetworkReply* reply) {

	// Create a new rate-limited request.
	std::unique_ptr<RateLimitedRequest> request = std::make_unique<RateLimitedRequest>(
		network_request, request_callback);

	// Get a reference to the endpoint since we use it a bunch.
	const QString endpoint = request->endpoint;

	// Handle requests that are not rate limited.
	if (reply->hasRawHeader("X-Rate-Limit-Policy") == false) {
		QLOG_DEBUG() << "The endpoint is not rate-limited:" << endpoint;
		endpoint_mapping[endpoint] = default_manager;
		default_manager->endpoints.push_back(endpoint);
		default_manager->QueueRequest(std::move(request));
		return;
	}

	// Parse the rate limit policy from the reply headers.
	const QString policy_name = ParseRateLimitPolicy(reply);
	QLOG_DEBUG() << policy_name << "encountered.";

	// Check to see if another manager already exists for this policy.
	if (policy_mapping.count(policy_name) > 0) {
		QLOG_DEBUG() << policy_name << "now handles" << endpoint;
		std::shared_ptr<PolicyManager> manager = policy_mapping[policy_name];
		endpoint_mapping[endpoint] = manager;
		manager->endpoints.push_back(endpoint);
		manager->policy = std::make_unique<Policy>(reply);
		manager->OnPolicyUpdate();
		manager->QueueRequest(std::move(request));
		return;
	};

	// Create a new policy manager.
	QLOG_DEBUG() << policy_name << "created for" << endpoint;
	std::unique_ptr<Policy> policy = std::make_unique<Policy>(reply);
	std::shared_ptr<PolicyManager> manager = std::make_shared<PolicyManager>(std::move(policy), this);
	connect(manager.get(), &PolicyManager::RequestReady, this, &RateLimiter::SendRequest);
	connect(manager.get(), &PolicyManager::ReplyReceived, this, &RateLimiter::DoStatusUpdate);
	connect(manager.get(), &PolicyManager::RateLimitingStarted, this, &RateLimiter::OnTimerStarted);
	endpoint_mapping[endpoint] = manager;
	policy_mapping[policy_name] = manager;
	manager->endpoints.push_back(endpoint);
	manager->QueueRequest(std::move(request));

	// Since we discovered a new rate limit policy, let's update the status.
	DoStatusUpdate();
}

void RateLimiter::OnTimerStarted() {

	// Make sure the program status is being updated.
	if (status_updater.isActive() == false) {
		QLOG_DEBUG() << "Starting rate limit status updates";
		QMetaObject::invokeMethod(&status_updater, "start");
	};
}

void RateLimiter::DoStatusUpdate()
{
	RateLimitStatus status = RateLimitStatus::OK;
	bool curious = false;
	QStringList lines;
	int minimum_pause = 0;

	// Check to see if any of the policy managers are busy.
	for (auto& pair : policy_mapping) {
		auto manager = pair.second;
		lines.push_back(manager->GetStatusMessage());
		lines.push_back("");
		switch (manager->GetStatus()) {
		case PolicyStatus::OK:
			break;
		case PolicyStatus::UNKNOWN:
		case PolicyStatus::INVALID:
			curious = true;
			break;
		case PolicyStatus::BORDERLINE:
		case PolicyStatus::VIOLATION:
			status = RateLimitStatus::PAUSED;
			int pause = manager->GetPauseDuration();
			if ((minimum_pause == 0) || (pause < minimum_pause)) {
				minimum_pause = pause;
			};
			break;
		};
	};

	if ((status == RateLimitStatus::OK) && (curious == false)) {
		// Stop the status updates if it's running and nobody is busy.
		if (status_updater.isActive() == true) {
			QLOG_DEBUG() << "Stopping rate limit status updates";
			QMetaObject::invokeMethod(&status_updater, "stop");
		};
	} else {
		// Start the timer if it's not running and someone is busy.
		if (status_updater.isActive() == false) {
			QLOG_DEBUG() << "Starting rate limit status updates status update";
			QMetaObject::invokeMethod(&status_updater, "start");
		};
	};

	// Emit a status update either way so the user can see that we aren't busy.
	StatusInfo update = { status, minimum_pause, lines.join('\n') };
	emit StatusUpdate(update);
}
