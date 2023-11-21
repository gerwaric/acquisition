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

Policy::Policy() :
	name("<<EMPTY-POLICY>>"),
	status(PolicyStatus::UNINITIALIZED),
	max_period(0) {};

Policy::Policy(QNetworkReply* const reply) :
	name(GetRateLimitPolicy(reply)),
	status(PolicyStatus::UNINITIALIZED),
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
	status = PolicyStatus::UNINITIALIZED;
	for (auto& rule : rules) {
		for (auto& item : rule.items) {
			const RuleItemData& limit = item.limit;
			const RuleItemData& state = item.state;
			if (item.limit.period != item.state.period) {
				item.status = PolicyStatus::INVALID;
			} else if (state.hits > limit.hits) {
				item.status = PolicyStatus::VIOLATION;
			} else if (state.hits >= (limit.hits - BORDERLINE_REQUEST_BUFFER)) {
				item.status = PolicyStatus::BORDERLINE;
			} else {
				item.status = PolicyStatus::OK;
			};
			if (status < item.status) {
				status = item.status;
			};
		};
	};
	if (status == PolicyStatus::UNINITIALIZED) {
		status = PolicyStatus::OK;
	};
}

//=========================================================================================
// Rate Limited Request
//=========================================================================================

// Total number of rate-limited requests that have been created.
unsigned long RateLimitedRequest::request_count = 0;

// Create a new rate-limited request.
RateLimitedRequest::RateLimitedRequest(const QString& endpoint_, const QNetworkRequest& request, const Callback callback) :
	id(++request_count),
	network_request(request),
	worker_callback(callback),
	endpoint(endpoint_),
	network_reply(nullptr),
	reply_time(QDateTime()),
	reply_status(-1)
{}

//=========================================================================================
// Policy Manager
//=========================================================================================

// Create a new rate limit manager based on an existing policy.
PolicyManager::PolicyManager(QObject* parent, std::unique_ptr<Policy> policy_) :
	QObject(parent),
	policy(std::move(policy_)),
	busy(false),
	active_request_timer(this),
	next_send(QDateTime::currentDateTime()),
	last_send(QDateTime()),
	violation(false)
{
	policy_name = (policy == nullptr) ? "default-policy" : policy->name;

	// Setup the active request timer to call SendRequest each time it's done.
	active_request_timer.setSingleShot(true);
	connect(&active_request_timer, &QTimer::timeout, this, &PolicyManager::SendRequest);

	// Check the policy for pre-existing violations, e.g. if Acquisition
	// has been recently restarted and we are still in time-out from a
	// prior rate limit violation.
	if (policy != nullptr) {
		OnPolicyUpdate();
	};
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
			<< "from" << known_reply_times.capacity() << "to" << policy->max_period;
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
	emit PolicyUpdated(*policy);
}

// If the rate limit manager is busy, the request will be queued.
// Otherwise, the request will be sent immediately, making the
// manager busy and causing subsequent requests to be queued.
void PolicyManager::QueueRequest(std::unique_ptr<RateLimitedRequest> request) {
	if (busy) {
		QLOG_TRACE() << policy_name << "queuing request" << request->id << "for" << request->network_request.url().toString();
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

	active_request_timer.setInterval(msec_delay);
	active_request_timer.start();
	if (msec_delay > 1000) {
		// Need to wait and rerun this function when it's safe to send.
		const QDateTime resend_time = QDateTime::currentDateTime().addMSecs(msec_delay);
		QLOG_TRACE() << policy_name
			<< "is waiting" << msec_delay / 1000
			<< "seconds to send request" << active_request->id
			<< "at" << resend_time.toLocalTime().toString();
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

	QLOG_TRACE() << policy_name
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
	active_request->reply_time = GetDate(reply);
	active_request->reply_status = GetStatus(reply);

	QLOG_TRACE() << policy_name
		<< "received reply for request" << active_request->id
		<< "with status" << active_request->reply_status;

	if (reply->hasRawHeader("X-Rate-Limit-Policy")) {

		const QString reply_policy_name = reply->rawHeader("X-Rate-Limit-Policy");

		// Check the rate limit policy name from the header.
		if (policy_name != reply_policy_name) {
			QLOG_ERROR() << "policy manager for" << policy_name
				<< "received header reply with" << reply_policy_name;
		};

		// Save the reply time if this was not a cached reply.
		known_reply_times.push_front(active_request->reply_time);

		// Read the updated policy limits and state from the network reply.
		policy = std::make_unique<Policy>(reply);

		// Now examine the new policy and update ourselves accordingly.
		OnPolicyUpdate();

		emit RateLimitingStarted();

	} else if (policy != nullptr) {
		QLOG_ERROR() << "policy manager for" << policy_name << "received a reply without a rate limit policy";
	};

	// Check for errors before dispatching the request
	if (active_request->reply_status == RATE_LIMIT_VIOLATION_STATUS) {

		// There was a rate limit violation.
		ResendAfterViolation();

	} else if (active_request->network_reply->error() != QNetworkReply::NoError) {

		// Some other HTTP error was encountered.
		QLOG_ERROR() << "policy manager for" << policy_name
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
	QLOG_ERROR() << policy_name
		<< "RATE LIMIT VIOLATION on request" << active_request->id << "of" << delay_sec << "seconds";
	for (const auto& header : active_request->network_reply->rawHeaderPairs()) {
		QLOG_DEBUG() << header.first << "=" << header.second;
	};

	// Update the time it will be safe to send again.
	next_send = active_request->reply_time.addMSecs(delay_msec);
	if (next_send.isValid() == false) {
		QLOG_DEBUG() << "policy manager for" << policy_name
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

bool PolicyManager::IsBusy() const {
	return (active_request != nullptr);
};

QString PolicyManager::GetCurrentStatus() const {

	if (policy == nullptr) {
		return "No Status for default-policy manager.";
	};

	const QString info = QString("%1 with %2 queued requests").arg(
		POLICY_STATE[policy->status],
		QString::number(request_queue.size()));

	const int delay = QDateTime::currentDateTime().secsTo(next_send);

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
static QByteArray GetHeader(QNetworkReply* const reply, const QByteArray& name) {
	if (reply->hasRawHeader(name)) {
		return reply->rawHeader(name);
	} else {
		QLOG_ERROR() << "GetHeader(): missing header:" << name;
		return QByteArray();
	};
}

// Get a header field and split into a list.
static QByteArrayList GetHeaderList(QNetworkReply* const reply, const QByteArray& name, const char delim) {
	QByteArray value = GetHeader(reply, name);
	QByteArrayList items = value.split(delim);
	if (items.isEmpty() == true) {
		QLOG_ERROR() << "GetHeaderList():" << name << "is empty";
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
	QString timestamp = QString(Util::FixTimezone(GetHeader(reply, "Date")));
	const QDateTime date = QDateTime::fromString(timestamp, Qt::RFC2822Date);
	if (date.isValid() == false) {
		QLOG_ERROR() << "invalid date parsed from" << timestamp;
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
// The application-facing Rate Limiter
//=========================================================================================

RateLimiter::RateLimiter() :
	network_manager_(this),
	default_manager(this),
	status_updater(this)
{
	// Create the worker thread.
	worker_thread = new QThread();
	connect(worker_thread, &QThread::finished, worker_thread, &QThread::deleteLater);
	connect(worker_thread, &QThread::finished, this, &RateLimiter::deleteLater);

	// This connection is used when Submit() is called from another thread.
	connect(this, &RateLimiter::Queue, this, &RateLimiter::OnSubmit, Qt::QueuedConnection);

	// Make sure requests from the default manager are handled.
	connect(&default_manager, &PolicyManager::RequestReady, this, &RateLimiter::SendRequest);

	// Setup the status update timer.
	status_updater.setSingleShot(false);
	status_updater.setInterval(1000);
	connect(&status_updater, &QTimer::timeout, this, &RateLimiter::DoStatusUpdate);

	// The current game patch (3.22.2) does not support HEAD requests against the new API,
	// so we need to setup some managers here.
	QLOG_WARN() << "============================================================";
	QLOG_WARN() << "==== HARD CODING INITAL POLICIES UNTIL 3.22.3 or 3.23 ======";
	QLOG_WARN() << "============================================================";
	std::map<QString, QStringList> setup = {
		{"league-request-limit",          {"GET /account/leagues"}},
		{"character-list-request-limit",  {"GET /character"}},
		{"character-request-limit",       {"GET /character/<name>"}},
		{"stash-list-request-limit",      {"GET /stash/<league>"}},
		{"stash-request-limit",           {"GET /stash/<league>/<stash_id>"}}
	};
	for (const auto& item : setup) {
		const QString& policy_name = item.first;
		const QStringList& policy_endpoints = item.second;
		auto p = std::make_unique<Policy>();
		auto pm = std::make_unique<PolicyManager>(this, std::move(p));
		pm->policy_name = policy_name;
		pm->policy->name = policy_name;
		pm->endpoints.append(policy_endpoints);
		QLOG_WARN() << "  INITIAL POLICY:" << policy_name;
		for (const auto& endpoint : policy_endpoints) {
			QLOG_WARN() << "    ENDPOINT:" << endpoint;
			endpoint_mapping[endpoint] = pm.get();
		};
		connect(pm.get(), &PolicyManager::RequestReady, this, &RateLimiter::SendRequest);
		connect(pm.get(), &PolicyManager::RateLimitingStarted, this, &RateLimiter::OnTimerStarted);
		connect(pm.get(), &PolicyManager::PolicyUpdated, this, &RateLimiter::OnPolicyUpdated);
		emit PolicyUpdated(*pm->policy);
		policy_mapping.emplace(policy_name, std::move(pm));
	};

	// Move ourselves (and our children) to the worker thread and start the worker.
	this->moveToThread(worker_thread);
	worker_thread->start();
}

void RateLimiter::SetAccessToken(const OAuthToken& token) {
	access_token = token;
	bearer_token = "Bearer " + token.access_token;
}

void RateLimiter::SendRequest(QNetworkRequest request) {
	PolicyManager* manager = qobject_cast<PolicyManager*>(sender());
	request.setHeader(QNetworkRequest::UserAgentHeader, USER_AGENT);
	if (request.url().host() == "api.pathofexile.com") {
		request.setRawHeader("Authorization", QByteArray::fromStdString(bearer_token));
	};
	QNetworkReply* reply = network_manager_.get(request);
	connect(reply, &QNetworkReply::finished, manager, &PolicyManager::ReceiveReply);
}

void RateLimiter::Submit(const QString endpoint, QNetworkRequest network_request, Callback request_callback)
{
	if (QThread::currentThread() == this->thread()) {
		OnSubmit(endpoint, network_request, request_callback);
	} else {
		emit Queue(endpoint, network_request, request_callback);
	};
};

void RateLimiter::OnSubmit(const QString& endpoint, QNetworkRequest network_request, Callback request_callback) {

	if (QThread::currentThread() != this->thread()) {
		QLOG_ERROR() << "OnSubmit is running on the wrong thread!";
		return;
	};

	if (endpoint_mapping.count(endpoint) > 0) {

		// This endpoint is known to use an existing policy manager.
		auto request = std::make_unique<RateLimitedRequest>(endpoint, network_request, request_callback);
		PolicyManager& manager = *endpoint_mapping[endpoint];
		manager.QueueRequest(std::move(request));

	} else {

		// This is a new endpoint.
		QNetworkReply* reply = network_manager_.head(network_request);
		connect(reply, &QNetworkReply::finished, this,
			[=]() { SetupEndpoint(endpoint, network_request, request_callback, reply); });

	};
}

void RateLimiter::OnPolicyUpdated(const Policy& policy) {
	emit PolicyUpdated(policy);
}

void RateLimiter::SetupEndpoint(const QString endpoint, QNetworkRequest network_request, Callback request_callback, QNetworkReply* reply) {

	// Create a new rate-limited request.
	auto request = std::make_unique<RateLimitedRequest>(endpoint, network_request, request_callback);

	// Handle requests that are not rate limited.
	if (reply->hasRawHeader("X-Rate-Limit-Policy") == false) {
		QLOG_DEBUG() << "The endpoint is not rate-limited:" << endpoint;
		endpoint_mapping[endpoint] = &default_manager;
		default_manager.endpoints.push_back(endpoint);
		default_manager.QueueRequest(std::move(request));
		return;
	}

	// Parse the rate limit policy from the reply headers.
	const QString policy_name = GetRateLimitPolicy(reply);
	QLOG_DEBUG() << policy_name << "encountered.";

	// Check to see if another manager already exists for this policy.
	if (policy_mapping.count(policy_name) > 0) {
		QLOG_DEBUG() << policy_name << "now handles" << endpoint;
		auto it = policy_mapping.find(policy_name);
		if (it != policy_mapping.end()) {
			auto manager = it->second.get();
			endpoint_mapping[endpoint] = manager;
			manager->endpoints.push_back(endpoint);
			manager->policy = std::make_unique<Policy>(reply);
			manager->OnPolicyUpdate();
			manager->QueueRequest(std::move(request));
		}
		return;
	};

	// Create a new policy manager.
	QLOG_DEBUG() << policy_name << "created for" << endpoint;
	auto policy = std::make_unique<Policy>(reply);
	auto manager = std::make_unique<PolicyManager>(this, std::move(policy));
	connect(manager.get(), &PolicyManager::RequestReady, this, &RateLimiter::SendRequest);
	connect(manager.get(), &PolicyManager::RateLimitingStarted, this, &RateLimiter::OnTimerStarted);
	connect(manager.get(), &PolicyManager::PolicyUpdated, this, &RateLimiter::OnPolicyUpdated);
	emit PolicyUpdated(*manager->policy);
	manager->endpoints.push_back(endpoint);
	manager->QueueRequest(std::move(request));
	endpoint_mapping[endpoint] = manager.get();
	policy_mapping.emplace(policy_name, std::move(manager));

	// Since we discovered a new rate limit policy, let's update the status.
	DoStatusUpdate();
}

void RateLimiter::OnTimerStarted() {

	// Make sure the program status is being updated.
	if (status_updater.isActive() == false) {
		QLOG_DEBUG() << "Starting rate limit status updates";
		status_updater.start();
	};
}

void RateLimiter::DoStatusUpdate()
{
	bool rate_limited = false;
	QStringList lines;

	QString limiting_policy;
	QDateTime limiting_send = QDateTime();

	// Check to see if any of the policy managers are busy.
	for (auto& pair : policy_mapping) {
		auto& manager = *pair.second;
		if (manager.IsBusy()) {
			if ((limiting_send.isValid() == false) || (manager.next_send < limiting_send)) {
				limiting_policy = manager.policy_name;
				limiting_send = manager.next_send;
			};
			rate_limited = true;
		};
	};
	if (limiting_send < QDateTime::currentDateTime()) {
		rate_limited = false;
	};

	if (rate_limited == false) {
		// Stop the status updates if it's running and nobody is busy.
		if (status_updater.isActive()) {
			QLOG_DEBUG() << "Stopping rate limit status updates";
			status_updater.stop();
		};
		emit StatusUpdate("Not rate limited.");
	} else {
		// Start the timer if it's not running and someone is busy.
		// (This should probably never happen, but I"ve seen it).
		if (status_updater.isActive() == false) {
			QLOG_WARN() << "The rate limiter is busy, but the status update timer was not running";
			status_updater.start();
		};
		long long msec = QDateTime::currentDateTime().msecsTo(limiting_send);
		const QString msg = (msec > 3000)
			? QString("Paused %1 seconds for \"%2\" policy.").arg(msec / 2000).arg(limiting_policy)
			: QString("Paused %1 ms for \"%2\" policy.").arg(msec).arg(limiting_policy);
		emit StatusUpdate(msg);
	};
}

void RateLimiter::OnUpdateRequested()
{
	for (auto& pair : policy_mapping) {
		emit PolicyUpdated(*pair.second->policy);
	}
	DoStatusUpdate();
}
