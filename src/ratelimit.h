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

#pragma once

#include <boost/circular_buffer.hpp>
#include <unordered_map>

#include <QDateTime>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QObject>
#include <QTimer>

class Application;
class OAuthManager;

//--------------------------------------------------------------------------
// Introduction to GGG's API Rate Limits
//--------------------------------------------------------------------------
// 
// As of August 2023, GGG has implemented "new" rate limit policies that are
// incompatible with how acquisition used to throttle requests. There was no
// obvious way to fix the network request code used by the ItemsManagerWorker.
// Instead, what I've done instead is create a wrapper that accepts network
// requests from the ItemsManagerWorker and hidden the implementation details.
// 
// See https://www.pathofexile.com/developer/docs for more details on the rate
// limit information that is included in the HTTP headers of the network reply
// to every API request.
//
// Those rate limit policies can change at any time, from one network
// call to the next. For example, GGG might decide to temporarily tighten rate
// limitations around a league launch.
// 
// This means that any solution that hard-codes things like delays will
// eventually break.
// 
// What this means for us is that we have to check the reply to every
// network request for the details of rate-limitation policy that were applied.
// 
// Each policy can have mulitple rules that apply to it. Each rule
// can have multiple elments. Each element defines the specific number of 
// API hits that are allowed, the period within which those hits are measured,
// and the timeout restriction if that item is violated.
// 
// The RateLimiter class defined below takes care of all of that.
// 
// Specifically, there are a number of helper functions and classes in
// ratelimit.cpp that are used to keep track of everything, limit network
// request as little as possible, and do all of that transparently, so that
// ItemsManagerWorker or the rest of the application don't need to be changed.
// 
// More specifically, the RateLimiter::Submit method takes a network request
// and a callback function. See the code in itemsmanagerworker.cpp for
// examples of how this is used.
// 
// Submitted requests are sent out serially. No request is sent until a reply
// to the previous request is received. If a rate limit violation is detected,
// a request will be resent after the required delay. This allows the wrapper
// to monitor the exact state of all the rate-limit policies and inject delays
// as necessary to avoid violating rate limit policies.
// 
// This approach also alows us to forgo hardcoding anything about the rate
// limits in the source code. Instead, everything about the rate limits is
// take from the relevant HTTP headers in each network reply.
//
// WARNINGS:
//
//      This code may also break where there are other sources of rate-limited
//      API requests, e.g. if someone is using two tools on the same computer
//      with the same account.

namespace RateLimit
{
	Q_NAMESPACE;

	enum class RateLimitStatus { OK, PAUSED };
	Q_ENUM_NS(RateLimitStatus);

	struct StatusInfo {
		RateLimitStatus status{ RateLimitStatus::OK };
		int duration{ 0 };
		QString message;
	};

	typedef boost::circular_buffer<QDateTime> RequestHistory;

	//=========================================================================================
	// Constants
	//=========================================================================================

	// The API endpoints used by acquisition that need to be checked for rate limits. Since
	// this is known at compile time, we don't have to anything dynamic here.
	extern const QStringList KNOWN_ENDPOINTS;

	// Consider a policy "borderline" when there are this many requests left before violation.
	// This gives us a small buffer, just in case.
	const int BORDERLINE_REQUEST_BUFFER = 0;

	// This HTTP status code means there was a rate limit violation.
	const int RATE_LIMIT_VIOLATION_STATUS = 429;

	// A delay added to make sure we don't get a violation.
	const int SAFETY_BUFFER_MSEC = 1000;

	// Minium time between sends for any given policy.
	const int MINIMUM_INTERVAL_MSEC = 100;

	// Minimum time to send after activating a request.
	const int MINIMUM_ACTIVATION_DELAY_MSEC = 100;

	// When there is a violation, add this much time to how long we
	// wait just to make sure we don't trigger another violation.
	const int EXTRA_RATE_VIOLATION_MSEC = 1000;

	//=========================================================================================
	// Rate Limited Requests
	//=========================================================================================

	// Request callback functions should look like "void Foo(QNetworkReply*) {...};". The pointer
	// to the QNetworkReply will be deleted after the callback is complete using deleteLater().
	typedef std::function<void(QNetworkReply*)> Callback;

	// Represents a single rate-limited request.
	struct RateLimitedRequest {

		// Construct a new rate-limited request.
		RateLimitedRequest(const QNetworkRequest& request, const Callback callback);

		// Unique identified for each request, even through different requests can be
		// routed to different policy managers based on different endpoints.
		const unsigned long id;

		// A copy of the network request that's going to be sent.
		const QNetworkRequest network_request;

		// The function to be called when a non-rate-limit reply is recieved.
		const Callback worker_callback;

		// A copy of this request's API endpoint, if any.
		const QString endpoint;

		// Pointer to network reply that will contain the http repsonse.
		QNetworkReply* network_reply;

		// The time a reply was made based on the reply's HTTP Date header.
		QDateTime reply_time;

		// The HTTP status of the reply.
		int reply_status;

	private:

		// Total number of requests that have every been constructed.
		static unsigned long request_count;
	};

	enum class PolicyStatus { UNKNOWN, OK, BORDERLINE, VIOLATION, INVALID };
	Q_ENUM_NS(PolicyStatus);

	//=========================================================================================
	// Next, declarations for the classes that represent a rate-limit policy
	//=========================================================================================
	//
	// Each API response has a rate-limit policy that applies to it.
	// Those responses are present in the HTTP reply headers. Here's
	// how they are concieved, briefly:
	// 
	//  Every endpoint only has one applicable policy.
	//  Different endpoints may share the same rate limit policy.
	// 
	//  A policy has a name.
	//  A policy has one or more rules.
	//  A policy applies to one or more endpoints.
	// 
	//  Each rule has a name.
	//  Each rule has one or more items.
	// 
	//  Each item has data that defines one set of limits.
	//  Each item has data on the state of those limts.
	//
	// For any request against a rate-limited endpoint, only one policy applies, but
	// all of limitations for each item of every rule within that policy are checked.

	class RuleItemData {
	public:
		RuleItemData(const QByteArray& header_fragment);
		int hits() const { return hits_; };
		int period() const { return period_; };
		int restriction() const { return restriction_; };
		operator QString() const;
	private:
		int hits_;
		int period_;
		int restriction_;
	};

	// Both RuleItemData and PolicyRule have default constructors because
	// they are the two policy-related classes where it's useful to preallocate
	// arrays of a known size.

	class RuleItem {
	public:
		RuleItem(const QByteArray& limit_fragment, const QByteArray& state_fragment);
		const RuleItemData& limit() const { return limit_; };
		const RuleItemData& state() const { return state_; };
		PolicyStatus status() const { return status_; };
		QDateTime NextSafeRequest(const RequestHistory& history) const;
		operator QString() const;
	private:
		RuleItemData limit_;
		RuleItemData state_;
		PolicyStatus status_;
	};

	class PolicyRule {
	public:
		PolicyRule(const QByteArray& rule_name, QNetworkReply* const reply);
		QString name() const { return name_; };
		const std::vector<RuleItem>& items() const { return items_; };
		PolicyStatus status() const { return status_; };
		operator QString() const;
	private:
		QString name_;
		std::vector<RuleItem> items_;
		PolicyStatus status_;
	};

	class Policy {
	public:
		Policy();
		Policy(QNetworkReply* const reply);
		QString name() const { return name_; };
		bool empty() const { return empty_; };
		const std::vector<PolicyRule>& rules() const { return rules_; };
		PolicyStatus status() const { return status_; };
		int max_hits() const { return max_hits_; };
		int min_delay_msec() const { return min_delay_msec_; };
		QDateTime NextSafeRequest(const RequestHistory& history) const;
	private:
		QString name_;
		const bool empty_;
		std::vector<PolicyRule> rules_;
		size_t max_hits_;
		size_t min_delay_msec_;
		PolicyStatus status_;
	};

	//=========================================================================================
	// Rate Limit Policy Managers
	//=========================================================================================

	class PolicyManager : public QObject {
		Q_OBJECT

	public:
		PolicyManager(QNetworkReply* reply, QObject* parent = nullptr);

		// Move a request into to this manager's queue.
		void QueueRequest(std::unique_ptr<RateLimitedRequest> request);

		// Keep a unique_ptr to the policy associated with this manager,
		// which will be updated whenever a reply with the X-Rate-Limit-Policy
		// header is received.
		std::unique_ptr<Policy> policy;

		// List of the API endpoints this manager's policy applies to.
		QStringList endpoints;

		// Return the status of this rate limit manager.
		PolicyStatus GetStatus() const { return policy->status(); };

		// Return the number of seconds this manager is paused.
		int GetPauseDuration() const;

		// Return a long string with the details of this manager's status.
		QString GetStatusMessage() const;

		// This is called whenever the policy is updated, which happens either
		// at construction or when a QNetworkReply with a X-Rate-Limit-Policy
		// header is received.
		void OnPolicyUpdate();

	signals:
		// Emitted when a network request is ready to go.
		void RequestReady(QNetworkRequest request);

		// Emmitted when a reply is recieved; used to update the rate limiter's status.
		void ReplyReceived();

		// Emitted when this policy manager pauses due to rate limiting.
		void RateLimitingStarted();

	public slots:
		// Called when a reply has been received. Checks for errors. Updates the
		// rate limit policy if one was received. Puts the response in the
		// dispatch queue for callbacks. Checks to see if another request is
		// waiting to be activated.
		void ReceiveReply();

	private:

		// Called right after active_request is loaded with a new request. This
		// will determine when that request can be sent and setup the active
		// request timer to send that request after a delay.
		void ActivateRequest();

		// Sends the currently active request and connects it to ReceiveReply().
		void SendRequest();

		// Resends the active request after a delay due to a violation.
		void ResendAfterViolation();

		// Keep track of wether or not there's an active request keeping this
		// policy manager busy.
		bool busy;

		// This time is used on a single-shot by ActivateRequest() to send
		// requests with a delay.
		QTimer active_request_timer;

		// The currently active request.
		std::unique_ptr<RateLimitedRequest> active_request;

		// Requests that are waiting to be activated.
		std::list<std::unique_ptr<RateLimitedRequest>> request_queue;

		// When a reply is recieved and the policy state has been updated or a 
		// rate violation has been detected, the next possible send time is calculated
		// and stored here.
		QDateTime next_send;

		// Store the time of the last send for this policy, just so we can have an
		// extra check to make sure we don't flood GGG with requests.
		QDateTime last_send;

		// Keep track of wether a violation was detected. Currently this is
		// only used to update the program status so the user knows wether
		// a reply is being delay to a violation or not.
		bool violation;

		// We use a history of the received reply times so that we can calculate
		// when the next safe send time will be. This allows us to calculate the
		// least delay necessary to stay compliant.
		//
		// A circular buffer is used because it's fast to access, and the number
		// of items we have to store only changes when a rate limit policy
		// changes, which should not happen regularly, but we handle that case, too.
		RequestHistory history;
	};

	//=========================================================================================
	// Top-Level RateLimiter
	//=========================================================================================

	class RateLimiter : public QObject {
		Q_OBJECT

	public:
		// Creat a rate limiter.
		RateLimiter(Application& app, QObject* parent = nullptr);

		// Submit a request-callback pair to the rate limiter. Note that the callback function
		// should not delete the QNetworkReply. That is handled after the callback finishes.
		void Submit(const QString& endpoint, QNetworkRequest network_request, Callback request_callback);

	public slots:
		// Slot for policy managers to send signals when they begin rate limiting.
		void OnTimerStarted();

	signals:
		// Signal sent to the UI so the user can see what's going on.
		void StatusUpdate(const StatusInfo& update);

	private slots:
		// Called when a policy manager has a request for us to send.
		void SendRequest(QNetworkRequest request);

	private:
		// Process the first request for an endpoint we haven't encountered before.
		void SetupEndpoint(QNetworkRequest network_request, Callback request_callback, QNetworkReply* reply);

		// Reference to the Application's network access manager.
		QNetworkAccessManager& network_manager_;

		// Reference to the Application's OAuth manager.
		OAuthManager& oauth_manager_;

		// Map endpoints to policy managers.
		std::unordered_map<QString, std::shared_ptr<PolicyManager>> endpoint_mapping;

		// Map policy names to policy managers.
		std::unordered_map<QString, std::shared_ptr<PolicyManager>> policy_mapping;

		// This timer updates the rate limit status.
		QTimer status_updater;

		// Called by the timer to emit status updates.
		void DoStatusUpdate();
	};
}
