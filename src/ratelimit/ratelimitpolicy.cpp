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

#include "ratelimitpolicy.h"

#include <QByteArray>
#include <QDateTime>
#include <QNetworkReply>

#include <util/spdlog_qt.h>
#include <util/util.h>

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
    , m_resolution(-1)
{
    // Determine the status of this item.
    if (m_state.period() != m_limit.period()) {
        m_status = RateLimit::Status::INVALID;
    } else if (m_state.hits() > m_limit.hits()) {
        m_status = RateLimit::Status::VIOLATION;
    } else if (m_state.hits() == m_limit.hits()) {
        m_status = RateLimit::Status::BORDERLINE;
    } else {
        m_status = RateLimit::Status::OK;
    };

    // Determine which timing resolution applies.
    m_resolution = (m_limit.period() <= RateLimit::INITIAL_VS_SUSTAINED_PERIOD_CUTOFF)
        ? RateLimit::INITIAL_TIMING_BUCKET_SECS
        : RateLimit::SUSTAINED_TIMING_BUCKET_SECS;
}

void RateLimitItem::Check(const RateLimitItem& other, const QString& prefix) const {
    if (m_limit.hits() != other.m_limit.hits()) {
        spdlog::warn("{} limit.hits changed form {} to {}", prefix, m_limit.hits(), other.m_limit.hits());
    };
    if (m_limit.period() != other.m_limit.period()) {
        spdlog::warn("{} limit.period changed from {} to {}", prefix, m_limit.period(), other.m_limit.period());
    };
    if (m_limit.restriction() != other.m_limit.restriction()) {
        spdlog::warn("{} limit.restriction changed from {} to {}", prefix, m_limit.restriction(), other.m_limit.restriction());
    };
}

QDateTime RateLimitItem::GetNextSafeSend(const boost::circular_buffer<RateLimit::Event>& history) const {
    spdlog::trace("RateLimit::RuleItem::GetNextSafeSend() entered");

    const QDateTime now = QDateTime::currentDateTime().toLocalTime();

    // We can send immediately if we have not bumped up against a rate limit.
    if (m_state.hits() < m_limit.hits()) {
        return now;
    };

    // Determine how far back into the history we can look.
    const size_t n = (m_limit.hits() > history.size()) ? history.size() : m_limit.hits();

    // Start with the timestamp of the earliest known 
    // reply relevant to this limitation.
    const QDateTime earliest = (n < 1) ? now : history[n - 1].reply_time;

    QDateTime next_send = earliest.addSecs(m_limit.period());

    // Add the timing bucket resolution if we are borderline to avoid rate limiting.
    if (m_status >= RateLimit::Status::BORDERLINE) {
        next_send = next_send.addSecs(m_resolution);
    };

    spdlog::trace(
        "RateLimit::RuleItem::GetNextSafeSend()"
        " n = {}"
        " earliest = {} ({} seconds ago)"
        " next_send = {} (in {} seconds)",
        n,
        earliest.toString(), earliest.secsTo(now),
        next_send.toString(), now.secsTo(next_send));

    // Calculate the next time it will be safe to send a request.
    return next_send;
}

int RateLimitItem::EstimateDuration(int request_count, int minimum_delay_msec) const {
    spdlog::trace("RateLimit::RuleItem::EstimateDuration() entered");

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
    , m_status(RateLimit::Status::UNKNOWN)
    , m_maximum_hits(-1)
{
    spdlog::trace("RateLimit::PolicyRule::PolicyRule() entered");
    const QByteArrayList limit_fragments = RateLimit::ParseRateLimit(reply, name);
    const QByteArrayList state_fragments = RateLimit::ParseRateLimitState(reply, name);
    const int item_count = limit_fragments.size();
    if (state_fragments.size() != limit_fragments.size()) {
        spdlog::error("Invalid data for policy role.");
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
    spdlog::trace("RateLimit::PolicyRule::Check() entered");

    // Check the rule name
    if (m_name != other.m_name) {
        spdlog::warn("{} rule name changed from {} to {}", prefix, m_name, other.m_name);
    };

    // Check the number of items in this rule
    if (m_items.size() != other.m_items.size()) {

        // The number of items changed
        spdlog::warn("{} rule {} went from {} items to {} items", prefix, m_name, m_items.size(), other.m_items.size());

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
    , m_status(RateLimit::Status::UNKNOWN)
    , m_maximum_hits(0)
{
    spdlog::trace("RateLimit::Policy::Policy() entered");
    const QByteArrayList rule_names = RateLimit::ParseRateLimitRules(reply);

    // Parse the name of the rate limit policy and all the rules for this reply.
    m_rules.reserve(rule_names.size());

    // Iterate over all the rule names expected.
    for (const auto& rule_name : rule_names) {

        // Create a new rule and add it to the list.
        const auto& rule = m_rules.emplace_back(rule_name, reply);

        // Check the status of this rule..
        if (rule.status() >= RateLimit::Status::VIOLATION) {
            spdlog::error(
                "Rate limit policy '{}:{}' is {})",
                m_name, rule.name(), Util::toString(rule.status()));
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
    spdlog::trace("RateLimit::Policy::Check() entered");

    // Check the policy name
    if (m_name != other.m_name) {
        spdlog::warn("The rate limit policy name change from {} to {}", m_name, other.m_name);
    };

    // Check the number of rules
    if (m_rules.size() != other.m_rules.size()) {

        // The number of rules changed
        spdlog::warn("The rate limit policy {} had {} rules, but now has {}", m_name, m_rules.size(), other.m_rules.size());

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

QDateTime RateLimitPolicy::GetNextSafeSend(const boost::circular_buffer<RateLimit::Event>& history) {
    spdlog::trace("RateLimit::Policy::GetNextSafeSend() entered");

    QDateTime next_send = QDateTime::currentDateTime().toLocalTime();
    for (const auto& rule : m_rules) {
        for (const auto& item : rule.items()) {
            const QDateTime t = item.GetNextSafeSend(history);
            spdlog::trace("RateLimit::Policy::GetNextSafeSend() {} {} t = {} (in {} seconds)",
                          m_name, rule.name(), t.toString(), QDateTime::currentDateTime().secsTo(t));
            if (next_send < t) {
                spdlog::trace("RateLimit::Policy::GetNextSafeSend() updating next_send");
                next_send = t;
            };
        };
    };
    return next_send;
}

QDateTime RateLimitPolicy::EstimateDuration(int num_requests, int minimum_delay_msec) const {
    spdlog::trace("RateLimit::Policy::EstimateDuration() entered");

    int longest_wait = 0;
    for (const auto& rule : m_rules) {
        for (const auto& item : rule.items()) {
            const int wait = item.EstimateDuration(num_requests, minimum_delay_msec);
            if (longest_wait < wait) {
                longest_wait = wait;
            };
        };
    };
    return QDateTime::currentDateTime().toLocalTime().addSecs(longest_wait);
}
