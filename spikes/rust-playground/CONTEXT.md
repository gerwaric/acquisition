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

Only the **cross-cutting** decisions are here, because an agent must know them before it knows which area it is in; the check caps their count. Every other decision lives in its area's file with what that area has parked, read before touching that area (`AGENTS.md`, "Read before changing anything"), and named again at the top of the module it governs:

| File | Area | Decisions |
| --- | --- | --- |
| `decisions/daemon.md` | Daemon, jobs, protocol, accounts | C3, C5, C6, C7, C8, C9, C10, C23, C27, C45, C31, C49, C50, C51 |
| `decisions/network.md` | Network and rate limiting | C17, C18, C19, C20, C21, C22, C24, C25, C26, C32, C33 |
| `decisions/store.md` | Store: facts, realm, characters | C28, C29, C30, C54, C55, C56, C57, C58, C59, C60, C61, C62, C63 |
| `decisions/plans.md` | Intent, plans, apply | C36, C37, C39, C40, C41, C42, C43, C44, C76, C77 |
| `decisions/pricing.md` | Pricing: intent values, listing, reference data, price plans, import, render | C64, C65, C66, C67, C68, C69, C70, C71, C72, C73, C74, C75, C78 |
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
- **C79 — Surfaces GGG does not sanction — the trade site, the forums, third-party feeds — are governed inputs, never runtime dependencies, and permission attaches to the access method.** The daemon never touches them; no store read, plan compile or apply depends on one; each is registered with its status, terms exposure, access method and cadence. A human may read one; tooling fetches one only from an official export or with explicit permission recorded in its entry. What a surface yields lands as claims or reviewed reference data, sources cited. One used as an *effect* needs its own boundary session first. *Why:* Acquisition predates the API and has always used such surfaces; the relationship is protected by keeping them deliberate, not by pretending otherwise. Ruled 2026-09-03.

## Interfaces (boundaries are specified; internals are not)

None of these interfaces are locked down more than anything else in this document.

### Daemon job protocol

The live definition is `crates/acquisition-core/src/protocol.rs` (request/response/event enums, job states). Its boundary properties are the decision lines above; the verb list is internals.

### CLI shape

The live verb list is `acq --help` and the README's "Try it" block. Properties: default mode is blocking-with-progress ("rate limited, starting in ~4m37s..."), `--detach` is the async/job mode, every command takes `--json`, and `daemon status|stop` exist for debugging only.

## Parked (cross-cutting only; do not build yet)

Scope deferred with its trigger, so deferral never needs re-arguing. An area's parked items and open questions live in its `decisions/<area>.md` under "Parked", read before touching the area; only what crosses every area is here. A fired trigger deletes its entry in the build's commit. An entry is the item, where it lands, and the trigger that reopens it, with at most one clause of why; a workaround is a README known gap, history is git.

- Shop / forum publishing (POESESSID, thread numbers, one post per page, bumping against the indexer's thread limit, auto-post after a clean refresh) → outward credentialed traffic **outside the API choke-point invariant**, the third apply target of the one loop; its own boundary session before any code. Until then the render (C74) writes pages to stdout and a human pastes them. Trigger: the render validated and the owner wanting the posts automated.

## Working style

The charter is `README.md`'s opening paragraph; who holds which boundary is `AGENTS.md`. The numbered principles are cited by the registry as `P<n>`.

- Tests pin behavior at boundaries, never mechanisms. A test that reaches into daemon internals pins this implementation, not the contract, and is disposable. The GGG-side contract surface is the send journal (`TESTING-NOTES.md`); the frontend-side surface is the protocol, not yet pinned.
- Decisions get recorded here after the code teaches us, not before. When the current internals get in the way of learning, record the finding and move on rather than polishing.
- Design discussion precedes code on `spikes/rust-playground` — "design" means updating this doc, not writing a spec.
- Prefer simplicity over flexibility when trade-offs arise. Prefer idiomatic Rust patterns over translations from Qt/C++.
- **P1.** Deep design sessions are evidence-driven, never calendar-driven; crystallize before building. Rulings land in this doc; session notes (`brainstorming-notes/`) are disposable history, never a second authority.
- **P2.** In product scope the validating consumer is real use, and each frontend contract needs its own — the owner's live use validates a CLI slice, not the GUI/MCP/TUI contracts; friction notes are data the way the send journal is data.
- **P3.** Generalize after two materially different consumers reveal the shared property — except where an early choice controls irreversible identity, durability, safety, or compatibility (those get first-consumer treatment; the uuid identity decision is the example).
- **P4.** Tactical taste is settled by a lint where mechanical and a recorded property where stakes are real — design discussion precedes a property's promotion to lint or test; everything else is agent-owned internals.
