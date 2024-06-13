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
// Local functions
//=========================================================================================

// Get a header field from an HTTP reply.
QByteArray ParseHeader(QNetworkReply* const reply, const QByteArray& name) {
	if (!reply->hasRawHeader(name)) {
		QLOG_ERROR() << "GetHeader(): missing header:" << name;
	};
	return reply->rawHeader(name);
}

// Get a header field and split into a list.
QByteArrayList ParseHeaderList(QNetworkReply* const reply, const QByteArray& name, const char delim) {
	const QByteArray value = ParseHeader(reply, name);
	const QByteArrayList items = value.split(delim);
	if (items.isEmpty()) {
		QLOG_ERROR() << "GetHeaderList():" << name << "is empty";
	};
	return items;
}

// Return the name of the policy from a network reply.
QByteArray ParseRateLimitPolicy(QNetworkReply* const reply) {
	return ParseHeader(reply, "X-Rate-Limit-Policy");
}

// Return the name(s) of the rule(s) from a network reply.
QByteArrayList ParseRateLimitRules(QNetworkReply* const reply) {
	return ParseHeaderList(reply, "X-Rate-Limit-Rules", ',');
}

// Return a list of one or more items that define a rule's limits.
QByteArrayList ParseRateLimit(QNetworkReply* const reply, const QByteArray& rule) {
	return ParseHeaderList(reply, "X-Rate-Limit-" + rule, ',');
}

// Return a list of one or more items that define a rule's current state.
QByteArrayList ParseRateLimitState(QNetworkReply* const reply, const QByteArray& rule) {
	return ParseHeaderList(reply, "X-Rate-Limit-" + rule + "-State", ',');
}

// Return the date from the HTTP reply headers.
QDateTime ParseDate(QNetworkReply* const reply) {
	const QString timestamp = QString(Util::FixTimezone(ParseHeader(reply, "Date")));
	const QDateTime date = QDateTime::fromString(timestamp, Qt::RFC2822Date).toLocalTime();
	if (!date.isValid()) {
		QLOG_ERROR() << "invalid date parsed from" << timestamp;
	};
	return date;
}

// Return the HTTP status from the reply headers.
int ParseStatus(QNetworkReply* const reply) {
	return reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
}

void Dispatch(std::unique_ptr<RateLimitedRequest> request);

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


RuleItem::RuleItem(const QByteArray& limit_fragment, const QByteArray& state_fragment) :
	limit_(limit_fragment),
	state_(state_fragment)
{
	// Determine the status of this item.
	if (state_.period() != limit_.period()) {
		status_ = PolicyStatus::INVALID;
	} else if (state_.hits() > limit_.hits()) {
		status_ = PolicyStatus::VIOLATION;
	} else if (state_.hits() == limit_.hits()) {
		status_ = PolicyStatus::BORDERLINE;
	} else {
		status_ = PolicyStatus::OK;
	};
}

QDateTime RuleItem::GetNextSafeSend(const RequestHistory& history) const {

	const QDateTime now = QDateTime::currentDateTime().toLocalTime();

	// We can send immediately if the policy is not borderline or in violation.
	if (status_ < PolicyStatus::BORDERLINE) {
		return now;
	};

	// Determine how far back into the history we can look.
	const size_t n = (limit_.hits() > history.size()) ? history.size() : limit_.hits();

	// Start with the timestamp of the earliest known
	// reply relevant to this limitation.
	const QDateTime start = (n < 1) ? now : history[n - 1];

	// Calculate the next time it will be safe to send a request.
	return start.addSecs(limit_.period());
}

PolicyRule::PolicyRule(const QByteArray& rule_name, QNetworkReply* const reply) :
	name_(rule_name),
	status_(PolicyStatus::UNKNOWN),
	maximum_hits_(-1)
{
	const QByteArrayList limit_fragments = ParseRateLimit(reply, rule_name);
	const QByteArrayList state_fragments = ParseRateLimitState(reply, rule_name);
	const int item_count = limit_fragments.size();
	if (state_fragments.size() != limit_fragments.size()) {
		QLOG_ERROR() << "Invalid data for policy role.";
	};
	items_.reserve(item_count);
	for (int j = 0; j < item_count; ++j) {

		// Create a new rule item from the next pair of fragments.
		const auto& item = items_.emplace_back(limit_fragments[j], state_fragments[j]);
		
		// Keep track of the max hits, max rate, and overall status.
		if (maximum_hits_ < item.limit().hits()) {
			maximum_hits_ = item.limit().hits();
		};
		if (status_ < item.status()) {
			status_ = item.status();
		};
	};
}

Policy::Policy() :
	status_(PolicyStatus::UNKNOWN),
	maximum_hits_(-1)
{}

QDateTime Policy::GetNextSafeSend(const RequestHistory& history) {
	QDateTime next_send = QDateTime::currentDateTime().toLocalTime();
	for (const auto& rule : rules_) {
		for (const auto& item : rule.items()) {
			const QDateTime t = item.GetNextSafeSend(history);
			if (next_send < t) {
				next_send = t;
			};
		};
	};
	return next_send;
}

void Policy::Update(QNetworkReply* const reply)
{
	// Parse the name of the rate limit policy and all the rules for this reply.
	const QString policy_name = ParseRateLimitPolicy(reply);
	const QByteArrayList rule_names = ParseRateLimitRules(reply);

	// Set or check the name of the policy found in the reply.
	if (name_.isEmpty()) {
		name_ = policy_name;
	} else if (name_ != policy_name) {
		QLOG_ERROR() << "Rate limit policy name changed from" << name_ << "to" << policy_name;
	};

	// Check to see if the rules seem to have changed, too.
	if (!rules_.empty()) {
		if (rules_.size() != rule_names.size()) {
			QLOG_ERROR() << "The number rules of for rate limit policy" << name_ << "is changing.";
		} else {
			for (int i = 0; i < rule_names.size(); ++i) {
				if (rule_names[i] != rules_[i].name()) {
					QLOG_ERROR() << "Rate limit policy" << name_ << "rule" << i
						<< "name changed from" << rules_[i].name() << "to" << rule_names[i];
				};
			};
		};
	};

	// Reset the list of rules.
	rules_.clear();
	rules_.reserve(rule_names.size());

	// Iterate over all the rule names expected.
	for (const auto& rule_name : rule_names) {

		// Create a new rule and add it to the list.
		const auto& rule = rules_.emplace_back(rule_name, reply);

		// Check the status of this rule..
		if (rule.status() > PolicyStatus::OK) {
			QLOG_WARN() << "Rate limit policy" << name_ << "(" << rule.name() << ") status is" << rule.status();
		};
		
		// Update metrics for this rule.
		if (maximum_hits_ < rule.maximum_hits()) {
			maximum_hits_ = rule.maximum_hits();
		};
		if (status_ < rule.status()) {
			status_ = rule.status();
		};
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
PolicyManager::PolicyManager(QObject* parent) :
	QObject(parent),
	next_send_(QDateTime::currentDateTime().toLocalTime()),
	last_send_(QDateTime())
{
	// Setup the active request timer to call SendRequest each time it's done.
	activation_timer_.setSingleShot(true);
	connect(&activation_timer_, &QTimer::timeout, this, &PolicyManager::SendRequest);
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

// If the rate limit manager is busy, the request will be queued.
// Otherwise, the request will be sent immediately, making the
// manager busy and causing subsequent requests to be queued.
void PolicyManager::QueueRequest(std::unique_ptr<RateLimitedRequest> request) {
	requests_.push_back(std::move(request));
	if (!active_request_) {
		ActivateRequest();
	};
}

// Send the active request at the next time it will be safe to do so
// without violating the rate limit policy.
void PolicyManager::ActivateRequest() {

	if (active_request_) {
		QLOG_DEBUG() << "RateLimit::PolicyManager::ActivateRequest: a request is already active";
		return;
	};

	if (requests_.empty()) {
		QLOG_DEBUG() << "RateLimit::PolicyManager::ActivateReuqest: no requests queued";
		return;
	};

	active_request_ = std::move(requests_.front());
	requests_.pop_front();

	if (next_send_.isValid() == false) {
		QLOG_ERROR() << "RateLimit::PolicyManager::ActivateRequest(): next_send is invalid";
	};

	QDateTime send = next_send_;
	if (last_send_.isValid()) {
		if (last_send_.msecsTo(send) < MINIMUM_INTERVAL_MSEC) {
			send = last_send_.addMSecs(MINIMUM_INTERVAL_MSEC);
		};
	};

	int delay = QDateTime::currentDateTime().msecsTo(send);
	if (delay < 0) {
		delay = 0;
	};
	QLOG_TRACE() << policy_.name() << "waiting" << (delay / 1000)
		<< "seconds to send request" << active_request_->id
		<< "at" << next_send_.toLocalTime().toString();

	activation_timer_.setInterval(delay);
	activation_timer_.start();
}

// Send the active request immediately.
void PolicyManager::SendRequest() {

	if (!active_request_) {
		QLOG_DEBUG() << "RateLimit::PolicyManager: no active request to send";
		return;
	};

	const auto& request = *active_request_;

	if (request.network_reply != nullptr) {
		QLOG_ERROR() << "The network reply for the active request is not empty";
		return;
	};

	QLOG_TRACE() << policy_.name()
		<< "sending request" << request.id
		<< "to" << request.endpoint
		<< "via" << request.network_request.url().toString();

	// Finally, send the request and note the time.
	last_send_ = QDateTime::currentDateTime().toLocalTime();
	emit RequestReady(this, request.network_request);
};

// Called when the active request's reply is finished.
void PolicyManager::ReceiveReply()
{
	QNetworkReply* reply = qobject_cast<QNetworkReply*>(sender());

	// Make sure the reply has a rate-limit header.
	if (!reply->hasRawHeader("X-Rate-Limit-Policy")) {
		QLOG_ERROR() << "Received a reply for" << policy_.name() << "without rate limit headers.";
		return;
	};

	auto& request = *active_request_;
	request.network_reply = reply;
	request.reply_time = ParseDate(reply).toLocalTime();
	request.reply_status = ParseStatus(reply);

	QLOG_TRACE() << policy_.name()
		<< "received reply for request" << request.id
		<< "with status" << request.reply_status;

	// Save the reply time.
	history_.push_front(request.reply_time);

	// Now examine the new policy and update ourselves accordingly.
	Update(reply);

	// Check for errors before dispatching the request
	if (policy_.status() == PolicyStatus::VIOLATION) {

		// There was a rate limit violation.
		ResendAfterViolation();

	} else if (reply->error() != QNetworkReply::NoError) {

		// Some other HTTP error was encountered.
		QLOG_ERROR() << "policy manager for" << policy_.name()
			<< "request" << request.id
			<< "reply status was " << request.reply_status
			<< "and error was" << reply->error();

	} else {

		// Dispatch the current active request.
		Dispatch(std::move(active_request_));

		// Activate the next queued reqeust.
		ActivateRequest();

	};
}

void PolicyManager::Update(QNetworkReply* reply) {

	// Update the rate limit policy.
	policy_.Update(reply);

	// Grow the history capacity if needed.
	const int capacity = history_.capacity();
	const int max_hits = policy_.maximum_hits();
	if (capacity < max_hits) {
		QLOG_DEBUG() << policy_.name()
			<< "increasing history capacity"
			<< "from" << capacity
			<< "to" << max_hits;
		history_.set_capacity(max_hits);
	};

	const QDateTime time = policy_.GetNextSafeSend(history_);
	if (next_send_ < time) {
		// Update this manager's send time only if it's later
		// than the manager thinks we need to wait.
		QLOG_TRACE() << "Updating next send:"
			<< "from" << next_send_.toString()
			<< "to" << time.toString();
		next_send_ = time;
	};

	emit PolicyUpdated(policy_);
}


// A violation was detected, so we need to wait to resend the active request.
void PolicyManager::ResendAfterViolation()
{
	auto& request = *active_request_;

	// Determine how long we need to wait.
	const int delay_sec = request.network_reply->rawHeader("Retry-After").toInt();
	const int delay_msec = (delay_sec * 1000) + VIOLATION_BUFFER_MSEC;
	QLOG_ERROR() << policy_.name()
		<< "RATE LIMIT VIOLATION on request" << request.id
		<< "of" << delay_sec << "seconds";
	for (const auto& header : request.network_reply->rawHeaderPairs()) {
		QLOG_DEBUG() << header.first << "=" << header.second;
	};

	// Update the time it will be safe to send again.
	next_send_ = request.reply_time.addMSecs(delay_msec);
	if (next_send_.isValid() == false) {
		QLOG_DEBUG() << "policy manager for" << policy_.name()
			<< "\n\tnext_send after violation is invalid:"
			<< "\n\t" << "request id" << request.id
			<< "\n\t" << "request endpoint" << request.endpoint
			<< "\n\t" << "Retry-After" << request.network_reply->rawHeader("Retry-After")
			<< "\n\t" << "reply time was" << request.reply_time.toLocalTime().toString();
	};

	// Reset this request before resending it, which means
	// letting QT know the assocated reply can be deleted.
	request.network_reply->deleteLater();
	request.network_reply = nullptr;
	request.reply_time = QDateTime();
	request.reply_status = -1;
	ActivateRequest();
}

//=========================================================================================
// The application-facing Rate Limiter
//=========================================================================================

RateLimiter::RateLimiter(Application& app, QObject* parent) :
	QObject(parent),
	network_manager_(app.network_manager()),
	oauth_manager_(app.oauth_manager())
{
	update_timer_.setSingleShot(false);
	update_timer_.setInterval(UPDATE_INTERVAL_MSEC);
	connect(&update_timer_, &QTimer::timeout, this, &RateLimiter::SendStatusUpdate);
}

void RateLimiter::Submit(const QString& endpoint, QNetworkRequest network_request, Callback request_callback)
{
	// Make sure the user agent is set according to GGG's guidance.
	network_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);

	auto it = manager_by_endpoint_.find(endpoint);
	if (it != manager_by_endpoint_.end()) {

		// This endpoint is handled by an existing policy manager.
		PolicyManager& manager = it->second;
		QLOG_DEBUG() << manager.policy().name() << "is handling" << endpoint;
		auto request = std::make_unique<RateLimitedRequest>(endpoint, network_request, request_callback);
		manager.QueueRequest(std::move(request));

	} else {

		// This is a new endpoint.
		QNetworkReply* reply = network_manager_.head(network_request);
		connect(reply, &QNetworkReply::finished, this,
			[=]() {
				SetupEndpoint(endpoint, network_request, request_callback, reply);
			});

	};
}

void RateLimiter::SetupEndpoint(const QString& endpoint, QNetworkRequest network_request, Callback request_callback, QNetworkReply* reply) {

	// All endpoints should be rate limited.
	if (!reply->hasRawHeader("X-Rate-Limit-Policy")) {
		QString url = reply->request().url().toString();
		QLOG_ERROR() << "The endpoint is not rate-limited:" << endpoint << "(" + url + ")";
		return;
	};

	const QString policy_name = ParseRateLimitPolicy(reply);

	auto request = std::make_unique<RateLimitedRequest>(endpoint, network_request, request_callback);

	PolicyManager& manager = GetManager(endpoint, policy_name);
	manager.Update(reply);
	manager.QueueRequest(std::move(request));

}

PolicyManager& RateLimiter::GetManager(const QString& endpoint, const QString& policy_name)
{
	auto it = manager_by_policy_.find(policy_name);
	if (it == manager_by_policy_.end()) {
		// Create a new policy manager.
		QLOG_DEBUG() << "Creating rate limit policy" << policy_name << "for" << endpoint;
		PolicyManager& manager = managers_.emplace_back(this);
		connect(&manager, &PolicyManager::RequestReady, this, &RateLimiter::SendRequest);
		connect(&manager, &PolicyManager::PolicyUpdated, this, &RateLimiter::OnPolicyUpdated);
		manager_by_policy_.emplace(policy_name, manager);
		manager_by_endpoint_.emplace(endpoint, manager);
		return manager;
	} else {
		// Use an existing policy manager.
		QLOG_DEBUG() << "Using an existing rate limit policy" << policy_name << "for" << endpoint;
		PolicyManager& manager = it->second;
		manager_by_endpoint_.emplace(endpoint, manager);
		return manager;
	};
}

void RateLimiter::SendRequest(PolicyManager* manager, QNetworkRequest request) {
	if (oauth_manager_.token()) {
		const std::string bearer = "Bearer " + oauth_manager_.token().value().access_token();
		request.setRawHeader("Authorization", QByteArray::fromStdString(bearer));
	};
	QNetworkReply* reply = network_manager_.get(request);
	connect(reply, &QNetworkReply::finished, manager, &PolicyManager::ReceiveReply);
}

void RateLimiter::OnUpdateRequested()
{
	for (const auto& manager : managers_) {
		emit PolicyUpdate(manager.policy());
	};
	SendStatusUpdate();
}

void RateLimiter::OnPolicyUpdated(const Policy& policy)
{
	emit PolicyUpdate(policy);
	SendStatusUpdate();
}

void RateLimiter::SendStatusUpdate()
{
	QDateTime next_send;
	QString limiting_policy;
	for (auto& manager : managers_) {
		if (!manager.isActive()) {
			continue;
		};
		if (next_send.isValid() && (next_send <= manager.next_send())) {
			continue;
		};
		next_send = manager.next_send();
		limiting_policy = manager.policy().name();
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
			QLOG_TRACE() << "Starting status updates (" << limiting_policy << ")";
			update_timer_.start();
		};
	} else {
		if (update_timer_.isActive()) {
			QLOG_TRACE() << "Stopping status updates";
			update_timer_.stop();
		};
	};

	emit Paused(pause, limiting_policy);
}
