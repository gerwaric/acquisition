// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <expected>

#include <QByteArray>
#include <QNetworkReply>
#include <QString>
#include <QUrl>

namespace RateLimit {

    // The value-only failure type at the network boundary (network-redesign
    // spec, D1/D8). Errors are values, never exceptions: nothing thrown ever
    // crosses a layer boundary, so every consumer handles the same struct
    // whether the failure came from the transport, the server, the rate
    // limiter, or a parse.
    struct FetchError
    {
        enum class Kind {
            // Transport failure — including a reply truncated after a 2xx
            // status arrived. Carries the Qt error code and string.
            Network,
            // A non-2xx status other than 429. Carries the status.
            Http,
            // A facade-level payload (JSON) failure.
            Parse,
            // A 2xx whose rate-limit headers fail to parse or name a
            // different policy — setup or steady state (D4/D8/IR1).
            Protocol,
            // A terminal 429: retries exhausted *or* non-retryable. One kind
            // for every terminal 429 (R7); carries the attempt count and
            // whether an acceptable Retry-After was present.
            RateLimited,
            // An exception escaped inside the pump or the facade — a bug,
            // contained as a value rather than propagated.
            Internal,
            // The caller requested stop (D2). An outcome, not a failure:
            // consumers return silently and it never counts toward
            // request-failure accounting.
            Canceled,
        };

        Kind kind = Kind::Internal;

        // The API endpoint label the request was submitted under, and the
        // request's URL. Always populated for requests that reached a pump.
        QString endpoint;
        QUrl url;

        // The HTTP status, when the reply carried a status line; 0 otherwise.
        int http_status = 0;

        // The Qt transport error, when there was one.
        QNetworkReply::NetworkError network_error = QNetworkReply::NoError;
        QString network_error_string;

        // Send attempts made for this request (1 unless the 429 retry loop
        // ran). Meaningful for RateLimited.
        int attempts = 0;

        // Whether the terminal 429 carried an acceptable Retry-After
        // (present, numeric, nonnegative, at or below the product cap).
        // Meaningful for RateLimited.
        bool retry_after_acceptable = false;

        // A human-readable description for logs; never parsed.
        QString message;
    };

    // A short stable name for a kind, for log messages.
    constexpr const char *ToString(FetchError::Kind kind)
    {
        switch (kind) {
        case FetchError::Kind::Network:
            return "Network";
        case FetchError::Kind::Http:
            return "Http";
        case FetchError::Kind::Parse:
            return "Parse";
        case FetchError::Kind::Protocol:
            return "Protocol";
        case FetchError::Kind::RateLimited:
            return "RateLimited";
        case FetchError::Kind::Internal:
            return "Internal";
        case FetchError::Kind::Canceled:
            return "Canceled";
        }
        return "Unknown";
    }

    // What the rate limiter delivers: the response body on success (D1). The
    // facade chains a parse onto this and yields typed payloads.
    using FetchOutcome = std::expected<QByteArray, FetchError>;

} // namespace RateLimit
