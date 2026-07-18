# Network Ground Truth

**Status: living research document, begun July 17, 2026.** This is the
output of the network ground-truth research phase that paused the F56–F59
fix work (see `docs/cleanup/findings.md` and the pause note in F56). Its
job is to capture everything we know, believe, or need to learn about how
Path of Exile's API actually limits requests — so that queueing/scheduling
designs can be derived from cited claims instead of code archaeology and
folklore. When a claim here falls, every design that cites it falls with
it; that is the point.

Rules of this document:

- **Claims are numbered N1, N2, … and never renumbered.** Designs and
  specs cite them by number.
- Every claim carries a **source tag** and a **confidence**. When better
  evidence lands (a verbatim email, an instrumented log), the claim is
  upgraded in place with the new citation — history stays in git.
- Hypotheses and open questions live here too, clearly marked. A
  hypothesis is a claim we are *acting on* without adequate evidence; an
  open question is one we have not resolved either way.

Source tags:

| Tag | Meaning |
|-----|---------|
| DOC | Official developer docs (`https://www.pathofexile.com/developer/docs/index`), read July 17, 2026 |
| GGG-EMAIL | Private correspondence between Tom and GGG support. Primary source; verbatim where quoted. Not public, not documented anywhere else |
| INCIDENT | A specific real-world event, reconstructed from memory |
| OBS | Instrumented observation from real sessions (none yet — see Instrumentation) |
| CODE | Inference from what the current client does and has survived doing |
| TOM | Tom's recollection, not yet backed by a retrievable artifact |
| HYP | Hypothesis — plausible, load-bearing, unverified |

Confidence: **Confirmed** (primary source or direct observation),
**High** (strong secondary evidence), **Provisional** (acting on it,
would like better evidence).

---

## The four-layer model

The API's request limiting is not one mechanism. Four distinct layers
exist, and only one of them is documented:

1. **Cloudflare / DDoS protection** — in front of everything, opaque
   even to GGG support, IP-facing, triggered by request *bursts*.
2. **Policy counters** — the documented `X-Rate-Limit-*` contract:
   named policies, rules, hit counters, restrictions, `Retry-After`.
3. **Timing buckets** — server-side quantization of the policy
   counters. Real, GGG-confirmed, deliberately not exposed to clients.
4. **Behavioral enforcement** — humans at GGG: the documented threat of
   application revocation for frequent violations, and privately granted
   allowances (HEAD probing) that exist only in email.

A correct client design must satisfy all four at once. The layers have
different failure modes: layer 1 blocks the user's IP outside the API's
own protocol; layer 2 returns structured 429s; layer 3 makes layer 2
fire "impossibly"; layer 4 kills the application id for everyone.

---

## Design postulates

Two conclusions strong enough to drive design, derived from the claims
below:

**P-A. Perfect violation prevention is impossible; graceful 429 recovery
is therefore a first-class requirement, not edge-case handling.**
The timing buckets (N11–N14) quantize the server's counters in ways the
headers do not expose, GGG says bucket sizes "might" vary and can only
be learned by asking support per policy, and limits are documented as
dynamic (N9). The client's model of the server counter is *inherently*
approximate. Occasional violations are a structural fact of the
platform. Consequence: F57 (one 429 wedges the update until restart) is
not a rare-path bug — under this postulate it is a violation of a core
requirement. Even the recovery path is bucket-quantized: `Retry-After`
likely needs padding too (N19). Cites: N9, N11, N12, N13, N14, N15,
N19.

**P-B. Parallelism across policies is safe at the policy layer, but
must be packaged with a deliberate global burst bound.**
The four data endpoints have four independent policies whose counters
run in parallel (N6, N7) — so cross-policy parallel requests do not
violate layer 2. But layer 1 watches burst concurrency invisibly (N2,
N4): the one known Cloudflare block was triggered by over a thousand
requests in a minute, and policy-compliant traffic is inherently slow
("seconds-per-request", N4), so a small in-flight cap plus strict HEAD
serialization (N18) is what keeps layer 1 invisible. Any design that
re-parallelizes (the F56 fix) must state its global bound explicitly.
Cites: N2, N4, N6, N7, N18.

---

## Claims ledger

### Layer 1 — Cloudflare / DDoS protection

**N1. A Cloudflare protection layer sits in front of the API; its rules
are opaque even to GGG support.** [INCIDENT + TOM — Confirmed]
Discovered via the incident in N2. The affected user knew a GGG
developer personally and escalated; the resulting exchanges established
that GGG support itself does not know the layer's details.

**N2. The one known Cloudflare block (July 2024, v0.11.1-alpha.11) was
triggered by over a thousand HEAD requests in one minute — redundant
per-tab probes, repeated ~20+ times per stash tab.** [INCIDENT +
GGG-EMAIL — Confirmed]
GGG support relayed the report, verbatim (July 2024):

> "Just letting you know that someone has come to us after getting
> rate-limited by cloudflare after using your tool. It made over a
> thousand requests in a minute, all were HEAD requests to what seems
> like every Standard stash tab on the account at least 20 times each.
> Hopefully this isn't the intended behaviour... so passing it along
> for you to look into."

The client was firing one HEAD per stash tab even though all tabs
share one endpoint — a single HEAD would have sufficed. The trigger
was a burst of redundant requests, not sustained rate. The fix Tom
shipped later that month is the origin of the current HEAD design and
the F5 standing constraint (one-HEAD-at-a-time, nested event loop in
`RateLimiter::SetupEndpoint`). The affected user (around since 2014,
personally acquainted with a GGG developer) escalated, which is also
how N1's "opaque even to GGG support" was learned. Remaining unknown:
the block from the user's side — presentation, duration, scope (Q7).

**N3. Cloudflare signals such blocks with error code 1015, which
Cloudflare documents as caused by (but not limited to) rate limit
violations.** [TOM + Cloudflare public docs — High]
The client already watches for 1015 in the league-list fetch
(`logindialog.cpp`).

**N4. A client that respects the policy limits and avoids request
bursts should never surface the Cloudflare layer.** [TOM reasoning —
HYP, Provisional]
Tom's framing: GGG's limits are effectively "seconds per request", not
"requests per second" — compliant traffic is slow. If layer 2 is
implemented well and bursts are bounded, layer 1 stays invisible.
Strategy consequence: we deliberately do **not** try to characterize
the Cloudflare boundary (probing it risks real users' IPs); we stay far
inside it. The only design obligation layer 1 imposes is the burst
bound of P-B.

### Layer 2 — Policy counters (the documented contract)

**N5. The header contract is: `X-Rate-Limit-Policy` (policy name),
`X-Rate-Limit-Rules` (comma-list of rule names),
`X-Rate-Limit-<rule>` (comma-list of `hits:period:restriction`
triplets), `X-Rate-Limit-<rule>-State` (comma-list of
`current-hits:period:restriction-active` triplets), and `Retry-After`
(seconds) on 429 responses.** [DOC — Confirmed]
The client's parsing (`src/ratelimit/ratelimit.cpp`,
`ratelimitpolicy.cpp`) matches this format. Docs: applications should
"parse and follow" these headers.

**N6. Policies with the same name share counters across endpoints;
differently-named policies count independently and in parallel.**
[DOC (same-name sharing) + TOM (parallel counting) — Confirmed]
Docs: "Policies may be the same across different API endpoints but are
treated the same for rate limiting purposes." Tom confirms the
counters for different policies run in parallel. This is the premise
that makes F56 (accidental cross-policy serialization) a real defect
rather than conservative correctness.

**N7. The four data endpoints acquisition uses map to four distinct
policies: `stash-list-request-limit`, `stash-request-limit`,
`character-list-request-limit`, `character-request-limit`.**
[GGG-EMAIL — Confirmed]
These four names come from Tom's email to GGG support (quoted under
N12), and GGG answered in terms of them without correction. Together
with N6, full four-way parallelism is safe at layer 2.

**N8. Rules within a policy can be IP-, account-, or client-scoped.**
[DOC — Confirmed (that these scopes exist)]
Open (Q2): whether the policy/rule topology differs between OAuth
sessions (client-scoped?) and POESESSID sessions (account/IP-scoped?),
and what that implies for a user running acquisition alongside the
website or other tools.

**N9. Limits are dynamic and can change at any time.** [DOC —
Confirmed]
Docs verbatim: "These limits are dynamic and can change at any time
depending on our requirements." The client must tolerate mid-session
policy changes (the `RateLimitPolicy::Check` machinery exists for
this; today a mismatch only logs). Tempering observation (TOM, July
2026): the policy definitions acquisition sees have not changed since
Tom first observed them — years of stability. The reservation is real
but rarely, if ever, exercised.

**N10. Frequently exceeding the limits results in application access
being revoked.** [DOC — Confirmed (that the threat exists)]
Docs verbatim: "Exceeding these limits frequently will result in your
application access being revoked." Acquisition is a registered
confidential client (`client_id=acquisition`), so this is an
application-wide, all-users consequence. What "frequently" means is
unknown (Q8). This is also why active 429-provoking experiments are
ruled out as a research method.

### Layer 3 — Timing buckets

**N11. The server buckets incoming requests into time intervals; the
counters the headers report are quantized by these buckets, which are
not exposed to clients.** [GGG-EMAIL — Confirmed]
GGG support, verbatim (on one policy, before the four-policy answer):

> "That particular policy has a resolution of 5 seconds, which means
> that it buckets incoming requests in 5 second intervals. To be safe
> you'd have to add 5 more seconds rather than 2. This isn't exposed
> anywhere for you to have known and we'll have to figure out some way
> of making that more obvious... in the meantime feel free to ask about
> other policies and we can track down what resolution they use for
> you."

**N12. For all four policies in N7: the initial (short-period) limit is
bucketed in 5-second intervals; the sustained (long-period) limit is
bucketed in 1-minute intervals.** [GGG-EMAIL — Confirmed, as of the
email date]
Tom asked GGG support for the resolution of exactly those four
policies. GGG support, verbatim:

> "They are all bucketed in 5 second intervals for the initial limit,
> and then 1 minute intervals for the sustained limit."

GGG had earlier said resolutions "might" vary per endpoint, so this
answer is per-policy fact, not a global rule — and per N9 it could
change. Provenance note: this email is the direct source of
`INITIAL_TIMING_BUCKET_SECS = 5` and `SUSTAINED_TIMING_BUCKET_SECS =
60` in `src/ratelimit/ratelimitpolicy.cpp`, which until this document
had no written provenance. (The `INITIAL_VS_SUSTAINED_PERIOD_CUTOFF =
75` used to classify a limit as initial-vs-sustained is a client-side
heuristic calibrated by eyeballing the observed policy definitions,
not from GGG — see Q4 for provenance and an untested alternative.)

**N13. The safe margin against bucket quantization is the full bucket
size, added on top of the computed wait.** [GGG-EMAIL — Confirmed]
From the N11 quote: "To be safe you'd have to add 5 more seconds rather
than 2." The current client adds bucket + 1s
(`TIMING_BUCKET_BUFFER_SECS`).

**N14. Bucket resolutions can only be learned by asking GGG support per
policy; there is no protocol mechanism.** [GGG-EMAIL — Confirmed]
GGG explicitly offered "feel free to ask about other policies and we
can track down what resolution they use for you." If new endpoints are
ever added (or Q3 suggests a change), this is the channel.

**N15. Before the buckets were known, sub-second pacing arithmetic
produced intermittent, undiagnosable rate limit violations.**
[TOM/INCIDENT — Confirmed]
This is the empirical face of P-A: the violations were real, the
client-side arithmetic was "correct", and the cause was invisible by
construction. Open (Q5): whether the current bucket-padded pacing has
reduced live violations to zero, or whether a residual rate remains —
the violation counter exists because violations were still being
observed. Instrumented sessions will answer this.

### Layer 4 — Sanctions and private allowances

**N16. HEAD probing is sanctioned by GGG — "intended to work" — with
the sanctioned usage pattern being a single HEAD at application
startup. All of this exists only in a private email thread (Nov 2023 –
Jul 2024); the docs say nothing.** [GGG-EMAIL — Confirmed]

Chronology, verbatim:

- **Nov 2023** — Tom asks whether OAuth HEADs need a special scope
  (they worked on legacy endpoints but failed on the new API with an
  "insufficient scope" error). GGG, a few days later:

  > "Unfortunately an oversight has prevented this from working for
  > quite some time... I'm surprised you're the first person to bring
  > it up! It is intended to work as you'd expected and we'll get a
  > fix out for it in the next game patch."

- **Dec 2023** — Tom reports that a HEAD against
  `/character/<name>` returns `x-rate-limit-policy` but none of the
  other rate-limit headers. GGG (Feb 2024): "Things are a bit hectic
  this year! I've identified the issue and we should be able to get a
  fix out for it next patch."

- **Apr 2024** — Tom asks for a recommended rate limit for HEADs,
  including possible periodic UI-refresh polling. GGG, a few days
  later:

  > "Ideally you only need to make one HEAD request to figure out what
  > state you're in as the application boots up. Is there a reason you
  > think the policy state would get out of sync enough to warrant it?
  > Such as after being rate-limited?"

- **Jul 2024** — the Cloudflare incident (N2); the fix that month
  produced the current one-HEAD-at-a-time design.

Reading: the sanction is real but *narrow* — one HEAD at boot to
discover unknown state. Periodic polling was met with a question, not
an endorsement; treat it as unsanctioned. Note that GGG's own reply
floats HEAD-after-429 resync as a conceivably legitimate case ("Such
as after being rate-limited?") — a door left open, not a grant.
Standing consequence: the technique is legitimate but fragile — the
sanction is invisible to any future maintainer or GGG employee who
has not seen this thread, which is part of why it is transcribed
here.

**N17. Acquisition was — per GGG — the first client to raise HEAD
probing on the new API; API behavior in this area has had real,
long-lived bugs.** [GGG-EMAIL — Confirmed]
GGG, Nov 2023: "I'm surprised you're the first person to bring it up!"
— about an oversight that had "prevented this from working for quite
some time." Tom has also personally found documented API object fields
that were simply wrong. Consequence for research method: community
prior art is a calibration source, not ground truth — other projects
likely never exercised these paths. See N20 for the design consequence
of the bug history.

**N18. HEAD probes must be strictly serialized — at most one in flight,
ever.** [INCIDENT + CODE — Confirmed as a constraint we impose]
Direct lesson of N2, implemented as the F5 nested-event-loop block in
`RateLimiter::SetupEndpoint`. Today's accidental full serialization
(F56) makes overlap impossible; any re-parallelization must preserve
this property deliberately (noted in the F56 pause note).

**N19. `Retry-After` alone is probably insufficient — the retried
request likely needs bucket padding on top.** [TOM — Provisional]
Tom is "almost certain" of this from his launch-window experimentation,
where the retry delay was one of the variables tuned against
intermittent violations. If confirmed (Q9), the F57 fix must schedule
the retried request at `Retry-After` plus the applicable bucket, not
`Retry-After` alone. Strengthens P-A: even the recovery path is
subject to bucket quantization.

**N20. HEAD responses are intended to carry the full rate-limit header
set — and this mechanism has silently regressed server-side at least
twice.** [GGG-EMAIL — Confirmed]
The Nov 2023 "insufficient scope" oversight and the Dec 2023
`/character/<name>` HEAD returning only `x-rate-limit-policy` (see the
N16 chronology) were both server-side breakages of exactly the
mechanism the client's startup depends on — each live for an extended
period, each fixed only after Tom reported it. Design consequence: the
client must degrade gracefully when a HEAD comes back partial. Today
`RateLimiter::ProcessHeadResponse` is fatal only on a missing
`X-Rate-Limit-Policy`; a Dec-2023-shaped reply (policy present, rules
absent) would parse into an empty policy whose status is OK — i.e. the
endpoint would run effectively unpaced until the first real reply
reseeds it. Whether the Dec 2023 fix actually shipped, and what the
degraded-HEAD behavior *should* be, are open (Q3).

### Other regimes — legacy website API and forum

Acquisition's traffic is not all OAuth-API traffic. Two other regimes
exist, with mutually exclusive auth. Design-history note (Tom, July
2026): this multi-regime reality — two hosts, incompatible auth, two
different limit-signaling protocols — is part of why the current
architecture centralizes scheduling and policy management inside the
rate limiter. Recorded as context for the M2 where-does-scheduling-live
question, not as an endorsement either way.

**N21. The legacy website API
(`www.pathofexile.com/character-window/*`) is rate-limited by the same
`X-Rate-Limit-*` header mechanism as the OAuth API, works only with
POESESSID auth, and actively rejects requests that carry an OAuth
bearer token.** [TOM — Confirmed, as of July 2026]
Consequences: (a) auth must be scoped per host — a bearer token must
never reach `www.pathofexile.com` (the current
`NetworkManager::createRequest` already restricts the bearer to
`api.pathofexile.com`, correct by construction, but nothing pins this
as a requirement); (b) GGG's bucket answer (N12) named only the four
`api.pathofexile.com` policies — the legacy endpoints' bucket
resolutions are unknown (the N14 ask-GGG channel applies); (c) open:
whether legacy policies share names/counters with the OAuth stash
policies (observable from the captured `X-Rate-Limit-Policy` on
`get-stash-items`).

**N22. The forum (`/forum/edit-thread/*`) is a separate rate-limiting
regime: no `X-Rate-Limit-*` protocol, limits signaled in the response
HTML ("You must wait N seconds."), POESESSID-only, and it also breaks
if an OAuth bearer token is present.** [TOM + CODE — Confirmed]
The client detects limits by scraping the body and resubmitting
(`Shop::OnShopSubmitted`), entirely outside the rate limiter. It
shares the user's IP, so layer 1 sees this traffic regardless of how
separate the protocol is.

---

### First capture — observed claims (July 18, 2026)

Source for all claims in this group: the first instrumented session
[OBS — Confirmed for this session; one account, OAuth auth, PC realm],
July 18, 2026: full refresh of 121 stash fetches (~47 top-level +
folder/special children) plus one shop stash-index call and one
character. 132 records, zero errors, zero violations. Capture file
retained locally (not committed — it contains account/stash
identifiers).

**N23. Observed policy topology and definitions (OAuth session).**
Five policies, confirming N7's four plus a distinct legacy policy:

| Policy | Rules | Limits (`hits:period:restriction`) |
|---|---|---|
| `stash-list-request-limit` | Account | `10:15:60, 30:60:300` |
| `stash-request-limit` | Account | `15:10:60, 30:300:300` |
| `character-list-request-limit` | Account | `2:10:60, 5:300:300` |
| `character-request-limit` | Account | `5:10:60, 30:300:300` |
| `backend-item-request-limit` (legacy `get-stash-items`) | Account, Ip | Account `30:60:60, 100:1800:600`; Ip `45:60:120, 180:1800:600` |

Under OAuth the API rules are all **Account**-scoped (no Client rule
observed — so other tools on the same account share these counters).
The legacy endpoint has its own policy — **no counter sharing with
`stash-request-limit`** (answers N21c) — and is the only one with an
Ip rule.

**N24. HEAD probes return the full header set and do not appear to
increment the counters.** API HEADs returned **204** (legacy: 200)
with complete limits+state+rules — the Dec 2023 regression (N16/N20)
is fixed. All four API states were `0` hits at HEAD time and the first
GET reported exactly `1`, so HEADs don't count against the policy.
The legacy policy's 1800s window showed 1 pre-existing hit at HEAD
time — confirmed by Tom as residue from an earlier acquisition
session. That residue is itself worth stating: **the counters are
server-side per account and persist across client restarts**, so a
freshly started client can begin mid-window with hits already
consumed — which is exactly why the HEAD-at-boot pattern (N16)
matters.

**N25. The state header is post-increment and tracks 1:1.** Each
reply's state included that request (first request = 1), incremented
exactly once per request through both windows, hit exactly `15/15` and
`30/30` at saturation and never over, and reset cleanly after the
padded wait. No bucket-quantization artifacts were *visible* — but at
full bucket padding none would be; the quantization only shows at
tighter margins (N15).

**N26. The client's pacing arithmetic is empirically exact, and the
resulting shape is burst-then-stall.** Observed cycle: 15 sends at
~0.2s spacing (10s window full) → wait ≈ 10+5+1s → 15 more sends
(300s window full) → wait ≈ 300+60+1s; the long-wait send landed
within 0.4s of `history[max_hits-1] + period + bucket + buffer`.
Cost of the padding: ~61s idle per 30-fetch cycle (~17% overhead);
121 tabs took ~24 minutes. Design note for later (not a finding):
even pacing at ~period/hits spacing would avoid saturation entirely —
no bucket stalls (~20% faster) and no 0.2s bursts (friendlier to
layer 1) — at the cost of never using the initial-burst allowance.
Also observed: the worker queue is not strictly stashes-first — the
character fetch (request id 54) sat behind only the ~47 initially
queued tabs, with ~74 children appended behind it as parents resolved
(refines F56's description).

## Open questions

- **Q1. HEAD sanction verbatim. RESOLVED July 18, 2026** — Tom
  retrieved the thread (Nov 2023 – Jul 2024). Chronology and verbatim
  quotes are captured in N16; consequences split into N17 (first
  reporter), N20 (degraded-HEAD handling), and the upgraded N2
  (incident details from GGG's own report).
- **Q2. Auth-mode topology. LARGELY RESOLVED July 18, 2026** (N23):
  under OAuth, API rules are Account-scoped (no Client rule); the
  POESESSID legacy endpoint has its own policy with Account+Ip rules.
  Residual: whether the API policies look different under any other
  auth arrangement is untested but no longer load-bearing.
- **Q3. HEAD mechanics. LARGELY RESOLVED July 18, 2026** (N24): full
  header sets confirmed on all five endpoints (Dec 2023 fix shipped);
  HEADs do not appear to increment counters (the one anomalous-looking
  state value was confirmed as cross-session residue). Residual: only
  the design decision of what a client *should* do with a degraded
  HEAD reply (today: run unpaced until the first real reply — N20).
- **Q4. Initial-vs-sustained classification.** The client classifies a
  limit as initial (5s bucket) vs sustained (60s bucket) by `period <=
  75s`. Provenance (Tom, July 2026): the cutoff came from eyeballing
  the actual policy definitions, which have not changed since first
  observed (N9 note) — calibrated to reality, but an absolute cutoff
  misclassifies if definitions ever shift (a 60s-period sustained
  limit would be under-padded → violations). Untested alternative
  (Tom's idea; he may have tried it and forgotten): classify
  *relatively* — pair up rules that differ only in period; the shorter
  of the pair is the initial limit, the longer is sustained. Captured
  policy definitions would let us evaluate both against the real
  shapes. **Sharpened July 18, 2026 (N23): the danger case is live.**
  `stash-list-request-limit`'s sustained rule is `30:60:300` — a
  60-second period, classified as *initial* (5s bucket) by the 75s
  cutoff. If GGG's "1 minute intervals for the sustained limit" (N12)
  applies to that rule, it is under-padded by 55s, and it is a prime
  suspect for the historically unexplained intermittent violations
  (N15). Not yet observed failing (the rule never saturated in the
  first capture); saturating stash-list — e.g. many rapid manual
  refreshes — would test it, but see the no-active-probing
  constraint. **Tom's hypothesis (July 18, 2026) [HYP]:**
  classification is *positional* — when a rule carries two
  `hits:period:restriction` triplets, the first is the initial limit
  (5s bucket) and the second is the sustained limit (60s bucket).
  This needs no period arithmetic, matches every rule in the first
  capture (shorter period always listed first, N23), and classifies
  the stash-list danger case conservatively where the 75s cutoff does
  not. Strictly safer than the cutoff on all observed data; still
  worth confirming with GGG alongside the bucket-tier question.
- **Q5. Current live violation rate.** With bucket-padded pacing, are
  violations actually zero in real sessions? If not, under what
  conditions? (Instrumented sessions, especially at saturation on the
  many-tab account.) First data point (July 18, 2026): a full
  24-minute saturated refresh, zero violations.
- **Q6. What does the reported state reflect at response time?
  LARGELY RESOLVED July 18, 2026** (N25): post-increment, 1:1 with
  requests, no backwards jumps, clean reset after the padded wait.
  Residual: quantization is only observable at margins tighter than
  the full bucket padding, so N25 cannot rule it out — it just
  confirms the padded regime is safe.
- **Q7. Cloudflare incident reconstruction.** Magnitude and shape now
  known from GGG's own report (N2): over a thousand HEADs in one
  minute, ~20+ repeats per tab. Still unknown, from the user's side:
  presentation (1015 page?), duration, and scope (API only, or
  website/game too). Deliberately *not* to be re-established
  experimentally (N4); memory and any surviving reports are the only
  sources.
- **Q8. What does "frequently" mean for revocation (N10)?** Probably
  unanswerable except by asking GGG; relevant to how much residual
  violation rate (Q5) is tolerable.
- **Q9. Is `Retry-After` sufficient?** Tom is almost certain it is
  *not* — bucket padding is needed on top (N19; the retry delay was
  one of the variables in his launch-window tuning). Design should
  assume N19; captures of any live 429 (retry timing vs outcome) would
  confirm it. (F57's wedge has hidden the answer so far — the retried
  request's outcome was never observed by the caller.)
- **Q10. Scope: legacy endpoints. LARGELY RESOLVED July 18, 2026** —
  the facts are recorded as N21/N22: both regimes are real
  rate-limited traffic the client carries, and their auth exclusivity
  (bearer tokens actively break both) is part of why the current
  design centralizes scheduling in the limiter. What remains of Q10:
  (a) research side — capture the legacy endpoint's policy identity
  and, if needed, ask GGG for its bucket resolutions (N21c); (b)
  design side — decide whether the redesign *coordinates* legacy and
  forum traffic with API traffic (they share the IP, hence layer 1)
  or merely tolerates them as-is.

---

## Instrumentation

The current logs cannot answer the open questions: raw
`X-Rate-Limit-*` headers are logged only once per endpoint (initial
HEAD probe, debug level), steady-state replies never log raw headers,
per-request send timestamps are trace-only, and non-violating state
evolution is invisible above trace.

**Implemented July 18, 2026** (`src/ratelimit/networkcapture.h/.cpp`,
pinned by `tests/tst_networkcapture.cpp`): a capture-only instrument —
it observes traffic and never influences it. Durable, off by default;
enable with `network_capture_enabled=true` in `settings.ini`. Writes
`network-capture.jsonl` in the data directory (20 MB cap, one rotated
`.1` backup, flushed per record so a crash loses nothing). One JSON
object per line, `v` = schema version (currently 1):

- `kind: "reply"` — every exchange through a `RateLimitManager`,
  including network failures and 429s: policy, endpoint, request id,
  URL, `scheduled` (the computed next-safe-send, buffers included;
  updated to the intended retry time on a Retry-After retry), `sent`
  and `received` (local, ms), HTTP `status`, `error`/`error_string`
  when present, and a `headers` object with the verbatim
  `x-rate-limit-*`, `retry-after`, and `date` values (names
  lowercased — Qt normalizes header case anyway; values verbatim).
- `kind: "head"` — every setup probe, recorded *before* validation so
  degraded HEAD replies (N20) are captured: endpoint, URL, status,
  error, headers.

Every normal refresh session with the setting on becomes research
data. Saturation sessions on a many-tab account are the most valuable
(that is where pacing decisions actually bind). Captures from both
auth modes serve Q2; a shop stash-index run captures the legacy
endpoint's policy identity (N21c).

## Research method constraints

- **No active probing of limits.** Provoking 429s burns layer-4
  goodwill (N10) with a registered client id shared by all users;
  probing layer 1 risks users' IPs (N2). Passive observation of normal
  usage, plus GGG support's explicitly offered ask-us channel (N14),
  are the sanctioned instruments.
- **Community prior art is calibration, not ground truth** (N17).
  A bounded sweep of other clients' rate-limit code and issues may
  corroborate bucket-like anomalies or Cloudflare encounters; do it
  after the email archaeology and first captures, not before.

---

## Appendix A — What the current client does (evidence, July 17, 2026)

Condensed factual map; anchors are to code as of branch
`fix-f57-f59-ratelimit-retry` (docs-only commits over `03b15a3`).

**Rate-limited endpoints** (via `RateLimiter::Submit`, keyed by
endpoint label, managers deduplicated by policy name):

| Label | URL | Issued from |
|---|---|---|
| `List Stashes` | `api.pathofexile.com/stash[/{realm}]/{league}` | `ItemsManagerWorker::SubmitStashListRequest` |
| `List Characters` | `api.pathofexile.com/character[/{realm}]` | `SubmitCharacterListRequest` |
| `Get Stash` | `api.pathofexile.com/stash[/{realm}]/{league}/{id}[/{sub}]` | `ProcessTab` / folder-children path |
| `Get Character` | `api.pathofexile.com/character[/{realm}]/{name}` | `OnCharacterListReceived` |
| (URL as key) | `www.pathofexile.com/character-window/get-stash-items?...` | `Shop::UpdateStashIndex` (POESESSID cookie; no bearer) |

All with a 10s transfer timeout and a versioned `User-Agent`.

**Bypassing the rate limiter entirely:** league list (login), OAuth
authorize/token, forum-shop GET/POST (`edit-thread`, 300s timeout,
rate limiting detected by scraping "You must wait (\d+) seconds." from
HTML), RePoE static files, GitHub release check, poecdn images, imgur
upload, Sentry.

**Concurrency, today:** strictly serial twice over. Each
`RateLimitManager` sends one request at a time per policy; on top of
that, `ItemsManagerWorker` (since `ea9dd95`, v0.17.0) keeps one item
request in flight globally and its queue is stashes-first — the F56
starvation. Net effect: at most one API request in flight at any
moment, ever (which incidentally satisfies N18 and keeps bursts at 1).

**Pacing arithmetic** (`RateLimitPolicy::GetNextSafeSend`): if any
rule's state has `current-hits >= max-hits`, look back `max_hits`
events in that manager's reply history; for that event take
`max(request_time, received_time, server Date)`; add the rule's period;
add the bucket (5s if period ≤ 75s, else 60s) plus 1s buffer; next send
is the max over all saturated rules. Non-saturated sends get a 100ms
buffer. A 1s global minimum-interval guard exists but is dead code
(`last_send` never assigned — F58).

**429 handling:** on 429 with `Retry-After`, the manager waits and
resends — but the caller's `RateLimitedReply` is destroyed by the
nulling line in `ReceiveReply`, so the caller never hears the retry's
outcome (F57: worker counters never reconcile; update wedges until
restart). On 429 without `Retry-After`: logged, no retry. Violations
increment a session counter (`RateLimiter::OnViolation`) and dump
policy history at error level.

**HEAD setup:** first request to an unknown endpoint triggers a HEAD
probe, blocking in a nested event loop (F5/N18); non-2xx or missing
`X-Rate-Limit-Policy` on the probe is fatal to the app.

**Logging gaps for research:** see Instrumentation plan above.

## Appendix B — Related registers

- `docs/cleanup/findings.md` — F56 (starvation), F57 (429 wedge), F58
  (dead spacing), F59 (reply ownership): the paused fixes this research
  feeds. F5/F29/F30 standing constraints.
- `docs/design/items-pipeline.md` — the M2 spec must state where
  scheduling lives; it consumes this document's conclusions.
- `docs/design/network-redesign.md` — the design derived from this
  document (July 18, 2026): typed facade, coroutine pumps, the gate.
  Cites claims here by number; supersedes the paused F56–F59 fix
  shapes.
