# Acquisition (Rust rewrite) — Agent Context

This doc is working memory for coding agents and a place for developers to inspect and tweak things, not a spec or requirements document. It records what's settled, what's open, and what must never happen. Terse on purpose. Current state only: superseded content is edited or deleted in place — git holds the history. Facts about GGG live in `docs/design/network-ground-truth.md` and are cited here by number, not restated. Decisions are a numbered registry (`C<n>`, below): the ruling and its rationale live here, one bullet each; the mechanism a decision implies lives as a doc comment on the code that implements it and in the tests that pin it, cited by id. Where code exists, code is the source of truth for implemented shape — this doc holds boundaries, properties, and open questions, never a parallel description of what the code already says.

## Orientation (3 sentences)

Acquisition is a Path of Exile inventory-management tool being rewritten from C++/Qt to Rust under the same GGG OAuth registration as the existing app. The system is a local daemon that owns all GGG API traffic, with three thin frontends: CLI, GUI, and MCP server. The GGG relationship (OAuth registration, rate-limit standing) is the project's most valuable asset and every design decision defers to protecting it.

## Invariants

As few as possible, each as simple as possible: properties, not mechanisms. An invariant enforced wholly inside one component doesn't need to be listed here unless there's a good reason.

1. **Every daemon-originated request to GGG goes through one rate-limit choke point and one send-lifetime global gate — API GET, HEAD probe, OAuth code exchange, and OAuth refresh** (the token endpoint has its own rate-limit policy: ground-truth N33). The browser-owned authorize navigation is outside this boundary; the daemon opens the URL and does not send that request. A daemon HTTP path that bypasses the choke point or gate is a critical bug regardless of how reasonable it looks locally.
2. **Rate-limit headers from GGG (`X-Rate-Limit-*`) are the source of truth.** Local rate-limit state is a prediction; headers correct it.
3. **Never retry through a Cloudflare block.** Recognition signatures and the one known incident: ground-truth N1–N4, N28.
4. **The user-agent string stays continuous with the existing registration** ("Acquisition 1.0, same registration, new capabilities" framing with GGG).
5. **Refresh tokens are never stored in plaintext on disk.** The OS keyring is their default home.

## Decisions — the registry

One entry per decision: a stable id (`C<n>`, never reused; a superseded decision is rewritten in place under its id and dated, or deleted — git holds the history), the ruling in the owner's words, *Why:* in a sentence, and where the rest lives — *Details:* the module doc that carries the mechanism as recorded, *Pinned:* the tests, *Evidence:* the claim or ledger row. An entry is one bullet under `tools/docs-check.sh`'s length limit; a decision that needs more is a decision plus a mechanism, and the mechanism goes to the code. Tests and docs cite decisions by id; the check refuses an id that does not exist and reports a decision nothing cites.

Only the **cross-cutting** decisions are here, because an agent must know them before it knows which area it is in; the check caps their count. Every other decision lives in its area's file, read before touching that area (`AGENTS.md`, "Read before changing anything"), and named again at the top of the module it governs:

| File | Area | Decisions |
| --- | --- | --- |
| `decisions/daemon.md` | Daemon, jobs, protocol, accounts | C3, C5, C6, C7, C8, C9, C10, C23, C27, C45, C31, C49, C50, C51 |
| `decisions/network.md` | Network and rate limiting | C17, C18, C19, C20, C21, C22, C24, C25, C26, C32, C33 |
| `decisions/store.md` | Store: facts, realm, characters | C28, C29, C30, C54, C55, C56, C57, C58, C59, C60, C61, C62, C63 |
| `decisions/plans.md` | Intent, plans, apply | C36, C37, C39, C40, C41, C42, C43, C44 |
| `decisions/frontends.md` | Frontends and output | C11, C13, C16, C52, C53 |

### Cross-cutting

- **C1 — Cargo workspace, library-centric.** `acquisition-core` holds OAuth, rate limiter, API client, models; `acquisition-store` and `acquisition-plan` the facts/intent and the planner; `acquisition-cli`, `acquisition-mcp` (and a future `acquisition-gui`) are thin frontends. *Why:* write/test logic once.
- **C2 — Daemon owns shared state; clients talk over local IPC.** *Why:* makes the single-choke-point invariant structural, not disciplinary.
- **C4 — API requests are jobs, not calls.** Rate-limit waits can reach 300 s, so the core abstraction is a job with ID, state, and priority. *Why:* blocking calls can't represent a 5-minute wait sanely across three frontends.
- **C12 — A frontend consumes exactly two surfaces: the daemon protocol and the store crate's read API — no third door.** This pins the boundary's *location*, not its content; a frontend that wants a third channel is a protocol or store change, recorded here first. *Why:* bespoke per-frontend channels turn three frontends into a review burden; two shared surfaces keep the contract in exactly two places, enforced by what frontends link against. Decided 2026-08-30.
- **C14 — Agent traffic against GGG is allowed; the daemon is the single gate.** Owner ruling 2026-09-01: GGG permits agent use of the API as long as the API rules are respected; the distinction between human, script and agent clients was never enforceable, and what is enforceable is one gate every client's traffic passes through (invariant 1). `acq-mcp` submits, applies and quotes in either mode against a running daemon; the live-test rails stay what `LIVE-TESTING.md` says.
- **C15 — Tokio + reqwest for async/HTTP.** *Why:* ecosystem default; core exposes `async fn`, frontends provide the runtime.
- **C34 — The system is four layers — facts, intent (annotations), derivations, effects — each with one authoritative mutation path, not one physical writer.** Facts mutate only through the store crate's ingest surface; intent only through its annotation write API; the effects ledger only through the daemon; derivations are always reproducible from declared inputs. The daemon is permanently blind to intent and creates work only in causal service of client-submitted work — never spontaneously: no schedules, no policy execution, no annotation reads; scheduled syncs are small frontends. *Why:* "a sync can never clobber intent" becomes structural, the way the choke point made rate-limit discipline structural. Decided 2026-08-31 (brainstorming-notes 06).
- **C35 — Annotations are the only irreplaceable local state.** A separate per-account file named by the account uuid, keyed on stable GGG ids, written only through the store crate with integer-revision compare-and-swap; no fact-side event ever deletes intent (an orphaned annotation is kept and surfaceable); backup is a store-managed consistent snapshot, never a raw file copy under WAL. *Why:* facts are refetchable at the cost of requests; intent has no server to refetch from. Decided 2026-08-31.
- **C38 — A Plan is a serializable, immutable authorization envelope, and plans are binding.** Derived from a named snapshot of facts + intent, computable with the daemon down; it carries provider, account uuid, operation and schema version, fact basis, annotation revision, the explicit action set, and optionally a quote. Applying executes exactly the listed actions or a strict subset, never an unreviewed addition; new facts produce a new Plan; operation-specific types first (`RefreshPlan`); a universal grammar waits for the second plan-bearing consumer. *Why:* reviewed work stays exact. Decided 2026-08-31; **confirmed live 2026-09-01** with no owner friction. *Details:* `acquisition-plan` doc, C38.
- **C46 — Shared semantics live in Rust; every frontend has a Rust adapter** (clap CLI, `rmcp` MCP, Tauri backend — the webview is presentation, never a second implementation — `dash` TUI). A proposed non-Rust frontend is a design event, recorded here first. *Why:* this premise is what makes "built once, inherited by every frontend" true; unstated premises erode silently. Decided 2026-08-31.
- **C47 — Panics are for broken internal invariants only; malformed external input — a GGG body, a store row, a protocol message — is a structured error with stable kinds and context.** The store and plan crates enforce it mechanically (`clippy::unwrap_used`/`expect_used` denied in production code); the daemon's `.lock().unwrap()` poisoning idiom stays. *Why:* the persisted queue makes crashes recoverable, which turns a reproducible panic on bad input into a crash loop — the one failure persistence cannot absorb. Decided 2026-08-31.
- **C48 — The SQLite schema is internal; raw SQL is not a surface.** Schema versions and compatibility errors; defended by making the store crate's API expressive enough that going around it is never worth it. No cached search service. *Why:* stale results mistaken for current truth is the failure a cache reintroduces; reopening needs a measured duplication or latency case. Decided 2026-08-31.

## Interfaces (boundaries are specified; internals are not)

None of these interfaces are locked down more than anything else in this document.

### Daemon job protocol

The live definition is `crates/acquisition-core/src/protocol.rs` (request/response/event enums, job states). Its boundary properties are the decision lines above; the verb list is internals.

ETA is computed from limiter state + queue depth ahead of the job — the daemon can predict, because it sees everything.

### CLI shape

The live verb list is `acq --help` and the README's "Try it" block. Properties: default mode is blocking-with-progress ("rate limited, starting in ~4m37s..."), `--detach` is the async/job mode, every command takes `--json`, and `daemon status|stop` exist for debugging only.

## Frontend boundary findings (from `acq pull`, 2026-08-24; `pull` itself was retired 2026-08-29 in favor of the store)

What a real consumer needed from the protocol and did not get. Facts, not decisions; each is a candidate protocol change for Tom to accept or refuse. Resolved ones become decisions above and are deleted here.

- **Collecting a subtree is N+1 round trips** (`list`, then `result` per child; 15 for a deep pull of the mock, hundreds for a real map tab). Fine over a Unix socket; shape-wise it wants either results delivered on the event channel as jobs finish, or a `results` verb over a subtree. Waits for a second consumer (GUI or MCP) to show which.
- **The denominator grows.** Children exist only once their parent runs, so progress reads "0/1" and then "8/8"; a deep pull grows again when each map/unique tab lands. Any progress UI must expect the tree to widen while it watches. A property, not a change request.
- **Nameless substashes.** Map/unique substashes carry an empty `name` (map ones have `metadata.map.name`); a frontend labels them `parent/id`. Tab identity for a substash is `(parent, id)`. Client-side; the returned JSON is not to be changed.

## Open topics

- Priority levels: how many, and named or numeric? (Interactive > background is the intuition, *regardless of frontend* — an agent in a live conversation is interactive; the caller states its urgency, the frontend doesn't imply it.)
- The ad-hoc `refresh --tabs`/`--all` kind beside the plan path: two doors to one task. Evidence in: across four live runs nobody reached for it (the tracer's friction prompt asked). Retiring it needs one design answer first — how a human fetches exactly one tab without authoring a policy (an explicit selection compiled as a policy-less plan through the same envelope and apply?). A candidate line for the pricing packet; the kind stays until then.
- Pending ground-truth claims from the 2026-09-02 documentation read (not observed, so not yet claims): realm segment semantics per endpoint and pc-by-omission; PoE2 on the character endpoints only; `inventoryId` undocumented; the invalid-request (4xx) threshold.

(2026-08-31: "delta/selection for refresh" and "user state on items" are resolved into the sync-policy / annotations / Plan decisions above; the tracer section above built them.)

## Explicitly deferred (do not build yet)

- Queue-management UI (drag-to-reorder, per-job progress bars). v1.0 only guarantees the architecture makes this a rendering problem.
- **Parking lot (2026-08-31, each with its trigger so deferral never needs re-arguing):**
  - Pricing-as-document → lands on the annotations layer + plan/apply after the tracer and after characters join the refresh plan (agreed 2026-09-02); the second plan-bearing consumer, and the test of whether Plan is one grammar or a family of operation-specific documents.
  - Legacy buyout import → a patch generator into the ordinary annotation plan/apply path; the wizard dissolves.
  - Shop / forum publishing → outward credentialed traffic (POESESSID) **outside the API choke-point invariant**; requires its own equally structural ownership/rate/safety boundary session before any implementation.
  - User-scoped annotations home (`user.db`) + scope taxonomy → trigger: the first user-scoped kind (currency ratios, saved searches).
  - Annotation event log → trigger: `diff --since` needs "what got repriced," or conflicts need history (row revisions exist from day one; the schema is shaped so the log is an addition, not a migration).
  - Wire-send budget → trigger: a consumer that needs enforcement over actual sends, not logical work.
  - Universal Plan grammar / five-verb surface → direction only; evidence at pricing.
  - Dynamic `--deep` fan-out under plans → trigger: tracer evidence that two-cycle reconciliation genuinely hurts (2026-09-01: the tracer rung produced none; stays parked).
  - Type-level sync-policy filters ("skip map tabs", "include unique tabs", "fetch folder children") → trigger: a policy author who needs a type exclusion the parent-covers-children rule cannot express; a policy-shape change (planner owns the schema).
  - Fact-path migration to uuid naming → opportunistic, or never (facts are refetchable).
  - Per-realm merge at `policy set` (today a set replaces the whole policy, so a poe2 run erases the pc policy — seen 2026-09-02) → trigger: a second realm in daily use; until then the author re-sets the pc policy, one command.
  - Search-at-scale (FTS at ingest, search-crate factoring behind the store API) → trigger per the two-surface stress test: a real consumer with a measured latency or duplication case.

## Working style

- This branch is the **reference implementation**. Its purpose is to find out what the daemon and rate limiter need to be and to pin that as tests and recorded decisions; the code is replaceable given a reason (a bug, performance, maintainability, understandability) no matter how complete it gets, and a fully operational CLI is still evidence, not a promotion. It may become the real implementation, or a fresh build may replace it — judged by the same tests and the same live ladder (ADR 0003 stays open; both paths share this goal function). The limiter's behavior is fully specified (`ratelimit.rs` test tables); the daemon's GGG-side boundary is proven by the closed live ladder (2026-08-27, `LIVE-TESTING.md`); the frontend boundary is where mapping continues.
- Tests pin behavior at boundaries, never mechanisms. A test that reaches into daemon internals pins this implementation, not the contract, and is disposable. The GGG-side contract surface is the send journal (`TESTING-NOTES.md`); the frontend-side surface is the protocol, not yet pinned.
- Decisions get recorded here after the code teaches us, not before. When the current internals get in the way of learning, record the finding and move on rather than polishing.
- Design discussion precedes code on `spikes/rust-playground` — "design" means updating this doc, not writing a spec.
- Owner (Tom) holds the boundaries: invariants, protocol, core API surface. Agents own internals behind those boundaries.
- Prefer simplicity over flexibility when trade-offs arise. Prefer idiomatic Rust patterns over translations from Qt/C++.
- Deep design sessions are evidence-driven, never calendar-driven; crystallize before building. Rulings land in this doc; session notes (`brainstorming-notes/`) are disposable history, never a second authority.
- In product scope the validating consumer is real use, and each frontend contract needs its own — the owner's live use validates a CLI slice, not the GUI/MCP/TUI contracts; friction notes are data the way the send journal is data.
- Generalize after two materially different consumers reveal the shared property — except where an early choice controls irreversible identity, durability, safety, or compatibility (those get first-consumer treatment; the uuid identity decision is the example).
- Tactical taste is settled by a lint where mechanical and a recorded property where stakes are real — design discussion precedes a property's promotion to lint or test; everything else is agent-owned internals.
