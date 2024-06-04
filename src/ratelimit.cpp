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

RuleItemData::RuleItemData(const QByteArray& header_fragment) :
	hits_(-1),
	period_(-1),
	restriction_(-1)
{
	const QByteArrayList parts = header_fragment.split(':');
	hits_ = parts[0].toInt();
	period_ = parts[1].toInt();
	restriction_ = parts[2].toInt();
}

RuleItemData::operator QString() const {
	return QString("%1:%2:%3").arg(
		QString::number(hits_),
		QString::number(period_),
		QString::number(restriction_));
}

RuleItem::RuleItem(const QByteArray& limit_fragment, const QByteArray& state_fragment) :
	limit_(limit_fragment),
	state_(state_fragment)
{
	if ((state_.period() != limit_.period())) {
		status_ = PolicyStatus::INVALID;
	} else if (state_.hits() > limit_.hits()) {
		status_ = PolicyStatus::VIOLATION;
	} else if (state_.hits() >= (limit_.hits() - BORDERLINE_REQUEST_BUFFER)) {
		status_ = PolicyStatus::BORDERLINE;
	} else {
		status_ = PolicyStatus::OK;
	};
}

QDateTime RuleItem::NextSafeRequest(const RequestHistory& history) const {

	// We can send immediately if the policy is not borderline or in violation.
	if (status_ < PolicyStatus::BORDERLINE) {
		return QDateTime::currentDateTime().toLocalTime();
	};

	// Determine how far back into the history we can look.
	size_t n = limit_.hits();
	if (n > history.size()) {
		n = history.size();
	};

	// Start with the timestamp of the earliest known
	// reply relevant to this limitation.
	QDateTime start;
	if (n < 1) {
		start = QDateTime::currentDateTime().toLocalTime();
	} else {
		start = history[n - 1];
	};

	// Calculate the next time it will be safe to send a request.
	return start.addSecs(limit_.period());
}

RuleItem::operator QString() const {
	return QString("%1/%2:%3:%4").arg(
		QString::number(state_.hits()),
		QString::number(limit_.hits()),
		QString::number(limit_.period()),
		QString::number(limit_.restriction()));
}

PolicyRule::PolicyRule(const QByteArray& rule_name, QNetworkReply* const reply) :
	name_(QString(rule_name)),
	status_(PolicyStatus::UNKNOWN)
{
	const QByteArrayList limit_fragments = ParseRateLimit(reply, rule_name);
	const QByteArrayList state_fragments = ParseRateLimitState(reply, rule_name);
	const int item_count = limit_fragments.size();
	items_.reserve(item_count);
	for (int j = 0; j < item_count; ++j) {
		RuleItem item(limit_fragments[j], state_fragments[j]);
		if (status_ < item.status()) {
			status_ = item.status();
		};
		items_.push_back(item);
	};
}

PolicyRule::operator QString() const {
	QStringList list;
	list.reserve(items_.size());
	for (const RuleItem& item : items_) {
		list.push_back(QString(item));
	};
	return QString("%1: %2").arg(name_, list.join(", "));
}

Policy::Policy() :
	name_("<<EMPTY-POLICY>>"),
	empty_(true),
	status_(PolicyStatus::UNKNOWN),
	max_hits_(0),
	min_delay_msec_(99999999)
{};

Policy::Policy(QNetworkReply* const reply) :
	name_(ParseRateLimitPolicy(reply)),
	empty_(false),
	status_(PolicyStatus::UNKNOWN),
	max_hits_(0),
	min_delay_msec_(99999999)
{
	const QByteArrayList names = ParseRateLimitRules(reply);

	rules_.reserve(names.size());

	// Iterate over all the rule names expected.
	for (const auto& name : names) {

		// Parse the next rule.
		const PolicyRule rule(name, reply);

		// Update the maximum number of hits and minimum delay.
		for (const auto& item : rule.items()) {
			const int hits = item.limit().hits();
			if (max_hits_ < hits) {
				max_hits_ = hits;
			};
			const int period = item.limit().period();
			const int delay_msec = (period * 1000) / hits;
			if (min_delay_msec_ > delay_msec) {
				min_delay_msec_ = delay_msec;
			};
		};

		// Check the status of the new rule.
		const PolicyStatus status = rule.status();
		if (status == PolicyStatus::INVALID) {
			QLOG_ERROR() << "Invalid rate limit state:" << name_ + "(" + rule.name() + ")";
		} else if (status == PolicyStatus::VIOLATION) {
			QLOG_ERROR() << "Rate limit violation detected:" << name_ + "(" + rule.name() + ")";
		} else if (status == PolicyStatus::BORDERLINE) {
			QLOG_WARN() << "Rate limit is borderline:" << name_ + "(" + rule.name() + ")";
		};
		if (status_ < status) {
			status_ = status;
		};

		// Add this rule to the list.
		rules_.push_back(rule);
	};
}

QDateTime Policy::NextSafeRequest(const RequestHistory& history) const {
	QDateTime result = QDateTime::currentDateTime().toLocalTime();
	for (const auto& rule : rules_) {
		for (const auto& item : rule.items()) {
			const QDateTime t = item.NextSafeRequest(history);
			if (result < t) {
				result = t;
			};
		};
	};
	return result;
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
{}

//=========================================================================================
// Policy Manager
//=========================================================================================

// Create a new rate limit manager based on an existing policy.
PolicyManager::PolicyManager(QNetworkReply* reply, QObject* parent) :
	QObject(parent),
	policy(std::make_unique<Policy>(reply)),
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
	if (history.capacity() < policy->max_hits()) {
		QLOG_DEBUG() << policy->name()
			<< "increasing history capacity"
			<< "from" << history.capacity()
			<< "to" << policy->max_hits();
		history.set_capacity(policy->max_hits());
	};

	QDateTime t = policy->NextSafeRequest(history);

	if (next_send < t) {
		// Update this manager's send time only if it's later
		// than the manager thinks we need to wait.
		QLOG_TRACE() << "Updating next send:"
			<< "\n\tfrom" << next_send.toLocalTime().toString()
			<< "\n\tto  " << t.toLocalTime().toString();
		next_send = t;
	};
}

// If the rate limit manager is busy, the request will be queued.
// Otherwise, the request will be sent immediately, making the
// manager busy and causing subsequent requests to be queued.
void PolicyManager::QueueRequest(std::unique_ptr<RateLimitedRequest> request) {
	if (busy) {
		QLOG_TRACE() << policy->name() << "queuing request" << request->id;
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
	if (msec_delay < policy->min_delay_msec()) {
		msec_delay = policy->min_delay_msec();
	};

	if (last_send.isValid()) {
		const QDateTime min_send = last_send.addMSecs(MINIMUM_INTERVAL_MSEC);
		const int min_delay = now.msecsTo(min_send);
		if (min_delay > msec_delay) {
			msec_delay = min_delay;
		};
	};

	// Need to wait and rerun this function when it's safe to send.
	QLOG_TRACE() << policy->name()
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

	QLOG_TRACE() << policy->name()
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
	active_request->reply_time = ParseDate(reply).toLocalTime();
	active_request->reply_status = ParseStatus(reply);

	QLOG_TRACE() << policy->name()
		<< "received reply for request" << active_request->id
		<< "with status" << active_request->reply_status;

	if (reply->hasRawHeader("X-Rate-Limit-Policy")) {

		const QString reply_policy_name = reply->rawHeader("X-Rate-Limit-Policy");

		// Check the rate limit policy name from the header.
		if (policy->name() != reply_policy_name) {
			QLOG_ERROR() << "policy manager for" << policy->name()
				<< "received header reply with" << reply_policy_name;
		};

		// Save the reply time if this was not a cached reply.
		history.push_front(active_request->reply_time);

		// Read the updated policy limits and state from the network reply.
		policy = std::make_unique<Policy>(reply);

		// Now examine the new policy and update ourselves accordingly.
		OnPolicyUpdate();

		// Alert listeners like the rate limiter
		emit ReplyReceived();

	} else {
		if (policy->empty() == false) {
			QLOG_ERROR() << "policy manager for" << policy->name()
				<< "received a reply without a rate limit policy";
		};
	};

	// Check for errors before dispatching the request
	if (active_request->reply_status == RATE_LIMIT_VIOLATION_STATUS) {

		// There was a rate limit violation.
		ResendAfterViolation();

	} else if (active_request->network_reply->error() != QNetworkReply::NoError) {

		// Some other HTTP error was encountered.
		QLOG_ERROR() << "policy manager for" << policy->name()
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
	QLOG_ERROR() << policy->name()
		<< "RATE LIMIT VIOLATION on request" << active_request->id << "of" << delay_sec << "seconds";
	for (const auto& header : active_request->network_reply->rawHeaderPairs()) {
		QLOG_DEBUG() << header.first << "=" << header.second;
	};

	// Update the time it will be safe to send again.
	next_send = active_request->reply_time.addMSecs(delay_msec);
	if (next_send.isValid() == false) {
		QLOG_DEBUG() << "policy manager for" << policy->name()
			<< "\n\tnext_send after violation is invalid:"
			<< "\n\t" << "request id" << active_request->id
			<< "\n\t" << "request endpoint" << active_request->endpoint
			<< "\n\t" << "Retry-After" << active_request->network_reply->rawHeader("Retry-After")
			<< "\n\t" << "reply time was" << active_request->reply_time.toLocalTime().toString();
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
		Util::toString(policy->status()),
		QString::number(request_queue.size()));

	const int delay = GetPauseDuration();

	QStringList lines;
	lines.push_back(policy->name());
	const std::vector<PolicyRule>& rules = policy->rules();
	for (auto& rule : rules) {
		lines.push_back(QString("( %1 )").arg(QString(rule)));
	};
	lines.push_back(info);

	switch (policy->status()) {
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
	// Setup the status update timer.
	status_updater.setSingleShot(false);
	status_updater.setInterval(1000);
	connect(&status_updater, &QTimer::timeout, this, &RateLimiter::DoStatusUpdate);
}

void RateLimiter::SendRequest(QNetworkRequest request) {
	PolicyManager* manager = qobject_cast<PolicyManager*>(sender());
	if (manager->policy->empty() == false) {
		if (oauth_manager_.access_token() != nullptr) {
			oauth_manager_.addAuthorization(request);
		};
	};
	QNetworkReply* reply = network_manager_.get(request);
	connect(reply, &QNetworkReply::finished, manager, &PolicyManager::ReceiveReply);
}

void RateLimiter::Submit(const QString& endpoint, QNetworkRequest network_request, Callback request_callback)
{
	// Make sure the user agent is set according to GGG's guidance.
	network_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);

	if (endpoint_mapping.count(endpoint) > 0) {

		// This endpoint is known to use an existing policy manager.
		auto request = std::make_unique<RateLimitedRequest>(network_request, request_callback);
		PolicyManager& manager = *endpoint_mapping[endpoint];
		QLOG_DEBUG() << manager.policy->name() << "is handling" << endpoint;
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
	auto request = std::make_unique<RateLimitedRequest>(network_request, request_callback);

	// Get a reference to the endpoint since we use it a bunch.
	const QString endpoint = request->endpoint;

	// Handle requests that are not rate limited.
	if (reply->hasRawHeader("X-Rate-Limit-Policy") == false) {
		QLOG_ERROR() << "The endpoint is not rate-limited:"
			<< endpoint + " (" + network_request.url().toString() + ")";
		return;
	}

	// Parse the rate limit policy from the reply headers.
	const QString policy_name = ParseRateLimitPolicy(reply);
	QLOG_DEBUG() << policy_name << "encountered.";

	if (policy_mapping.count(policy_name) > 0) {

		// A policy manager already exists for this endpoint.
		QLOG_DEBUG() << "Using existing rate limit policy" << policy_name << "for" << endpoint;
		std::shared_ptr<PolicyManager> manager = policy_mapping[policy_name];
		endpoint_mapping[endpoint] = manager;
		manager->endpoints.push_back(endpoint);
		manager->policy = std::make_unique<Policy>(reply);
		manager->OnPolicyUpdate();
		manager->QueueRequest(std::move(request));

	} else {

		// Create a new policy manager.
		QLOG_DEBUG() << "Creating rate limit policy" << policy_name << "for" << endpoint;
		auto manager = std::make_shared<PolicyManager>(reply, this);
		connect(manager.get(), &PolicyManager::RequestReady, this, &RateLimiter::SendRequest);
		connect(manager.get(), &PolicyManager::ReplyReceived, this, &RateLimiter::DoStatusUpdate);
		connect(manager.get(), &PolicyManager::RateLimitingStarted, this, &RateLimiter::OnTimerStarted);

		for (auto& rule : manager->policy->rules()) {
			QLOG_DEBUG() << manager->policy->name() + "(" + rule.name() + ")" << "is" << QString(rule);
		};

		endpoint_mapping[endpoint] = manager;
		policy_mapping[policy_name] = manager;
		manager->endpoints.push_back(endpoint);
		manager->QueueRequest(std::move(request));

		// Since we discovered a new rate limit policy, let's update the status.
		DoStatusUpdate();
	};
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
		auto& manager = pair.second;
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
