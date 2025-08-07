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

// GGG has stated that when they are keeping track of request times,
// they have a timing resolution, which they called a "bucket".
//
// This explained some otherwise mysterious rate violations that I
// was seeing very intermittently. Unless there's a away to find out
// where those timing buckets begin and end precisely, all we can do
// is use the bucket size as a minimum delay.
//
// GGG has also stated that this bucket resolution may be different
// for different policies, but the one I had been asking them about
// was 5.0 seconds. They also noted that this number is currently
// not documented or exposed to api users in any way.
//
// As of June 2025, GGG has confirmed that all endpoints used by
// acquisition have a 5 second timing bucket for the "fast" rate
// limit, and a 1 minute bucket for the "slow" rate limit.

constexpr const int INITIAL_TIMING_BUCKET_SECS = 5;
constexpr const int SUSTAINED_TIMING_BUCKET_SECS = 60;

// There's nothing in the rate limit policy that says there's only
// a fast and slow rate limit, but that's what email from GGG has
// implied, so this is used as a heuristic for determining which
// is which.

constexpr const int INITIAL_VS_SUSTAINED_PERIOD_CUTOFF = 75;

// Since we don't know how the server buckets are aligned or what
// the error is, let's add a buffer.

constexpr const int TIMING_BUCKET_BUFFER_SECS = 1;

//=========================================================================================
// RateLimitData
//=========================================================================================

RateLimitData::RateLimitData(const QByteArray &header_fragment)
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

RateLimitItem::RateLimitItem(const QByteArray &limit_fragment, const QByteArray &state_fragment)
    : m_limit(limit_fragment)
    , m_state(state_fragment)
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
    }
}

void RateLimitItem::Check(const RateLimitItem &other, const QString &prefix) const
{
    if (m_limit.hits() != other.m_limit.hits()) {
        spdlog::warn("{} limit.hits changed form {} to {}",
                     prefix,
                     m_limit.hits(),
                     other.m_limit.hits());
    }
    if (m_limit.period() != other.m_limit.period()) {
        spdlog::warn("{} limit.period changed from {} to {}",
                     prefix,
                     m_limit.period(),
                     other.m_limit.period());
    }
    if (m_limit.restriction() != other.m_limit.restriction()) {
        spdlog::warn("{} limit.restriction changed from {} to {}",
                     prefix,
                     m_limit.restriction(),
                     other.m_limit.restriction());
    }
}

//=========================================================================================
// RateLimitRule
//=========================================================================================

RateLimitRule::RateLimitRule(const QByteArray &name, QNetworkReply *const reply)
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
    }
    m_items.reserve(item_count);
    for (int j = 0; j < item_count; ++j) {
        // Create a new rule item from the next pair of fragments.
        const auto &item = m_items.emplace_back(limit_fragments[j], state_fragments[j]);

        // Keep track of the max hits, max rate, and overall status.
        if (m_maximum_hits < item.limit().hits()) {
            m_maximum_hits = item.limit().hits();
        }
        if (m_status < item.status()) {
            m_status = item.status();
        }
    }
}

void RateLimitRule::Check(const RateLimitRule &other, const QString &prefix) const
{
    spdlog::trace("RateLimit::PolicyRule::Check() entered");

    // Check the rule name
    if (m_name != other.m_name) {
        spdlog::warn("{} rule name changed from {} to {}", prefix, m_name, other.m_name);
    }

    // Check the number of items in this rule
    if (m_items.size() != other.m_items.size()) {
        // The number of items changed
        spdlog::warn("{} rule {} went from {} items to {} items",
                     prefix,
                     m_name,
                     m_items.size(),
                     other.m_items.size());

    } else {
        // Check each item
        for (size_t i = 0; i < m_items.size(); ++i) {
            const QString item_prefix = QString("%1 item #%2").arg(prefix, QString::number(i));
            const auto &old_item = m_items[i];
            const auto &new_item = other.m_items[i];
            old_item.Check(new_item, item_prefix);
        }
    }
}

//=========================================================================================
// RateLimitPolicy
//=========================================================================================

RateLimitPolicy::RateLimitPolicy(QNetworkReply *const reply)
    : m_name(RateLimit::ParseRateLimitPolicy(reply))
    , m_status(RateLimit::Status::UNKNOWN)
    , m_maximum_hits(0)
{
    spdlog::trace("RateLimit::Policy::Policy() entered");
    const QByteArrayList rule_names = RateLimit::ParseRateLimitRules(reply);

    // Parse the name of the rate limit policy and all the rules for this reply.
    m_rules.reserve(rule_names.size());

    // Iterate over all the rule names expected.
    for (const auto &rule_name : rule_names) {
        // Create a new rule and add it to the list.
        const auto &rule = m_rules.emplace_back(rule_name, reply);

        // Check the status of this rule..
        if (rule.status() >= RateLimit::Status::VIOLATION) {
            spdlog::error("Rate limit policy '{}:{}' is {})",
                          m_name,
                          rule.name(),
                          Util::toString(rule.status()));
        }

        // Update metrics for this rule.
        if (m_maximum_hits < rule.maximum_hits()) {
            m_maximum_hits = rule.maximum_hits();
        }
        if (m_status < rule.status()) {
            m_status = rule.status();
        }
    }
}

void RateLimitPolicy::Check(const RateLimitPolicy &other) const
{
    spdlog::trace("RateLimit::Policy::Check() entered");

    // Check the policy name
    if (m_name != other.m_name) {
        spdlog::warn("The rate limit policy name change from {} to {}", m_name, other.m_name);
    }

    // Check the number of rules
    if (m_rules.size() != other.m_rules.size()) {
        // The number of rules changed
        spdlog::warn("The rate limit policy {} had {} rules, but now has {}",
                     m_name,
                     m_rules.size(),
                     other.m_rules.size());

    } else {
        // The number of rules is the same, so check each one
        for (size_t i = 0; i < m_rules.size(); ++i) {
            const QString prefix = QString("Rate limit policy %1, rule #%2:")
                                       .arg(m_name, QString::number(i));
            const auto &old_rule = m_rules[i];
            const auto &new_rule = other.m_rules[i];
            old_rule.Check(new_rule, prefix);
        }
    }
}

QDateTime RateLimitPolicy::GetNextSafeSend(const boost::circular_buffer<RateLimit::Event> &history)
{
    const QDateTime now = QDateTime::currentDateTime().toLocalTime();
    const bool is_borderline = m_status == RateLimit::Status::BORDERLINE;

    spdlog::trace("Rate Limiting: calculating next safe send for {} policy", m_name);

    if (is_borderline) {
        m_report.clear();
        m_report.append(QString("<----- BORDERLINE_REPORT for '%1' at %2 ----->")
                            .arg(m_name, now.toString(Qt::ISODateWithMs)));
        m_report.append("<POLICY_STATE>");
        for (const auto &rule : m_rules) {
            for (const auto &item : rule.items()) {
                const auto &limit = item.limit();
                const auto &state = item.state();
                const QString line = QString("%1/%2[%3s] = %4/%5:%6:%7")
                                         .arg(m_name,
                                              rule.name(),
                                              QString::number(limit.period()),
                                              QString::number(state.hits()),
                                              QString::number(limit.hits()),
                                              QString::number(limit.period()),
                                              QString::number(limit.restriction()));
                m_report.append(line);
            }
        }
        m_report.append("</POLICY_STATE>");
        m_report.append("<HISTORY_STATE>");
        for (size_t i = 0; i < history.size(); ++i) {
            const auto &event = history[i];
            const QString line = QString("%1 #%2 (request_id=%3): sent %4, received %5, reply %6 "
                                         "(status=%7, url='%8')")
                                     .arg(m_name,
                                          QString::number(i + 1),
                                          QString::number(event.request_id),
                                          event.request_time.toString("yyyy-MMM-dd HH:mm:ss.zzz"),
                                          event.received_time.toString("yyyy-MMM-dd HH:mm:ss.zzz"),
                                          event.reply_time.toString("yyyy-MMM-dd HH:mm:ss.zzz"),
                                          QString::number(event.reply_status),
                                          event.request_url);
            m_report.append(line);
        }
        m_report.append("</HISTORY_STATE>");
    }

    QDateTime next_send(now);

    for (const auto &rule : m_rules) {
        for (const auto &item : rule.items()) {
            const auto period = item.limit().period();
            const auto max_hits = item.limit().hits();
            const auto current_hits = item.state().hits();

            const QString prefix = QString("%1/%2[%3s]")
                                       .arg(m_name, rule.name(), QString::number(period));

            // If this item is not limiting, we can skip it.
            if (current_hits < max_hits) {
                spdlog::trace("{}: (state {}/{}), skipping", prefix, current_hits, max_hits);
                continue;
            }

            // Determine how far back into the history we can look.
            const size_t hits = static_cast<size_t>(max_hits);
            const size_t len = history.size();
            const size_t n = (len < hits) ? len : hits;

            spdlog::trace("{}: (state {}/{}), n={}/{}", prefix, current_hits, max_hits, n, len);

            // Start with the timestamp of the earliest known
            // reply relevant to this limitation.
            QDateTime t;
            if (n < 1) {
                t = now;
                spdlog::trace("{}: using current time: {}", prefix, t.toString(Qt::ISODateWithMs));
                if (is_borderline) {
                    m_report.append(
                        QString("Using current time: %1").arg(t.toString(Qt::ISODateWithMs)));
                }
            } else {
                const auto &event = history[n - 1];
                if (is_borderline) {
                    m_report.append(QString("<EVENT index=%1, history_size=%2>").arg(n).arg(len));
                    m_report.append(QString("request_id    = %1").arg(event.request_id));
                    m_report.append(QString("request_url   = %1").arg(event.request_url));
                    m_report.append(QString("request_time  = %1")
                                        .arg(event.request_time.toString(Qt::ISODateWithMs)));
                    m_report.append(QString("received_time = %1")
                                        .arg(event.received_time.toString(Qt::ISODateWithMs)));
                    m_report.append(QString("reply_time    = %1")
                                        .arg(event.reply_time.toString(Qt::ISODateWithMs)));
                    m_report.append(QString("reply_status  = %1").arg(event.reply_status));
                    m_report.append(QString("</EVENT>"));
                }
                t = event.reply_time;
                spdlog::trace("{}: candidate send is {} from history event {}/{}",
                              prefix,
                              t.toString(Qt::ISODateWithMs),
                              n,
                              len);
            }

            // Add the measurement period.
            t = t.addSecs(period);
            spdlog::trace("{}: candidate send is {} after adding {}s period",
                          prefix,
                          t.toString(Qt::ISODateWithMs),
                          period);

            // Determine which timing resolution applies.
            const int delay = ((period <= INITIAL_VS_SUSTAINED_PERIOD_CUTOFF)
                                   ? INITIAL_TIMING_BUCKET_SECS
                                   : SUSTAINED_TIMING_BUCKET_SECS)
                              + TIMING_BUCKET_BUFFER_SECS;

            // Add the timing resolution.
            t = t.addSecs(delay);
            spdlog::trace("{}: candidate send is {} after adding {}s for timing bucket",
                          prefix,
                          t.toString(Qt::ISODateWithMs),
                          delay);

            // Check to see if we need to update the final result.
            if (next_send < t) {
                next_send = t;
                spdlog::trace("{}: next send is {} now",
                              prefix,
                              next_send.toString(Qt::ISODateWithMs));
            }
        }
    }
    return next_send;
}

QDateTime RateLimitPolicy::EstimateDuration(int num_requests, int minimum_delay_msec) const
{
    spdlog::trace("RateLimit::Policy::EstimateDuration() entered");

    int longest_wait = 0;

    for (const auto &rule : m_rules) {
        for (const auto &item : rule.items()) {
            int wait = 0;

            const int current_hits = item.state().hits();
            const int max_hits = item.limit().hits();
            const int period_length = item.limit().period();
            const int restriction = item.limit().restriction();

            int initial_burst = max_hits - current_hits;
            if (initial_burst < 0) {
                initial_burst = 0;
                wait += restriction;
            }

            int remaining_requests = num_requests - initial_burst;
            if (remaining_requests < 0) {
                remaining_requests = 0;
            }

            const int full_periods = (remaining_requests / max_hits);
            const int final_burst = (remaining_requests % max_hits);

            const int a = initial_burst * minimum_delay_msec;
            const int b = full_periods * period_length * 1000;
            const int c = final_burst * minimum_delay_msec;
            const int total_msec = a + b + c;

            wait += (total_msec / 1000);

            if (longest_wait < wait) {
                longest_wait = wait;
            }
        }
    }

    return QDateTime::currentDateTime().toLocalTime().addSecs(longest_wait);
}
