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
so the token is read after admission.

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

- **B. Code-level debt** — small, under the gate, agent-owned; a commit
  each.
- **C. Questions for the owner** — each a one-line answer; nothing
  moves until it arrives. Batch them into one message.
- **D. Fired or near triggers** — parked items whose trigger has fired
  or is calendar-visible. These are the slice candidates.
- **E. Waits on GGG or an observation** — nothing to build; a watch
  list with the cheapest way to close each.
- **F. Parked, trigger unfired** — the registry's own backlog, listed
  once so this census is complete. Nothing to do.

Suggested order: B over two sessions (below); C as one batch to the
owner; D waits behind item search; E and F are ambient.

## B. Code-level debt (under the gate; a commit each)

| # | Where | What | Fix |
| --- | --- | --- | --- |
| B6 | the developer's machine, not the repo | the unsigned debug binary makes macOS Keychain prompt twice per login after every rebuild (REFRESH observation; the live-run skill's "known cost"). | the owner creates a self-signed local identity (a stable designated requirement; a keychain action on the machine); then a `codesign` step in `tools/preflight.sh`, a ten-minute change that rides with the B2/B4 session. Dev-experience only |

## C. Questions for the owner (one batch)

- **C1. Note 14 — retiring the mock default (2026-09-07).** Four
  questions for the owner, never answered; no ruling, no parked entry
  anywhere (`grep` finds neither "mock default" nor `ACQ_PROVIDER` in
  the registry). The daemon split took the trap that motivated it
  (C10's observe tier) but not the flip. Rule it, park it with a
  trigger, or delete the note.
- **C2. The pricing "reading-1 questions"** (`PRICING-SLICE.md`,
  observations), four one-liners never put to you: (a) should `price
  status` lead with the residue count when priced tabs are not public;
  (b) should "unlisted" be visible by default in `price list`; (c)
  should `set` say, at write time, that the facts do not hold the
  target; (d) should `set` resolve an alias instead of refusing it.
- **C3. `PRICING-SLICE.md` owner question 2:** C73 (the 0.18 import)
  parked as "a 0.18 user asks" — the park stands unless you say
  otherwise.
- **C4. Note 13 (at `decda84e`):** "the owner has a second charter topic for the next
  stopping point." This may be that stopping point.
- **C5. The per-realm merge trigger's second clause:** did C72's
  remedy (printing the whole edited policy) prove unusable at the
  pricing readings? If yes, the trigger fired (see D3).
- **C6. Shop publishing** (`CONTEXT.md`, Parked): the render is
  validated (reading 2); the other half of the trigger is "the owner
  wanting the posts automated". Yes, no, or not yet.
- **C8. Lint and toolchain ratchet** (was B5): the C47 lint is a
  per-crate `#![cfg_attr(not(test), deny(...))]` in three crates; no
  `[workspace.lints]`, no `rust-toolchain.toml`, no `cargo-deny`/`audit`.
  Does a spike want any of it? A workspace lints table would make the
  ratchet one line.

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
- **D5. Shop/forum publishing** — half-fired; see C6.

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
| cross-cutting | shop/forum publishing | render validated **and** the owner wants it (C6) |
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

1. ~~B1~~ — done 2026-09-12.
2. ~~B4, B2~~ — done 2026-09-12. B6's preflight step rides along once
   the owner has made the identity.
3. One message to the owner: C1–C6, C8.
4. Then item search, the owner's direction. D1 (the explicit-selection
   door) stays a fired trigger in `decisions/plans.md` until it is
   picked; the other D items keep their triggers.
