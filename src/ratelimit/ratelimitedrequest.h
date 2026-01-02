// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#pragma once

#include <QDateTime>
#include <QNetworkRequest>
#include <QString>

#include "ratelimit/ratelimitedreply.h"

class QNetworkRequest;

// Represents a single rate-limited request.
struct RateLimitedRequest
{
    // Construct a new rate-limited request.
    RateLimitedRequest(const QString &endpoint_,
                       const QNetworkRequest &network_request_,
                       RateLimitedReply *reply_)
        : id(++s_request_count)
        , endpoint(endpoint_)
        , network_request(network_request_)
        , reply(reply_)
    {}

    // Unique identified for each request, even through different requests can be
    // routed to different policy managers based on different endpoints.
    const unsigned long id;

    // A copy of this request's API endpoint, if any.
    const QString endpoint;

    // A copy of the network request that's going to be sent.
    QNetworkRequest network_request;

    // The time the request was made.
    QDateTime send_time;

    std::unique_ptr<RateLimitedReply> reply;

private:
    // Total number of requests that have every been constructed.
    static unsigned long s_request_count;
};
