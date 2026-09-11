# 19 — Open findings census (2026-09-11)

**Written 2026-09-11 at `decda84e`**, at the owner's request: step back
from the work streams and collect every open item the documents and
the code comments carry, then group them by what it takes to close
each one. Disposable like every note here (P1): an item is real only
where it already lives — a parked entry, a known gap, a closed record's
observation, a doc comment — and this note only indexes those homes.
It proposes no ruling.

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
`tools/docs-check.sh` is green with one note (A7 below).

## How the buckets were cut

By what closes the item, because that is what decides who does it and
when:

- **A. Stale records** — a document or doc comment says something the
  repo has since moved past. A doc edit; agent-owned; one commit.
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

Suggested order: A and B in one cleanup session (two or three
commits); C as one batch to the owner; D is the next-slice
conversation; E and F are ambient.

## A. Stale records (doc edits; one commit)

| # | Where | What is stale | Fix |
| --- | --- | --- | --- |
| A1 | `DAEMON-SPLIT-SLICE.md`, "Where the open observations went" | names three items as parked in `decisions/daemon.md` that have since landed: the unbounded handshake (C86, `decda84e`), C47's lint on the protocol crate (`d302f438`), one harness module per crate with the `test-hooks` retirement (`79e2d441`, `3ff46a9e`). The parked entries no longer exist; the closed record points at nothing. | reword to name the fix commits |
| A2 | `acquisition-store/src/lib.rs` C60 doc (line ~187); `acquisition-daemon/src/mockggg.rs` `mock_character_list` doc | both say the `Character.realm` field's `poe2` value is "open until a PoE2 body is seen"; N42 (2026-09-02) observed it. The mock's comment also cites `CONTEXT.md, "Characters in the refresh plan"`, a heading that now lives in `decisions/store.md`. | cite N42; fix the pointer |
| A3 | `acquisition-daemon/src/daemon.rs` ~7925 | `// ---- persistence (CONTEXT.md, "The job queue persists")` — that mechanism is the C27 paragraph in `daemon.rs`'s own module doc since the routing pass. | point at the module doc |
| A4 | `TESTING-NOTES.md` line 13 | says `CONTEXT.md` holds "the branch's status as the **reference implementation**"; CONTEXT no longer says it (note 13 left exactly this open on 2026-09-07). | date the clause or drop it |
| A6 | `README.md`, Known gaps | "A failed fetch child's result carries only the error string" has no parked entry and no trigger — it reads as an accepted shape, not a gap. | delete the bullet, or park it in `decisions/daemon.md` with a trigger |
| A7 | `tools/docs-check.sh` note | 11 uncited decisions: C2, C4, C15, C16, C17, C22, C28, C33, C45, C46, C48. Most are architectural choices no test pins; the module docs that implement them can cite them (store `lib.rs` for C28/C48, `daemon.rs` for C2/C4, `ratelimit.rs` for C17/C33, the jobs facade for C45, the MCP `main.rs` for C46). C16 (Tauri) has nothing to cite until the GUI; C15 and C22 are fine uncited. | cite from module docs; halves the note |

## B. Code-level debt (under the gate; a commit each)

| # | Where | What | Fix |
| --- | --- | --- | --- |
| B1 | `daemon.rs` — `last_activity`, the idle loop (`IDLE_POLL`), the errors ring | `TESTING-NOTES.md`: "Not yet on the clock: the idle-shutdown and activity sites in `daemon.rs`" — still true (`Instant::now()` at ~972, 1532, 3321, 3440, 3875). Open since 2026-08-24. | put them on the daemon `Clock` so idle tests do not sleep real time; check first what the idle tests do today |
| B2 | `TESTING-NOTES.md` journal table, the 401 rule | "after a 401 the next send is `POST oauth-token`" is pinned but "armed, no offline breaker yet" — the one row never broken deliberately. | a breaker, or a line saying why it cannot have one |
| B3 | `acquisition-plan/src/lib.rs` 428–445; `price_notes_fixture.rs` 10–13 | five `#[allow(dead_code)]`: the `version` field of `SyncPolicyWireV1/2/3` (parsed, never read) and two fixture fields. | read `version` (assert it matches the variant — a strictness gain) or rename to `_version`; the fixture fields likewise |
| B4 | `tools/tracer-rung.sh` | REFRESH observation: after a failed apply the driver stops with the daemon up so the jobs stay readable; a scripted caller wants the stop automated once the evidence is copied. | a flag, or stop after the evidence copy |
| B5 | `Cargo.toml` | the C47 lint is a per-crate `#![cfg_attr(not(test), deny(...))]` in three crates; no `[workspace.lints]`, no `rust-toolchain.toml`, no `cargo-deny`/`audit`. | owner's call whether a spike wants any of it; a workspace lints table would make the ratchet one line |
| B6 | the developer's machine, not the repo | the unsigned debug binary makes macOS Keychain prompt twice per login after every rebuild (REFRESH observation; the live-run skill's "known cost"). | a self-signed local identity gives a stable designated requirement; a `codesign` step in `tools/preflight.sh`. Dev-experience only |

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
- **C7. Which of D is next.**

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
- **D2. Pricing on the MCP.** `MCP-REFERENCE.md` lists 17 tools; none
  touches pricing (`price`, `reference`, `shop`). Neither built nor
  parked — `decisions/pricing.md` and `decisions/frontends.md` do not
  mention it. The frontends' "read economy" ruling waits on "the first
  MCP pricing consumer's re-read record (the pricing slice's MCP
  step)", which never happened, so that trigger cannot fire. Record
  it: park with a trigger, or build it as the MCP's pricing step — it
  is also the second consumer P3 wants before C53's read economy
  becomes a ruling.
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
| frontends | the read economy as a ruling | the MCP pricing consumer (blocked, D2) |
| frontends | results over a subtree | a second consumer showing which shape |
| frontends | queue UI, watch-that-waits, standalone TUI, error taxonomy | the GUI slice (D4) |

## What a cleanup session would do, in order

1. One commit: A1–A4, A6, A7, B3 — "record hygiene after the daemon
   split".
2. One commit each, under the gate: B1, B2, B4.
3. One message to the owner: C1–C7.
4. The next slice's framing note: D1 (the explicit-selection door),
   with D2 recorded — parked or built — in the same session, since
   both are debts the pricing close left.
