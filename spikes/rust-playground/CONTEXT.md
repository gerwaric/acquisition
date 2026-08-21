# Acquisition (Rust rewrite) — Agent Context

This doc is working memory for coding agents and a place for developers to inspect and tweak things, not a spec or requirements document. It records what's settled, what's open, and what must never happen. Grows and changes structure as needed. Terse on purpose. Current state only: superseded content is edited or deleted in place — git holds the history. Facts about GGG live in `docs/design/network-ground-truth.md` and are cited here by number, not restated. Where code exists, code is the source of truth for implemented shape — this doc holds boundaries, properties, and open questions, never a parallel description of what the code already says.

## Orientation (3 sentences)

Acquisition is a Path of Exile inventory-management tool being rewritten from C++/Qt to Rust under the same GGG OAuth registration as the existing app. The system is a local daemon that owns all GGG API traffic, with three thin frontends: CLI, GUI, and MCP server. The GGG relationship (OAuth registration, rate-limit standing) is the project's most valuable asset and every design decision defers to protecting it.

## Invariants

As few as possible, each as simple as possible: properties, not mechanisms. An invariant enforced wholly inside one component doesn't need to be listed here unless there's a good reason.

1. **Every request to GGG goes through the daemon's single rate-limit choke point — API, OAuth token, everything** (the token endpoint has its own rate-limit policy: ground-truth N33). A code path that bypasses it is a critical bug regardless of how reasonable it looks locally.
2. **Rate-limit headers from GGG (`X-Rate-Limit-*`) are the source of truth.** Local rate-limit state is a prediction; headers correct it.
3. **Never retry through a Cloudflare block.** Recognition signatures and the one known incident: ground-truth N1–N4, N28.
4. **The user-agent string stays continuous with the existing registration** ("Acquisition 1.0, same registration, new capabilities" framing with GGG).
5. **Refresh tokens are never stored in plaintext on disk.** The OS keyring is their default home.

## Decisions

Short one-liners with rationale, optimized for agents to reason from. Kept current, not append-only: a superseded decision is rewritten or deleted, and git holds the history.

- **Cargo workspace, library-centric.** `acquisition-core` holds OAuth, rate limiter, API client, models; `acquisition-cli`, `acquisition-gui`, `acquisition-mcp` are thin frontends. Rationale: write/test logic once.
- **Daemon owns shared state; clients talk over local IPC.** Rationale: makes the single-choke-point invariant structural, not disciplinary.
- **Lazy spawn + idle auto-shutdown** (gpg-agent model). Any client that can't connect spawns the daemon; daemon exits after a reasonably short period with no connections and no queued work. `daemon stop|status` subcommands exist for debugging only — normal use never requires manual lifecycle. Rationale: no stray processes, no user-visible ceremony.
- **API requests are jobs, not calls.** Rate-limit waits can reach 300s, so the core abstraction is a job with ID, state, and priority. Rationale: blocking calls can't represent a 5-minute wait sanely across three frontends.
- **Priority queue from day one, even if v1 uses one priority level.** Rationale: reorder becomes a field write, not a data-structure migration.
- **Jobs are in-memory for v1.** SQLite-backed persistence is a known upgrade path (same job table schema). Rationale: simplicity first.
- **Daemon protocol includes a subscribe/event channel from the start** (job-state-changed events over the same socket). Rationale: GUI push updates and `jobs --watch` come nearly free; bolting on later is painful.
- **Protocol transport: JSON lines over Unix socket / named pipe** (tokio + serde). Rationale: boring, debuggable, cross-platform.
- **Version handshake in the protocol.** Daemon reports its version; client kills and respawns on mismatch. Rationale: CLI and running daemon may be from different builds.
- **CLI emits structured output** (`--json` on every command). Rationale: the CLI is itself an API; makes MCP and agent use nearly free.
- **Tokio + reqwest for async/HTTP.** Rationale: ecosystem default; core exposes `async fn`, frontends provide the runtime.
- **Tauri for GUI** (webview frontend). Rationale: item search/grid/filter UIs are a strength of web tech; egui considered and passed on for data-heavy views.
- **Rate limiter: custom policy layer parsing GGG headers, with a simple enforcement mechanism underneath.** Rationale: GGG's header-driven limits are too specific for off-the-shelf policy.
- **Limiter state is keyed by policy name and learned only from headers.** No local token counting; waits are computed from the last `X-Rate-Limit-*` state plus response arrival times, padded by the server's timing bucket. Bucket classification is positional (first window initial/5s, later windows sustained/60s — ground truth Q4 hypothesis). Rationale: same-name policies share counters across endpoints (N6), so endpoint-keyed state would be a migration later; positional classification is conservative on every observed policy shape.
- **Rate limiter spec will be expressed as test tables, not prose.** `docs/design/network-ground-truth.md` (the claims registry; it indexes the deeper spike evidence) is the input; "given these headers, wait N seconds" tests are the permanent, enforced spec.

## Interfaces (boundaries are specified; internals are not)

None of these interfaces are locked down more than anything else in this document.

### Daemon job protocol

The live definition is `crates/acquisition-core/src/protocol.rs` (request/response/event enums, job states). Its boundary properties are the decision lines above; the verb list is internals.

ETA is computed from limiter state + queue depth ahead of the job — the daemon can predict, because it sees everything.

### CLI shape

```
acquisition auth
acquisition get-character <name> [--json]
acquisition fetch-stashes [--detach]     # returns job_id immediately
acquisition jobs [--watch]
acquisition result <job_id>
acquisition daemon status|stop           # debugging only
```

Default CLI mode is blocking-with-progress ("rate limited, retrying in 4m37s..."); `--detach` is the async/job mode.

## Open topics

- Ctrl-C semantics in blocking CLI mode: cancel the job, or leave it running in the daemon?
- Multi-request job cancellation and/or requests that spawn substash requests?
- Priority levels: how many, and named or numeric? (Interactive GUI > CLI > background/MCP is the intuition.)
- MCP server: in-process with the daemon, or a fourth thin client?

## Explicitly deferred (do not build yet)

- Caching API results in a local database that lives behind the daemon.
- Job persistence across daemon restarts (SQLite-backed queue).
- Queue-management UI (drag-to-reorder, per-job progress bars). v1.0 only guarantees the architecture makes this a rendering problem.
- Agent/MCP rate-limit stress-testing against GGG — blocked on verifying GGG's policy stance on agent traffic before the MCP path ships.

## Working style

- Spike-and-stabilize: throwaway builds are cheap; decisions get recorded here after spikes teach us, not before.
- Design discussion precedes code on `spikes/rust-playground` — "design" means updating this doc, not writing a spec.
- Owner (Tom) holds the boundaries: invariants, protocol, core API surface. Agents own internals behind those boundaries.
- Prefer simplicity over flexibility when trade-offs arise. Prefer idiomatic Rust patterns over translations from Qt/C++.
