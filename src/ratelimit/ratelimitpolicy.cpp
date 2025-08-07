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

bool RateLimitItem::Check(const RateLimitItem &other) const
{
    if (m_limit.hits() != other.m_limit.hits()) {
        spdlog::warn("Rate Limit Policy: maximum hits hits changed form {} to {}",
                     m_limit.hits(),
                     other.m_limit.hits());
        return false;
    }
    if (m_limit.period() != other.m_limit.period()) {
        spdlog::warn("Rate Limit Policy: period changed from {} to {}",
                     m_limit.period(),
                     other.m_limit.period());
        return false;
    }
    if (m_limit.restriction() != other.m_limit.restriction()) {
        spdlog::warn("Rate Limit Policy: restriction changed from {} to {}",
                     m_limit.restriction(),
                     other.m_limit.restriction());
        return false;
    }
    return true;
}

//=========================================================================================
// RateLimitRule
//=========================================================================================

RateLimitRule::RateLimitRule(const QByteArray &name, QNetworkReply *const reply)
    : m_name(name)
{
    const QByteArrayList limit_fragments = RateLimit::ParseRateLimit(reply, name);
    const QByteArrayList state_fragments = RateLimit::ParseRateLimitState(reply, name);
    const int item_count = limit_fragments.size();
    if (state_fragments.size() != limit_fragments.size()) {
        spdlog::error("Invalid data for policy role.");
        return;
    }
    m_items.reserve(item_count);
    for (int j = 0; j < item_count; ++j) {
        // Create a new rule item from the next pair of fragments.
        m_items.emplace_back(limit_fragments[j], state_fragments[j]);
    }
}

bool RateLimitRule::Check(const RateLimitRule &other) const
{
    // Check the rule name
    if (m_name != other.m_name) {
        spdlog::warn("Rate Limit Policy: rule name changed from {} to {}", m_name, other.m_name);
        return false;
    }

    // Check the number of items in this rule
    if (m_items.size() != other.items().size()) {
        // The number of items changed
        spdlog::warn("Rate Limit Policy: rule {} went from {} items to {} items",
                     m_name,
                     m_items.size(),
                     other.items().size());
        return false;
    }

    // Check each item
    for (size_t i = 0; i < m_items.size(); ++i) {
        if (!m_items[i].Check(other.items()[i])) {
            return false;
        }
    }
    return true;
}

//=========================================================================================
// RateLimitPolicy
//=========================================================================================

RateLimitPolicy::RateLimitPolicy(QNetworkReply *const reply)
    : m_name(RateLimit::ParseRateLimitPolicy(reply))
    , m_status(RateLimit::Status::OK)
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

        // Process each item in this rule.
        for (const auto &item : rule.items()) {
            // Log any violations.
            if (item.status() >= RateLimit::Status::VIOLATION) {
                spdlog::error("Rate limit policy '{}/{}[{}s]' is {})",
                              m_name,
                              rule.name(),
                              item.limit().period(),
                              Util::toString(item.status()));
            }
            // Update metrics for this rule.
            if (m_maximum_hits < item.limit().hits()) {
                m_maximum_hits = item.limit().hits();
            }
            if (m_status < item.status()) {
                m_status = item.status();
            }
        }
    }
}

bool RateLimitPolicy::Check(const RateLimitPolicy &other) const
{
    // Check the policy name.
    if (m_name != other.m_name) {
        spdlog::warn("The rate limit policy name change from {} to {}", m_name, other.m_name);
        return false;
    }

    // Check the number of rules.
    if (m_rules.size() != other.rules().size()) {
        // The number of rules changed
        spdlog::warn("The rate limit policy {} had {} rules, but now has {}",
                     m_name,
                     m_rules.size(),
                     other.rules().size());
        return false;
    }

    // The number of rules is the same, so check each one
    for (size_t i = 0; i < m_rules.size(); ++i) {
        if (!m_rules[i].Check(other.rules()[i])) {
            return false;
        }
    }
    return true;
}

QString RateLimitPolicy::Timestamp(const QDateTime &t)
{
    return t.toString("yyyy-MMM-dd HH:mm:ss.zzz");
}

QString RateLimitPolicy::GetPolicyReport() const
{
    QStringList lines;
    lines.append(QString("<POLICY_STATE policy='%1'>").arg(m_name));
    lines.append(QString("  status = %1:").arg(Util::toString(m_status)));
    for (const auto &rule : m_rules) {
        for (const auto &item : rule.items()) {
            lines.append(QString("  %1/%2[%3s] = (%4/%5):%6:%7")
                             .arg(m_name, rule.name())
                             .arg(item.limit().period())
                             .arg(item.state().hits())
                             .arg(item.limit().hits())
                             .arg(item.limit().period())
                             .arg(QString::number(item.limit().restriction())));
        }
    }
    lines.append("</POLICY_STATE>");
    return lines.join("\n");
}

QString RateLimitPolicy::GetHistoryReport(
    const boost::circular_buffer<RateLimit::Event> &history) const
{
    QStringList lines;
    lines.append(QString("<HISTORY_STATE policy='%1'>").arg(m_name));
    for (size_t i = 0; i < history.size(); ++i) {
        const auto &event = history[i];
        const QString line = QString("  %1 #%2 (request_id=%3): sent %4, received %5, reply %6 "
                                     "(status=%7, url='%8')")
                                 .arg(m_name)
                                 .arg(i + 1)
                                 .arg(event.request_id)
                                 .arg(Timestamp(event.request_time),
                                      Timestamp(event.received_time),
                                      Timestamp(event.reply_time))
                                 .arg(event.reply_status)
                                 .arg(event.request_url);
        lines.append(line);
    }
    lines.append("</HISTORY_STATE>");
    return lines.join("\n");
}

QDateTime RateLimitPolicy::GetNextSafeSend(const boost::circular_buffer<RateLimit::Event> &history)
{
    const QDateTime now = QDateTime::currentDateTime().toLocalTime();

    // We can send immediately if the status is OK.
    if (m_status < RateLimit::Status::BORDERLINE) {
        return now;
    }

    spdlog::debug("Rate Limiting: calculating next send for BORDERLINE policy: {}", m_name);

    QDateTime next_send(now);

    QStringList lines;
    lines.append(QString("===== BORDERLINE_REPORT(%1) =====").arg(Timestamp(now)));
    lines.append(GetPolicyReport());
    lines.append(GetHistoryReport(history));

    for (const auto &rule : m_rules) {
        for (const auto &item : rule.items()) {
            const auto period = item.limit().period();
            const auto max_hits = item.limit().hits();
            const auto current_hits = item.state().hits();

            const QString tag = QString("%1/%2[%3s]").arg(m_name, rule.name()).arg(period);

            // If this item is not limiting, we can skip it.
            if (current_hits < max_hits) {
                lines.append(QString("%1: skipping rule because state is %2/%3")
                                 .arg(tag)
                                 .arg(current_hits)
                                 .arg(max_hits));
                continue;
            }

            // Determine how far back into the history we can look.
            const size_t hits = static_cast<size_t>(max_hits);
            const size_t len = history.size();
            const size_t n = (len < hits) ? len : hits;
            lines.append(QString("%1: n=%2/%3").arg(tag).arg(n).arg(len));

            // Start with the timestamp of the earliest known
            // reply relevant to this limitation.
            QDateTime t;
            if (n < 1) {
                t = now;
                lines.append(QString("Using current time: %1").arg(Timestamp(t)));
            } else {
                const auto &event = history[n - 1];
                lines.append(QString("%1: using history event:").arg(tag));
                lines.append(QString("<EVENT index=%1, history_size=%2>").arg(n).arg(len));
                lines.append(QString("  request_id    = %1").arg(event.request_id));
                lines.append(QString("  request_url   = %1").arg(event.request_url));
                lines.append(QString("  request_time  = %1").arg(Timestamp(event.request_time)));
                lines.append(QString("  received_time = %1").arg(Timestamp(event.received_time)));
                lines.append(QString("  reply_time    = %1").arg(Timestamp(event.reply_time)));
                lines.append(QString("  reply_status  = %1").arg(event.reply_status));
                lines.append(QString("</EVENT>"));
                t = event.reply_time;
            }

            // Add the measurement period.
            t = t.addSecs(period);
            lines.append(QString("%1: send is %2 adding %3 seconds for period")
                             .arg(tag, Timestamp(t))
                             .arg(period));

            // Determine which timing resolution applies.
            const int delay = ((period <= INITIAL_VS_SUSTAINED_PERIOD_CUTOFF)
                                   ? INITIAL_TIMING_BUCKET_SECS
                                   : SUSTAINED_TIMING_BUCKET_SECS)
                              + TIMING_BUCKET_BUFFER_SECS;

            // Add the timing resolution.
            t = t.addSecs(delay);
            lines.append(QString("%1: send is %2 after adding %3 seconds for timing bucket")
                             .arg(tag, Timestamp(t))
                             .arg(delay));

            // Check to see if we need to update the final result.
            if (next_send < t) {
                lines.append(QString("%1: updating next send from %1 to %2")
                                 .arg(tag, Timestamp(next_send), Timestamp(t)));
                next_send = t;
            } else {
                lines.append(QString("Next send is unchanged").arg(tag));
            }
        }
    }
    spdlog::debug("Rate Limiting: next send for '{}' is {}", m_name, Timestamp(next_send));
    lines.append(QString("Next send for '%1' is %2").arg(m_name, Timestamp(next_send)));
    lines.append(QString("================================="));

    m_report = lines.join("\n");

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
