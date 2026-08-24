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
anyone noticed the codebase had settled the other way.

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

## Open topics

1. **The clock fork.** Extend `ratelimit.rs`'s `Clock` trait with a
   wall-clock method so one fake drives limiter and daemon together, or
   add a separate daemon-level clock that is less invasive but leaves
   two fakes to keep consistent? R8 is a bug about two clocks
   disagreeing, which argues for one fake able to make them disagree
   deliberately — but that touches the limiter's trait and every impl.
2. **Is the journal sufficient as a contract surface?** It records what
   was sent and when, but not *why* the limiter waited. Does pinning
   pacing need that? Is the CLI's `--json` outcome a necessary second
   surface for job results and data correctness? Note that the journal
   is stronger here than it first appears: under a manual clock its
   timestamps are exact, so "this refresh completes within N virtual
   seconds" is a deterministic, free-to-run *performance* invariant off
   a surface that already exists — which covers the degradation half of
   the goal without new instrumentation.
3. **Should provenance be a rail** — the daemon refusing to run a binary
   that does not match its checkout — or is that a process fix rather
   than a code one?
4. **What does the live ladder shrink to?** Arguably only what the mock
   cannot answer, which is already written down as README's "Known
   gaps". If so, most future confidence is bought offline and the ladder
   becomes short and rare.
5. **The 16 existing assertions.** Leave them, convert them, or let the
   simplification eat them. They pass today and they are cheap; they are
   also coupled to `ScriptedTransport`, so a transport change
   invalidates all of them at once.
6. **How much of this is a hedge that pays either way on ADR-0003?**
   Behavior pinned at the network boundary survives both rewrite and
   evolve. Does that change how much to invest now, before the decision?

## Standing constraints carried from the soak

- Verify the **binary**, not the checkout, before any live run:
  `strings target/debug/acq | grep 'token rejected'` returns two lines
  with the R8 fix present, one without.
- A future soak must derive its ceiling from cadence × intended
  duration; 200 sends at one per 10 min is 33 h, not "several days".
- Express the HEAD condition per `(pid, route)` — the journal records
  `pid` — so it means something in both the pinned and restarting
  shapes.
- Ground-truth claims are authored on the master-side branch and
  cherry-picked to the spike, never the reverse.
