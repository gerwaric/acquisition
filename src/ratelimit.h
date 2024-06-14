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

#include <QByteArray>;
#include <QByteArrayList>;
#include <QDateTime>
#include <QNetworkRequest>
#include <QString>

class QNetworkReply;

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

	class RateLimitedReply : public QObject {
		Q_OBJECT
	signals:
		void complete(QNetworkReply* reply) const;
		void failed(QNetworkReply* reply) const;
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
		void Check(const RuleItem& other, const QString& prefix) const;
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
		void Check(const PolicyRule& other, const QString& prefix) const;
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
		Policy(QNetworkReply* const reply);
		void Check(const Policy& other) const;
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
	
	QByteArray ParseHeader(QNetworkReply* const reply, const QByteArray& name);
	QByteArrayList ParseHeaderList(QNetworkReply* const reply, const QByteArray& name, const char delim);
	QByteArray ParseRateLimitPolicy(QNetworkReply* const reply);
	QByteArrayList ParseRateLimitRules(QNetworkReply* const reply);
	QByteArrayList ParseRateLimit(QNetworkReply* const reply, const QByteArray& rule);
	QByteArrayList ParseRateLimitState(QNetworkReply* const reply, const QByteArray& rule);
	QDateTime ParseDate(QNetworkReply* const reply);
	int ParseStatus(QNetworkReply* const reply);
}
