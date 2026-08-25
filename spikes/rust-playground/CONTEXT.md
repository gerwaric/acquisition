# Acquisition (Rust rewrite) — Agent Context

This doc is working memory for coding agents and a place for developers to inspect and tweak things, not a spec or requirements document. It records what's settled, what's open, and what must never happen. Grows and changes structure as needed. Terse on purpose. Current state only: superseded content is edited or deleted in place — git holds the history. Facts about GGG live in `docs/design/network-ground-truth.md` and are cited here by number, not restated. Where code exists, code is the source of truth for implemented shape — this doc holds boundaries, properties, and open questions, never a parallel description of what the code already says.

## Orientation (3 sentences)

Acquisition is a Path of Exile inventory-management tool being rewritten from C++/Qt to Rust under the same GGG OAuth registration as the existing app. The system is a local daemon that owns all GGG API traffic, with three thin frontends: CLI, GUI, and MCP server. The GGG relationship (OAuth registration, rate-limit standing) is the project's most valuable asset and every design decision defers to protecting it.

## Invariants

As few as possible, each as simple as possible: properties, not mechanisms. An invariant enforced wholly inside one component doesn't need to be listed here unless there's a good reason.

1. **Every daemon-originated request to GGG goes through one rate-limit choke point and one send-lifetime global gate — API GET, HEAD probe, OAuth code exchange, and OAuth refresh** (the token endpoint has its own rate-limit policy: ground-truth N33). The browser-owned authorize navigation is outside this boundary; the daemon opens the URL and does not send that request. A daemon HTTP path that bypasses the choke point or gate is a critical bug regardless of how reasonable it looks locally.
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
- **Jobs are in-memory for v1.** SQLite-backed persistence is a known upgrade path (same job table schema). Rationale: simplicity first. First frontend requirement on it (2026-08-24, from `acq pull`): a client that comes back after its refresh finished and the daemon has since idled out has no way to find that refresh's results; the sends were already spent. See the persistence open topic.
- **A job's `params` travel on `JobInfo`, verbatim and public.** Every connected client sees them, so a job's params must never carry a secret (tokens are obtained inside the daemon, never passed in). Rationale: a job a person cannot identify (`stash`, `stash`, `stash`) cannot be managed — queue management, when it comes, needs this row; and a client labelling a failed child otherwise has to zip parent payload arrays by position. Rendering (`acq jobs`' `target` column, `JobInfo::target`) is the client's business. Decided 2026-08-24.
- **Daemon protocol includes a subscribe/event channel from the start** (job-state-changed events over the same socket). Rationale: GUI push updates and `jobs --watch` come nearly free; bolting on later is painful.
- **Protocol transport: JSON lines over Unix socket / named pipe** (tokio + serde). Rationale: boring, debuggable, cross-platform.
- **Version handshake in the protocol.** Daemon reports its version; client kills and respawns on mismatch. Rationale: CLI and running daemon may be from different builds.
- **CLI emits structured output** (`--json` on every command). Rationale: the CLI is itself an API; makes MCP and agent use nearly free.
- **Tokio + reqwest for async/HTTP.** Rationale: ecosystem default; core exposes `async fn`, frontends provide the runtime.
- **Tauri for GUI** (webview frontend). Rationale: item search/grid/filter UIs are a strength of web tech; egui considered and passed on for data-heavy views.
- **Rate limiter: custom policy layer parsing GGG headers, with a simple enforcement mechanism underneath.** Rationale: GGG's header-driven limits are too specific for off-the-shelf policy.
- **Limiter state is keyed by policy name and learned only from headers.** No local token counting; waits are computed from the last `X-Rate-Limit-*` state plus response arrival times, padded by the server's timing bucket. For the observed paired API-policy windows, classification is positional (first window initial/5s, later windows sustained/60s — ground-truth Q4 hypothesis). That hypothesis does not classify N33's single-window `token-request-limit`; use the conservative 60s bucket for it until GGG confirms its hidden resolution through N14's support channel. Rationale: same-name policies share counters across endpoints (N6), so endpoint-keyed state would be a migration later; positional classification is conservative on the observed paired API shapes without pretending the new one-window shape is known.
- **Endpoint discovery is a visible `probe` job: one HEAD per endpoint per daemon lifetime, queued by the daemon before the first real send.** Rationale: N16's sanctioned pattern is one HEAD at startup; doing it lazily per endpoint sends the same count and nothing for endpoints never used. Visible rather than internal because everything that touches GGG is a job, and HEAD has regressed server-side before (N20) — the probe's headers need to be inspectable.
- **The global burst bound belongs at the daemon's HTTP send boundary and spans each actual request from immediately before dispatch until its response/body completes.** API requests use policy serialization plus ordinary permits from this common gate; HEAD probes take its exclusive, writer-preferred permit. OAuth code exchange and refresh use ordinary permits and serialize under stable route key `oauth-token` before discovery, then learned policy name `token-request-limit`; no HEAD probes the token endpoint. Authentication completes before an API request performs its final limiter check and acquires its send permit, so neither auth nor rate-limit waiting occupies an API permit. Browser authorize remains outside because the browser owns it. The dispatcher may keep one active job task per scheduling key to preserve priority and FIFO, but it has no global job-task cap; auth and pacing waits therefore cannot block progress on independent keys. Rationale: P-B and N33 — Cloudflare watches bursts across policies and the token endpoint is itself Cloudflare-fronted and IP-limited; HTTP capacity must describe actual sends, not waiting work.
- **The frozen C++ network design (`docs/design/network-redesign.md`) is a property source, not a Rust architecture template.** The Rust code conforms to its D3/D4/D5/D8 properties with three intentional adaptations: OAuth token traffic is inside the common gate (N33 postdates the C++ design); ordinary FIFO is among currently eligible policies, so a same-policy waiter cannot idle an independent global slot; and the C++ gate's 250 ms inter-send spacing is not adopted. Rationale: the properties protect the GGG relationship; the mechanisms were shaped by Qt. Changing an adaptation is a design change, recorded here first.
- **Work that needs many requests is a parent job that submits child jobs; a parent finishes when its last descendant does, gives up its dispatcher task and scheduling key while waiting, and cancels its descendants when cancelled.** Rationale: the queue, dispatcher, priorities, ETAs, and events already work per job, so children get all of it for free; a job-internal loop would need its own scheduler and hide the requests from every tool. Observed API shapes (2026-08-20): folder children are in the stash list (a folder holds tabs only — never items, never another folder; confirmed against GGG patch notes 2026-08-24); map/unique substashes only appear on fetching the tab (one map tab listed 234); substash stubs carry `metadata.items` counts. Following substashes is opt-in per tab.
- **A `refresh_token` grant the provider rejects with a 4xx other than 429 is terminal: no further refresh is sent until `acq auth` or logout.** Rationale: R1/L0-R5 in `LIVE-TESTING.md` — a dead grant re-sent per flight is pointless traffic on a Cloudflare-fronted endpoint, and the rail that stopped it (rail 2) was opt-in, so the shipped default still did it (`TESTING-NOTES.md`, surprise 12). Decided and built 2026-08-24; the mark persists in the rails state file and is honored regardless of the tripwire.
- **A 429 re-queues the job (keeping its place) behind the limiter's hold; after `MAX_429_RETRIES` (2) it fails with the evidence. 403/503 are never retried.** Rationale: P-A — violations are structural, so recovery is a requirement; N10 — frequent violations revoke the app, so it's bounded; invariant 3 for the Cloudflare shapes. No new job state: `running → waiting` with a retry counter on the job.
- **The first real consumer is `acq pull` (2026-08-24): a full-league stash pull, snapshot on the client's disk, diff on the next run.** It uses only the existing protocol verbs (`submit`, `status`, `list`, `result`) on purpose: every place the walk is awkward is a fact about what the frontend boundary needs, recorded under "Frontend boundary findings" below rather than patched in the daemon. Snapshots live with the frontend (`$ACQ_SNAPSHOTS`); a frontend remembering what it fetched is not the deferred daemon-side cache. Rationale: `TESTING-NOTES.md` — the GGG side is mapped to diminishing returns; nothing had ever collected a refresh's children, so the product did the network work and threw the answer away.
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

## Frontend boundary findings (from `acq pull`, 2026-08-24)

What a real consumer needed from the protocol and did not get. Facts, not decisions; each is a candidate protocol change for Tom to accept or refuse.

- **`JobInfo` had no `params`** — resolved 2026-08-24 (decision above): `params` on `JobInfo`, `acq jobs` shows a `target` column, `pull` reads a child's tab from its own params.
- **Collecting a subtree is N+1 round trips** (`list`, then `result` per child; 15 for a deep pull of the mock, hundreds for a real map tab). Fine over a Unix socket; shape-wise it wants either results delivered on the event channel as jobs finish, or a `results` verb over a subtree.
- **The denominator grows.** Children exist only once their parent runs, so progress reads "0/1" and then "8/8"; a deep pull grows again when each map/unique tab lands. Any progress UI must expect the tree to widen while it watches.
- **Results outlive the client but not the daemon.** `pull` killed mid-hold (a 2-minute wall clock hit while the limiter held a 300 s window) left the refresh running to completion in the daemon with nobody to collect it. Two separate problems, resolved separately (2026-08-24): (1) *the refresh is still running* — the common case, since a hold can last minutes — `pull` now records `{daemon pid, job id, params}` at submit and reattaches on the next run (verified: a pull killed 4 s in was collected 342 s later by the next invocation); (2) *the refresh finished and the daemon idled out* — a job-persistence requirement, deferred with the persistence open topic. Queue management (cancel/reorder unsent work) is a third, unrelated thing and should not be pulled in by either. Ctrl-C answer implied: leave the job running; the sends are committed either way.
- **Nameless substashes.** Map/unique substashes carry an empty `name` (map ones have `metadata.map.name`); a frontend labels them `parent/id`. Tab identity for a substash is `(parent, id)`.
- **Two surfaces for one hold agree.** A pre-emptive pacing hold shows as the ETA on every waiting child and as `wait_ms` on exactly the one send that was held (12 923 ms for a 10 s window + timing-bucket pad); the sends released after it read 0. Not a finding against `wait_ms`; recorded because it was misread once.

## Open topics

- Ctrl-C semantics in blocking CLI mode: cancel the job, or leave it running in the daemon? Current lean (from `acq pull`): leave it running and reattach; the sends happen either way.
- **Persistence — two questions, not one.** (1) *Job persistence* (queue + outcomes surviving disconnect/restart): daemon-owned, shape already decided (SQLite job table), timing deferred; now has a real frontend requirement (results finished while the client was away). (2) *Data persistence* (items, tabs, buyouts, search state — the app's model): undecided whether the daemon owns it or each frontend does. Daemon-owned means one store shared by GUI/CLI/MCP and one refresh, but every item-model decision becomes a protocol decision and the daemon becomes an application server rather than the GGG traffic owner. Deferred until the first GUI consumer shows whether it wants items *from the daemon* or from its own store fed by jobs. `acq pull`'s client-side snapshot is a prototype of the frontend-owned answer. Neither question changes `JobInfo.params` (input, stored verbatim either way); only result *shape* could be affected.
- Priority levels: how many, and named or numeric? (Interactive GUI > CLI > background/MCP is the intuition.)
- MCP server: in-process with the daemon, or a fourth thin client?

## Explicitly deferred (do not build yet)

- Caching API results in a local database that lives behind the daemon.
- Job persistence across daemon restarts (SQLite-backed queue).
- Queue-management UI (drag-to-reorder, per-job progress bars). v1.0 only guarantees the architecture makes this a rendering problem.
- Agent/MCP traffic against GGG — blocked on verifying GGG's policy stance on agent traffic before the MCP path ships. (Owner-driven live baseline testing of the daemon against GGG is not deferred; it has its own control document.)

## Working style

- This branch is the **reference implementation**. Its purpose is to find out what the daemon and rate limiter need to be and to pin that as tests and recorded decisions; the code is replaceable given a reason (a bug, performance, maintainability, understandability) no matter how complete it gets, and a fully operational CLI is still evidence, not a promotion. It may become the real implementation, or a fresh build may replace it — judged by the same tests and the same live ladder (ADR 0003 stays open; both paths share this goal function). The limiter's behavior is fully specified (`ratelimit.rs` test tables); the daemon's is still being mapped at its two boundaries, GGG and the frontends.
- Tests pin behavior at boundaries, never mechanisms. A test that reaches into daemon internals pins this implementation, not the contract, and is disposable. The GGG-side contract surface is the send journal (`TESTING-NOTES.md`); the frontend-side surface is the protocol, not yet pinned.
- Decisions get recorded here after the code teaches us, not before. When the current internals get in the way of learning, record the finding and move on rather than polishing.
- Design discussion precedes code on `spikes/rust-playground` — "design" means updating this doc, not writing a spec.
- Owner (Tom) holds the boundaries: invariants, protocol, core API surface. Agents own internals behind those boundaries.
- Prefer simplicity over flexibility when trade-offs arise. Prefer idiomatic Rust patterns over translations from Qt/C++.
