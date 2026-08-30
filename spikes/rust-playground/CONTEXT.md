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
- **Jobs are in-memory for v1.** SQLite-backed persistence is a known upgrade path (same job table schema). Rationale: simplicity first. The first real requirement on it is in the persistence open topic.
- **A job's `params` travel on `JobInfo`, verbatim and public.** Every connected client sees them, so a job's params must never carry a secret (tokens are obtained inside the daemon, never passed in). Rationale: a job a person cannot identify (`stash`, `stash`, `stash`) cannot be managed — queue management, when it comes, needs this row; and a client labelling a failed child otherwise has to zip parent payload arrays by position. Rendering (`acq jobs`' `target` column, `JobInfo::target`) is the client's business. Decided 2026-08-24.
- **Daemon protocol includes a subscribe/event channel from the start** (job-state-changed events over the same socket). Rationale: GUI push updates and `jobs --watch` come nearly free; bolting on later is painful.
- **Protocol transport: JSON lines over Unix socket / named pipe** (tokio + serde). Rationale: boring, debuggable, cross-platform.
- **Version handshake in the protocol.** Daemon reports its version; client kills and respawns on mismatch. Rationale: CLI and running daemon may be from different builds.
- **CLI emits structured output** (`--json` on every command). Rationale: the CLI is itself an API; makes MCP and agent use nearly free.
- **Tokio + reqwest for async/HTTP.** Rationale: ecosystem default; core exposes `async fn`, frontends provide the runtime.
- **Tauri for GUI** (webview frontend). Rationale: item search/grid/filter UIs are a strength of web tech; egui considered and passed on for data-heavy views.
- **Rate limiter: custom policy layer parsing GGG headers, with a simple enforcement mechanism underneath.** Rationale: GGG's header-driven limits are too specific for off-the-shelf policy.
- **Limiter state is keyed by policy name and learned only from headers.** No local token counting; waits are computed from the last `X-Rate-Limit-*` state plus response arrival times, padded by the server's timing bucket. For the observed paired API-policy windows, classification is positional (first window initial/5s, later windows sustained/60s — ground-truth Q4 hypothesis). That hypothesis does not classify N33's single-window `token-request-limit`; use the conservative 60s bucket for it until GGG confirms its hidden resolution through N14's support channel. Rationale: same-name policies share counters across endpoints (N6), so endpoint-keyed state would be a migration later; positional classification is conservative on the observed paired API shapes without pretending the new one-window shape is known.
- **A counted send whose response carries no usable policy headers is paced as if the server counted it — the same treatment as a send lost in transport.** One predicted hit in every window as of that response; the definition and the last real observation are untouched; the next response with headers replaces the prediction (invariant 2). The job still fails and is never retried. Rationale: rung 10's origin 503 (2026-08-24, N35) was an HTML page with no `X-Rate-Limit-*`; before this, such a response left pacing reading a state that predated it, so a run of 503s went out unpaced. This bounds an outage at the policy's own rate (30 per 300 s on stash) with no new policy; backing off harder than that waits for evidence. Decided 2026-08-24.
- **Endpoint discovery is a visible `probe` job: one HEAD per endpoint per daemon lifetime, queued by the daemon before the first real send.** Rationale: N16's sanctioned pattern is one HEAD at startup; doing it lazily per endpoint sends the same count and nothing for endpoints never used. Visible rather than internal because everything that touches GGG is a job, and HEAD has regressed server-side before (N20) — the probe's headers need to be inspectable.
- **The global burst bound belongs at the daemon's HTTP send boundary and spans each actual request from immediately before dispatch until its response/body completes.** API requests use policy serialization plus ordinary permits from this common gate; HEAD probes take its exclusive, writer-preferred permit. OAuth code exchange and refresh use ordinary permits and serialize under stable route key `oauth-token` before discovery, then learned policy name `token-request-limit`; no HEAD probes the token endpoint. Authentication completes before an API request performs its final limiter check and acquires its send permit, so neither auth nor rate-limit waiting occupies an API permit. Browser authorize remains outside because the browser owns it. The dispatcher may keep one active job task per scheduling key to preserve priority and FIFO, but it has no global job-task cap; auth and pacing waits therefore cannot block progress on independent keys. Rationale: P-B and N33 — Cloudflare watches bursts across policies and the token endpoint is itself Cloudflare-fronted and IP-limited; HTTP capacity must describe actual sends, not waiting work.
- **The frozen C++ network design (`docs/design/network-redesign.md`) is a property source, not a Rust architecture template.** The Rust code conforms to its D3/D4/D5/D8 properties with three intentional adaptations: OAuth token traffic is inside the common gate (N33 postdates the C++ design); ordinary FIFO is among currently eligible policies, so a same-policy waiter cannot idle an independent global slot; and the C++ gate's 250 ms inter-send spacing is not adopted. Rationale: the properties protect the GGG relationship; the mechanisms were shaped by Qt. Changing an adaptation is a design change, recorded here first.
- **Work that needs many requests is a parent job that submits child jobs; a parent finishes when its last descendant does, gives up its dispatcher task and scheduling key while waiting, and cancels its descendants when cancelled.** Rationale: the queue, dispatcher, priorities, ETAs, and events already work per job, so children get all of it for free; a job-internal loop would need its own scheduler and hide the requests from every tool. Observed API shapes (2026-08-20): folder children are in the stash list (a folder holds tabs only — never items, never another folder; confirmed against GGG patch notes 2026-08-24); map/unique substashes only appear on fetching the tab (one map tab listed 234); substash stubs carry `metadata.items` counts. Following substashes is opt-in per tab.
- **A `refresh_token` grant the provider rejects with a 4xx other than 429 is terminal: no further refresh is sent until `acq auth` or logout.** Rationale: R1/L0-R5 (`LIVE-TESTING.md` history, `9fa99459`) — a dead grant re-sent per flight is pointless traffic on a Cloudflare-fronted endpoint, and the rail that stopped it (rail 2) was opt-in, so the shipped default still did it (`TESTING-NOTES.md`, "rails-conditional fixes"). Decided and built 2026-08-24; the mark persists in the rails state file and is honored regardless of the tripwire.
- **A 429 re-queues the job (keeping its place) behind the limiter's hold; after `MAX_429_RETRIES` (2) it fails with the evidence. 403/503 are never retried.** Rationale: P-A — violations are structural, so recovery is a requirement; N10 — frequent violations revoke the app, so it's bounded; invariant 3 for the Cloudflare shapes. No new job state: `running → waiting` with a retry counter on the job.
- **A client that disappears leaves its jobs running.** Ctrl-C, a closed terminal, or a crash cancels nothing; the sends are committed either way, and a hold can last minutes. A client that wants the results reattaches (`acq pull` records `{daemon pid, job id, params}` at submit and does so on its next run). Results that finish after the daemon idles out are lost — that is the persistence open topic. Decided 2026-08-24.
- **Persistence is a shared library + file, not a process: `acquisition-store` (SQLite, one file per provider), written by the daemon and read directly by every frontend.** The daemon's whole involvement is `record(endpoint, params, status, body)` after each API success; it never reads the store and never looks inside a body. Search and the item model live in the store crate as plain functions, so the CLI, GUI, and an agent on the CLI call the same code and see the same data. Rationale (2026-08-29): frontend-owned stores duplicate GGG traffic when two frontends pull (the one real rule); daemon-served queries make the daemon an application server; a shared file gives one fetch for all consumers and keeps net/store/frontend separable for testing.
- **Bodies are stored verbatim except at the item seams; `items` is the only place to look for an item.** Every item array (tab `items`, character `inventory`/`equipment`/`jewels`/`rucksack`, each `socketedItems`) is lifted into `items`, one row per GGG item id (stable across moves), keyed by location `(kind, id)`; the envelope keeps the counts under `_split`, so envelope + rows is the response exactly. Derived columns come from the row's own JSON (`rebuild` re-extracts; never a refetch). Ingest compares with the previous state and records `item_events` — this replaces `pull`'s snapshot diff. Rationale: raw-plus-parsed duplicated every body (a league spans 1000× in size); raw-only made every query a body scan and gave user state (buyouts, notes) no key. Decided 2026-08-29; the real-snapshot replay (322 tabs, 19,210 rows, 2.3 s, zero false changes 8 h apart) is the evidence.
- **Multi-account is one daemon holding many sessions, never one daemon per account.** The Cloudflare bound (`SendGate`, 2 live sends) is a per-IP property (P-B, ground truth §1) held as per-process state; two daemons on one machine make it a 4-wide burst that neither sees, with separate tripwires. Rung 11 (2026-08-30) showed the other half: `Account` rules count per account on GGG's side, so two accounts never contend on layer 2 — the only thing they share is layer 1 and the `Ip`-scoped token endpoint, which is exactly what the single gate exists for. Built in two halves with different blast radii (option C): **account as first-class identity now** (store path, job field, keyring key — leaves), **many live sessions later** (a refactor confined to the session layer). Limiter and probe scope keying — `(account, policy)` for `Account` rules, policy alone for `Ip` rules, scope learned from `X-Rate-Limit-Rules` — is a **precondition of the session map, not an optimization**: with two live sessions on one policy each response would overwrite shared state with a different account's counters, and the next send from the other account floods (a 429 path; the "over-waits, never floods" reading only held for rung 11's sequential switch). Decided 2026-08-29, amended 2026-08-30 after review across sessions; design below in "Multi-account design"; not started.
- **Rate limiter spec will be expressed as test tables, not prose.** `docs/design/network-ground-truth.md` (the claims registry; it indexes the deeper spike evidence) is the input; "given these headers, wait N seconds" tests are the permanent, enforced spec.

## Interfaces (boundaries are specified; internals are not)

None of these interfaces are locked down more than anything else in this document.

### Daemon job protocol

The live definition is `crates/acquisition-core/src/protocol.rs` (request/response/event enums, job states). Its boundary properties are the decision lines above; the verb list is internals.

ETA is computed from limiter state + queue depth ahead of the job — the daemon can predict, because it sees everything.

### CLI shape

The live verb list is `acq --help` and the README's "Try it" block. Properties: default mode is blocking-with-progress ("rate limited, starting in ~4m37s..."), `--detach` is the async/job mode, every command takes `--json`, and `daemon status|stop` exist for debugging only.

### Multi-account design (decided 2026-08-30, not started)

**Complexity rule:** the only code that interprets accounts is the
session layer. Everywhere else account is data — a field on the job, a
path segment for the store, an opaque key component for the limiter
(which never reads it; scope comes from `X-Rate-Limit-Rules`). An
`if account == …` outside the session layer is the smell.

- **Identity is the token response's `username`** (`name#discriminator`),
  which every login already returns — no fetch at login, no new failure
  mode. The profile `uuid` is recorded opportunistically whenever
  `/profile` has been called for that account and then accepted as an
  exact match; a name change is one re-auth plus an orphaned store file.
- **No daemon-side default account; stateless selection.** Every submit
  carries `account`. Omitted, it resolves only when exactly one session
  exists; otherwise the daemon refuses with the list. While the daemon
  holds one session, a submitted `account` is validated against it and
  refused on mismatch (so the selector is testable before the session
  map exists). The CLI resolves `--account` / `ACQ_ACCOUNT` client-side
  against a non-secret index file, `store/<provider>/accounts.json`
  (username, uuid when known, last login), so reads never spawn a daemon.
  Matching is exact — name with or without discriminator, or uuid —
  never by prefix. GUI/MCP hold their own selection and pass it.
- One-off (non-persisted) sessions are accounts: listed, selectable,
  marked "not persisted".
- A job has exactly one account; no cross-account `refresh --all`.
  Cross-account work is a frontend loop.
- `account` is a protocol field on `Submit`/`JobInfo`, not a params
  entry; shown in `jobs`, `dash`, and the journal.
- Store: `store/<provider>/<account>.db`, opened lazily on first record;
  `tabs`/`items`/`store` take the selector and never span accounts.
  Keyring: one entry per account; the index file is how the daemon knows
  which entries to restore (the keyring crate cannot enumerate). Restore
  continues past a dead grant — the terminal-grant mark is per session.
  The existing single keyring entry is orphaned, not migrated: one
  re-auth.
- The future `jobs` table lives in a per-daemon `daemon.db`, not inside
  an account file, and carries the account column from day one.
- Mock: the login page accepts any username and policies count per
  username (the access token carries it, `at-<user>-<rand>`), so
  two-account tests can distinguish per-account from shared counting —
  the property rung 11 established for GGG.

Build order, each step gate-green, single-session behaviour unchanged
through (5): (1) identity — username key, index file, per-account keyring
and store; (2) `account` on jobs, validated against the sole session;
(3) the selector; (4) mock any-username login and per-username counting;
(5) limiter and probe scope keying as test-table rows, verified on (4);
(6) the session map; (7) cheap live samples of `/profile`,
`/character/{name}`, `/league` under `LIVE-TESTING.md`'s replacement
rule, uuid recorded opportunistically.

## Frontend boundary findings (from `acq pull`, 2026-08-24; `pull` itself was retired 2026-08-29 in favor of the store)

What a real consumer needed from the protocol and did not get. Facts, not decisions; each is a candidate protocol change for Tom to accept or refuse. Resolved ones become decisions above and are deleted here.

- **Collecting a subtree is N+1 round trips** (`list`, then `result` per child; 15 for a deep pull of the mock, hundreds for a real map tab). Fine over a Unix socket; shape-wise it wants either results delivered on the event channel as jobs finish, or a `results` verb over a subtree. Waits for a second consumer (GUI or MCP) to show which.
- **The denominator grows.** Children exist only once their parent runs, so progress reads "0/1" and then "8/8"; a deep pull grows again when each map/unique tab lands. Any progress UI must expect the tree to widen while it watches. A property, not a change request.
- **Some item fields are volatile.** `veiledMods` placeholder ids change
  on every fetch (N36; rung 10, 2026-08-25: 10 items "changed" between two
  pulls an hour apart with nothing touched). The store's ingest ignores them
  (`acquisition_store::VOLATILE_ITEM_FIELDS`), so every consumer shares the
  list. Resolved 2026-08-29.
- **Nameless substashes.** Map/unique substashes carry an empty `name` (map ones have `metadata.map.name`); a frontend labels them `parent/id`. Tab identity for a substash is `(parent, id)`. Client-side; the returned JSON is not to be changed.

- **A rails halt fails never-sent children.** Rung 10 (2026-08-24): a
  server 503 tripped the tripwire with 82 children queued; all failed
  without a send, so the rerun refetches all 322 tabs. The client half is
  done — the store keeps what landed (every fetched tab is recorded as it
  arrives; 2026-08-29). What remains is daemon-side and the owner's:
  should a halt leave queued jobs *waiting* (resumable after
  `reset-tripwire`, but a halted daemon then never idles out)? The larger
  saving — refetching only the failed set — is the deferred
  delta/selection gap, not a halt question.

## Open topics

- **Job persistence** (queue + outcomes surviving daemon restart): shape decided (a `jobs` table in the same store), timing still deferred. Results themselves no longer die with the daemon — every API body is in the store the moment it lands — so the remaining requirement is the *queue* (a halted or restarted daemon resuming waiting jobs), not the results.
- **Delta/selection for refresh.** The store now knows each tab's `fetched_at` and the last listing; with the real API's `metadata.items` counts on substash stubs, a refresh could skip tabs that cannot have changed. Undesigned; the one remaining reason a client would want its own snapshot.
- **User state on items** (buyouts, notes, ignore flags): the store has the key (`items.id`) but no table yet; needs the first frontend that writes.
- Priority levels: how many, and named or numeric? (Interactive GUI > CLI > background/MCP is the intuition.)
- MCP server: in-process with the daemon, or a fourth thin client?

## Explicitly deferred (do not build yet)

- Job persistence across daemon restarts (SQLite-backed queue) (open topic above).
- Multi-account build steps (design and order decided 2026-08-30, "Multi-account design"); starts on the owner's go, after the `LIVE-TESTING.md` rewrite lands.
- Queue-management UI (drag-to-reorder, per-job progress bars). v1.0 only guarantees the architecture makes this a rendering problem.
- Agent/MCP traffic against GGG — blocked on verifying GGG's policy stance on agent traffic before the MCP path ships. (Owner-driven live baseline testing of the daemon against GGG is not deferred; it has its own control document.)

## Working style

- This branch is the **reference implementation**. Its purpose is to find out what the daemon and rate limiter need to be and to pin that as tests and recorded decisions; the code is replaceable given a reason (a bug, performance, maintainability, understandability) no matter how complete it gets, and a fully operational CLI is still evidence, not a promotion. It may become the real implementation, or a fresh build may replace it — judged by the same tests and the same live ladder (ADR 0003 stays open; both paths share this goal function). The limiter's behavior is fully specified (`ratelimit.rs` test tables); the daemon's GGG-side boundary is proven by the closed live ladder (2026-08-27, `LIVE-TESTING.md`); the frontend boundary is where mapping continues.
- Tests pin behavior at boundaries, never mechanisms. A test that reaches into daemon internals pins this implementation, not the contract, and is disposable. The GGG-side contract surface is the send journal (`TESTING-NOTES.md`); the frontend-side surface is the protocol, not yet pinned.
- Decisions get recorded here after the code teaches us, not before. When the current internals get in the way of learning, record the finding and move on rather than polishing.
- Design discussion precedes code on `spikes/rust-playground` — "design" means updating this doc, not writing a spec.
- Owner (Tom) holds the boundaries: invariants, protocol, core API surface. Agents own internals behind those boundaries.
- Prefer simplicity over flexibility when trade-offs arise. Prefer idiomatic Rust patterns over translations from Qt/C++.
