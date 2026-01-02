// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#include "ratelimitedrequest.h"

// Total number of rate-limited requests that have been created.
unsigned long RateLimitedRequest::s_request_count = 0;
