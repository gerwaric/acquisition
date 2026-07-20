// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#include "ratelimit/ratelimitedrequest.h"

// Total number of rate-limited requests that have been created.
unsigned long RateLimitedRequest::s_request_count = 0;

void CompleteRequest(RateLimitedRequest &entry, RateLimit::FetchOutcome outcome)
{
    if (entry.completed) {
        return;
    }
    entry.completed = true;
    entry.promise.addResult(std::move(outcome));
    entry.promise.finish();
}

void CompleteRequest(RateLimitedRequest &entry,
                     RateLimit::FetchError::Kind kind,
                     const QString &message)
{
    RateLimit::FetchError error;
    error.kind = kind;
    error.endpoint = entry.endpoint;
    error.url = entry.network_request.url();
    error.message = message;
    CompleteRequest(entry, RateLimit::FetchOutcome(std::unexpected(std::move(error))));
}
