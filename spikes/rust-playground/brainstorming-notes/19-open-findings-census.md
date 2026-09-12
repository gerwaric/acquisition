# 19 — Open findings census (2026-09-11)

**Written 2026-09-11 at `decda84e`**, at the owner's request: step back
from the work streams and collect every open item the documents and
the code comments carry, then group them by what it takes to close
each one. Disposable like every note here (P1): an item is real only
where it already lives — a parked entry, a known gap, a closed record's
observation, a doc comment — and this note only indexes those homes.
It proposes no ruling. **Erased so far** (the commits hold what each
row was): A, 2026-09-11, after `44b4befd` and `64bda152` (C45, which A7
had listed as uncited and which turned out unbuilt); B3 (`1f4c284c`)
and D2 (`e9db23b2`: the MCP pricing step had been parked at the
2026-09-04 re-scope and the record cut dropped the entry unrouted —
restored, then postponed behind item search by the owner, `f9ea6c1d`);
C7, answered by the owner in conversation 2026-09-11: "I want to get to
the real heart of acquisition--item search" — the census items first,
then item search, so D is not the next slice; B5 moved to C8. B1,
2026-09-12: its premise was wrong — no test drove the idle watchdog, so
nothing slept real time; the sites went on the clock and the verdict is
pinned (C3). B4, 2026-09-12: its reason ("so the jobs stay readable") had
been answered by C45; the driver now saves the lifetime's evidence and
stops. B2, 2026-09-12: not a missing breaker — no journal had ever
carried a 401, and the product re-sent a rejected bearer; the owner
ruled fix, not record: C87 — and an external audit the same day found
the first fix blind to the gate (a waiter kept its copy of the token),
so the token is read after admission. B6, 2026-09-12: its premise was
half wrong — the owner's machine already held signing identities, and a
new Apple Development one was made for it; `tools/sign-acqd.sh` signs
the daemon's deps twin after the build (cargo re-copies the daemon from
it on every build, so signing the copy alone was undone by a no-op), run by
preflight and the live-run skill (its effect on the prompts is the
2026-09-12 ledger row: once per keychain item, then none; the
`REFRESH-SLICE.md` observation is closed); the
same investigation found the
Linux keyring gap (a mock store reporting `keyring: ok`) and fixed it
(`e65cbb07`). C, 2026-09-12: the owner answered all seven in one
message. C1 "agree to flip the default" — ruled in conversation; the
entry (`decisions/network.md`) and the build are the next slice, which
deletes note 14. C2 (a), (b), (d) stay as built; (c) built: `set` notes a
target the facts do not hold. C3 "park the 0.18 import indefinitely" —
C73's trigger is now a shipping decision. C4 "i forget what this was,
lets remove this one" — removed. C5 "no objection" — the clause named a
mechanism never built; reworded. C6 "no shop publishing. that will
happen much later or not at all" — the trigger is the owner asking. C8
"agree with your recommendation" — no workspace lints table (it cannot
exempt test code; C47 says so), no toolchain pin (no drift observed; the
pin and cargo-deny's other checks in the packaging lot), advisories in
the Linux workflow (`deny.toml`).

Scope of this pass: what is *written up* — the README's known gaps,
`CONTEXT.md` and the six `decisions/<area>.md` "Parked" sections, the
three closed slice records' "Observations still open", `TESTING-NOTES.md`,
`LIVE-TESTING.md`, `SURFACES.md`, the two ground-truth registers'
open questions, the later brainstorming notes, and every doc comment
in `crates/` that says parked, deferred, open, unobserved or "until".
Not in this pass (a second pass, if wanted): the quality gate's own
output, a dependency audit, dead code the compiler cannot see, and the
closed records' findings tables (every row there names a fix commit).

Verified negatives, so the next reader need not re-check: the code
carries no `TODO`/`FIXME`/`HACK` marker and no `#[ignore]` test; the
tree is clean; every directory under `runs/` has its ledger row;
`tools/docs-check.sh` is green; its one note names C16 as uncited,
which stays so until the GUI exists.

## How the buckets were cut

By what closes the item, because that is what decides who does it and
when:

- **D. Fired or near triggers** — parked items whose trigger has fired
  or is calendar-visible. These are the slice candidates.
- **E. Waits on GGG or an observation** — nothing to build; a watch
  list with the cheapest way to close each.
- **F. Parked, trigger unfired** — the registry's own backlog, listed
  once so this census is complete. Nothing to do.

Suggested order: D waits behind item search; E and F are ambient.

## D. Fired or near triggers (slice candidates)

- **D1. The explicit-selection door (C76).** Parked in
  `decisions/plans.md` with the trigger "pricing closed"; pricing
  closed 2026-09-09. This is the one entry in the whole register whose
  trigger has fired with nothing built. The ad-hoc `refresh` kind still
  exists (`main.rs` ~825 submits it; `tools/persist-check.sh` line 182
  drives it), and the entry says its listing, freshness and two-cycle
  semantics are ruled in its own slice, with `persist-check.sh` moved
  onto the selection plan. Its own design session (a framing note
  first, in 17's mold).
- **D3. Per-realm `policy set` merge** (README known gap;
  `decisions/plans.md`). Trigger: "a second realm in daily use". PoE2
  1.0 ships December 2026 and the owner ruled realm a coordinate above
  league everywhere; the trigger is calendar-visible. Plan for it
  before December.
- **D4. The GUI cluster.** Six parked items trigger on the GUI slice:
  queue-management UI, a watch that waits for the daemon, the
  standalone queue TUI with `Dashboard` reshaped, the error taxonomy
  beyond `kind` (`decisions/frontends.md`); a GUI-hosted daemon
  refused, correlation ids on one connection (`decisions/daemon.md`);
  and C16 itself. Note 17 (at `decda84e`) says at least one GUI or TUI is coming. One
  design session, not six.

## E. Waits on GGG or an observation (watch list)

- **E1. `/account/leagues` counted HEAD (N39).** GGG said it will be
  corrected; the no-probe declaration stays until the free HEAD is
  observed. Cheapest close: a three-send first-contact re-probe under
  the standing rule after any GGG release note that mentions it.
- **E2. `/profile` policyless (N38).** Until headers appear; strict
  observation already covers the arm. Nothing to do.
- **E3. Rate-limit classification.** Q4's positional hypothesis is what
  `ratelimit.rs` implements, conservatively; the single-window
  `token-request-limit` takes the 60 s bucket "until GGG confirms"
  through N14's channel. Q5, Q8, Q9 and Q11 are unprovokable by design.
- **E4. Mock gaps** (README): no timing-bucket quantization; the mock
  flags every window on a 429 where real GGG's behavior is unobserved.
- **E5. Claims awaiting a first contact** (parked as claims, authored
  master-side): the invalid-request 4xx threshold (`decisions/network.md`);
  realm segment semantics per endpoint, PoE2 on the character endpoints
  only, `inventoryId` undocumented (`decisions/store.md`).
- **E6. Trade ground truth, the owner's hand experiments.** Q6's forum
  half, Q7, Q8, and Q10's seven unexpanded groups; the render's policy
  table fails closed on each cell until observed (`shop.rs`). One hand
  experiment each, on `SURFACES.md`'s cadence. Q9 is moot for the spike.
- **E7. Observations that need a reader**, from the closed records:
  whether the C72 "moved or reindexed" line ever changes what the owner
  does; whether the quote is useful on a long-lived daemon; whether a
  freshness window is the right handle when cycles are long; whether
  all-zero change lines read as reassurance; the parent-failure report
  has never been exercised live; the first probe of a lifetime queues
  behind the token POST (noted, not investigated).

## F. Parked, trigger unfired (the registry's backlog, for completeness)

| Area | Item | Trigger |
| --- | --- | --- |
| cross-cutting | shop/forum publishing | the owner asks for it (2026-09-12: much later or not at all) |
| daemon | priority levels | a second level in use |
| daemon | the §5 packaging lot: bundle, signing, installer, CLI symlink; a second install; a keeper for non-spawning frontends; correlation ids; Windows; `-Z bindeps`; `world.rs` as a crate; GUI-hosted daemon; `spawned_by`; hash-only identity | ADR 0003 / a shipping decision; each sub-item names its own |
| network | wire-send budget | a consumer that needs it |
| store | user-scoped annotations home (`user.db`) | the first user-scoped kind written |
| store | fact-path migration to uuid naming | opportunistic, or never |
| store | search-at-scale | a measured latency or duplication case |
| plans | coverage advice in `refresh --plan` | C72's report proving the wrong place for it |
| plans | `aging` in the plan (C77) | a trusted cycle estimate |
| plans | universal plan grammar | a second plan-bearing consumer (C71 parked) |
| plans | dynamic `--deep` fan-out | two-cycle reconciliation genuinely hurting (none so far) |
| plans | type-level policy filters | a policy author who needs one |
| pricing | C71 price plans, C73 legacy import, C78 receipts (parked in place) | a batch a human cannot review row by row; a 0.18 user asking |
| pricing | third-party price feeds; the bulk-exchange table; row-granularity history; one change cursor; batch pricing by query; `~c/o` as a value; PoE2 currencies; currency totals; name→id for price targets; "what did I last post" | each names its own; currency totals is "when I ask for it" |
| frontends | the read economy as a ruling | the MCP pricing consumer's re-read record (the frontends park, postponed 2026-09-11 behind item search) |
| frontends | results over a subtree | a second consumer showing which shape |
| frontends | queue UI, watch-that-waits, standalone TUI, error taxonomy | the GUI slice (D4) |

## What a cleanup session would do, in order

Agreed with the owner 2026-09-11: a session boundary buys a clean
reading set, one commit story and one close; it costs the orientation
pass. Split where the second item needs another area's decisions file,
not by census row.

1. Item search, the owner's direction (the C answers and the
   mock-default flip landed 2026-09-12, `20234cf9` and `515bf034`).
   D1 (the explicit-selection door) stays a fired trigger in
   `decisions/plans.md` until it is picked; the other D items keep
   their triggers.
