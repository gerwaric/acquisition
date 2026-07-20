// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#pragma once

#include <deque>
#include <expected>
#include <vector>

#include <QMetaObject>
#include <QString>

#include "ratelimit/ratelimit.h"

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
//
// Parsing is a total function (network-redesign spec, D8): validation IS the
// parser — RateLimitPolicy::Parse() either yields a policy whose every triplet
// is three in-range integers, or a description of the first grammar failure.
// Nothing downstream ever indexes an unvalidated header fragment.

class RateLimitData
{
public:
    RateLimitData(int hits, int period, int restriction)
        : m_hits(hits)
        , m_period(period)
        , m_restriction(restriction)
    {}
    int hits() const { return m_hits; }
    int period() const { return m_period; }
    int restriction() const { return m_restriction; }

private:
    int m_hits;
    int m_period;
    int m_restriction;
};

class RateLimitItem
{
public:
    RateLimitItem(const RateLimitData &limit, const RateLimitData &state);
    bool Check(const RateLimitItem &other) const;
    const RateLimitData &limit() const { return m_limit; }
    const RateLimitData &state() const { return m_state; }
    RateLimit::Status status() const { return m_status; }

private:
    RateLimitData m_limit;
    RateLimitData m_state;
    RateLimit::Status m_status;
};

class RateLimitRule
{
public:
    RateLimitRule(const QString &name, std::vector<RateLimitItem> items);
    bool Check(const RateLimitRule &other) const;
    const QString &name() const { return m_name; }
    const std::vector<RateLimitItem> &items() const { return m_items; }

private:
    QString m_name;
    std::vector<RateLimitItem> m_items;
};

class RateLimitPolicy
{
    Q_GADGET
public:
    // The total parse (D8): Full means this succeeds, which requires a
    // nonempty policy name; a nonempty rules list; every rule name nonempty,
    // with at least one item and limit/state lists of equal length; every
    // triplet exactly three in-range integers — limit: hits > 0, period > 0,
    // restriction >= 0; state: hits >= 0, period > 0, restriction >= 0 — and
    // each state period matching its limit period. The error is a
    // human-readable description of the first failure, for logging and
    // capture.
    static std::expected<RateLimitPolicy, QString> Parse(QNetworkReply *const reply);

    bool Check(const RateLimitPolicy &other) const;
    const QString &name() const { return m_name; }
    const std::vector<RateLimitRule> &rules() const { return m_rules; }
    RateLimit::Status status() const { return m_status; }
    int maximum_hits() const { return m_maximum_hits; }
    QDateTime GetNextSafeSend(const std::deque<RateLimit::Event> &history);

    // Report generators for logging.
    QString GetPolicyReport() const;
    QString GetHistoryReport(const std::deque<RateLimit::Event> &history) const;
    QString GetBorderlineReport() const { return m_report; }

private:
    RateLimitPolicy(const QString &name, std::vector<RateLimitRule> rules);

    QString m_name;
    std::vector<RateLimitRule> m_rules;
    RateLimit::Status m_status;
    int m_maximum_hits;

    // This report is generated whenever GetNextSafeSend() is called, which should
    // only be when a rate limit policy is about to be violated. Then if a violation
    // occurs the report can be written to the log.
    QString m_report;

    // Internal helper for format dates for logging.
    static QString Timestamp(const QDateTime &t);
};
