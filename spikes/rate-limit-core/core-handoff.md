# Composite hand-off: the implemented core

Status: review artifact per `slice-review.md` §2, written 2026-08-09
after the external review noted its absence for the composite core
(the hand-off process postdates the first slices). Covers everything
implemented through the external-review fix series and the follow-up
verifier fixes of 2026-08-10 (probe-lane 429 recording, byte
ceilings, 6 h period ceiling). Supersedes nothing; future slices
attach their own hand-offs — `bootstrap-handoff.md`, `mock-handoff.md`,
and `actor-handoff.md`, in build order. §3's deliberate-coverage
paragraph is superseded in effect by all three; see the note there.

Historical record — live state lives in `status.md`. (2026-08-12)

*[Marker, 2026-08-13 (DS-R1): this composite slice has no single
closure record — its reviews were the 2026-08-09 audit,
external-review, and follow-up verifier rounds (`result-draft.md` §3
registers), which predate the `slice-review.md` closure rule. The
chain treats it as closed; noted so the missing record reads as
history, not an oversight. Accepted as-is at the DS-R1 closure,
2026-08-13: the chain and this marker are the record — no
retroactive closure entry is manufactured.]*

## 1. Silences taken (doc gaps, the reading chosen, consequence traced)

| Silence | Reading taken | Next-call consequence |
|---|---|---|
| 429 with valid policy headers but unusable `Retry-After` | record a `RETRY_AFTER_CAP`-length restriction, refuse the request | `try_reserve` answers `NotBefore` until cap + bucket + buffer |
| Late 429 after halt / suspension | send-promising dispositions (Requeue, ProbeReady) refuse; outcome-delivering ones still deliver | requeue queues can no longer receive unkeepable promises |
| Zombie 429 from an expired confirmation | joins the episode regardless of generation | episode state unchanged; no double-escalation, no abort |
| Abandonment vs slow live token | age past the largest padded window = written off (shell obligation: resolve tokens well inside that horizon) | attempt consumed; a contract-violating shell risks a bounded double-confirmation window, never a wedge |
| Unbounded wire values | absolute ceilings at parse: 8 rules, 8 triplets, 10 000 hits, 21 600 s periods, 256/64-byte names, 64-byte diagnostics, and a 1024-byte whole-value gate on raw bytes (incl. Retry-After) before any conversion or scan | out-of-range headers are typed refusals without wire-sized allocations or wire-sized parsing work |
| Physical retention | retire entries past the largest padded window at both mutation surfaces | an observation window longer than every configured padded window may re-synthesize — pessimistic |
| Duplicate rule names / duplicated headers | parse as-is; first header value wins | harmless under max-not-sum reconciliation; pinned as current behavior |
| Zero-hit windows | wire-refused (D8); engine answers `Blocked` for constructed policies | defense in depth only |

All entries have register records in `result-draft.md` §3.

## 2. Seam map and invariants walk

State each mechanism touches across slice boundaries: abandonment
expiry × episodes (resolves the slot as a failed attempt);
retirement × abandonment (same horizon — a retired confirmation
entry reads as aged); retirement × token consumption (tolerant
paths); terminal state × both response lanes; reconciliation ×
retirement (counts run after retiring).

1. **No permanent wedge** — confirmation slots and entries both age
   out on the padded-window horizon; `Blocked` is re-asked on state
   change, never slept on.
2. **One send, one entry** — interleaving property now spans
   reserve/rollback/observe/unknown in any token order; retirement
   removes only aged entries, and consuming a retired token is a
   no-op on history.
3. **Pessimism direction** — synthesis targets min(reported, cap);
   retirement can only lower local counts where re-synthesis
   restores them; unknown outcomes keep entries until the horizon.
4. **Single scheduling authority** — no new scheduling paths;
   refusals on terminal state *remove* unkeepable promises rather
   than adding timing channels.
5. **Entry-point invariant** — swept across nine response shapes;
   the halted/suspended gates return Refuse, never a cross-lane
   disposition.
6. **Truthful notifications** — StateChanged still tracks actual
   mutation; the hoisted restriction recording sets it on every 429.

## 3. Coverage confession and property reachability

Not covered, deliberately: mock/M-series, C3/C4, X1/X2, actor
(unbuilt, build-order); M5 remap and M6 shrink (deferred by
decision); bootstrap seeding (accepted design, unimplemented).

**Superseded in effect, 2026-08-12** (the paragraph above is kept as
the dated 2026-08-09 record, per the seeding-review precedent for
`core-design.md`). Every item in it has since been built and reviewed:
bootstrap seeding in `708b32d8..17363429` (`bootstrap-handoff.md`), the
mock and M-series harness in `4353fb03..74d589fe` (`mock-handoff.md`),
and the Tokio actor shell in `d0eabcae..02b60f47` (`actor-handoff.md`),
which carries C3, C4, X1, X2, M5 remap, and M6 shrink. For what remains
open, read `actor-handoff.md` §3 — not this paragraph.

Not covered, honestly: no property sweeps the halted/suspended
dimension (example tests only); the retirement × long-observation
re-synthesis interaction is reasoned, not property-tested; the
double-confirmation window under a contract-violating shell is
untested; drop-bomb tests are debug-profile only (release runs 71 of
73).

*[Marker, 2026-08-13 (DS-R1): the paragraph above is the dated
2026-08-09 record, like the one before it — the test counts are of
that date (the matrix is now 135 debug / 133 release), and current
coverage state for every item is the registry's
(`src/obligations.rs`), not this confession.]*

Reachability accounting: C1 asserts on every generated branch
(grant → oracle; NotBefore → re-ask must grant → oracle); the
interleaving property generates ≥ 1 operation and asserts the full
id/kind map after each; both reconciliation properties assert
unconditionally; the C2 round-trip domain equals the accepted
grammar, with every rejected complement pinned by name.

## 4. Judgment calls

- Zombie-429-joins (Tom-confirmed) extended to *any* generation by
  deleting the open_or_join assert rather than tracking expired-
  token generations — less machinery, same conservatism.
- Restrictions are recorded on every valid 429 even when the
  disposition refuses (halted/suspended/unusable-Retry-After) —
  uniform pessimism over minimal mutation. The probe lane initially
  violated this (gates ran before the 429 branch); caught by the
  follow-up verifier review and fixed 2026-08-10.
- The probe lane's suspension gate goes beyond the reviewer's
  finding (they named halt only) — same unkeepable-promise
  reasoning.
- Retirement horizon = abandonment horizon (one aging concept, not
  two).
- Ceiling values: counts and hits (8/8/10 000) are mine, far above
  the observed 2 rules / 2 triplets / 180 hits, small enough to
  bound worst-case synthesis at ~240 KB per policy. The original
  3600 s period ceiling rested on a wrong evidence claim (45 hits /
  300 s; ground-truth N23's legacy Ip rule reaches 180 / 1800 s) —
  caught by the follow-up verifier review. Tom set 21 600 s (6 h,
  12x observed, 2026-08-10): a period sizes no allocation, and an
  over-ceiling policy refuses the endpoint, so availability wins.
- Byte ceilings (256-byte policy names, 64-byte rule names,
  64-byte truncated diagnostics) close the follow-up review's
  finding that names and error payloads were still wire-sized. The
  verifier's second pass (P2) added the 1024-byte whole-value gate:
  the field ceilings bounded copies but not scan work — `to_str`,
  trim, split, and digit validation still ran over the full wire
  length. The gate checks raw bytes first, in `required_header` and
  `parse_retry_after` both.
