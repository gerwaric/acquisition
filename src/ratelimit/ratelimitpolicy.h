/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include <QMetaObject>
#include <QString>

#include <vector>

#include <boost/circular_buffer.hpp>

class QByteArray;
class QDateTime;
class QNetworkReply;

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

class RateLimitRule;

class RateLimitPolicy {
    Q_GADGET
public:
    enum class Status { UNKNOWN, OK, BORDERLINE, VIOLATION, INVALID };
    Q_ENUM(Status)

    RateLimitPolicy(QNetworkReply* const reply);
    void Check(const RateLimitPolicy& other) const;
    const QString& name() const { return m_name; };
    const std::vector<RateLimitRule>& rules() const { return m_rules; };
    Status status() const { return m_status; };
    int maximum_hits() const { return m_maximum_hits; };
    QDateTime GetNextSafeSend(const boost::circular_buffer<QDateTime>& history);
    QDateTime EstimateDuration(int request_count, int minimum_delay_msec) const;
private:
    const QString m_name;
    std::vector<RateLimitRule> m_rules;
    Status m_status;
    int m_maximum_hits;
};


class RateLimitData {
public:
    RateLimitData(const QByteArray& header_fragment);
    int hits() const { return m_hits; };
    int period() const { return m_period; };
    int restriction() const { return m_restriction; };
private:
    int m_hits;
    int m_period;
    int m_restriction;
};

class RateLimitItem {
public:
    RateLimitItem(const QByteArray& limit_fragment, const QByteArray& state_fragment);
    void Check(const RateLimitItem& other, const QString& prefix) const;
    const RateLimitData& limit() const { return m_limit; };
    const RateLimitData& state() const { return m_state; };
    RateLimitPolicy::Status status() const { return m_status; };
    QDateTime GetNextSafeSend(const boost::circular_buffer<QDateTime>& history) const;
    int EstimateDuration(int request_count, int minimum_delay_msec) const;
private:
    RateLimitData m_limit;
    RateLimitData m_state;
    RateLimitPolicy::Status m_status;
};

class RateLimitRule {
public:
    RateLimitRule(const QByteArray& name, QNetworkReply* const reply);
    void Check(const RateLimitRule& other, const QString& prefix) const;
    const QString& name() const { return m_name; };
    const std::vector<RateLimitItem>& items() const { return m_items; };
    RateLimitPolicy::Status status() const { return m_status; };
    int maximum_hits() const { return m_maximum_hits; };
private:
    const QString m_name;
    std::vector<RateLimitItem> m_items;
    RateLimitPolicy::Status m_status;
    int m_maximum_hits;
};
