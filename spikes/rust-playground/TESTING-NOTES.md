# Testing notes — closed record

How this project checks its own work. Opened 2026-08-24 as a discussion
register after the rung-8 soak stopped without passing; closed the same
day after three experiments. This is the permanent short record: the
purpose, the findings that changed our minds, the boundary of what the
offline suite can see, and what a rewrite keeps. Per-experiment narratives
(breakers, commit-by-commit builds, the thirteen numbered surprises) live
in git history — the file at `7aee76ee` holds the full text.

`LIVE-TESTING.md` remains the control document for anything run against
the real API; `CONTEXT.md` holds invariants and decisions, including the
branch's status as the **reference implementation** and the rule that
tests pin behavior at boundaries, never mechanisms.

## Purpose

The tests, constraints, and abstractions here serve two things: hardening
the current implementation, and pinning behavior so that the daemon's
internals can be rewritten — in place, or by a fresh build judged by the
same suite — without losing trust in our ability to catch rate-limit
violations **and performance degradation**. When they block or impede that
service we break or change them.

The central tension is that "simple but generic" and "catches real
regressions" pull against each other at the level of a single assertion:
too tight breaks on every legitimate refactor until someone loosens it into
meaninglessness; too loose is a check that reads as coverage and cannot
fail. Where the right tightness is obvious the invariant is probably not
pinning anything interesting; where it is genuinely hard is where the
contract lives. The method that located the line was empirical: walk the
bug register and ask of each entry what invariant over observable output
would have caught it.

## What changed our minds

Kept short; each of these is a shape we expect to meet again.

- **The rails verify behavior; nothing verified which code.** The soak ran
  a binary predating the fix it had been restarted to pick up, and every
  rail was correct and blind to it. Provenance is its own category. Since
  then the binary carries its commit (`build.rs`), the daemon's first log
  line and the journal header print it, `acq --version` prints it, and
  `soak-check.sh` refuses a journal it cannot trust.
- **"Looks like coverage, isn't."** Met five times in two days in different
  clothes: a stop condition that could not fail (no restarts, so "one HEAD
  per route" was a tautology); a checker that compared timestamps as
  strings and dropped the evidence it existed to find; a journal stamped
  with real time that would have made a virtual-time assertion vacuous; a
  `wait_ms` measured where the wait could not be seen; a test that reached
  past the fake clock for `SystemTime::now()` and hung. The general
  question — *what else here reads as a check but cannot fail?* — is the
  most useful one this project has, and it will matter more, not less, if
  a goal-seeking agent ever builds against this suite.
- **A clean run proves less than it feels like it does.** R8 was found by
  reading code and confirmed live only because the fix was missing.
- **Read the sites before deciding what to do with them.** The "16
  sequence-literal assertions" were 7 redundant literals and 9 counts, and
  the counts were already the right shape. The plan changed from *convert*
  to *cross-check*.
- **Rails-conditional fixes.** R1 and L0-R5 were recorded as resolved; they
  are resolved only while the tripwire is armed. With rails off — the
  shipped default — the product still re-sends a dead grant per flight.
  Now L0-R13 and a CONTEXT.md decision.
- **Commit the check, then break the code.** A chained `cd` that failed
  silently into a `git checkout` discarded an uncommitted invariant once.
- **Two clocks in one test is a harness bug that looks like coverage.**
  The N6 stress drove `mockggg` on real time and the daemon on a manual
  clock, so the mock's windows never expired during the test and the
  sixth fetch's retries always met a still-restricted mock. Its "one job
  fails after three 429s" was that mismatch, not a property. On one clock
  the limiter holds the sixth send and nothing violates — which is what
  N4 says should happen. Found by putting the mock on the `Clock`
  (2026-08-24, `7762d967`).

## The journal is the contract surface

The send journal was built for live forensics (rail 4) and turned out to
be the better test oracle, because it records what the *product* emits —
pid, method, route, status, `counted`, every rate header, a timestamp on
the daemon's clock, and `wait_ms` (held from dispatcher-ready to transport
dispatch) — rather than what a test double happened to observe. Every
harness daemon journals; `assert_journal_matches_wire` holds the journal
equal to what the test server received, so the journal's own failure (R4)
is caught by the recorder and the recorder's coupling is caught by the
journal.

Invariants that run over every harness journal, derived from the register
rather than from the code:

| Invariant | Source | Pins |
| --- | --- | --- |
| journal send methods == wire, in order, header as line 0 | R4 | the oracle itself |
| per `(pid, route ≠ oauth-token)` the first send is `HEAD` | N16/N24 | probe before send |
| `counted == (method != HEAD)` | N24 | the product's accounting |
| after a 401 the next send is `POST oauth-token` | N34/R8 | refresh on rejection (armed, no offline breaker yet) |
| a send is held only if the previous landed response on its route was a 429, then for `[Retry-After, Retry-After + RETRY_BUCKET_PAD + BUFFER]`; otherwise `wait_ms == 0` | N19, N33 | safety floor and performance ceiling at once |
| R8 scenario: after a laptop sleep past expiry, no 401, a refresh reaches the wire, all sends within 60 virtual s | R8 | wall-clock expiry |
| no send answered 429; every reported window state `hits ≤ max` with no restriction active (`assert_never_over_the_limit`) | N4/N25 | never over the limit — real only against a counting mock; runs in the N6 stress |

Each has been broken deliberately and seen to fail (the 401 rule excepted,
noted above). Reading note: a pre-emptive hold appears as `wait_ms` on
exactly the send that was held and as ETA on every job behind it; the
sends released after it read 0. A column of zeros with one large value
is the expected shape, not a broken column (misread once, 2026-08-24). The pacing rule is the first assertion here loose enough to
survive a rewrite and tight enough to have a breaker on each side. Its
"otherwise `wait_ms == 0`" arm is a scripted-server property: against a
counting mock the limiter holds *before* a violation, so that arm does not
apply there and the never-over-the-limit rule takes its place.

Time is a test input: one `Clock` with two faces (`now()` monotonic,
`wall()` system) drives limiter, token expiry, journal, and `mockggg`'s
policy counters; the test `ManualClock` can advance both or
`laptop_sleep()` the wall alone. Tests must speak the daemon's clock. Not
yet on the clock: the idle-shutdown and activity sites in `daemon.rs`.

## The boundary — three kinds of "cannot"

The register walk (R1–R8, L0-R1–R12, the N-claims) gives about half the
entries a journal invariant. The rest fall into three kinds, and the kinds
matter more than the list:

1. **Not on the wire** — reporting, persistence, config, job identity,
   the reason a job was refused. The right side of the line: a rewrite is
   allowed to change all of it.
2. **Absence** — R3. A line-based oracle cannot see a send that never
   completes. The journal will never close this; the answer is the
   client's timeouts (rail 3) and, if wanted, a `started` line.
3. **Vacuous offline** — N4/N6/N25/N26, *never over the limit* and
   *same-name policies share counters*. The journal carries them and the
   product is subject to them, but the harness's scripted server echoes
   whatever headers the script names, so asserting them would pin the
   script. They are real only live, or against a mock that keeps counters.
   `mockggg.rs` keeps them (`MockPolicy`: sliding windows, restrictions,
   self-generated 429s) and since `7762d967` runs on the test's `Clock`
   (`start_with_clock`), so the N6 stress now asserts never-over-the-limit
   non-vacuously (breaker: a limiter that never waits → "a send was
   answered 429"). The scripted-server tests remain kind 3 by nature.

Kind 3 is also the list of things a goal-seeking builder could get wrong
without the suite noticing. The live ladder is the un-gameable half of the
goal function and stays human-run.

## What a rewrite keeps

Sorting the tests (95 at `9d5f3b62`) by what they need from a daemon:

- **Component unit tests** (limiter 40, gate 7, rails 9, auth 2; the
  CLI's 7 for `pull`'s diff and snapshot store) survive with the component
  and vanish with it. The limiter's are its spec.
- **The journal invariants and the R8 scenario** need only a driving
  surface of about five calls: construct a daemon with a base URL, a
  clock, and a credential store; submit a job; await it terminal; read the
  journal. Any rewrite that keeps that surface keeps these for free.
  `wait_ms` is therefore a requirement on any design — every send records
  how long it was held from ready — not a coupling to this one.
- **Harness tests that reach into internals** (`shared.auth.refresh_flight`,
  `wait_for_refresh_waiters`, the scripted server's per-connection
  method/path asserts) pin this implementation's concurrency mechanics.
  They die with a rewrite and should.

## Next direction

Test infrastructure is at diminishing returns for mapping the GGG side.
The frontend side — what a GUI, CLI, or MCP client actually needs from the
protocol, whether the job model survives a real stash import, how
priorities behave under contention — is unmapped, and no journal work
maps it. The next thing to build is a real consumer: one end-to-end task a
user wants (a full-league stash pull with a diff on the next run), on the
real API under the ladder rules. Every rough edge it finds is a fact about
what the daemon's internals need to be, which is the input a
rearchitecture needs. Alongside, at no dev cost: the re-soak below.

Started 2026-08-24: `acq pull` (CLI-side snapshot + diff over the existing
verbs); its findings are in `CONTEXT.md` under "Frontend boundary findings".
Not yet run live.

Done 2026-08-24, closing the register's open items: rail 2 promoted to
product behavior (L0-R13, `ee5131e0`); `mockggg` on the `Clock` with
never-over-the-limit asserted in the N6 stress (`7762d967`).

## Before any fresh build against this suite

Prerequisites for a goal-function build, not a plan to start on now:

1. Move the harness out of `daemon.rs` into an integration-test crate that
   touches only the driving surface. This is what makes the goal function
   portable. Do it the day a fresh build starts, not before.
2. Re-soak the fixed binary (`LIVE-TESTING.md`, next action) for the first
   live `wait_ms` baseline; teach `soak-check.sh` to summarize it.
3. Pin the frontend boundary — after the consumer has validated the
   protocol, not before. Nothing tests the protocol yet.

## Standing constraints carried from the soak

- Verify the **binary**, not the checkout, before any live run:
  `acq --version` must equal `git rev-parse --short=12 HEAD` with no
  `-dirty`; the journal header and first log line say the same.
- A future soak derives its ceiling from cadence × intended duration;
  200 sends at one per 10 min is 33 h, not "several days".
- Express the HEAD condition per `(pid, route)` so it means something in
  both the pinned and restarting shapes.
- Ground-truth claims are authored on the master-side branch and
  cherry-picked to the spike, never the reverse.
- Commit the check, then break the code.
