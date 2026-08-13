# Slice hand-off: clause registry

Status: **closed 2026-08-12 — review round one (REG-R1).** Tom closed
with no blocking findings, after an independent row-for-row
reconciliation by the obligation-map audit session (125 map rows
confirmed; per-section coverage-class distributions exact across all
27 groups; `OPEN_UNTESTED` matched the set predicted from the map;
two mutation checks replayed on fresh instances, both failing on the
intended rule). Four observations REG-R1-F1–F4, dispositions recorded
in the `result-draft.md` §9 closure entry — including F3: the
judgment-call-4 candidate collapse of the two tripwire-feed rows is
**declined** (they are distinct call-site lanes). Coding completed
2026-08-12; per AGENTS.md the slice ended at review, not at green.

Historical record — live state lives in `status.md`. (2026-08-12)

Contract: `clause-registry-design.md` (accepted rev 1, `ce5730d4`).
Migration source and acceptance oracle: `obligation-map.md` at
`f03632b9` (code at `e2034807`), now marked superseded-by-registry.
Deliverables: `src/obligations.rs` (122-entry `CLAUSES`,
`OPEN_UNTESTED` with 13 ids), `tests/obligations.rs` (six
verification tests), the supersession marker, the §9 changelog
entry, and — kickoff commit 1 — the finding-ID namespaces subsection
in `slice-review.md` §5.

Reconciliation arithmetic: 122 entries = 125 map rows − 2 collapses
− 1 omission (all three below). Coverage tallies: 63 Full /
33 Partial / 14 Untested (13 open + 1 accepted) / 12 Excluded —
matching the map class-for-class.

## 1. Silences taken

1. **Disposition strings needed literals.** The design requires
   every `Untested` note to "name a disposition" but does not fix a
   spelling. Chosen: the note must contain the literal
   `open — flagged for Tom` or `accepted —` (with its register
   citation); the computed open set keys on the open literal.
   Consequence one step further: those two spellings are now
   load-bearing constants in `tests/obligations.rs` — a future
   accepted-not-fixed disposition written any other way fails
   verification until it adopts the spelling, and a note containing
   *both* literals counts as open (the conservative side).
2. **Composite owning-row cells.** Map rows like "M2; padding
   arithmetic owned by C1", "judge/B13", "B8/B2 (mock side)", and
   "ambiguous — see §8.5 item 3" had to fit a single `owner` field
   from the fixed vocabulary. Chosen: the primary (first-named or
   bolded) owner — `m9-race-exposure-attribution` → B13,
   `m6-mock-hits-are-facts` → B8, `m7-threshold-tuning` → M7 with
   the ambiguity recorded in its note. Consequence: series
   reachability and any future "all owned clauses Full" derivation
   key off that primary owner; re-allocating one (e.g. Tom's §8.5
   item 2 ownership sentence for M11a) is a one-field diff, plus the
   `OPEN_UNTESTED` line if coverage state moves with it.
3. **ID namespace for cross-owned rows.** The design says IDs are
   "namespaced by owner", but rows that live in an M table while
   owned elsewhere (M3's parse clause → C2, M12's trip logic → C4,
   M12's ladder → M8) would then collide with or shadow the owner's
   own §2 rows. Chosen: IDs are namespaced by the map section the
   row came from (`m3-header-parse-typed`, owner `C2`), keeping
   row-for-row reconciliation greppable. Consequence: grepping by
   owner uses the `owner` field, not the id prefix; three of the
   design's four example IDs still match verbatim (see judgment
   call 3 for the fourth).
4. **`Excluded` owner restriction made structural.** The design
   describes `Excluded` as "U/O rows" without demanding a check; the
   verification test asserts it. Consequence: a future exclusion
   outside U-/O-series fails the build and forces an explicit
   vocabulary decision instead of quiet reuse of the state.
5. **Non-row map text was not migrated.** The §6 trailing "bounds
   hygiene (cross-cutting)" line and the §5 trailing gate-summary
   note are commentary, not rows; they have no entries. Recorded
   here so the absence reads as deliberate; the bounds-hygiene tests
   remain cited from the B-series notes they support.
6. **Anchors for C/X/U/G/B/O entries.** The map carries
   `scenarios.md` anchors only for M-series headers and scattered §8
   references; the design's `text` field wants an anchor per entry.
   Taken mechanically from `scenarios.md` heading/ID lines at
   `e2034807` — a lookup, not an interpretation; no contract text
   was read for meaning beyond the map's own wording.

## 2. Seam map and cross-slice invariants

This slice adds const data (`src/obligations.rs`) and a read-only
test. It mutates no engine, mock, judge, or driver state, and no
existing file changed except the three documents. The one production
cost is the design-D2 accepted one: a few KB of const data now lives
in the lib target; nothing reads `CLAUSES` at runtime (payoff wiring
is explicitly deferred, design §6).

The six AGENTS.md cross-slice invariants are untouched because no
code path changed: (1) no-permanent-wedge — no state added that
could wedge; (2) one-send-one-entry — reservation identity code
untouched; (3) pessimism direction — no history mutation introduced;
(4) `try_reserve` sole authority — no new scheduling input;
(5) entry-point invariant — no dispatch path touched;
(6) notifications tell the truth — no `StateChanged` emission point
added. The verification test never calls production code at all — it
reads source files as text (so it is not a mirror-oracle of
anything; there is no arithmetic to mirror).

One seam deserves the reviewer's eye anyway: the registry now
*duplicates in structured form* what the evidence registers say in
prose. Until the doc-split slice collapses the result-draft M-row
"remaining" prose onto registry pointers (design §6), the two can
drift; the supersession marker on the map closes the largest such
window, but `result-draft.md` §3 rows still carry their own coverage
wording, unchanged by this slice on the kickoff's instruction.

*[Marker, 2026-08-13 (DS-R1): the doc-split slice ran and
deliberately declined the collapse — its kickoff downgraded design
§6's prediction to a single §3 preamble sentence. The drift window
this paragraph warned about is therefore standing, now tracked in
`status.md` §5 item 2.]*

## 3. Coverage confession

What the verification tests deliberately do not check:

- **Assert strength (audit class 3).** A cited test that exists but
  asserts less than its clause passes verification. This is the
  design's stated limitation; the `must_assert` line on every
  citation is the reviewer's hook, and those lines were written from
  the map's own row notes — reviewing their fidelity is part of this
  slice's review, not automated.
- **The content search is textual.** `fn <name>(` found in a file
  satisfies source-existence even if the match were a comment or a
  non-test helper; the check does not prove the fn is a test, is not
  `#[ignore]`d, or runs in the release profile. Accepted (design D3)
  — the census cross-check below is the compensating control.
- **`must_assert` truthfulness.** Non-emptiness is asserted;
  accuracy is not machine-checkable.
- **No vacuity exposure:** the six tests are plain loops over a
  table asserted non-empty; every clause hits every applicable
  assertion — there is no generated input to leave a branch
  unvisited.
- **The registry does not yet feed the judge.** `ContractCoverage`,
  `verdict_eligible()`, and the driver's fragment declarations are
  untouched (non-goal, design §3); a green registry changes no run's
  verdict eligibility.

Census: at migration time the source inventory was exactly the
kickoff's pinned 129 test fns (127 release); with the six registry
tests the matrix is now 135 debug / 133 release.

Readiness notes for the open `Untested` clauses (tests remain out of
scope here — Tom sequences): `m12-tripwire-feed` /
`m1-probe-429-tripwire-feed` (assert a 4xx increments the counter
through each feed call site; C4 already pins the counter itself) and
`c3-trip-latched` (advance past a trip, re-ask, assert still halted)
are one-line-scale tests, ready whenever scheduled.
`x1-trip-drain-publish` needs a fuse-trip variant of the existing
Cloudflare drain test. `x2-single-send-path` is blocked on Tom's
decision about what a spike-scope structural test even is (map §8.2
item 2; design §7 item 1 kept it open deliberately). *[Marker,
2026-08-13: dated readiness analysis — sequencing and the X2
decision live in `status.md` (§5, §3 item 2).]*

## 4. Judgment calls

1. **Six focused tests, not one** (licensed): each mutation class
   fails a test whose name states the broken rule.
2. **Primary-owner rule for composite cells** (silence 2 above) —
   a different session might have introduced multi-owner support;
   that seemed like schema invention beyond the accepted design.
3. **`b1-header-protocol`, not the design example
   `b1-retry-after-emission`.** The map row is the whole header
   protocol; the retry-after wire emission is its recorded gap, in
   the note. The design's example IDs were illustrative; the other
   three (`m12-tripwire-feed`, `c3-trip-latched`,
   `x2-single-send-path`) match verbatim.
4. **Two map rows kept separate rather than merged:**
   `m1-probe-429-tripwire-feed` and `m12-tripwire-feed` — §8.1
   item 4 treats them as one clause with two call sites, but the map
   presents two rows and the design says row collapses are rev-2
   review edits (design §7 item 3). Candidate collapse for review.
5. **Note text lightly normalized where the schema demanded it:**
   two Partial rows with empty map note cells
   (`m6-g1-post-announcement`, `m8-no-follow-on-violation`) got
   their implied delta ("fragment scale") written out; map cells
   that named tests parenthetically as *non*-discharging evidence
   (`m4-watch-status-published`, `m5-stale-window-exposure`) had
   those names moved into notes, since `Untested` forbids citations.
   Owner, coverage class, and discharging citations were never
   altered.

## 5. Migration findings (recorded, not fixed)

1. **`c4-halt-semantics-shared`: the map's `partial` rests on a code
   citation.** The map's discharging column for this row is
   `src/actor.rs:254–268` (the shared `halted` latch) — code, not a
   test, which the registry schema cannot hold as a citation.
   Kept `Partial` per the no-reclassification rule; cited the
   nearest real test (`c4_pins_burst_sustained_and_exact_window_edges`,
   which pins the trip itself) and recorded the mismatch in the
   entry's note. Tom may prefer `Untested` for the halt-sharing
   half; that is a coverage-state decision, so it is his.
2. **The dropped-dispatched-`RequestTicket` lifecycle remains
   unowned and unrepresented** (map §8.1 item 2). Design §7 item 2
   (a `SHELL`-owned entry) was still undecided when the slice ran,
   so per the acceptance disposition the entry is omitted and the
   gap is carried here: it is currently the only known obligation
   living solely in prose confessions. When Tom decides, adding it
   is a two-line diff (entry + `OPEN_UNTESTED`).
3. **Two U-register pointer rows collapsed** (licensed, recording
   required): the M5 table's "remap triggers beyond reactive are U1"
   row and the M12 table's "server-side restriction behavior" row
   are the U1/U2 exclusions respelled; each U entry's note names the
   absorbed row.
4. **No dangling citations found during migration** — every test
   name the map cites resolved by content at `e2034807`, consistent
   with the audit's §8.4 clean check.

## 6. Mutation checks (design §5 criterion 2 — all six demonstrated)

Each mutation was applied to the committed tree, observed to fail
`cargo test --locked --test obligations` with the message quoted,
then reverted (`git checkout`); the suite was re-run green after the
last revert.

1. **Remove one citation** (the sole citation of
   `c2-missing-headers-typed`) →
   `coverage_and_citation_counts_are_consistent`: *"clause
   c2-missing-headers-typed claims Full coverage with zero
   citations"*.
2. **Point a citation at a nonexistent fn**
   (`fuse_uses_the_documented_half_open_boundariez`) →
   `every_cited_test_fn_exists_in_source`: *"cites src/actor.rs ::
   … but no such fn exists"*.
3. **Misspell a cited file** (`src/mock/modle.rs`) →
   `every_cited_test_fn_exists_in_source`: *"clause
   b3-server-owned-phase cites unreadable file src/mock/modle.rs:
   No such file or directory"*.
4. **Add a citation to an `Excluded` clause** (`u1-proactive-remap`)
   → `coverage_and_citation_counts_are_consistent`: *"is Excluded
   but cites tests — either the coverage state is stale or an
   exclusion is drifting"*.
5. **Remove an id from `OPEN_UNTESTED` without changing the entry**
   (`c3-trip-latched`) → `open_untested_matches_the_computed_set`:
   declared/computed disagree.
6. **Duplicate an id** (`b14-zero-skew-date` →
   `b13-observation-log`) → `ids_are_unique_and_owners_are_known`:
   *"duplicate clause id: b13-observation-log"*.

## 7. Gate matrix

All at the slice head, in `spikes/rate-limit-core/`:

- `cargo test --locked` — 135 passed, debug.
- `cargo test --locked --release` — 133 passed (the two drop-bomb
  tests are debug-gated, as recorded).
- `PROPTEST_CASES=4096 cargo test --locked` — 135 passed; all
  properties at 4,096 cases.
- `cargo clippy --locked --all-targets -- -D warnings` — clean.
- `cargo fmt --check` — clean.
- `git diff --check` / clean tree — clean.
