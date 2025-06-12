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

#include <QDateTime>
#include <QNetworkRequest>
#include <QString>

#include "ratelimitedreply.h"

class QNetworkRequest;

// Represents a single rate-limited request.
struct RateLimitedRequest {

    // Construct a new rate-limited request.
    RateLimitedRequest(const QString& endpoint_, const QNetworkRequest& network_request_, RateLimitedReply* reply_) :
        id(++s_request_count),
        endpoint(endpoint_),
        network_request(network_request_),
        reply(reply_) {
    }

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
