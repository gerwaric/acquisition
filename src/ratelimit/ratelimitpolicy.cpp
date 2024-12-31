/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include "ratelimitpolicy.h"

#include <QByteArray>
#include <QDateTime>
#include <QNetworkReply>

#include <QsLog/QsLog.h>

#include "util/util.h"

#include "ratelimit.h"

//=========================================================================================
// RateLimitData
//=========================================================================================

RateLimitData::RateLimitData(const QByteArray& header_fragment)
    : m_hits(-1)
    , m_period(-1)
    , m_restriction(-1)
{
    const QByteArrayList parts = header_fragment.split(':');
    m_hits = parts[0].toInt();
    m_period = parts[1].toInt();
    m_restriction = parts[2].toInt();
}

//=========================================================================================
// RateLimitItem
//=========================================================================================

RateLimitItem::RateLimitItem(const QByteArray& limit_fragment, const QByteArray& state_fragment)
    : m_limit(limit_fragment)
    , m_state(state_fragment)
{
    // Determine the status of this item.
    if (m_state.period() != m_limit.period()) {
        m_status = RateLimitPolicy::Status::INVALID;
    } else if (m_state.hits() > m_limit.hits()) {
        m_status = RateLimitPolicy::Status::VIOLATION;
    } else if (m_state.hits() == m_limit.hits()) {
        m_status = RateLimitPolicy::Status::BORDERLINE;
    } else {
        m_status = RateLimitPolicy::Status::OK;
    };
}

void RateLimitItem::Check(const RateLimitItem& other, const QString& prefix) const {
    if (m_limit.hits() != other.m_limit.hits()) {
        QLOG_WARN() << prefix << "limit.hits changed"
            << "from" << m_limit.hits()
            << "to" << other.m_limit.hits();
    };
    if (m_limit.period() != other.m_limit.period()) {
        QLOG_WARN() << prefix << "limit.period changed"
            << "from" << m_limit.period()
            << "to" << other.m_limit.period();
    };
    if (m_limit.restriction() != other.m_limit.restriction()) {
        QLOG_WARN() << prefix << "limit.restriction changed"
            << "from" << m_limit.restriction()
            << "to" << other.m_limit.restriction();
    };
}

QDateTime RateLimitItem::GetNextSafeSend(const boost::circular_buffer<QDateTime>& history) const {
    QLOG_TRACE() << "RateLimit::RuleItem::GetNextSafeSend() entered";

    const QDateTime now = QDateTime::currentDateTime().toLocalTime();

    // We can send immediately if we have not bumped up against a rate limit.
    if (m_state.hits() < m_limit.hits()) {
        return now;
    };

    // Determine how far back into the history we can look.
    const size_t n = (m_limit.hits() > history.size()) ? history.size() : m_limit.hits();

    // Start with the timestamp of the earliest known 
    // reply relevant to this limitation.
    const QDateTime earliest = (n < 1) ? now : history[n - 1];

    const QDateTime next_send = earliest.addSecs(m_limit.period());

    QLOG_TRACE() << "RateLimit::RuleItem::GetNextSafeSend()"
        << "n =" << n
        << "earliest =" << earliest.toString() << "(" << earliest.secsTo(now) << "seconds ago)"
        << "next_send =" << next_send.toString() << "(in" << now.secsTo(next_send) << "seconds)";

    // Calculate the next time it will be safe to send a request.
    return next_send;
}

int RateLimitItem::EstimateDuration(int request_count, int minimum_delay_msec) const {
    QLOG_TRACE() << "RateLimit::RuleItem::EstimateDuration() entered";

    int duration = 0;

    const int current_hits = m_state.hits();
    const int max_hits = m_limit.hits();
    const int period_length = m_limit.period();
    const int restriction = m_limit.restriction();

    int initial_burst = max_hits - current_hits;
    if (initial_burst < 0) {
        initial_burst = 0;
        duration += restriction;
    };

    int remaining_requests = request_count - initial_burst;
    if (remaining_requests < 0) {
        remaining_requests = 0;
    };

    const int full_periods = (remaining_requests / max_hits);
    const int final_burst = (remaining_requests % max_hits);

    const int a = initial_burst * minimum_delay_msec;
    const int b = full_periods * period_length * 1000;
    const int c = final_burst * minimum_delay_msec;
    const int total_msec = a + b + c;

    duration += (total_msec / 1000);

    return duration;
}

//=========================================================================================
// RateLimitRule
//=========================================================================================

RateLimitRule::RateLimitRule(const QByteArray& name, QNetworkReply* const reply)
    : m_name(name)
    , m_status(RateLimitPolicy::Status::UNKNOWN)
    , m_maximum_hits(-1)
{
    QLOG_TRACE() << "RateLimit::PolicyRule::PolicyRule() entered";
    const QByteArrayList limit_fragments = RateLimit::ParseRateLimit(reply, name);
    const QByteArrayList state_fragments = RateLimit::ParseRateLimitState(reply, name);
    const int item_count = limit_fragments.size();
    if (state_fragments.size() != limit_fragments.size()) {
        QLOG_ERROR() << "Invalid data for policy role.";
    };
    m_items.reserve(item_count);
    for (int j = 0; j < item_count; ++j) {

        // Create a new rule item from the next pair of fragments.
        const auto& item = m_items.emplace_back(limit_fragments[j], state_fragments[j]);

        // Keep track of the max hits, max rate, and overall status.
        if (m_maximum_hits < item.limit().hits()) {
            m_maximum_hits = item.limit().hits();
        };
        if (m_status < item.status()) {
            m_status = item.status();
        };
    };
}

void RateLimitRule::Check(const RateLimitRule& other, const QString& prefix) const {
    QLOG_TRACE() << "RateLimit::PolicyRule::Check() entered";

    // Check the rule name
    if (m_name != other.m_name) {
        QLOG_WARN() << prefix << "rule name changed"
            << "from" << m_name << "to" << other.m_name;
    };

    // Check the number of items in this rule
    if (m_items.size() != other.m_items.size()) {

        // The number of items changed
        QLOG_WARN() << prefix << "rule" << m_name << "went"
            << "from" << m_items.size() << "items"
            << "to" << other.m_items.size() << "items";

    } else {

        // Check each item
        for (int i = 0; i < m_items.size(); ++i) {
            const QString item_prefix = QString("%1 item #%2").arg(prefix, QString::number(i));
            const auto& old_item = m_items[i];
            const auto& new_item = other.m_items[i];
            old_item.Check(new_item, item_prefix);
        };

    };
}

//=========================================================================================
// RateLimitPolicy
//=========================================================================================

RateLimitPolicy::RateLimitPolicy(QNetworkReply* const reply)
    : m_name(RateLimit::ParseRateLimitPolicy(reply))
    , m_status(RateLimitPolicy::Status::UNKNOWN)
    , m_maximum_hits(0)
{
    QLOG_TRACE() << "RateLimit::Policy::Policy() entered";
    const QByteArrayList rule_names = RateLimit::ParseRateLimitRules(reply);

    // Parse the name of the rate limit policy and all the rules for this reply.
    m_rules.reserve(rule_names.size());

    // Iterate over all the rule names expected.
    for (const auto& rule_name : rule_names) {

        // Create a new rule and add it to the list.
        const auto& rule = m_rules.emplace_back(rule_name, reply);

        // Check the status of this rule..
        if (rule.status() >= RateLimitPolicy::Status::VIOLATION) {
            QLOG_ERROR() << QString("Rate limit policy '%1:%2' is %4)").arg(
                m_name, rule.name(), Util::toString(rule.status()));
        } else if (rule.status() == RateLimitPolicy::Status::BORDERLINE) {
            QLOG_DEBUG() << QString("Rate limit policy '%1:%2' is BORDERLINE").arg(
                m_name, rule.name());
        };

        // Update metrics for this rule.
        if (m_maximum_hits < rule.maximum_hits()) {
            m_maximum_hits = rule.maximum_hits();
        };
        if (m_status < rule.status()) {
            m_status = rule.status();
        };
    };
}

void RateLimitPolicy::Check(const RateLimitPolicy& other) const {
    QLOG_TRACE() << "RateLimit::Policy::Check() entered";

    // Check the policy name
    if (m_name != other.m_name) {
        QLOG_WARN() << "The rate limit policy name change from" << m_name << "to" << other.m_name;
    };

    // Check the number of rules
    if (m_rules.size() != other.m_rules.size()) {

        // The number of rules changed
        QLOG_WARN() << "The rate limit policy" << m_name
            << "had" << m_rules.size() << "rules,"
            << "but now has" << other.m_rules.size();

    } else {

        // The number of rules is the same, so check each one
        for (int i = 0; i < m_rules.size(); ++i) {
            const QString prefix = QString("Rate limit policy %1, rule #%2:").arg(m_name, QString::number(i));
            const auto& old_rule = m_rules[i];
            const auto& new_rule = other.m_rules[i];
            old_rule.Check(new_rule, prefix);
        };

    };
}

QDateTime RateLimitPolicy::GetNextSafeSend(const boost::circular_buffer<QDateTime>& history) {
    QLOG_TRACE() << "RateLimit::Policy::GetNextSafeSend() entered";

    QDateTime next_send = QDateTime::currentDateTime().toLocalTime();
    for (const auto& rule : m_rules) {
        for (const auto& item : rule.items()) {
            const QDateTime t = item.GetNextSafeSend(history);
            QLOG_TRACE() << "RateLimit::Policy::GetNextSafeSend()"
                << m_name << rule.name() << "t =" << t.toString()
                << "(in" << QDateTime::currentDateTime().secsTo(t) << "seconds)";
            if (next_send < t) {
                QLOG_TRACE() << "RateLimit::Policy::GetNextSafeSend() updating next_send";
                next_send = t;
            };
        };
    };
    return next_send;
}

QDateTime RateLimitPolicy::EstimateDuration(int num_requests, int minimum_delay_msec) const {
    QLOG_TRACE() << "RateLimit::Policy::EstimateDuration() entered";

    int longest_wait = 0;
    while (true) {
        for (const auto& rule : m_rules) {
            for (const auto& item : rule.items()) {
                const int wait = item.EstimateDuration(num_requests, minimum_delay_msec);
                if (longest_wait < wait) {
                    longest_wait = wait;
                };
            };
        };
    };
    return QDateTime::currentDateTime().toLocalTime().addSecs(longest_wait);
}
