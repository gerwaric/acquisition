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

#include <array>
#include <boost/circular_buffer.hpp>
#include <deque>
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

	//=========================================================================================
	// Constants
	//=========================================================================================

	// This HTTP status code means there was a rate limit violation.
	const int VIOLATION_STATUS = 429;

	// A delay added to make sure we don't get a violation.
	const int SAFETY_BUFFER_MSEC = 1000;

	// Minium time between sends for any given policy.
	const int MINIMUM_INTERVAL_MSEC = 500;

	// When there is a violation, add this much time to how long we
	// wait just to make sure we don't trigger another violation.
	const int VIOLATION_BUFFER_MSEC = 1000;

	//=========================================================================================
	// Rate Limited Requests
	//=========================================================================================

	// Request callback functions should look like "void Foo(QNetworkReply*) {...};". The pointer
	// to the QNetworkReply will be deleted after the callback is complete using deleteLater().
	typedef std::function<void(QNetworkReply*)> Callback;

	// Represents a single rate-limited request.
	struct RateLimitedRequest {

		// Construct a new rate-limited request.
		RateLimitedRequest(const QString& endpoint, const QNetworkRequest& request, const Callback callback);

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

	typedef boost::circular_buffer<QDateTime> RequestHistory;

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
	private:
		int hits_;
		int period_;
		int restriction_;
	};

	class RuleItem {
	public:
		RuleItem(const QByteArray& limit_fragment, const QByteArray& state_fragment);
		const RuleItemData& limit() const { return limit_; };
		const RuleItemData& state() const { return state_; };
		PolicyStatus status() const { return status_; };
		QDateTime GetNextSafeSend(const RequestHistory& history) const;
	private:
		RuleItemData limit_;
		RuleItemData state_;
		PolicyStatus status_;
	};

	class PolicyRule {
	public:
		PolicyRule(const QByteArray& rule_name, QNetworkReply* const reply);
		const QString& name() const { return name_; };
		const std::vector<RuleItem>& items() const { return items_; };
		PolicyStatus status() const { return status_; };
		int maximum_hits() const { return maximum_hits_; };
	private:
		QString name_;
		std::vector<RuleItem> items_;
		PolicyStatus status_;
		int maximum_hits_;
	};

	class Policy {
	public:
		Policy();
		void Update(QNetworkReply* const reply);
		const QString& name() const { return name_; };
		const std::vector<PolicyRule>& rules() const { return rules_; };
		PolicyStatus status() const { return status_; };
		int maximum_hits() const { return maximum_hits_; };
		QDateTime GetNextSafeSend(const RequestHistory& history);
	private:
		QString name_;
		std::vector<PolicyRule> rules_;
		PolicyStatus status_;
		int maximum_hits_;
	};

	//=========================================================================================
	// Rate Limit Policy Managers
	//=========================================================================================

	class PolicyManager : public QObject {
		Q_OBJECT

	public:
		PolicyManager(QObject* parent = nullptr);

		// Move a request into to this manager's queue.
		void QueueRequest(std::unique_ptr<RateLimitedRequest> request);

		void Update(QNetworkReply* reply);

		const Policy& policy() const { return policy_; };

		const QDateTime& next_send() const { return next_send_; };

		bool isActive() const { return !requests_.empty(); };

	signals:
		// Emitted when a network request is ready to go.
		void RequestReady(PolicyManager* manager, QNetworkRequest request);

		// Emmitted when a reply is recieved; used to update the rate limiter's status.
		//void ReplyReceived(PolicyManager* manager);

		// Emitted when the underlying policy has been updated.
		void PolicyUpdated(const Policy& policy);

	public slots:
		// Called when a reply has been received. Checks for errors. Updates the
		// rate limit policy if one was received. Puts the response in the
		// dispatch queue for callbacks. Checks to see if another request is
		// waiting to be activated.
		void ReceiveReply();

	private slots:

		// Sends the currently active request and connects it to ReceiveReply().
		void SendRequest();

	private:

		// Called right after active_request is loaded with a new request. This
		// will determine when that request can be sent and setup the active
		// request timer to send that request after a delay.
		void ActivateRequest();

		// Resends the active request after a delay due to a violation.
		void ResendAfterViolation();

		// Keep a unique_ptr to the policy associated with this manager,
		// which will be updated whenever a reply with the X-Rate-Limit-Policy
		// header is received.
		Policy policy_;

		// Used to send requests after a delay.
		QTimer activation_timer_;

		// Requests that are waiting to be activated.
		std::deque<std::unique_ptr<RateLimitedRequest>> requests_;

		std::unique_ptr<RateLimitedRequest> active_request_;

		// When a reply is recieved and the policy state has been updated or a 
		// rate violation has been detected, the next possible send time is calculated
		// and stored here.
		QDateTime next_send_;

		// Store the time of the last send for this policy, just so we can have an
		// extra check to make sure we don't flood GGG with requests.
		QDateTime last_send_;

		// We use a history of the received reply times so that we can calculate
		// when the next safe send time will be. This allows us to calculate the
		// least delay necessary to stay compliant.
		//
		// A circular buffer is used because it's fast to access, and the number
		// of items we have to store only changes when a rate limit policy
		// changes, which should not happen regularly, but we handle that case, too.
		RequestHistory history_;
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

		const int UPDATE_INTERVAL_MSEC = 1000;

	public slots:
		// Used by the GUI to request a manual refresh.
		void OnUpdateRequested();

	signals:
		// Emitted when one of the policy managers has signalled a policy update.
		void PolicyUpdate(const Policy& policy);

		// Signal sent to the UI so the user can see what's going on.
		void Paused(int seconds, const QString& policy_name);

	private slots:
		// Called when a policy manager has a request for us to send.
		void SendRequest(PolicyManager* manager, QNetworkRequest request);

		void SendStatusUpdate();

		// Received signals from the policy managers.
		void OnPolicyUpdated(const Policy& policy);

	private:
		// Process the first request for an endpoint we haven't encountered before.
		void SetupEndpoint(const QString& endpoint, QNetworkRequest network_request, Callback request_callback, QNetworkReply* reply);

		PolicyManager& GetManager(const QString& endpoint, const QString& policy_name);

		// Reference to the Application's network access manager.
		QNetworkAccessManager& network_manager_;

		// Reference to the Application's OAuth manager.
		OAuthManager& oauth_manager_;

		std::list<PolicyManager> managers_;
		std::map<const QString, PolicyManager&> manager_by_policy_;
		std::map<const QString, PolicyManager&> manager_by_endpoint_;

		QTimer update_timer_;
	};
}
