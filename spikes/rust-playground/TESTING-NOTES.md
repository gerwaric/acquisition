# Testing notes — open questions

Discussion register, opened 2026-08-24 after rung 8 closed. **These are
open questions, not decisions.** Nothing here is settled and nothing here
authorizes work. `LIVE-TESTING.md` remains the control document for
anything run against the real API; `CONTEXT.md` holds invariants.

Context in one line: the rung-8 soak stopped without passing (see its
postmortem in `LIVE-TESTING.md`), and what it turned up is less about
that run than about how this project checks its own work.

## Why any of this exists (Tom, 2026-08-24)

The constraints and plans and abstractions are here to serve us in
(a) hardening the current implementation and (b) pinning things so that
when we drastically simplify the internals we break as little as
possible. **When they block or impede that service we can break or
change them.** This is a playground for experimenting, not an enterprise
development environment with layers of organization and process. We are
exploring, so complexity is growing, but the goal is to map out the
terrain so that future builders can make something better and simpler,
because they will have in hindsight everything we are discovering now
the hard way.

The other half is capturing system behavior in simple but generic
invariants that let us iterate on implementation and design without
losing trust in our ability to catch or prevent rate-limit violations
**and performance degradation**.

Read the rest of this file through that. Everything below is an
observation, a question, or a fact carried out of a run; the framing
above is the only thing here with any authority, and it is permission
rather than constraint.

## Surprises

These changed our minds, so they lead.

**1. Every rail was armed and correct, and none could see that the wrong
code was armed.** The soak ran a binary predating the fix it had been
restarted to pick up. Tripwire, ceiling, journal, dead-token stop — all
working, all blind to provenance. The rails verify *behavior*; nothing
verified *which code*. That is a category the rails design does not
address at all.

**2. R8 was confirmed live only because the fix was missing.** It had
been found by reading code, before any occurrence. Had the rebuild
happened, the soak would have been clean and there would be no live
evidence the hazard was real. A clean run proves less than it feels
like it does.

**3. A stop condition that could not fail.** `ACQ_IDLE_SHUTDOWN=604800`
means no restarts, so "more than one HEAD per route per day" was a
tautology rather than a guardrail. It read as coverage for 34 hours.

**4. The checker hid the evidence it existed to find.** `soak-check.sh`
compared timestamps as strings; `.` sorts below `Z`, so every send in
the start second was dropped — including the HEAD probe whose stop
condition it was evaluating. Fixed in `41305e5f`.

**5. The test suite already does what we had just decided not to do.**
16 sites in `daemon.rs` assert `requests.lock() == ["HEAD","GET"]` —
literal sequence matching on method names, against an in-memory
recorder attached to the test transport. The "invariants, not literal
sequences" position was reasoned out from first principles before
anyone noticed the codebase had settled the other way. *(Corrected by
experiment 2: this paragraph miscounted. See surprise 10.)*

**6. Ground truth stayed in sync by discipline, not topology.** Both
branches' copies of `network-ground-truth.md` were byte-identical, via
different commits on each side. It held, but nothing enforced it.

## Themes

**"Looks like coverage, isn't."** Surprises 3, 4, and 5 are three
independent instances inside one run. That is a failure mode this
project appears prone to, and the useful question is the general one:
*what else here reads as a check but cannot fail?*

**Provenance versus behavior.** Surprise 1 generalizes well past
binaries — version handshakes, build freshness, "is the artifact under
test the artifact I believe it is."

**Time as a test input.** The root cause of R8, the reason soaks are
slow, and the unlock for cheap long scenarios are one and the same
thing. `ratelimit.rs` already has a `Clock` trait and a `ManualClock`;
`daemon.rs` reads clocks directly at the two sites that matter (token
expiry, idle shutdown).

**Contract surface.** The send journal was built for live forensics but
may be the better test oracle, because it records what the *product*
emits — path, route, status, counted, rate headers, timestamp — rather
than what a test double happened to observe.

**Duration accidentally welded to risk.** The ladder escalates on blast
radius; wall-clock cost rode along invisibly until a low-risk rung
implied a week of waiting. The two axes are independent and were not
separated.

## The central tension

**"Simple but generic" and "catches real regressions" pull against each
other, and that is useful rather than unfortunate.** An invariant loose
enough to survive a drastic simplification may be too loose to catch a
subtle bug; one tight enough to catch the bug may break on every
legitimate refactor until someone quietly loosens it into meaninglessness.

This is not an incidental difficulty. It is goals (a) and (b) meeting at
the level of a single assertion: hardening wants invariants tight enough
to catch today's defects, pinning wants them loose enough to survive
tomorrow's rewrite. Any suite serving both lives on that line.

Two reasons to treat it as a working tool rather than a problem to
design away:

- **It is diagnostic.** Where the right tightness is obvious, the
  invariant is probably not pinning anything interesting. Where the
  choice is genuinely hard is where the real contract lives, and the
  argument about it is how you find out what the contract *is*.
- **Both failure directions are already named here.** Too tight is the
  16 sequence-literal assertions in surprise 5. Too loose is
  "looks like coverage, isn't" — an invariant that cannot fail is the
  same failure as a stop condition that cannot fail, arrived at from the
  other side.

The method below is the empirical way to locate the line rather than
argue it in the abstract.

## A method worth trying

Rather than inventing invariants — the four floated so far are guesses —
**derive them from the bug register**. Walk the L0 review R1–R12 in
`LIVE-TESTING.md`, `docs/cleanup/findings.md`, and the N-claims, and ask
of each: *what invariant over observable output would have caught this?*

Run the same walk for **performance**, not only safety. Every register
here is about violations and floods, and a suite that guards only those
will happily bless a rewrite that makes a 261-tab refresh three times
slower while staying perfectly inside every limit. Degradation needs its
own invariants and they are not free riders on the safety ones.

That yields a grounded list instead of a plausible one. The more
valuable half is the negative result: the bugs that **no** invariant
would have caught mark the boundary of what this approach can protect
during an internal rewrite. Knowing that boundary before the rewrite is
worth more than the invariant list itself.

## Experiment 1 — R8 as a scenario (2026-08-24)

The first thing tried, chosen because it touches topics 1, 2, and 3 at
once on the smallest piece of work. Plain-English version: give the
daemon a clock the test controls, pretend the laptop slept and then ten
hours passed, ask for a job, and check the daemon's own send log for a
rejected token.

**What was built.** `Clock` grew a second face, `wall()`, and a `kind()`.
One `SystemClock` now drives limiter, token expiry, and journal; the test
`ManualClock` can `advance()` both faces or `laptop_sleep()` the wall
alone. The journal stamps lines from that clock and opens each daemon
lifetime with a header line — `{"event":"open","pid","build","clock"}` —
where `build` is the git commit stamped into the binary by `build.rs`
(`<hash>` or `<hash>-dirty`). `soak-check.sh` refuses a journal with a
`manual` lifetime and prints the `(pid, build)` of each `system` one.

**The invariant** (`expired_token_after_laptop_sleep_is_refreshed_before_any_send`):
over the journal, not the sequence — no send is answered 401, a `POST
/token` reached the wire, and every send falls within 60 virtual seconds
of the scenario start. The server answers by *what was sent* (a stale
bearer gets 401 on any route), so it encodes no expected order.

**The breaker.** With expiry measured on the monotonic face again, the
test fails on its first send: `HEAD /character` carrying the stale bearer,
401 — the same shape the soak saw live at 21:50Z. Restored; 88 pass.

**Three more surprises, from an afternoon's work:**

7. **Topic 2's premise was false when written.** The journal stamped
   lines with `SystemTime::now()`, so under a manual clock a "within N
   virtual seconds" assertion would have read a 2 ms test as 2 ms and
   never failed. Caught by reading the code before building on it; this
   is exactly the "reads as a check, cannot fail" theme and it was one
   step from being built.
8. **An existing test hung once the daemon spoke scenario time.** It
   marked a token expired with the *machine's* `SystemTime::now()`
   (2026) while the daemon now compared against the scenario wall
   (2000): the token looked valid for 26 years and the awaited refresh
   never came. Tests must speak the daemon's clock. Expect this shape
   again anywhere a test reaches past the fake for real time.
9. **The bug-register walk gives about 5 of 13.** R1, R3, R8-env, R9,
   and clock-R8 have journal invariants; R4 (the journal itself failing),
   R5 (a race), R6/R7/R10/R11/R12 (config, reporting, persistence) do
   not. The boundary is clean: **the journal guards the wire; it cannot
   guard what the daemon reports or persists.** That is the right place
   for the boundary, because a rewrite is most dangerous on the wire and
   is allowed to change the reporting side.

**Still open from this experiment:** the idle-shutdown and activity sites
in `daemon.rs` read `Instant::now()` directly and are not on the clock
yet. (`acq --version` prints `<version> (<build>)` since the follow-up
commit, so the pre-run check is one command.)

## Experiment 2 — the 16, and journal == wire (2026-08-24)

Chosen next because topic 5 was the concrete "too tight" case and topic
2 needed the journal to be trustworthy before anything else could be
built on it. The first step was to read the 16 sites rather than argue
about them.

**What the 16 are.** Seven are sequence literals (`["HEAD","GET"]` and
variants); nine are counts (`len() == N`, `is_empty()`); two of the
counts also check refresh-token bodies. Every count is a *negative-space*
assertion — "exactly N reached the wire and nothing more" — which is the
safety property in its most honest form. They are not the too-tight case.

**The sequence is enforced by the script, not the assertions.**
`scripted_server` asserts method and path per connection inside its
spawned task, so the seven literals are redundant with the script, and
converting them would loosen nothing. The real coupling to the test
transport is the script. A wrong-order send panics in the server task
and surfaces as a hang at `server.await` — a check that fails badly.

**The recorder is the journal's independent witness.** The recorder is
what the server received; the journal is what the daemon *claims* it
sent. Surprise 9 put R4 (the journal itself failing) outside the
boundary. It is — unless something compares journal to wire, and until
today nothing did. Converting the 16 to journal assertions would have
*discarded* the only witness that could catch R4.

**What was built (b).** Every harness daemon now journals, on the
test's clock, to a file beside its log. `assert_journal_matches_wire`
reads the journal back and requires its send methods to equal the
recorder's, in order, with the header as line 0. It runs in the 13
tests that hold a recorder (via `finish_harness_wire` or explicitly),
including the three token-server tests and the two with an empty wire.
The 16 assertions are untouched; `tripwire_rails()` became
`tripwire_config()` and the rails handle comes from
`daemon.choke.rails()`.

**The breaker.** Skip the journal write for `HEAD` in
`Rails::journal_line` and 8 of the 13 fail on "journal (left) disagrees
with what the server received (right)"; the five that pass are the ones
whose wire has no HEAD. Restored; 88 pass.

**Surprise 10.** The count in surprise 5 was wrong in a way that
mattered: "16 sequence literals" was really 7 literals and 9 counts,
and the counts were already the right shape. The position that the
suite "settled the other way" was itself reached without reading the
suite. Reading the sites before deciding what to do with them changed
the plan from *convert* to *cross-check*.

**(d) — `wait_ms`, same afternoon.** Every journal line now carries
`wait_ms`: how long the send was held from *ready* to transport
dispatch. "Ready" is the instant the dispatcher picked the job
(`Daemon::process`), threaded down as `ready` through `execute` /
`api_get` / the probe, and into the choke's `get` / `get_bearer` /
`head` / `post_form` as `since`; a token refresh is ready when it
enters the choke. The four `clock.slept()` assertions became
`journal_waits(&log_path) == [..]` per send, `ManualClock` lost its
sleep counter, and one script now answers `Retry-After: 5` so the third
GET pins `5_000 + hold` rather than `hold` again.

**Surprise 11 — the trap, met while building.** The first draft
measured the wait inside the choke, from method entry to permit. That
reads 0 in exactly the retry tests, because the dispatcher pre-waits
the whole hold in `daemon.rs` *before* calling the choke. A `wait_ms`
that cannot see the wait is surprise 7 again: a field that reads as
observability. Caught by reading the dispatcher before wiring it.

**The invariant that fell out — a first piece of (c), as predicted.**
`assert_pacing_follows_responses` runs over every harness journal from
`finish_harness_wire`, derived from N19 rather than from the code: a
send on a route is held *only* if the previous landed response on that
route was a 429, and then for at least its `Retry-After` and at most
`Retry-After + RETRY_BUCKET_PAD + BUFFER`. No test names a number for
it. It is both halves at once: the floor is safety (a retry that goes
out too soon), the ceiling and the "otherwise zero" arm are performance
(a rewrite that paces slower, or paces when nothing asked). It is the
first assertion here that is loose enough to survive a rewrite and
tight enough to have a breaker on each side.

**Breakers (verified 2026-08-24).** One extra second of sleep before
every send in `process()`: 9 tests fail, all through "nothing demanded
a hold on fetch, yet the send waited". Drop `Retry-After` from the hold
in `ratelimit.rs`: the 5-second floor fails in the daemon test, and two
limiter unit tests that already pinned it. That second result is worth
saying plainly: the *limiter* was already tested for this; what is new
is that the same fact is now checked at the product boundary, where a
rewrite of the limiter cannot take the test with it.

**Still open:** (c) as a deliberate reading pass over the register with
`wait_ms` in hand — the pacing invariant is one derived from N19; the
walk should say which others the journal can now carry and which it
still cannot. `soak-check.sh` does not yet read `wait_ms`; a live
journal now carries real pacing data that nothing summarizes.

## Experiment 3 — the register walk (2026-08-24)

The reading pass the method section asked for, done with `wait_ms` and
journal == wire in hand. For each entry: *what invariant over the
journal would have caught it*, and whether it now runs over every
harness journal (`finish_harness_wire` and the R8 scenario).

| Entry | Journal invariant | Status |
| --- | --- | --- |
| R1 dead grant re-sent per flight | no `POST oauth-token` after a 400 on it, same pid | expressible; **rails-conditional** — with rails off the product still does this, so it pins the rail, not the product. Per-test only. |
| R2 no violation budget | ≤ N 429s per pid | expressible; the product has no such rule to pin. Not applied. |
| R3 no HTTP timeout | — | **cannot**: a hung send never journals; absence is invisible to a line-based oracle. Only the test's own timeout sees it. |
| R4 journal itself | journal == wire | **applied** (experiment 2 b). |
| R5 no ceiling | sends per pid ≤ max | per-test (`ceiling_halts…`), config-dependent. |
| R6 `profile` reaches token endpoint | no sends at all in that scenario | per-test; the *reason* (job kind) is not on the wire. |
| R7 keyring save failure | — | cannot (persistence). |
| R8 monotonic expiry | no 401; refresh reaches the wire; all sends within N virtual s | **applied** (experiment 1). |
| L0-R1 parked send after trip | no send after a tripping status until reset | per-test; the harness cannot see resets. |
| L0-R4 journal open failure | — | cannot (the journal's own failure, from the other side). |
| L0-R5 second dead-token POST (race) | same as R1 | the *outcome* of the race is on the wire even though the race is not. Same rails caveat as R1. |
| L0-R9 `ACQ_MAX_SENDS=0` | empty journal | per-test. |
| L0-R2/3/6/7/8/10/11/12 | — | cannot (config, reporting, persistence). |
| N16/N24 probe before send | per (pid, route ≠ oauth-token) the first send is HEAD | **applied**, `assert_wire_contract`. Listed under "sound and unchanged" in the blast-radius review and until today tested nowhere generically. Breaker: fall through on `Unknown` → 10 fail, incl. the R8 scenario. |
| N24 HEAD not counted | `counted == (method != HEAD)` | **applied**. Pins the product's accounting, not GGG's. Breaker: `true` in `head()`'s record → 9 fail. |
| N19 Retry-After + bucket | pacing floor/ceiling | **applied** (experiment 2 d). |
| N33 token endpoint paced | pacing invariant covers the route | **applied** by construction. |
| N34/R8 401 then refresh | after a 401 the next send is `POST oauth-token` | **applied**; no offline breaker — the harness has no scenario where a 401 lands and the refresh does not follow. Honest status: armed, unproven. |
| N18/F5 one HEAD in flight | overlap of HEAD intervals | **cannot as-is**: the journal stamps completion only. A `started` timestamp (or `duration_ms`) would make it expressible. Cheap; not built. |
| N4/N25/N26 never over the limit; burst-then-stall shape | per line `state ≤ limit`, `restriction-active == 0` on non-429; spacing per policy from `ts` + headers | expressible — the journal carries the policy's limits and state on every line — but **vacuous in the harness**: the scripted server echoes whatever headers the script names, so it would pin the script. Real only live (`soak-check.sh`) or against a mock that keeps counters. |
| N6 same-name policies share counters | as above, keyed by policy name | same: needs a counting mock. |
| ≤ MAX_429_RETRIES+1 attempts per job | — | cannot: job identity is not on the wire, and a rewrite is allowed to change it. Right side of the boundary. |

**The boundary, restated after the walk.** Three kinds of "cannot":

1. *Not on the wire* — reporting, persistence, config, job identity.
   The right side; a rewrite may change all of it.
2. *Absence* — R3. A line-based oracle cannot see a send that never
   completes. This is a real gap and the journal will never close it;
   the answer is elsewhere (timeouts, or a `started` line).
3. *Vacuous offline* — N4/N6/N25/N26. The journal can carry them, the
   product is subject to them, and the harness cannot exercise them
   because the mock has no state. This is the interesting one: the
   most important live properties are the ones the offline suite is
   structurally blind to. A `mockggg` that keeps real per-policy
   counters and answers 429 on its own would move the whole row into
   the harness — and is probably the single largest remaining gain in
   offline confidence. It also bears on topic 4.

**Surprise 12.** The walk's most valuable output was not an invariant
but a *kind*: rails-conditional. R1 and L0-R5 are "fixed" only while
the tripwire is armed. With rails off — the shipped default — the
product still re-sends a dead grant per flight. The register had
recorded them as resolved; they are resolved *for the ladder*.

**Surprise 13, process.** While running breakers I chained a `cd` that
failed silently into a `git checkout` that discarded an uncommitted
invariant. Recovered from the transcript, re-applied, committed
*before* the next breaker. Rule: commit the check, then break the code.

## Open topics

1. **The clock fork — resolved by experiment 1:** one clock, two faces.
   Remaining: the idle/activity `Instant::now()` sites, when a scenario
   needs restarts.
2. **Is the journal sufficient as a contract surface?** Since
   experiment 2 (d) it records *how long* each send was held
   (`wait_ms`); pinning pacing did not need *why*, because the
   preceding response on the route already says why. Remaining: Is the CLI's `--json` outcome a necessary second
   surface for job results and data correctness? Since experiment 1 the
   journal's timestamps *are* the scenario's, so "this refresh completes
   within N virtual seconds" is a deterministic, free-to-run
   *performance* invariant off a surface that already exists — which
   covers the degradation half of the goal without new instrumentation.
   (Before experiment 1 this paragraph was wrong; see surprise 7. The
   `wait_ms` field proposed here was built in experiment 2 (d).)
3. **Provenance — resolved as process with a code assist:** the binary
   carries its commit, the journal and log say which one, and
   `soak-check.sh` refuses what it cannot trust. The daemon does not
   refuse to run; a script that refuses to *evaluate* is enough.
4. **What does the live ladder shrink to?** Arguably only what the mock
   cannot answer, which is already written down as README's "Known
   gaps". If so, most future confidence is bought offline and the ladder
   becomes short and rare. Experiment 3 sharpened this: the mock's
   *statelessness* is what keeps N4/N6/N25/N26 live-only. A counting
   mock is the lever.
5. **The 16 existing assertions — resolved by experiment 2: leave them,
   cross-check them.** They are counts and redundant literals, coupled
   to the *script* rather than to the assertions; the journal is now
   held equal to the wire in every test that has one. The simplification
   may still eat them; that is its call.
6. **How much of this is a hedge that pays either way on ADR-0003?**
   Behavior pinned at the network boundary survives both rewrite and
   evolve. Does that change how much to invest now, before the decision?

## Standing constraints carried from the soak

- Verify the **binary**, not the checkout, before any live run. Since
  experiment 1: the daemon's first log line and the journal header carry
  `build`, as does `acq --version`; it must equal
  `git rev-parse --short=12 HEAD` with no `-dirty`.
  (The older check, `strings target/debug/acq | grep 'token rejected'`
  returning two lines, still works for the R8 fix specifically.)
- A future soak must derive its ceiling from cadence × intended
  duration; 200 sends at one per 10 min is 33 h, not "several days".
- Express the HEAD condition per `(pid, route)` — the journal records
  `pid` — so it means something in both the pinned and restarting
  shapes.
- Ground-truth claims are authored on the master-side branch and
  cherry-picked to the spike, never the reverse.
