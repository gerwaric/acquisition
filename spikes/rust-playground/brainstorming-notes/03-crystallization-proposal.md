# Crystallization proposal — decision lines, tracer, parking lot

**Written 2026-08-31.** This is a **proposal packet, deliberately
disposable**: every line here is phrased for the owner to accept, amend,
or refuse. Accepted lines move into `CONTEXT.md` (decisions, open topics,
working style) in its format; this file then remains as history in the
notes directory and is never a second authority. Sources:
`00-framing.md`, `01-first-round-brainstorming.md`,
`02-practicality-evaluation.md`, and the code checks recorded in 02.

Each item ends with **Ruling:** — fill with *accept*, *amend: …*, or
*refuse: …*. Bucket labels are the framing's triage: (a) already built,
(b) fits an open topic, (c) new scope, (d) conflicts with a recorded
decision — where (d) here means *recorded rejections*, so nothing is ever
re-argued or silently adopted.

---

## 1. Candidate decision lines

### Bucket (a) — already built; wants recording or exposure

**A1. The effects ledger is frontend-readable through the store crate.**
`daemon.db` access already lives in `acquisition-store` (`jobs.rs`, WAL,
busy timeout); orientation reads (`acq state`-style) may merge job-ledger
and fact-ledger reads with no daemon running — a dead daemon implies
nothing in flight, so orientation degrades gracefully rather than
failing. Rationale: structurally true today; recording it prevents
protocol-only thinking about job visibility.
**Ruling:**

**A2. Rails graduate to product features by exposure, not new
machinery.** Budget *visibility* is the quote (D6) plus the journal;
`ACQ_MAX_SENDS` and the tripwire remain the enforcement substrate.
Per-operation enforcement is separately priced (D8). Rationale: the
trust mechanism and the product feature are the same object (framing's
synthesis seed, confirmed in 02).
**Ruling:**

### Bucket (b) — the heart: landings on open topics

**D1. The system is four layers — facts, annotations, derivations,
effects — and each layer has exactly one writer class.** The daemon
writes facts through `record` and never reads them; frontends write
annotations through one shared store-crate write API; nobody writes
derivations (computed, recomputable); the daemon alone writes the
effects ledger. Rationale: "a sync can never clobber intent" becomes a
structural property, the way the choke point made rate-limit discipline
structural.
**Ruling:**

**D2. Annotations are the only irreplaceable local state, and the store
treats them that way.** They live in a **separate per-account file**
beside the facts file, keyed on stable GGG ids, written only by
frontends through the store crate. No fact-side event may cascade into
deleting intent: an annotation whose item is removed is kept and
surfaceable as orphaned, never dropped. Backup/export is a file copy by
construction. Rationale: facts are refetchable at the cost of API
requests; intent has no server to refetch from — the legacy-buyout saga
is the C++ app paying full price for getting this wrong. Separate file
also gives zero daemon/frontend write contention and immunity to
fact-schema migrations. This **amends** the recorded "written by the
daemon and read directly by every frontend" line to: *frontends read
facts and read/write intent, all through the store crate*.
**Ruling:**

**D3. The daemon never initiates GGG traffic; every job has a
submitting client.** Rationale: true today, and it is what makes the
daemon's permanent blindness to annotations safe to pin — scheduled or
policy-driven syncs are small frontends (a cron/login-session process
running plan→apply), never daemon capabilities. On macOS a cron-spawned
daemon has no keychain anyway; the architecture and the platform agree.
**Ruling:**

**D4. The sync policy is the first annotation.** A per-account,
inspectable declaration of what should be kept fresh, compiled by a
frontend-side planner into minimal requests. Rationale: the triple
convergence (C++ tracked-set/clean-refresh semantics, the
delta/selection open topic, both essays' declarative sync policy) is
three descriptions of this one object; per-account scope makes it the
easy first row of the new layer.
**Ruling:**

**D5. A Plan is a frontend-side derivation, composed from both
surfaces, living in a shared crate.** Inputs: facts and annotations
(door 2) plus a cost quote (door 1). The Plan type carries `account`
and its **epistemics from v1**: request counts are exact; ETA/headroom
may be *unknown-until-probe* (the limiter starts each lifetime empty),
and a Plan must say so rather than guess. Rationale: neither surface
can compute a plan alone, and no third door is needed; a Plan that
pretends to know an ETA it doesn't have violates the design's own first
principle.
**Ruling:**

**D6. The daemon answers a pure cost question: `quote` — requests, ETA,
headroom after — as its own protocol request, not a flag on `Submit`.**
Mechanically it extends `eta_for` (which already simulates the pacing
rule forward); single-route quotes suffice for the tracer, cross-policy
quotes wait for a consumer that needs them. Rationale for the separate
verb: `Submit`'s contract is heavily loaded (id allocation,
persistence, rollback-on-write-failure); a dry run that allocates no id
and persists nothing is a different verb wearing a flag. *(Owner taste
explicitly solicited — a `--plan` flag on submit is the equally valid
alternative.)*
**Ruling:**

**D7. "An error's remedy is a plan" is a door-2 idiom, not a
system-wide law.** Stale-read refusals and planner errors carry Plans;
daemon protocol errors keep their current shapes. Rationale: the daemon
cannot compute plans — it is blind to facts and annotations by design —
and scoping the unification removes the one pressure that would
otherwise argue for un-blinding it.
**Ruling:**

**D8. Per-operation budget enforcement (`--max-requests` on a plan) is
new daemon machinery, priced as such.** A counter scoped to a parent's
descendant tree; tripping it mid-fan-out terminalizes like an
interrupted fan-out — never success over a partial set (the semantics
template already exists in the restart rules). Budget *visibility*
(D6/A2) is nearly free; enforcement is a real feature and a late tracer
step. Rationale: honesty about cost — this is a promotion in concept
but not in code.
**Ruling:**

**D9. "Built once, inherited by every frontend" rests on a premise now
made explicit: all frontends are Rust linking the shared crates.** CLI
(clap), MCP (`rmcp`), GUI (Tauri — its backend is Rust; Tauri commands
wrap the shared crates), TUI (`acq dash`). A proposed non-Rust frontend
is a design event, recorded before built. Rationale: the load-bearing
fact under every "ergonomics are built once" claim; unstated premises
erode silently.
**Ruling:**

**D10. The planner lives in its own crate (`acquisition-plan`),
depending on core's client/protocol types and the store, linked by
frontends only.** Rationale: `acquisition-core` already links the store
(the daemon writes through it), so a planner in core would compile —
but it would erode "the daemon never reads the store" from structural
to disciplinary. A separate crate keeps the blindness enforced by the
dependency graph.
**Ruling:**

**D11. Panics are for broken internal invariants only; malformed
external input — a GGG body, a store row, a protocol message — is
always a structured error.** The store crate ratchets this mechanically
(`clippy::unwrap_used` / `expect_used` on that crate; its production
code is at zero today). Not workspace-wide: the daemon's
`.lock().unwrap()` mutex-poisoning idiom and checked-invariant
`.expect("…")` calls are the correct register and the lint would fight
them. Rationale: the persisted queue makes crashes recoverable, which
turns a *reproducible* panic on bad input into a crash loop — the one
failure persistence cannot absorb.
**Ruling:**

### Bucket (c) — new scope: goes to the parking lot (§4), not built now

Pricing-as-document, shop, currency ratios, saved searches, `user.db`,
PoB export, catalogs, the annotation change-log, the five-verb grammar
surface. Each has a named landing and trigger in §4.
**Ruling (that these are parked, not scoped in):**

### Bucket (d) — recorded rejections (with rationale, so they stay dead)

**R1. No fused reads (`--sync-if-stale`).** A read that fetches breaks
the property that store reads are daemon-free and network-free by
construction — the architectural boundary and the epistemic one are the
same boundary. The idiom instead: the read *refuses with the exact plan
it would take* (D5/D7), and the caller decides to spend.
**Ruling:**

**R2. No daemon-readable annotations.** Blindness is load-bearing (D1,
D3, D10). The scenario that would break it — daemon-resident scheduled
syncs — is foreclosed by D3 and by the platform (no keychain off a
login session). Reopening requires a concrete consumer that a
frontend-side scheduler demonstrably cannot serve.
**Ruling:**

**R3. No cached search service; no third surface.** Inherited from the
framing's stress test with its tripwires for reopening (duplicated
expensive indexes across long-lived frontends, or a measured latency
floor in-process reads cannot meet). Restated here so the brainstorm's
adoption is explicit.
**Ruling:**

**R4. The SQLite schema is internal; raw SQL is not a surface.**
Defended by making door 2 expressive enough that going around it is
never worth it — which the annotations write API and planner extend to
the write side.
**Ruling:**

---

## 2. The tracer: refresh-with-plan

The one slice built next. It is the smallest slice that touches all
four layers — the policy is an annotation, the plan a derivation, the
apply an effect, the next read a fact — so every boundary the roadmap
needs gets validated by one consumer.

### Pinned going in (assuming §1 rulings)

- Plan type: `account` + epistemics fields from v1 (D5).
- Sync policy: per-account, in the new annotations file (D2, D4).
- Ambiguous account: refuse-and-list, exactly like every job command
  (stateless-selection precedent).
- Quote: single-route is enough; no cross-policy simulation (D6).

### Deliberately open going in (decided by what the code teaches)

- **Plan-staleness semantics** — the tracer's known hard question. A
  plan is computed from the *stored* listing; the world can move before
  apply, and the refresh parent re-lists. Candidates: **(binding)**
  apply submits exactly the planned children, no relist — cheap,
  fetches into a possibly-moved world; **(advisory)** apply runs the
  parent with the plan as filter + budget, recomputing at fan-out —
  honest, but what runs is not exactly what was reviewed. This is the
  C++ clean-refresh gate one level down. Decide at build step 6, record
  in `CONTEXT.md` before the code is written.
- Budget-trip terminal semantics detail (D8's template applied).
- Whether the policy wants `--max-age` per tab, per set, or both — the
  policy schema starts minimal and the owner's real use decides.

### Build order (each step gate-green; observable behavior unchanged until 5)

1. **Annotations file + store-crate write API + sync-policy table.**
   Tests are the spec, as ingest's were. Schema shaped so a later
   change-log is an addition, not a migration (rowids, `updated_at`).
   Lint ratchet on the store crate lands here (D11).
2. **The store-side planner input:** a pure function answering "which
   fetches does this policy imply," over `fetched_at`, the last
   listing, and `metadata.items` counts where present. This *is* the
   delta/selection open topic, closed. Test-tabled.
3. **`quote` on the protocol** + the limiter extension (simulate N
   sends on a route, honest about unknown state). Mock-validated;
   test rows beside `eta_for`'s.
4. **`acquisition-plan` crate:** the Plan type, composing 2 + 3;
   remedy/refusal share the type (D7).
5. **`acq refresh --plan`** — prints the plan, spends nothing, `--json`
   total as always. First user-visible change.
6. **Apply:** plan → refresh parent. The staleness ruling happens
   *here*, recorded before code.
7. **Budget enforcement** (`--max-requests`, D8).
8. **MCP exposure** of plan/quote in mock mode (the real-GGG
   `submit_job` deferral is untouched; a *quote* sends nothing and may
   be allowed in either mode — owner call at this step).
9. **Live rung** under `LIVE-TESTING.md`'s standing rule: the owner
   uses `--plan`/apply on a real league. Friction notes are collected
   as data (§3).

### Done criteria

Gate green throughout; new boundaries pinned by tests; the owner has
used it live; findings and rulings recorded in `CONTEXT.md`; the
protocol additions pinned or revised — plausibly closing the "frontend
boundary not yet pinned" frontier, since this tracer is the real
consumer the pin has been waiting for; the method-test verdict (§3)
written.

### Anti-scope (what this tracer must NOT build, with reopening triggers)

- **No annotation framework or scope taxonomy in the schema** —
  per-account only. Trigger: the first user-scoped annotation kind
  (currency ratios, saved searches) arrives.
- **No `user.db`.** Same trigger.
- **No annotation change-log.** Trigger: `diff --since` needs "what got
  repriced," or write conflicts need visibility. Schema stays shaped
  for it (step 1).
- **No cross-policy quote.** Trigger: a consumer whose plan genuinely
  spans policies.
- **No five-verb CLI surface** (look/assert/plan/commit/attend stays a
  direction line). Trigger: the second plan-bearing feature (pricing)
  validates the grammar's generality.
- **No pricing table, no shop, no GUI work.** Parking lot.

---

## 3. The method test

This slice tests the *process*, not only the design: pin-after-the-
consumer has been proven only in daemon land, where truth is external
(headers, Cloudflare shapes) and a test table settles arguments. The
tracer is the first product-land slice, where the validating consumer
is the owner's real use and "did it feel simple" is evidence.

Evidence to collect, on purpose:

- The owner's friction notes from step 9, treated as data the way the
  send journal is data.
- Whether the deliberately-open questions (staleness, policy shape)
  were genuinely cheaper to decide with code in hand — or whether the
  slice thrashed for lack of an up-front answer.
- Whether the anti-scope held, and if it was breached, what pressure
  breached it.
- Whether agent-built internals stayed simple behind pinned boundaries
  without further intervention — the transmission question, measured.

Verdict recorded at the end (a short section in the next session's
notes): *does the method transfer to product scope, and what amendment
does it need if not?* That verdict is the first input to the pricing
session.

---

## 4. Parking lot (real, not next; landings named so deferral never
needs re-arguing)

- **Pricing-as-document** → lands on the annotations layer + plan/apply
  grammar, after the tracer validates both. The second consumer that
  triggers grammar generalization.
- **Legacy buyout import** → dissolves into a patch generator feeding
  the ordinary annotation plan/apply path (no wizard).
- **Shop / forum publishing** → own boundary session first: POESESSID
  traffic is outward-facing and *outside* the choke-point invariant,
  which is scoped to the API. Needs its own boundary thinking, not
  inheritance.
- **Currency ratios, saved searches** → first user-scoped annotation
  kinds; trigger for `user.db` + the scope taxonomy.
- **Annotation change-log** → trigger recorded in anti-scope.
- **Five-verb grammar as surface** → direction line now; generalized at
  pricing (the second consumer).
- **PoB export, catalogs/enumerable vocabularies** → real; after
  pricing.
- **Search-at-scale (FTS at ingest, search crate factoring)** → per the
  framing's stress test; trigger is a real consumer with a latency or
  scale case.

---

## 5. Process delta — candidate working-style lines

Four lines, not a process document; the process that worked is already
recorded where it worked.

**P1.** Deep design sessions are evidence-driven, never calendar-driven;
each new session waits for a new evidence pile. Crystallize before
building: rulings land in `CONTEXT.md`; session notes are disposable
history, never a second authority.
**Ruling:**

**P2.** In product scope, the validating consumer is the owner's real
use; "did it feel simple" is evidence, and friction notes are data.
**Ruling:**

**P3.** Generalize at the second consumer, never at the first.
**Ruling:**

**P4.** Tactical taste is settled by a lint where mechanical and a
recorded property where stakes are real — never adjudicated in prose;
everything else is agent-owned internals.
**Ruling:**
