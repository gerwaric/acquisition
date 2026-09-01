# Final ruling packet

**Written 2026-08-31.** The merge of `03-crystallization-proposal.md`
(the packet), `04-crystallization-audit.md` (the external audit — all
code citations verified), and `05-audit-response.md` (the endorsement
with three trims). **This is the document the owner rules on**; it
supersedes 03's ruling slots. Accepted lines are harvested into
`CONTEXT.md` in its style, then this file is history, never a second
authority.

Provenance tags: [03] proposal as written · [04] audit amendment ·
[05] response trim.

**RULED 2026-08-31: everything accepted; D2a accepted as amended by the
owner (uuid required at login — the stable key is the profile uuid, and
a login that cannot fetch it fails whole). Two wording corrections
applied on acceptance: A1's facade phrasing, and D1 reclassifying
`rebuild` as maintenance of materialized derivations rather than fact
ingestion. Next: harvest into `CONTEXT.md`, then build by §2.**

---

## 1. Decision lines

### Already built; wants recording or exposure

**A1. The effects ledger is frontend-readable through a read-only
facade in the store crate** (`JobLedgerReader` or equivalent — not the
open `JobDb`, whose write methods are daemon-only in fact and should be
kept out of the frontend surface). An offline orientation report distinguishes: daemon
offline · zero sends in flight · persisted work (N waiting, M recorded
running) · ledger observed-at · runtime state unavailable. "Daemon
offline" never collapses into "no outstanding work." [03 amended per 04]
**Ruling:** accept (owner, 2026-08-31)

**A2. Rails machinery is reused for product budget features; rail
semantics are not promoted wholesale.** The tripwire and lifetime
ceiling stay what `LIVE-TESTING.md` says they are. Product budget
*visibility* is the quote (D6) + the journal; product *enforcement* is
D8's admission-time check. [03 amended per 04]
**Ruling:** accept (owner, 2026-08-31)

### The heart: landings on open topics

**D1. Four layers — facts, intent (annotations), derivations, effects —
each with one authoritative mutation path, not one physical writer.**
Facts mutate only through the store crate's ingest surface (daemon
`record`; `store import` is the existing frontend-triggered example
that made "one writer" false). Intent mutates only through the store
crate's annotation write API (frontends). The effects ledger mutates
only through the daemon. Derivations have no independent authority:
computed or materialized, always reproducible from declared inputs —
`rebuild` is their maintenance operation (re-materializing derived
columns from stored JSON), not fact ingestion. Rationale: "a sync can never clobber intent" as structure;
authority phrasing is what the code already practices. [03 amended per 04]
**Ruling:** accept (owner, 2026-08-31)

**D2. Annotations are the only irreplaceable local state, and the store
treats them that way.** A separate per-account file, named by the
stable account key — the profile uuid (D2a) — keyed on stable GGG ids, written only
through the store crate with integer-revision compare-and-swap writes
(optimistic conflict detection now; the full change log stays parked).
No fact-side event may cascade into deleting intent: an annotation
whose item is removed is kept and surfaceable as orphaned. Export and
backup are a store-managed consistent snapshot (`VACUUM INTO` / SQLite
backup API) — a raw file copy under WAL is **not** a backup. Amends the
recorded store line to: *frontends read facts and read/write intent,
all through the store crate.* [03 amended per 04]
**Ruling:** accept (owner, 2026-08-31)

**D2a. The stable account key is the GGG profile uuid, and login
requires it.** Login completes only when, after token exchange, a
profile fetch — an ordinary job through the choke point, submitted in
causal service of the client's `acq auth` (D3) — returns the uuid; the
session is registered, the keyring written, and `accounts.json` updated
only then. A login whose profile fetch fails **fails whole**: no
provisional identity, no locally minted keys, no alias/rename-repair
machinery — if `/profile` is broken, something is broken and login says
so. `accounts.json` maps username/discriminator/provider → uuid; a
rename is a pure mapping update with intent untouched; `--account`
matches name-or-uuid exactly, as already designed. Annotation paths are
uuid-named; fact paths stay username-named (refetchable;
rename-orphaning remains tolerable; migration deferred/opportunistic).
Existing entries without a uuid are not migrated: one re-auth (house
precedent). The mock serves deterministic per-username uuids so
two-account tests keep distinguishing accounts. **Amends the recorded
multi-account identity line** ("no fetch at login, no new failure
mode"): the failure mode is accepted deliberately for the machinery it
deletes, and a retry of a failed login repeats the token exchange —
`Ip`-scoped (N33) — so the limiter already bounds a retry loop.
`/profile` stays policyless-declared (N38), no-probe: one GET per
login, negligible. [04+05, amended by owner ruling]
**Ruling:** accept as amended above — uuid required (owner, 2026-08-31)

**D3. The daemon creates work only in causal service of
client-submitted work — probes, children, retries — and never
originates work spontaneously: no schedules, no policy execution, no
annotation reads.** Rationale: architectural — this is what makes R2's
blindness safe to pin. (Probes are daemon-submitted roots today, which
falsified both earlier wordings; macOS keychain behavior is
corroboration, not the reason.) [03 amended per 04+05]
**Ruling:** accept (owner, 2026-08-31)

**D4. The sync policy is the first annotation: a per-account,
inspectable declaration of desired coverage and freshness — not a
scheduler.** Compiled by the frontend-side planner into minimal
requests. `metadata.items` counts are heuristic evidence: they can
prove a tab changed; they can never prove it didn't. [03 accepted with
04's constraints]
**Ruling:** accept (owner, 2026-08-31)

**D5. A Plan is a serializable, immutable authorization envelope: the
bounded work the user authorized, derived from a named snapshot of
facts and intent, computable with the daemon down.** It carries:
provider + stable account key; operation kind + plan schema version;
fact basis (response/listing ids or timestamps); annotation revision;
the explicit action set (or a declared upper bound); generated-at;
freshness/completeness assumptions; and optionally a quote with its own
observation time. Work has two dimensions: `logical_requests` (exact or
bounded) and `wire_sends` — in v1 a **coarse** projection: a range plus
named prerequisites (probe may be needed; token refresh may occur; 429
retries possible), never a precise accounting (that is the deferred
wire-budget feature). Start with an operation-specific `RefreshPlan`;
a universal Plan grammar waits for the second plan-bearing consumer.
[03 amended materially per 04, trimmed per 05]
**Ruling:** accept (owner, 2026-08-31)

**D5a. Plans are binding: applying a Plan executes exactly the listed
actions or a strict subset — never an action that was not reviewed; new
facts produce a new Plan.** v1 excludes dynamic `--deep` fan-out; a
vanished tab fails or is reported skipped; newly discovered tabs wait
for the next plan (honest eventual reconciliation). Requires an
exact-action-set parent, or the existing refresh parent constrained to
never expand its set (it re-lists today — reused unchanged, the plan
would silently become advice). **Recorded as revisable by tracer
evidence**: if the owner's live use fights subset-only reconciliation,
that friction is data and the next session re-rules on it. [04,
revisability per 05; supersedes 03's deliberately-open staleness
question — pre-deciding it is justified because it shapes the schema
and parent design, P3's exception]
**Ruling:** accept (owner, 2026-08-31)

**D6. `quote` is its own protocol request: a read-only, non-reserving
projection over current daemon knowledge.** It reports observation
time, basis, per-policy/per-scope estimates, and unknown prerequisites;
applying a plan may receive a different schedule (`eta_for`'s own doc:
"an estimate, not a promise"). Headroom is per policy/window and scope,
conditional on no intervening sends — never one scalar. Never a flag on
`Submit`. [03's verb accepted, contract amended per 04]
**Ruling:** accept (owner, 2026-08-31)

**D7. Plans-as-remedies attach to unmet freshness/planner
preconditions, with stable structured error codes — ordinary stale
reads do not fail.** Reads return facts with freshness/completeness
metadata; only a caller-asserted freshness condition yields the
unmet-precondition error carrying a `RefreshPlan`. Three honest
operations: observe (with age) · assert (Plan if unmet) · apply and
observe again. [03 narrowed per 04]
**Ruling:** accept (owner, 2026-08-31)

**D8. v1 budget is logical-work, enforced at admission.** A submitted
plan carries an explicit bounded action set; the daemon refuses it
before any child submission if the logical bound exceeds
`max_requests`. Mid-fan-out terminalization is never the normal path
(binding plans make the bound knowable up front). An actual-wire-send
budget — causal operation id through probes, OAuth, retries;
shared-token-refresh semantics — is a separate feature, priced and
parked until a consumer needs it. [03 replaced per 04]
**Ruling:** accept (owner, 2026-08-31)

**D9. Shared semantics live in Rust; every frontend has a Rust
adapter.** CLI (clap), MCP (`rmcp`), GUI (Tauri backend — the webview
is presentation, never a second implementation), TUI (`acq dash`). A
proposed non-Rust frontend is a design event, recorded before built.
[03 accepted, rephrased per 04]
**Ruling:** accept (owner, 2026-08-31)

**D10. `acquisition-plan` owns policy compilation and Plan
construction; the store exposes neutral snapshots** — policy rows, tab
identities, freshness, listing basis, metadata — facts and intent,
never half a planner. Crate depends on core's client/protocol types +
the store; linked by frontends only, so "the daemon never reads the
store" stays enforced by the dependency graph. [03 accepted with 04's
responsibility shift]
**Ruling:** accept (owner, 2026-08-31)

**D11. Panics are for broken internal invariants only; malformed
external input — a GGG body, a store row, a protocol message — is
always a structured error with stable error kinds and context** (not
only an `anyhow` string), because persisted poison inputs plus the
recovering queue would otherwise make a crash loop. Store crate
ratchets mechanically (`clippy::unwrap_used`/`expect_used`; production
code is at zero today). Not workspace-wide — the daemon's
`.lock().unwrap()` poisoning idiom and checked-invariant `.expect`s are
the correct register. [03 accepted, sharpened per 04]
**Ruling:** accept (owner, 2026-08-31)

### Recorded rejections (so nothing is re-argued or silently adopted)

**R1. Store reads never initiate network traffic.** That is the
invariant — nothing more. Explicit frontend orchestration (a GUI
Refresh button; a CLI `apply → await → read`) is workflow, not a fused
read, and is fine. Stale facts stay readable with their metadata (D7).
[03 narrowed per 04]
**Ruling:** accept (owner, 2026-08-31)

**R2. The daemon is permanently blind to annotations.** Rationale is
architectural (D1/D3/D10); platform keychain behavior is corroboration.
Reopening requires a concrete consumer a frontend-side scheduler
demonstrably cannot serve. [03 accepted, rationale re-grounded per 04]
**Ruling:** accept (owner, 2026-08-31)

**R3. No cached search service; no third surface.** The framing's
reopening tripwires stand. [03/04 agree]
**Ruling:** accept (owner, 2026-08-31)

**R4. The SQLite schema is internal; raw SQL is not a surface.** Add
schema versions and compatibility errors; an accessible file is not a
supported SQL contract. Defended by making door 2 expressive. [03
accepted plus 04's versioning]
**Ruling:** accept (owner, 2026-08-31)

### Working-style lines

**P1. Deep design sessions are evidence-driven, never calendar-driven;
crystallize before building.** Rulings land in `CONTEXT.md`; session
notes are disposable history. [03/04 agree]
**Ruling:** accept (owner, 2026-08-31)

**P2. In product scope the validating consumer is real use — and each
frontend contract needs its own.** The owner's live use validates this
CLI tracer; it does not close the GUI/MCP/TUI contracts (pagination,
event replay after disconnect, cancellation presentation, partial
results, subscriptions may each surface new requirements). [03 amended
per 04]
**Ruling:** accept (owner, 2026-08-31)

**P3. Generalize after two materially different consumers reveal the
shared property — except where an early choice controls irreversible
identity, durability, safety, or compatibility.** D2a is the
demonstration: stable identity and intent durability get
first-consumer treatment because repairing them later risks the
irreplaceable state. [03 amended per 04]
**Ruling:** accept (owner, 2026-08-31)

**P4. Tactical taste is settled by a lint where mechanical and a
recorded property where stakes are real — with design discussion
preceding a property's promotion to lint or test; everything else is
agent-owned internals.** [03 amended per 04]
**Ruling:** accept (owner, 2026-08-31)

---

## 2. The tracer: refresh-with-plan (revised order)

The audit's sequence with 05's trims. Each step gate-green; observable
behavior unchanged until step 6.

1. **Record the v1 semantics first** — the accepted rulings above land
   in `CONTEXT.md` before schema work: binding Plan (D5a), no deep
   fan-out, logical vs. wire work (D5), observe/assert/apply (D7).
2. **Stable account identity — uuid required at login (D2a) — + the
   annotation file: revisioned writes, orphan retention, store-managed
   export/backup.** Includes the login-flow change (profile job before
   session registration), the mock's deterministic uuids, and the
   store-crate lint ratchet (D11).
3. **Neutral store snapshots** (D10): sync policy rows, tab identities,
   freshness, listing basis, metadata. Compilation stays out of the
   store.
4. **`RefreshPlan` built offline** from facts + intent in
   `acquisition-plan` — operation-specific type, no universal grammar.
5. **`quote` on the protocol** (D6) + optional Plan enrichment. A
   single-route limiter primitive internally; the quote **names what it
   does not cover** (listing policy, probe, OAuth) rather than claiming
   completeness.
6. **`acq refresh --plan`** — human and JSON forms, spends nothing.
7. **Apply**: the exact action set through a parent that never expands
   it (new parent kind, or the existing one constrained — decided here,
   recorded before code), with admission-time logical budget (D8).
8. **MCP exposure in mock mode**; verify MCP needs no parallel
   semantics. (Whether `quote` — which sends nothing — is allowed in
   real-GGG mode is an owner call at this step.)
9. **Owner live rung** under `LIVE-TESTING.md`'s standing rule;
   friction notes collected as data.

**Done criterion** (corrected per 04): **pin the refresh
Plan/quote/apply slice and the annotation API it exercised.** A CLI
tracer cannot close the whole frontend frontier; pinning one validated
slice is the method — declaring the frontier closed would not be.

### Anti-scope (unchanged from 03 except as noted, with triggers)

- No annotation framework or scope taxonomy in the schema — per-account
  only. Trigger: first user-scoped annotation kind.
- No `user.db`. Same trigger.
- No annotation **event log** — but row **revisions are in scope now**
  (D2). Trigger for the log: `diff --since` needs "what got repriced,"
  or conflicts need history.
- No cross-policy quote completeness — the quote names its gaps (step
  5). Trigger: a consumer that needs the full-span number.
- No wire-send budget (D8). Trigger: a consumer that needs enforcement
  over actual sends, not logical work.
- No universal Plan grammar / five-verb surface. Trigger: pricing — and
  pricing may instead prove Plan is a *family* of operation-specific
  documents; let that be evidence, not a forced enum. [04]
- No dynamic `--deep` fan-out under plans (D5a). Trigger: tracer
  evidence that two-cycle reconciliation genuinely hurts.
- No pricing table, no shop, no GUI work. Parking lot.

### The method test (carried from 03, sharpened)

This slice tests whether pin-after-the-consumer survives product land.
Evidence collected on purpose: the owner's step-9 friction notes
(especially against D5a's binding semantics — its named revisability);
whether the anti-scope held and what pressure breached it; whether
agent-built internals stayed simple behind pinned boundaries. Verdict
recorded at the end as first input to the pricing session.

---

## 3. Parking lot (landings named; deferral never re-argued)

- **Pricing-as-document** → annotations layer + plan/apply, after the
  tracer. The second plan-bearing consumer — and the test of whether
  Plan is one grammar or a family [04].
- **Legacy buyout import** → a patch generator into the ordinary
  annotation plan/apply path; the wizard dissolves.
- **Shop / forum publishing** → outward credentialed traffic outside
  the API choke-point invariant; requires its own equally structural
  ownership/rate/safety boundary before implementation [04]. Own
  session first.
- **Currency ratios, saved searches** → first user-scoped kinds;
  trigger for `user.db` + scope taxonomy.
- **Annotation event log** → trigger in anti-scope.
- **Wire-send budget** → trigger in anti-scope (D8).
- **Universal Plan grammar / five-verb surface** → direction only;
  evidence at pricing.
- **PoB export, catalogs/enumerable vocabularies** → after pricing.
- **Search-at-scale** (FTS at ingest, search-crate factoring) → per the
  framing's stress test; trigger is a real consumer with a latency or
  scale case.
- **Fact-path migration to stable account keys** → deferred by D2a;
  opportunistic, or never.
