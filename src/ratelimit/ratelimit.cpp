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

#include "ratelimit.h"

#include <QNetworkReply>
#include <QUrl>

#include <QsLog/QsLog.h>

#include "util/util.h"

using namespace RateLimit;

// Get a header field from an HTTP reply.
QByteArray RateLimit::ParseHeader(QNetworkReply* const reply, const QByteArray& name) {
    if (!reply->hasRawHeader(name)) {
        QLOG_ERROR() << "RateLimit: the network reply is missing a header:" << name;
    };
    return reply->rawHeader(name);
}

// Get a header field and split into a list.
QByteArrayList RateLimit::ParseHeaderList(QNetworkReply* const reply, const QByteArray& name, const char delim) {
    const QByteArray value = ParseHeader(reply, name);
    const QByteArrayList items = value.split(delim);
    if (items.isEmpty()) {
        QLOG_ERROR() << "GetHeaderList():" << name << "is empty";
    };
    return items;
}

// Return the name of the policy from a network reply.
QByteArray RateLimit::ParseRateLimitPolicy(QNetworkReply* const reply) {
    return ParseHeader(reply, "X-Rate-Limit-Policy");
}

// Return the name(s) of the rule(s) from a network reply.
QByteArrayList RateLimit::ParseRateLimitRules(QNetworkReply* const reply) {
    return ParseHeaderList(reply, "X-Rate-Limit-Rules", ',');
}

// Return a list of one or more items that define a rule's limits.
QByteArrayList RateLimit::ParseRateLimit(QNetworkReply* const reply, const QByteArray& rule) {
    return ParseHeaderList(reply, "X-Rate-Limit-" + rule, ',');
}

// Return a list of one or more items that define a rule's current state.
QByteArrayList RateLimit::ParseRateLimitState(QNetworkReply* const reply, const QByteArray& rule) {
    return ParseHeaderList(reply, "X-Rate-Limit-" + rule + "-State", ',');
}

// Return the date from the HTTP reply headers.
QDateTime RateLimit::ParseDate(QNetworkReply* const reply) {
    const QString timestamp = QString(Util::FixTimezone(ParseHeader(reply, "Date")));
    const QDateTime date = QDateTime::fromString(timestamp, Qt::RFC2822Date).toLocalTime();
    if (!date.isValid()) {
        QLOG_ERROR() << "invalid date parsed from" << timestamp;
    };
    return date;
}

// Return the HTTP status from the reply headers.
int RateLimit::ParseStatus(QNetworkReply* const reply) {
    return reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
}
