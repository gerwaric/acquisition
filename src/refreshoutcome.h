// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <variant>
#include <vector>

#include "fetchsourcekey.h"
#include "ratelimit/fetcherror.h"

// The typed terminal vocabulary (items-pipeline M2, D4): a sum type so
// invalid states — an error on a completion, skips on a failure — are
// unrepresentable (R1-6). Emitted exactly once per accepted Update() via
// ItemsManagerWorker::RefreshFinished (forwarded by ItemsManager); the
// ordering guarantee is the identity (every delta of an update precedes its
// terminal event; nothing of that update follows).
struct SkippedSource
{
    FetchSourceKey source;       // same key as the deltas (D3)
    RateLimit::FetchError error; // the deterministic failure (D5)
};

struct CompletedRefresh
{
    std::vector<SkippedSource> skipped; // empty on a clean completion
};

struct FailedRefresh
{
    RateLimit::FetchError error; // the FIRST terminal error
};

using RefreshOutcome = std::variant<CompletedRefresh, FailedRefresh>;
