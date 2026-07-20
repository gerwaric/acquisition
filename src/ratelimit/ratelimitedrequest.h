// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#pragma once

#include <stop_token>

#include <QDateTime>
#include <QFuture>
#include <QNetworkRequest>
#include <QPromise>
#include <QString>

#include "ratelimit/fetcherror.h"

// One entry in a policy pump's queue: what to send, where it came from, the
// timestamps the network capture compares, and the promise the caller is
// waiting on (network-redesign spec, D1). The entry owns the only handle to
// that promise, so completion has exactly one owner — the ownership problem
// F59 described is unconstructible at this boundary. (F59 was resolved in
// phase 4b, which deleted the legacy Submit() adapter and its
// RateLimitedReply — the last thing that still handed callers an object
// under the contradictory contract.)
struct RateLimitedRequest
{
    RateLimitedRequest(const QString &endpoint_,
                       const QNetworkRequest &network_request_,
                       std::stop_token token_)
        : id(++s_request_count)
        , endpoint(endpoint_)
        , network_request(network_request_)
        , token(std::move(token_))
    {
        promise.start();
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

    // The time the manager decided this request should be sent (the computed
    // next safe send, buffers included). Recorded so the network capture can
    // compare predicted against actual timing.
    QDateTime scheduled_time;

    // The caller's cancellation channel (D2): one token per update, checked
    // at every pump checkpoint. Callers with no abort story pass a default,
    // never-stopped token.
    std::stop_token token;

    // The caller's outcome. Started at construction so the future exists
    // before the entry is queued, and finished exactly once by
    // CompleteRequest() — every promise reaches a final state (D2/ER1), so
    // awaiting is unconditionally safe.
    QPromise<RateLimit::FetchOutcome> promise;

    // Set by CompleteRequest(): a completed entry is never completed again.
    bool completed = false;

private:
    // Total number of requests that have every been constructed.
    static unsigned long s_request_count;
};

// Complete an entry exactly once. Repeat calls are no-ops, which is what
// makes the drain's scoped completion guard safe: it can unconditionally
// complete an abandoned entry without knowing whether the entry's own path
// already did.
void CompleteRequest(RateLimitedRequest &entry, RateLimit::FetchOutcome outcome);

// Complete an entry with a FetchError of the given kind, filling in the
// endpoint and URL from the entry itself.
void CompleteRequest(RateLimitedRequest &entry,
                     RateLimit::FetchError::Kind kind,
                     const QString &message);
