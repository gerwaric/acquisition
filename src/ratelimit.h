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

#pragma once

#include <boost/circular_buffer.hpp>

#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QObject>
#include <QTimer>

//--------------------------------------------------------------------------
// Introduction to GGG's API Rate Limits
//--------------------------------------------------------------------------
// 
// As of August 2023, GGG has implemented "new" rate limit policies that are
// incompatible with how acquisition used to throttle requests. There was no
// obvious way to fix the network request code used by the ItemsManagerWorker.
// Instead, I've done instead is create a wrapper that accepts network requests
// from the ItemsManagerWorker and hidden the implementation details.
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
//      Rate limit violations may occur due to bugs, but they can also happen
//      on application startup. For example, someone runs into a rate limit
//      violation, then restarts the program to try again. If the restart
//      happens before the restriction expires, the new application instance
//      could hit the rate limt on its very first request.
// 
//      This code is "fragile". It uses asserts and throws exceptions when
//      some errors are detected. As a result, this code won't call the
//      requested callback function unless an error-freeresponse was received.
//
//      This code may also break where there are other sources of rate-limited
//      API requests. Capturing all the requests coming from one computer would
//      require hooking into the network at a deeper level than a single
//      application, but there's probably very few users with use case like this.

namespace RateLimit
{
    //=========================================================================================
    // Constants
    //=========================================================================================

    // The API endpoints used by acquisition that need to be checked for rate limits.
    const QStringList KNOWN_ENDPOINTS = {
        "https://www.pathofexile.com/character-window/get-stash-items",
        "https://www.pathofexile.com/character-window/get-items",
        "https://www.pathofexile.com/character-window/get-characters",
        "https://www.pathofexile.com/character-window/get-passive-skills",
        "https://api.pathofexile.com/leagues"};
    
    // Consider a policy "borderline" when there are this many requests left before violation.
    static const int BORDERLINE_REQUEST_BUFFER = 2;

    // This HTTP status code means there was a rate limit violation.
    static const int RATE_LIMIT_VIOLATION_STATUS = 429;

    // A delay added to make sure we don't get a violation.
    static const int SAFETY_BUFFER_MSEC = 1000;

    // Minium time between sends for any given policy.
    static const int MINIMUM_INTERVAL_MSEC = 100;

    // Minimum time to send after activating a request.
    static const int MINIMUM_ACTIVATION_DELAY_MSEC = 100;

    // When there is a violation, add this much time to how long we
    // wait just to make sure we don't trigger another violation.
    static const int EXTRA_RATE_VIOLATION_MSEC = 1000;

    //=========================================================================================
    // Rate Limited Requests
    //=========================================================================================

    // Request callback function
    typedef std::function<void(QNetworkReply*)> Callback;

    struct RateLimitedRequest {

        // Construct a new rate-limited request.
        RateLimitedRequest(QNetworkAccessManager& manager, const QNetworkRequest& request, const Callback callback);

        // Unique identified for each request, even through different requests can be
        // routed to different policy managers based on different endpoints.
        const unsigned long id;

        // A reference to the network manager used to send this request.
        QNetworkAccessManager& network_manager;

        // A copy of the network request that's going to be sent.
        const QNetworkRequest network_request;

        // The function to be called when a non-rate-limit reply is recieved.
        const Callback worker_callback;

        // A copy of this request's API endpoint, if any.
        const QString endpoint;

        // A pointer where the reply can be found after this request is sent.
        QNetworkReply* network_reply;

        // The time a reply was made based on the reply's HTTP Date header.
        QDateTime reply_time;

        // The HTTP status of the reply.
        int reply_status;

        // True is the reply came from the network cache
        bool reply_cached;

    private:

        // Total number of requests that have every been constructed.
        static unsigned long request_count;
    };

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
    //  Each rule has one or more limitation items.
    // 
    //  Each limitation item defines one set of limits
    //  Each limitation item comes with information on the state of that limitation.
    //
    // For any request against a rate-limited endpoint, only one policy applies, but
    // all of limitations for each item of every rule within that policy are checked.

    struct RuleItemData {
        RuleItemData();
        RuleItemData(const QByteArray& header_fragment);
        int hits;
        int period;
        int restriction;
        operator QString() const;
    };

    // Both RateLimitItem and RateLimitRule have default constructors because
    // they are the two policy-related classes where it's useful to preallocate
    // arrays of known size.
    // 
    // Specifically, this is used by RateLimit::Init(), by the RateLimitPolicy
    // constructor which parses a network reply.
    //
    // I avoided using default arguments because I find them confusing and 
    // more likely to result in unintended consequences.

    struct RuleItem {
        RuleItemData limit;
        RuleItemData state;
        operator QString() const;
    };

    struct PolicyRule {
        PolicyRule();
        PolicyRule(const QByteArray& rule_name, QNetworkReply* const reply);
        QString name;
        std::vector<RuleItem> items;
        operator QString() const;
    };

    enum class PolicyStatus { UNKNOWN, INVALID, OK, BORDERLINE, VIOLATION };
    static std::map<PolicyStatus, QString> POLICY_STATE = {
        {PolicyStatus::UNKNOWN,    "UNKNOWN"},
        {PolicyStatus::INVALID,    "INVALID"},
        {PolicyStatus::OK,         "OK"},
        {PolicyStatus::BORDERLINE, "BORDERLINE"},
        {PolicyStatus::VIOLATION,  "VIOLATION"}};

    struct Policy {
        Policy(const QString& name_);
        Policy(QNetworkReply* const reply);
        QString name;
        std::vector<PolicyRule> rules;
        PolicyStatus status;
        size_t max_period;
    private:
        void CheckStatus();
    };

    //=========================================================================================
    // Rate Limit Policy Managers
    //=========================================================================================

    class PolicyManager : public QObject 
    {
        Q_OBJECT
    signals:
        void TimerStarted();
    public:
        // Construct a rate limit manager with the specified policy.
        PolicyManager(std::unique_ptr<Policy>);

        // Move a request into to this manager's queue.
        void QueueRequest(std::unique_ptr<RateLimitedRequest> request);

        // Keep a unique_ptr to the policy associated with this manager,
        // which will be updated whenever a reply with the X-Rate-Limit-Policy
        // header is received.
        std::unique_ptr<Policy> policy;

        QStringList endpoints;

        bool IsBusy() const;

        QString GetCurrentStatus() const;

    private:

        // This is called whenever the policy is update, which happens either
        // at construction inside Init() or when a QNetworkReply with a
        // X-Rate-Limit-Policy header is received.
        void OnPolicyUpdate();

        // If you guessed that this sends the currently active request. The
        // active request is stored as a member variable so that it doesn't have
        // to be passed around in lambda functors, which is tough to do with
        // unique pointers, beacuse they are fussy about being copied.
        void ActivateRequest();

        void SendRequest();

        // This is called when a reply has been received. Like the unique pointer
        // to the active request, the network reply is stored as a member.
        void ReceiveReply();

        // Resend the active request after the delay specified in the active reply.
        void ResendAfterViolation();

        // Simple flag to check if the active_request is in use. It's possible
        // to detect this by checking the status of the active request pointer,
        // but using a separate boolean flag is claer.
        bool busy;

        // This timer sends the active request.
        QTimer active_request_timer;

        // Unique pointer to the currently active request.
        std::unique_ptr<RateLimitedRequest> active_request;

        // Other requests in this list have been queued up. When an active request
        // has been successfully send and replied, the next request will be taken
        // from this queue.
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
        boost::circular_buffer<QDateTime> known_reply_times;
    };

    void Error(const QString& message);

    //=========================================================================================
    // Top-Level RateLimiter
    //=========================================================================================

    class RateLimiter : public QObject {
        Q_OBJECT
    public:

        // Creat a rate limiter.
        RateLimiter(QNetworkAccessManager& manager);

        // Submit a request/callback to the rate limiter. Requets will be deleted
        // after the callback is triggered.
        void Submit(QNetworkAccessManager& manager, QNetworkRequest request, Callback callback);

    public slots:
        // Triggered by rate limit policy managers so the rate limiter knows to start status updates.
        void OnTimerStarted();

    signals:
        // Signal sent to the rate-limit status panel so the user can see what's going on.
        void StatusUpdate(const QString message);

    private:

        void DispatchRequest(std::unique_ptr<RateLimitedRequest> request);
        void SendInitRequest();
        void ReceiveInitReply();
        void FinishInit();

        bool initialized;
        QNetworkAccessManager& initial_manager;
        std::vector<QString> initial_endpoints;
        std::vector<QNetworkReply*> initial_replies;
        int initial_replies_received;

        // Save requests that are submitted before the policy managers
        // have been created.
        std::vector<std::unique_ptr<RateLimitedRequest>> staged_requests;

        // One manager for each rate limit policy.
        std::vector<std::unique_ptr<PolicyManager>> managers;

        // Keep around one extra manager for non-rate-limited, non-api requests.
        std::unique_ptr<PolicyManager> default_manager;

        // This timer updates the rate limit status.
        QTimer status_updater;

        // Called by the timer to emit status updates.
        void DoStatusUpdate();
    };
}
