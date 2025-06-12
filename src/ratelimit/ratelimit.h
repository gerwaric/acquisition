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

#include <QByteArray>
#include <QByteArrayList>
#include <QDateTime>
#include <QString>

#include <boost/circular_buffer.hpp>

class QNetworkReply;

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

namespace RateLimit {
    Q_NAMESPACE

    enum class Status { UNKNOWN, OK, BORDERLINE, VIOLATION, INVALID };
    Q_ENUM_NS(Status)

    struct Event {
        unsigned long request_id;
        QString request_url;
        QDateTime request_time;
        QDateTime reply_time;
        int reply_status;
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
