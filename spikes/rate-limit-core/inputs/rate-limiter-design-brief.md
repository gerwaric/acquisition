# PoE API Client in Rust — Rate Limiter Design Brief

Status: external input to the rate-limit-core design session —
consumed by, not produced by, this spike.
Provenance: written by Tom with Fable High in a separate chat
session (August 2026) that had **no access to this codebase** — it
was informed by Tom's description of concepts and lessons learned,
not by the code, the frozen specs, or `network-ground-truth.md`.
That independence is the point: it could not anchor on the C++
implementation, so it serves as an independent second opinion on the
design space.
Caveat for the design session: claims in this document sit in **no
evidence lane yet**. Anything it asserts about GGG API behavior must
be reconciled against the N-claims in
`docs/design/network-ground-truth.md` before it may influence the
design. Divergences are agenda items, not errors — each one is
either a mistake here or a gap in the N-claims, and both outcomes
are findings.

---

# PoE API Client in Rust — Rate Limiter Design Brief

Seed document for a spike on the `redesign` branch. This captures the design decisions, invariants, and testing strategy agreed in prior discussion. The rate limiter is the load-bearing component of the entire client: the project's standing with GGG depends on it. When in doubt, choose the more conservative behavior.

## Goal of the spike

Build the **pure core** of the rate limiter first: types and functions with no IO, no clock access, no HTTP. Specifically:

1. `Policy`, `RuleWindow`, and related types for the policy store
2. The endpoint state machine (`Unprobed → Probing → Known`) as an enum with data-carrying variants
3. `earliest_safe_send(...) -> Instant` — the scheduling function
4. Header parsing into typed values (`Result`-returning)
5. Unit and property tests for the above

Async shell, HTTP transport, and OAuth come after the core is solid. Do not start with them.

## External API facts (from GGG developer docs)

Docs: https://www.pathofexile.com/developer/docs/index and /authorization and /reference

- Endpoints are plain HTTP returning JSON. Rate limiting is **discovered, not configured**: policies are dynamic, can change at any time, and are communicated via response headers that must be parsed and obeyed.
- Response headers:
  - `X-Rate-Limit-Policy` — named policy applying to this request. Multiple endpoints can share one policy.
  - `X-Rate-Limit-Rules` — comma-delimited rule names (e.g. `ip`, `account`, `client`).
  - `X-Rate-Limit-{rule}` — comma-delimited triplets `max_hits:period_seconds:restriction_seconds`. A rule may have multiple triplets (multiple windows).
  - `X-Rate-Limit-{rule}-State` — matching triplets `current_hits:period_seconds:restricted_seconds`.
  - `Retry-After` — seconds to wait, present on 429.
- **Invalid-request threshold**: too many HTTP 4xx responses (including 401, 403, 429) in a short period leads to restriction/revocation. 4xx responses spend a second budget; never retry-loop into them.
- **OAuth 2.1, public client** (desktop app): authorization code + PKCE only, localhost redirect URI (e.g. `http://127.0.0.1:8080/callback`), access tokens last 10 hours, refresh tokens 7 days (refresh does not extend expiry). Public clients **share rate limits with all other public clients** — the budget is a communal pool.
- Required `User-Agent` format: `OAuth {clientId}/{version} (contact: {contact})`.
- **Cloudflare sits in front of everything** with its own invisible DDoS protection. It publishes no headers and no policy. A prior bug that emitted ~2,000 rapid HEAD requests got a user blocked at the Cloudflare level. This is why request-spacing and the fuse (below) govern *every* outbound request, including probes.
- Known undocumented quirk: the server quantizes hits into invisible timing buckets, typically ~5s or ~60s. Exact per-second aging math is false precision.

## Architecture

**Workspace**: `poe-api` (client library), `poe-cli` (thin consumer). UI (Tauri) later. Library is UI-agnostic.

**Single choke point**: every outbound request — including HEAD probes — flows through one internal `async fn send(...)`. This makes "no request bypasses the limiter" structurally enforced, not conventional. There is no other path to the network.

**Pure core / thin shell**: all scheduling and reconciliation logic is pure functions over values (`policy state + history + now → wait duration`; `headers + state → updated state`). The async shell around it only sleeps, sends, and feeds results back into the core.

**Observable status**: limiter publishes a `RateLimitStatus` snapshot via `tokio::sync::watch`. CLI prints it, future UI subscribes to it, tests assert on it.

## Rate limiter model

The limiter maintains **its own model of the world** and uses server headers only to correct that model pessimistically. It is proactive, not reactive.

### Policy store and history

- Keyed by **policy name**. Endpoints map many-to-one into policies; the send-history log lives in the **policy record**, not the endpoint record. A policy remap is a pointer swap plus a pessimistic history merge.
- Per rule-window, keep a log of send timestamps (`VecDeque<Instant>`), pruned as entries age out. Local history — not the server's state header — is the source of truth for *when* hits occurred, because the server only reports counts.

### Pessimistic reconciliation (headers vs. local history)

- **Whichever is more pessimistic wins.** If the server reports more hits than local history knows about (shared public-client pool; user may run other tools), adopt the server's count.
- The ages of phantom hits are unknown → assume **they all just happened** (age out a full period from now).
- Never trust the server's count *downward* against local history.

### Bucket-quantized arithmetic

- Treat every hit as landing at the **latest possible moment**: round its timestamp up to the next bucket boundary before adding the period. Wait calculation is roughly `ceil_to_bucket(oldest_hit) + period + margin`, not `oldest_hit + period`.
- Bucket size is a per-policy parameter with a **pessimistic default (60s)**. Do not attempt to infer bucket size empirically — the only way to learn it is by getting 429'd.

### Headroom (max-N)

- Effective limit = `max_hits − headroom`, with headroom **configurable per policy** and a conservative default. Applied before the wait calculation runs.

### Endpoint discovery via HEAD probes

- HEAD requests do **not** count against GGG policies, but they do hit Cloudflare, so probes flow through the choke point (spacing + fuse) like everything else — serial, never a parallel batch.
- Endpoint state machine: `Unprobed → Probing → Known`. The `Unprobed → Probing` transition is guarded to happen **exactly once per endpoint per session**; concurrent callers await the in-flight probe's result (stored in the `Probing` variant) rather than issuing their own. There must be no code path that can issue a second probe for an endpoint not in `Unprobed`.
- Startup probes double as **restart recovery**: state headers on probe responses repopulate hit counts after a restart, merged with the worst-case-age rule. No limiter state is persisted to disk — that entire category of complexity is deleted.

### Internal spacing limit

- A global minimum interval between *any* two outbound requests, enforced at the choke point before any policy logic. One `Instant` behind a mutex.
- This is the Cloudflare-facing protection and the backstop when everything else is wrong. **Not configurable from outside the crate** (or floor-clamped at minimum).

### Global fuse

- Session-wide rolling counter with a generous ceiling (hundreds of requests per rolling minute — far above legitimate use). When tripped, the client **halts with an error** rather than continuing to send. Rate limiters slow traffic; the fuse stops it.
- This converts any request storm — regardless of cause, including bugs in the limiter itself and runaway agent loops — into a loud stop and a bug report instead of a Cloudflare block.

### 429 handling

A 429 may be exogenous (another client sharing the pool) or a model error. Either way:

1. Honor `Retry-After` plus margin; adopt server state pessimistically; mark policy restricted.
2. Retry **once**.
3. A second consecutive 429 on the same policy escalates: multiply the wait or suspend that policy's queue and surface it to the user. Never politely re-knock.
4. Log with a distinction: "model predicted safe, got 429" vs. "model had no prediction, got 429". Occurrences are rare (none observed since 2023) and every one is signal.
5. A 401 means **stop and refresh auth** — never retry-loop (4xx threshold).

In tests against the simulated server, any 429 is a **test failure**, not a retry.

### Live policy changes

- Headers on every response are watched; a policy's values changing under a stable name goes through normal pessimistic reconciliation.
- **Endpoint→policy remaps** happen (precedent: some website endpoints report different policies depending on login state). Auth-state transitions (token acquired / expired / refreshed) mark all endpoint→policy mappings **provisional**, reactivating careful first-request behavior: one request goes out, headers confirm or remap, traffic resumes. On remap, seed the new policy's history with the endpoint's recent sends, merged pessimistically with the new policy's state header.
- The first request after idleness treats the policy as provisional — an idle endpoint's stale policy gets corrected by the first response's headers before a second request can be misscheduled.

## Invariants (what the test harness asserts)

1. Every outbound request passes the choke point (spacing + fuse), HEAD probes included.
2. No endpoint is probed twice in a session.
3. Under the worst-case bucket assumption, no bucketized window ever reaches the effective (headroom-adjusted) limit.
4. A 429 triggers at most one retry before escalation; a 401 triggers zero retries.
5. Auth transitions invalidate endpoint→policy mappings.
6. The fuse converts any storm into a halt.
7. Pessimistic reconciliation: server counts adopted upward, never downward; unknown hits assumed fresh.

## Testing strategy

- **Pure core**: unit tests feed header sequences and simulated request storms into the scheduling/reconciliation functions. Property tests (e.g. `proptest`) assert invariant 3 over arbitrary storms and bucket sizes (test at least 5s and 60s buckets).
- **Deterministic time**: `tokio::time::pause()` — simulate hours of traffic in milliseconds. No clock-injection trait needed.
- **HTTP layer**: `wiremock` replaying real header sequences, including 429 + `Retry-After`, policy value changes, and policy remaps.
- **Simulated server**: implements the bucket quirk with configurable bucket size and a shared-pool mode (background hits from a phantom second client) to exercise reconciliation. Agent-loop stress scenario: thousands of rapid requests from the caller side; assert wire behavior stays compliant and the fuse behavior is correct.

## Rust idioms to lean into

- **Make invalid states unrepresentable.** Data that only exists in a given state lives inside that enum variant (`Probing` holds the awaitable handle; `Known` holds the policy name). No boolean flags or `Option` fields that are "only valid when." Exhaustive `match` everywhere — adding a state should break compilation until handled.
- **One narrow waist, one owner.** The `Client` owns the transport and the policy store. `Arc<Mutex<PolicyStore>>` appears in at most one place; if it spreads, the design is wrong. An actor-shaped alternative (one task owns the store outright, others message it via channel) is the growth path if the mutex version gets awkward — don't start there.
- **`Result`/`Option` everywhere fallibility or absence exists.** Header parsing returns `Result`; malformed headers are a visible code path with a decided behavior (conservative: treat as unknown policy), not a panic.
- **Own the data.** `String` not `&str` in structs; clone small values freely; no structs holding references, no lifetime annotations. Performance is irrelevant at a few requests per second — zero-copy here is over-engineering.
- **`std::sync::Mutex` for short lock-and-release on the store; never held across an `await`.** `tokio::sync` primitives only where an await-holding lock is genuinely required (it shouldn't be).
- **Minimal traits.** Testability comes from the pure core, `tokio::time::pause()`, and `wiremock` — not from `IHttpClient`/`IClock` abstractions. The token store may earn one small trait (CLI vs. UI storage). Otherwise, zero-trait design is the target.

## Non-goals / over-engineering guards

- No endpoint codegen; hand-written serde types. **Do not use `deny_unknown_fields`** — GGG changes responses without notice.
- No `tower` middleware stack; plain owned client with an internal choke point.
- No persisted limiter state (probes handle restart).
- No keyring integration in v1 (token file, permissions-restricted).
- No hard-coded policy names or limit values anywhere.
- No speculative UI plumbing beyond the `watch` status channel.

## OAuth (later phase, for context)

- `oauth2` crate for PKCE mechanics; one-shot localhost listener for the callback.
- 7-day refresh ceiling means re-auth is a **routine state**, not an error: the library surfaces "needs interactive auth" as a first-class condition; CLI prints the URL, UI opens a browser.
- Small token-store abstraction; proactive refresh before the 10-hour access-token expiry.

## Resources

- An existing, battle-tested rate limiter implementation lives in this repo's current application code and has run in production since 2023. It may be consulted to cross-check behaviors and edge cases (header parsing quirks, real policy names/values seen in the wild) — but the Rust design should be built from this brief's model first and validated against it second, not derived from it.

## Suggested spike sequence

1. Types: `Policy`, `RuleWindow`, `PolicyState`, endpoint state enum.
2. Header parser (`&HeaderMap → Result<ParsedRateHeaders, ParseError>`), with fixture tests from real header examples in the GGG docs.
3. Pure scheduler: `earliest_safe_send`, bucket-ceiling math, headroom.
4. Pure reconciler: pessimistic merge of server state into local history.
5. Property tests for invariants 3 and 7.
6. Only then: the async choke point, spacing limit, fuse, and wiremock tests.