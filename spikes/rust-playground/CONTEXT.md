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
- **The job queue persists: a `jobs` table in a per-daemon `daemon.db` (SQLite, in the provider's store directory beside the account files), written through at every state change and read back at start.** Memory stays the runtime source of truth — the table is a mirror, "the HashMap, but it survives" — and the one thing read from it while the daemon runs is `result` for an id this lifetime never held (history, not state). On restore only open jobs are loaded (terminal rows carry bodies; a week of them does not belong in memory): `waiting` jobs resume; a `running` job is **re-queued** where the replay premise holds — every network kind is an idempotent GET and the restart probe reads GGG's current counters before it sends, so the duplicate costs one seen hit. Two exceptions (2026-08-30 review): on a declared **no-probe route** the premise fails — a replay would go out against an empty limiter — so the job fails as interrupted instead; and a running **parent whose children exist is mid-fan-out** (its held result was not yet written) — re-running it would submit a duplicate child set, so it holds for the children it has and then finishes as **interrupted, never success**: how many children were never submitted is unknowable, so a partial fan-out must not claim completeness (the children that did run recorded their responses; resubmitting completes the set). A parent whose held result was written resumes holding it; `probe` rows are dropped (one HEAD per lifetime, N16); ids continue from where they were (`AUTOINCREMENT`, never reused, so a stale `acq result <id>` can never name a different job) — a daemon that cannot open or read `daemon.db` refuses to start rather than risk reissuing them, and a queue **write** failure at runtime is sticky: a submit whose insert fails is refused with its id rolled back (a job exists only once its row does), later submits are refused outright, and the dispatcher stops picking while running jobs finish — **ids** never run ahead of disk. Completions are the accepted residual, stated plainly: a job already running when the flag trips finishes in memory but its outcome write fails, so disk still says running and the next daemon replays it (probed route: one seen duplicate hit; no-probe: fails as interrupted) — the send already happened, so refusing to finish it would record nothing at all. The same teeth apply per transition: a `waiting→running` write that fails reverts the job instead of running it (a send the queue cannot see must not happen), `cancel`/`set-priority` report a failed write instead of claiming success (the cancel still wins the job's terminal surface in this lifetime, though an already-running job may still complete its in-flight send — sends are committed once dispatched), and a `result` read failure is an error, never "no job". `submit_child` is refused once its parent is terminal or asked to cancel, under the lock `cancel` takes, so cancellation cannot race an active fan-out into submitting unseen children; a stopped fan-out finishes cancelled or failed, never as success over a partial set; a cancellation that lands after the last child is honored when the held result is installed, cancelled children never count toward a parent's success, and `finish` arbitrates a pending `cancel_requested` under the final lock — a cancel can land at any instant before terminalization and still win. Terminal rows stay so `acq result` has a memory across restarts (`acq jobs` lists live jobs only), pruned at start by age — `ACQ_JOB_RETENTION_DAYS` (default 7) for done/cancelled and `ACQ_FAILED_JOB_RETENTION_DAYS` (default 30) for failed, misread values logged as `CONFIG` errors like the rails knobs. Outcomes are stored verbatim, bodies included (a full refresh is ~50 MB, bounded by retention); compression was considered and deferred — it costs a crate and makes the column unreadable in `sqlite3`, and compressing one column later is a local change. **One daemon per store directory is an invariant, not a lock**: parallel daemons are for the mock and already require `ACQ_STORE_DIR=<scratch>` next to `ACQ_SOCKET` (`AGENTS.md`); two daemons on one `daemon.db` would each restore and run the same queue. Rationale: the queue was the one thing a restart lost once results moved to the store (2026-08-29); a mirror written under the same lock as the memory change keeps disk equal to memory at the `process::exit` the daemon leaves by (up to the declared write-failure residual above); SQLite because it is the crate's one persistence idiom, debuggable with `sqlite3`, and readable by a frontend without a daemon. Decided 2026-08-30.
- **A job's `params` travel on `JobInfo`, verbatim and public.** Every connected client sees them, so a job's params must never carry a secret (tokens are obtained inside the daemon, never passed in). Rationale: a job a person cannot identify (`stash`, `stash`, `stash`) cannot be managed — queue management, when it comes, needs this row; and a client labelling a failed child otherwise has to zip parent payload arrays by position. Rendering (`acq jobs`' `target` column, `JobInfo::target`) is the client's business. Decided 2026-08-24.
- **Daemon protocol includes a subscribe/event channel from the start** (job-state-changed events over the same socket). Rationale: GUI push updates and `jobs --watch` come nearly free; bolting on later is painful.
- **Protocol transport: JSON lines over Unix socket / named pipe** (tokio + serde). Rationale: boring, debuggable, cross-platform.
- **Version handshake in the protocol; the protocol is single-version on purpose.** Kill-and-respawn is the entire migration mechanism — no deprecation, no compat matrix. The stamp compared is the **build** (`VERSION_WITH_BUILD`: package version + git commit), not the package version: the latter is fixed at `0.0.1` across the playground, and comparing it let a daemon from an older commit serve a newer client silently (review finding 2026-09-02: a pre-realm daemon accepted a console job and rendered the pc URL). A `-dirty` stamp is the same for any dirty tree — the standing rule "never rebuild under a live daemon" covers it. Replacing is the *interactive CLI's* policy only (`ConnectOptions::interactive` — the caller is the human expressing intent); an autonomous client (MCP) never kills or replaces a daemon: the mismatch it sees may be a human's live GGG run, so it reports and stops (`ConnectOptions::autonomous`, `client.rs`). Known caveat, accepted: two frontends built from different commits would thrash by respawning each other's daemons — theoretical in a one-workspace playground, recorded so it isn't relearned live. Rationale: CLI and running daemon may be from different builds; three frontends with a compat matrix is the reconciliation swamp, three frontends with respawn is a one-line diff. Amended 2026-08-30 (autonomous policy).
- **CLI emits structured output** (`--json` on every command, the error path included: failures are `{"error": …}` on stdout with exit 1, and a failed job's outcome exits 1 in both output modes). Rationale: the CLI is itself an API; makes MCP and agent use nearly free. Trued up 2026-08-30 (four commands ignored the flag; `--json` mode exited 0 on a failed job).
- **A frontend consumes exactly two surfaces: the daemon protocol and the store crate's read API — no third door.** This pins the boundary's *location*, not its content: the verbs stay revisable until a consumer validates them (TESTING-NOTES.md, "pin after the consumer"). A frontend that wants a third channel is a protocol or store change, recorded here first. Rationale: bespoke per-frontend channels are what turns three frontends into a review burden; two shared surfaces keep the contract in exactly two places, enforced by what frontends link against rather than by process. Decided 2026-08-30.
- **The MCP server is a fourth thin client (`acquisition-mcp`, binary `acq-mcp`, official `rmcp` SDK over stdio), never in-process with the daemon.** Same reasoning that moved reads to the store: daemon-hosted queries make the daemon an application server. The binary embeds `daemon run` like `acq` (lazy spawn execs `current_exe`). Two structural rules in the rail-6 mold: it never kills or replaces a daemon (autonomous connect policy above), and, while the agent-traffic deferral stood (2026-08-30 → 2026-09-01), it refused `submit_job` in real-GGG mode — store reads and observing a live daemon were always allowed, they send nothing. In real mode it still never spawns a daemon (a human's act: keychain, browser); it talks to the one that is running. It lazy-spawns only in mock mode; login stays human, via the CLI. The tracer is the consumer that validates the protocol: when it has proven the shape, the protocol gets pinned — the GUI arrives to a pinned boundary and proposes changes against it, rather than reopening the question. Decided 2026-08-30.
- **Agent traffic against GGG is allowed; the daemon is the single gate.** Owner ruling 2026-09-01 on outside information: GGG permits agent use of the API as long as the API rules are respected. A CLI is already agent-drivable, and so increasingly is a desktop app, so the distinction between human, script and agent clients was never enforceable — what is enforceable is one gate that every client's traffic passes through, and that is the daemon (invariant 1). Consequences: the agent-traffic deferral is lifted; `acq-mcp` submits, applies and quotes in either mode against a running daemon; `quote` over MCP in real mode is simply allowed (it sends nothing). What stays: the MCP never spawns or replaces a daemon in real mode (login and the keychain are human), the live-test rails stay what `LIVE-TESTING.md` says, and every client — human or agent — is paced, journaled and halted by the same code. Decided 2026-09-01.
- **Tokio + reqwest for async/HTTP.** Rationale: ecosystem default; core exposes `async fn`, frontends provide the runtime.
- **Tauri for GUI** (webview frontend). Rationale: item search/grid/filter UIs are a strength of web tech; egui considered and passed on for data-heavy views.
- **Rate limiter: custom policy layer parsing GGG headers, with a simple enforcement mechanism underneath.** Rationale: GGG's header-driven limits are too specific for off-the-shelf policy.
- **Limiter state is keyed by policy name — per account for `Account`-scoped rules since 2026-08-30 (keying decision below) — and learned only from headers.** No local token counting; waits are computed from the last `X-Rate-Limit-*` state plus response arrival times, padded by the server's timing bucket. For the observed paired API-policy windows, classification is positional (first window initial/5s, later windows sustained/60s — ground-truth Q4 hypothesis). That hypothesis does not classify N33's single-window `token-request-limit`; use the conservative 60s bucket for it until GGG confirms its hidden resolution through N14's support channel. Rationale: same-name policies share counters across endpoints (N6), so endpoint-keyed state would be a migration later; positional classification is conservative on the observed paired API shapes without pretending the new one-window shape is known.
- **A counted send whose response carries no usable policy headers is paced as if the server counted it — the same treatment as a send lost in transport.** One predicted hit in every window as of that response; the definition and the last real observation are untouched; the next response with headers replaces the prediction (invariant 2). The job still fails and is never retried. Rationale: rung 10's origin 503 (2026-08-24, N35) was an HTML page with no `X-Rate-Limit-*`; before this, such a response left pacing reading a state that predated it, so a run of 503s went out unpaced. This bounds an outage at the policy's own rate (30 per 300 s on stash) with no new policy; backing off harder than that waits for evidence. Decided 2026-08-24.
- **Endpoint discovery is a visible `probe` job: one HEAD per endpoint per daemon lifetime, queued by the daemon before the first real send.** Rationale: N16's sanctioned pattern is one HEAD at startup; doing it lazily per endpoint sends the same count and nothing for endpoints never used. Visible rather than internal because everything that touches GGG is a job, and HEAD has regressed server-side before (N20) — the probe's headers need to be inspectable.
- **The global burst bound belongs at the daemon's HTTP send boundary and spans each actual request from immediately before dispatch until its response/body completes.** API requests use policy serialization plus ordinary permits from this common gate; HEAD probes take its exclusive, writer-preferred permit. OAuth code exchange and refresh use ordinary permits and serialize under stable route key `oauth-token` before discovery, then learned policy name `token-request-limit`; no HEAD probes the token endpoint. Authentication completes before an API request performs its final limiter check and acquires its send permit, so neither auth nor rate-limit waiting occupies an API permit. Browser authorize remains outside because the browser owns it. The dispatcher may keep one active job task per scheduling key to preserve priority and FIFO, but it has no global job-task cap; auth and pacing waits therefore cannot block progress on independent keys. Rationale: P-B and N33 — Cloudflare watches bursts across policies and the token endpoint is itself Cloudflare-fronted and IP-limited; HTTP capacity must describe actual sends, not waiting work.
- **The frozen C++ network design (`docs/design/network-redesign.md`) is a property source, not a Rust architecture template.** The Rust code conforms to its D3/D4/D5/D8 properties with three intentional adaptations: OAuth token traffic is inside the common gate (N33 postdates the C++ design); ordinary FIFO is among currently eligible policies, so a same-policy waiter cannot idle an independent global slot; and the C++ gate's 250 ms inter-send spacing is not adopted. Rationale: the properties protect the GGG relationship; the mechanisms were shaped by Qt. Changing an adaptation is a design change, recorded here first.
- **Work that needs many requests is a parent job that submits child jobs; a parent finishes when its last descendant does, gives up its dispatcher task and scheduling key while waiting, and cancels its descendants when cancelled.** Rationale: the queue, dispatcher, priorities, ETAs, and events already work per job, so children get all of it for free; a job-internal loop would need its own scheduler and hide the requests from every tool. Observed API shapes (2026-08-20): folder children are in the stash list (a folder holds tabs only — never items, never another folder; confirmed against GGG patch notes 2026-08-24); map/unique substashes only appear on fetching the tab (one map tab listed 234); substash stubs carry `metadata.items` counts. Following substashes is opt-in per tab.
- **A `refresh_token` grant the provider rejects with a 4xx other than 429 is terminal: no further refresh is sent until `acq auth` or logout.** Rationale: R1/L0-R5 (`LIVE-TESTING.md` history, `9fa99459`) — a dead grant re-sent per flight is pointless traffic on a Cloudflare-fronted endpoint, and the rail that stopped it (rail 2) was opt-in, so the shipped default still did it (`TESTING-NOTES.md`, "rails-conditional fixes"). Decided and built 2026-08-24; the mark persists in the rails state file and is honored regardless of the tripwire.
- **A rails halt leaves queued network jobs waiting; nothing fails for lack of a send.** The dispatcher does not pick a network job while halted, a job that finds the halt after being picked gives its key back, and `reset-tripwire` wakes the queue. A halted daemon with only waiting jobs counts as idle and exits — the queue is on disk, and its successor (started with the tripwire, which honors the persisted trip) holds it until the reset. Rationale: rung 10 (2026-08-24) failed 82 never-sent children on a 503 and the rerun refetched all 322 tabs; the two reasons for failing them — results died with the daemon, and a daemon with waiting jobs never idled out — are both gone with persistence. Caveat for `LIVE-TESTING.md`: the ceiling is per lifetime and not persisted, so a queue halted by `ACQ_MAX_SENDS` resumes under the next daemon's fresh ceiling — `acq jobs` and `acq cancel` before respawning. Decided 2026-08-30.
- **A 429 re-queues the job (keeping its place) behind the limiter's hold; after `MAX_429_RETRIES` (2) it fails with the evidence. 403/503 are never retried.** Rationale: P-A — violations are structural, so recovery is a requirement; N10 — frequent violations revoke the app, so it's bounded; invariant 3 for the Cloudflare shapes. No new job state: `running → waiting` with a retry counter on the job.
- **A client that disappears leaves its jobs running.** Ctrl-C, a closed terminal, or a crash cancels nothing; the sends are committed either way, and a hold can last minutes. A client that wants the results reattaches by job id (`acq result <id>`), which the persisted queue answers across daemon restarts. Decided 2026-08-24; results outlive the daemon since 2026-08-30.
- **Persistence is a shared library + file, not a process: `acquisition-store` (SQLite, one facts file per account under a per-provider directory, plus the account index); the daemon writes facts, and frontends read facts and read/write intent, all through the store crate.** The daemon's fact-side involvement is `record(endpoint, params, status, body)` after each API success; it never reads the store and never looks inside a body. Amended 2026-08-31 (intent write path — the four-layer decision below). Search and the item model live in the store crate as plain functions, so the CLI, GUI, and an agent on the CLI call the same code and see the same data. Rationale (2026-08-29): frontend-owned stores duplicate GGG traffic when two frontends pull (the one real rule); daemon-served queries make the daemon an application server; a shared file gives one fetch for all consumers and keeps net/store/frontend separable for testing.
- **Bodies are stored verbatim except at the item seams; `items` is the only place to look for an item.** Every item array (tab `items`, character `inventory`/`equipment`/`jewels`/`rucksack`/`guardian`/`skills`, each `socketedItems`) is lifted into `items`, one row per GGG item id (stable across moves), keyed by its location's **full coordinate** — realm, league for a stash, kind, id (2026-09-02: the same tab id under two realms is two locations, and events carry the whole address); the envelope keeps the counts under `_split`, so envelope + rows is the response exactly — with one ruled exception: a fetch of a location a listing has retired is **withheld** (2026-09-02): its whole body stays verbatim on the response row (`responses.withheld`, the daemon log, `store status`) and nothing else lands, because membership is the listing's — only a listing revives a location, and it clears the row's `fetched_at` doing so, so the next plan fetches again. Derived columns come from the row's own JSON (`rebuild` re-extracts; never a refetch). Ingest compares with the previous state and records `item_events` — this replaces `pull`'s snapshot diff. Rationale: raw-plus-parsed duplicated every body (a league spans 1000× in size); raw-only made every query a body scan and gave user state (buyouts, notes) no key. Decided 2026-08-29; the real-snapshot replay (322 tabs, 19,210 rows, 2.3 s, zero false changes 8 h apart) is the evidence.
- **Multi-account is one daemon holding many sessions, never one daemon per account.** The Cloudflare bound (`SendGate`, 2 live sends) is a per-IP property (P-B, ground truth §1) held as per-process state; two daemons on one machine make it a 4-wide burst that neither sees, with separate tripwires. Rung 11 (2026-08-30) showed the other half: `Account` rules count per account on GGG's side, so two accounts never contend on layer 2 — the only thing they share is layer 1 and the `Ip`-scoped token endpoint, which is exactly what the single gate exists for. Built in two halves with different blast radii (option C): **account as first-class identity now** (store path, job field, keyring key — leaves), **many live sessions later** (a refactor confined to the session layer). Limiter and probe scope keying — `(account, policy)` for `Account` rules, policy alone for `Ip` rules, scope learned from `X-Rate-Limit-Rules` — is a **precondition of the session map, not an optimization**: with two live sessions on one policy each response would overwrite shared state with a different account's counters, and the next send from the other account floods (a 429 path; the "over-waits, never floods" reading only held for rung 11's sequential switch). Decided 2026-08-29, amended 2026-08-30 after review across sessions; design below in "Multi-account design"; built 2026-08-30 through step (6) — step (7)'s live samples are in `LIVE-TESTING.md`'s run ledger.
- **Per-route knowledge about GGG that headers cannot teach lives in one place (`Daemon::declare_route_knowledge`), and strict observation is the default everywhere else.** `GET /profile` (first contact 2026-08-30) answers 200 with no `X-Rate-Limit-*` headers at all and 403 to HEAD, which strict observation ("every endpoint has a policy", post-N33) classed as a protocol failure and discarded. Now: a route *declared* policyless accepts a 2xx with **no** rate-limit header (a partial set is still a failure; a policy that later appears is learned strictly), becomes `EndpointState::Policyless`, and is paced by nothing but the send gate; a declared no-probe route goes straight to its GET. Only `/profile` is declared, and it is called at most once per login. Not generalised on purpose: "any headerless 2xx is fine" reopens the blind spot strict observation closed. Owner decision 2026-08-30; GGG confirmed the same day (Q12/N38): `/profile` is not rate limited at present, so the declaration is confirmed and stays until headers ever appear — strict observation covers that arm.
- **Rate limiter spec will be expressed as test tables, not prose.** `docs/design/network-ground-truth.md` (the claims registry; it indexes the deeper spike evidence) is the input; "given these headers, wait N seconds" tests are the permanent, enforced spec.
- **The system is four layers — facts, intent (annotations), derivations, effects — each with one authoritative mutation path, not one physical writer.** Facts mutate only through the store crate's ingest surface (daemon `record`; `store import`); intent only through the store crate's annotation write API (frontends); the effects ledger only through the daemon; derivations have no independent authority — computed or materialized, always reproducible from declared inputs (`rebuild` is their maintenance, not fact ingestion). The daemon is permanently blind to intent, and it creates work only in causal service of client-submitted work (probes, children, retries) — never spontaneously: no schedules, no policy execution, no annotation reads; scheduled syncs are small frontends. Rationale: "a sync can never clobber intent" becomes structural, the way the choke point made rate-limit discipline structural — and blindness is safe exactly because the daemon never initiates. Decided 2026-08-31 (brainstorming-notes 06, ruled).
- **Annotations are the only irreplaceable local state.** A separate per-account file named by the account uuid (identity decision in "Multi-account design"), keyed on stable GGG ids, written only through the store crate with integer-revision compare-and-swap; no fact-side event ever deletes intent — an annotation whose item is removed is kept and surfaceable as orphaned; export/backup is a store-managed consistent snapshot (`VACUUM INTO` / SQLite backup API — a raw file copy under WAL is not a backup). Rationale: facts are refetchable at the cost of requests; intent has no server to refetch from — the C++ legacy-buyout saga is the full price of getting this wrong. Decided 2026-08-31.
- **The sync policy is the first annotation: a per-account, inspectable declaration of desired coverage and freshness — not a scheduler — compiled by the frontend-side planner into minimal requests.** `metadata.items` counts are heuristic evidence: they can prove a tab changed, never that it didn't. Rationale: C++ tracked-set/clean-refresh semantics, the old delta/selection topic, and both redesign essays independently describe this one object. Decided 2026-08-31.
- **A tab id in the sync policy covers that tab and its children.** A tab is covered when its own id is listed or its parent's id is; one rule, no per-type logic, and it gives every case the owner meant (tracer rung, 2026-09-01): a map or unique tab's substashes are planned once their stubs are on record — the cycle after the parent's first fetch, since discovery still waits for facts (binding untouched: every action stays an explicit reviewed tuple, nothing is added at apply); a folder's children are planned at once, because the listing already carries them (folders themselves are never fetched); a child named directly still works. A substash stub whose `metadata.items` is 0 is skipped with a named reason rather than fetched — GGG appears to list only non-empty substashes (64 of 64 stubs on the real account carry a count ≥ 1), so this is a guard, not a saving; in-flight change is not designed around (a count is still never proof of freshness). Type-level filters ("skip map tabs", "include unique tabs", "fetch folder children") are parked with a trigger. Evidence: the rung's five-id run left 64 substashes under the two selected parents in the store and outside the policy, closed the loop after one cycle, and the owner's intent for a named map tab was its contents. Consequence: the same policy rerun plans those 64 as cycle 2 (~15 min of limiter holds) — the first live two-cycle discovery sample. Built the same day: `TabSelection::covers` (own id or parent's; `covers_tab` over `Selection` since step (3) of the characters work), `SkipReason::EmptyStub`, plan schema **v4** (a new skip kind is an envelope shape change). Decided 2026-09-01.
- **A Plan is a serializable, immutable authorization envelope, and plans are binding.** Derived from a named snapshot of facts + intent, computable with the daemon down; it carries provider + account uuid, operation kind + plan schema version, fact basis (response/listing ids or timestamps), annotation revision, the explicit action set (or a declared upper bound), generated-at, freshness assumptions, and optionally a quote with its own observation time. Work has two dimensions: `logical_requests` (exact or bounded) and `wire_sends` (a coarse range plus named prerequisites — probe, token refresh, possible 429 retries — never a precise accounting). Applying a Plan executes exactly the listed actions or a strict subset, never an unreviewed addition; new facts produce a new Plan; v1 excludes dynamic `--deep` fan-out (a vanished tab fails or is reported skipped; newly discovered tabs wait for the next plan). Operation-specific types first (`RefreshPlan`); a universal grammar waits for the second plan-bearing consumer. Binding was revisable on tracer evidence (the owner's live-use friction notes are the data). Decided 2026-08-31. **Confirmed 2026-09-01** on the tracer rung's live run (`LIVE-TESTING.md`, run ledger and friction notes): subset-only reconciliation produced no owner friction, the two-cycle discovery of substashes cost nothing observable, and the parking-lot trigger for dynamic fan-out ("two-cycle reconciliation genuinely hurts") did not fire. What the run did surface was a *coverage* question, ruled separately below (a policy id covers the tab and its children); binding itself stands as written.
- **The planner lives in `acquisition-plan`** — depends on core's client/protocol types + the store, linked by frontends only — and owns policy compilation and Plan construction; the store exposes neutral snapshots (policy rows, tab identities, freshness, listing basis, metadata), never half a planner. Rationale: keeps "the daemon never reads the store" enforced by the dependency graph, not discipline. Decided 2026-08-31.
- **`quote` is its own protocol request: a read-only, non-reserving projection over current daemon knowledge** — observation time, basis, per-policy/per-scope estimates, and unknown prerequisites; applying may receive a different schedule (`eta_for` is "an estimate, not a promise"). Headroom is per policy/window and scope, never one scalar. Never a flag on `Submit`, whose contract is loaded with id/persistence/rollback semantics. Decided 2026-08-31.
- **Reads observe, assertions plan, apply spends.** Store reads never initiate network traffic, and stale facts stay readable with freshness/completeness metadata; only a caller-asserted freshness condition fails — a stable structured error carrying the exact `RefreshPlan` it would take — and explicit frontend orchestration (refresh → await → read) is workflow, not a fused read. Plans-as-remedies are a store-side idiom only: the daemon cannot compute plans and its errors keep their shapes. Decided 2026-08-31.
- **v1 request budget is logical work, enforced at admission: the daemon refuses a plan before any child submission if its logical bound exceeds `max_requests`.** Mid-fan-out terminalization is never the normal path. An actual-wire-send budget (a causal operation id through probes, OAuth, retries) is a separate, deferred feature. The live-test rails are not promoted wholesale: the tripwire and lifetime ceiling stay what `LIVE-TESTING.md` says they are; product budget *visibility* is the quote + the journal. Decided 2026-08-31.
- **Apply is its own pure fan-out parent job kind (`apply`), never the `refresh` parent.** The refresh parent re-lists by construction — it fans out from the listing it just fetched — which contradicts "executes exactly the listed actions": a plan's listing is an optional action and its fetches derive from reviewed facts. So `apply`'s params carry the plan's actions as explicit `(kind, params)` child tuples; the parent performs no send of its own, submits exactly one child per tuple, and holds for them. "Never expands" is structural, not disciplinary: the daemon stays plan-blind (it cannot link the store or plan crates), so what it admits is **vocabulary, not meaning** — only single-request kinds (`stashes`, or `stash` with `deep` false), each of which submits no children. Admission is at submit, before a job id exists: a malformed tuple list, an empty one, or a logical bound over the caller's `max_requests` refuses the submit whole — the mid-fan-out terminalization path is never the budget's normal mechanism (D8). Plan validation, the staleness check, and rendering actions to tuples are the frontend's (`acq refresh --apply`, through the planner's validating parse). The ad-hoc `refresh` kind (`--all`/`--tabs`) stays untouched as the explicit client-stated-selection surface; whether it retires rides on step 9's friction notes. Decided 2026-09-01.
- **Step 7's staleness ruling: apply refuses a plan whose sync-policy revision is no longer the stored one.** A plan is authorization *derived from intent at a revision*; intent edited since revokes the derivation — the CAS reasoning extended from writing intent to spending it, and the refusal is cheap because replanning is offline and free. Checked frontend-side against a fresh read of the policy row immediately before submit (the daemon is intent-blind by the four-layer decision, so only a frontend can compare); a missing row, a different revision, a mismatched account uuid, or a mismatched provider each refuse with the remedy named. Two accepted residuals: the check races a concurrent policy write between read and submit (the same human-boundary register as `policy set` without `--if-revision`), and **fact drift does not refuse** — the authorization is the bounded action set, not a world-state assertion; the actions stay idempotent GETs of exactly the reviewed tuples (a since-vanished tab's fetch fails its own child honestly) and the next plan reconciles what a newer listing changed. Decided 2026-09-01.
- **The effects ledger is frontend-readable through a read-only facade in the store crate** — not the open `JobDb`; write methods stay out of the frontend surface. An offline orientation distinguishes "daemon offline, zero sends in flight" from persisted waiting or recorded-running work; "daemon offline" never collapses into "no outstanding work". Decided 2026-08-31.
- **Shared semantics live in Rust; every frontend has a Rust adapter** (clap CLI, `rmcp` MCP, Tauri backend — the webview is presentation, never a second implementation — `dash` TUI). A proposed non-Rust frontend is a design event, recorded here first. Rationale: this premise is what makes "built once, inherited by every frontend" true; unstated premises erode silently. Decided 2026-08-31.
- **Panics are for broken internal invariants only; malformed external input — a GGG body, a store row, a protocol message — is a structured error with stable kinds and context.** The store crate enforces this mechanically (`clippy::unwrap_used`/`expect_used`; its production code is at zero); not workspace-wide — the daemon's `.lock().unwrap()` poisoning idiom and checked-invariant `.expect`s are the correct register. Rationale: the persisted queue makes crashes recoverable, which turns a *reproducible* panic on bad input into a crash loop — the one failure persistence cannot absorb. Decided 2026-08-31.
- **The SQLite schema is internal; raw SQL is not a surface.** Schema versions and compatibility errors; defended by making the store crate's API expressive enough that going around it is never worth it. No cached search service (stale results mistaken for current truth); reopening needs a measured duplication or latency case that in-process reads over the file cannot meet. Decided 2026-08-31.

## Interfaces (boundaries are specified; internals are not)

None of these interfaces are locked down more than anything else in this document.

### Daemon job protocol

The live definition is `crates/acquisition-core/src/protocol.rs` (request/response/event enums, job states). Its boundary properties are the decision lines above; the verb list is internals.

ETA is computed from limiter state + queue depth ahead of the job — the daemon can predict, because it sees everything.

### CLI shape

The live verb list is `acq --help` and the README's "Try it" block. Properties: default mode is blocking-with-progress ("rate limited, starting in ~4m37s..."), `--detach` is the async/job mode, every command takes `--json`, and `daemon status|stop` exist for debugging only.

### Multi-account design (decided 2026-08-30, built through step 6; step 7's live samples partial)

**Complexity rule:** the only code that interprets accounts is the
session layer. Everywhere else account is data — a field on the job, a
path segment for the store, an opaque key component for the limiter
(which never reads it; scope comes from `X-Rate-Limit-Rules`). An
`if account == …` outside the session layer is the smell.

- **Identity: the stable account key is the profile `uuid`, fetched at
  login and required** (amended 2026-08-31; was username-only with
  opportunistic uuid). After token exchange the daemon submits a profile
  job — causal service of the client's `acq auth` — and the session is
  registered, the keyring written, and `accounts.json` updated only when
  the uuid lands; a login whose profile fetch fails **fails whole**: no
  provisional identity, no minted keys, no rename-repair machinery — if
  `/profile` is broken, something is broken and login says so. A retry
  repeats the token exchange, paced by the `Ip`-scoped token policy, so
  a retry loop is already bounded. `accounts.json` maps
  username/discriminator/provider → uuid; a rename is a mapping update
  with intent untouched. The token response's `username`
  (`name#discriminator`) stays the display name and selector; fact files
  stay username-named (refetchable; rename-orphaning tolerable);
  annotation files are uuid-named. Entries without a uuid: one re-auth,
  no migration. The mock serves deterministic per-username uuids.
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
  entry; fixed at submit (resolved against the live session, refused
  before a job exists otherwise), checked again at the moment a token is
  taken (a mismatch fails the job with no send), and it selects the store
  file — never the session at landing time. Shown in `jobs`, `dash`, the
  daemon log, and the journal (the `route` field is the endpoint key,
  `stash@Alice#1234`).
- **Limiter keying as built (step 5):** the endpoint key is
  `route@account`; a policy's state is keyed `name@account` only when
  *every* rule of the policy is `Account`-scoped and the send had an
  account — `Ip` rules, mixed scopes, and accountless sends share the
  bare name (over-waits at worst). One notch more conservative than
  "Account rules per account" if GGG ever mixes scopes in one policy.
  The token route (`oauth-token`) is deliberately accountless: it is
  `Ip`-scoped and has no probe, so an accounted key would be unpaced on
  an account's first login.
- **The free HEAD probe (N24) is per endpoint, not an API property.**
  First contact 2026-08-30: `HEAD /account/leagues` is answered 200 and
  counted as a hit (the free HEADs answer 204); `HEAD /profile` is 403.
  Both routes are declared no-probe in `Daemon::declare_route_knowledge`
  and taught by their first GET; pacing was never wrong (headers are
  post-increment and trusted), a probe there is just a wasted hit.
- Store: `store/<provider>/<account>.db`, opened lazily on first record;
  `tabs`/`items`/`store` take the selector and never span accounts.
  Keyring: one entry per account; the index file is how the daemon knows
  which entries to restore (the keyring crate cannot enumerate). Restore
  continues past a dead grant — the terminal-grant mark is per session.
  The existing single keyring entry is orphaned, not migrated: one
  re-auth.
- The `jobs` table lives in a per-daemon `daemon.db`, not inside an
  account file, and carries the account column (persistence decision above).
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
`/character/{name}`, `/account/leagues` under `LIVE-TESTING.md`'s replacement
rule, uuid recorded opportunistically (superseded 2026-08-31: uuid is now
required at login — identity bullet above).

### Annotations & plans — the refresh tracer (decided 2026-08-31; steps 1–9 done — step 9 run live 2026-09-01, pass; follow-up: the policy-handle planner change)

The one slice built next: refresh-with-`plan`, the smallest slice that
touches all four layers (policy = intent, plan = derivation, apply =
effect, the next read = facts). Full deliberation is history in
`brainstorming-notes/` 00–06; the binding text is the 2026-08-31
decision lines above. The built steps' properties are pinned in the
named code and its tests — the review-round narratives that used to
live here are in git history (pruned 2026-09-01, the section at
`35fb35d9` holds the full text).

Built, each step gate-green and owner-reviewed:

1. Semantics — the 2026-08-31 decision lines above.
2. uuid-at-login + the annotation file (2026-08-31,
   `acquisition-store/src/annotations.rs` and the daemon's staged login
   flow — a login fails whole, and only its own flow-terminal result on
   the protocol counts as success).
3. Neutral store snapshots (2026-08-31, `Store::refresh_snapshot` — then `stash_snapshot` — in
   `acquisition-store/src/snapshot.rs`) — with the malformed-2xx
   ruling: a malformed body is a typed refusal that writes nothing and
   fails the *job*; only `acq store import` keeps the legacy tolerance,
   at its own boundary.
4. `RefreshPlan` compiled offline in `acquisition-plan` (2026-09-01).
   `REFRESH_PLAN_SCHEMA` was **3** here (5 since the realm step, 6 with
   characters); any shape change anywhere in the envelope — the
   embedded `Quote` included — is a schema bump, so an older reader
   reports "newer schema", never "malformed".
5. `quote` on the protocol + `with_quote` plan enrichment (2026-09-01;
   `Daemon::quote`, protocol `Quote`/`QuoteScope`).
6. `acq refresh --plan` + the policy's first write surface, `acq policy
   show|set` (2026-09-01; `acquisition-cli/src/plan_cmd.rs`; the
   `--json` stdout contract is pinned at the process level in
   `acquisition-cli/tests/plan_json.rs` — step 7's apply consumes that
   envelope).
7. Apply (2026-09-01; the two `apply` decision lines above — the
   fan-out parent and the staleness ruling). `acq refresh
   --apply[=FILE]` runs the offline gates (planner's validating parse,
   identity, policy revision), then submits the actions as one `apply`
   parent the daemon admits or refuses whole at submit (vocabulary +
   `max_requests`); an empty plan is a no-op with no daemon contact.
   The loop is pinned at process level: `acquisition-cli/tests/
   apply_loop.rs` closes plan→apply→replan against a real daemon over
   the mock in a bootstrap listing plus two reconciliation cycles, and
   `tests/plan_json.rs`
   pins the offline gates (stale revision refused before any daemon;
   tampered envelope refused by the parse; empty plan spends nothing).
8. MCP exposure (2026-09-01): `acq-mcp` carries the slice as tools —
   `sync_policy` / `set_sync_policy` (intent), `refresh_plan`
   (derivation, quote-enriched by a running mock daemon), `apply_plan`
   (effect, returning the parent id for the MCP's submit-then-poll
   idiom). The second consumer's arrival triggered the factoring rule:
   the shared semantics moved to `acquisition-plan` —
   `put_sync_policy` (validate-then-CAS), `RefreshPlan::check_spendable`
   (the step-7 staleness/identity gate), `RefreshPlan::apply_params`
   (action→tuple rendering) — and the store names the policy's
   annotation address once (`SYNC_POLICY_SCOPE`/`KEY`/`KIND`); the CLI
   now goes through the same functions. Two agent-boundary defaults,
   owner-revisable: `set_sync_policy` works in either mode (intent is
   local — the deferral is about traffic) but has no blind-replace
   form — replacing an existing policy must name the revision it
   replaces, so an agent never clobbers intent it has not read; and
   — until the 2026-09-01 agent-traffic ruling — ggg-mode quote
   enrichment attempted no connection and `apply_plan` was refused in
   ggg mode alongside `submit_job`; both now work in either mode against
   a running daemon, and what `tests/ggg_refusal.rs` pins is the rule
   that remains: real mode never spawns a daemon. Pinned at process level (review
   round 1 closed two coverage gaps): `acquisition-mcp/tests/
   plan_loop.rs` — login over the protocol (the daemon rides inside
   `acq-mcp`), the tools carrying policy→plan→apply→replan against a
   real daemon over the mock, the create-only CAS and admission-budget
   gates, and the offline claims proven offline (the daemon is stopped
   before the staleness refusal and the empty-plan no-op, and the
   socket checked dead afterwards, so a regression that contacted or
   lazy-spawned a daemon cannot pass) — and `tests/ggg_refusal.rs`: an
   `ACQ_GGG=1` server with no daemon, store, or login refuses
   `apply_plan` and `submit_job` on the mode alone, before even the
   envelope parse.

Accepted residuals, recorded so they need no re-litigating:

- No process-level mismatched-daemon test; the structural
  connect-options pin covers lifecycle safety (the quote path never
  spawns or replaces a daemon).
- Real GGG's `/profile` `name` may lack the `#discriminator` the
  session username carries; if so, `--plan`'s quote enrichment surfaces
  it at step (9) as a graceful "daemon quote rejected" note — an
  observation to collect on the live rung, not a bug to pre-fix.

Prepared, not yet run (the run is the owner's, from a terminal):

9. Owner live rung under `LIVE-TESTING.md`'s standing rule, friction
   notes collected as data. Prepared 2026-09-01: the rung section in
   `LIVE-TESTING.md` ("Tracer rung") — the step table, expected totals,
   residuals, and the friction-note prompts — and `tools/tracer-rung.sh`,
   which drives it (one fresh daemon per wire phase under a ceiling
   derived from the plan's own wire estimate; the offline claims checked
   with the socket dead; the journal verified per cycle against the plan;
   friction notes prompted at each phase). Review round 1 (2026-09-01,
   six findings, all fixed before any live run): the ceiling is exact
   and the rails trip on `sends >= max`, so the bound reached right
   after the last planned send is a cycle's expected end, not a failure;
   the envelope applied is the quoted file, checked equal to the offline
   one plus the quote and shown before apply; every probe's hits are
   bounded by the run's own earlier sends (the first probe on a route
   must see 0); readback failures fail the run. One planner fact the
   review surfaced, recorded for the owner rather than pre-fixed: policy
   ids match exactly, so an id list never covers the substashes a
   map/unique fetch discovers — the loop closes with them uncovered
   (reported), and only `all` runs the discovery cycle; whether a parent
   should cover its substashes is a friction question. `--mock`
   rehearsal green in both shapes (`all`: listing, seven fetches, seven
   substashes, empty; ids: listing, three fetches, empty with four
   uncovered substashes reported). Round 2 (same day, five findings,
   fixed): the same-plan check ignores the derived `age_seconds` that
   drift between compiles (reason kinds still compared); probe hits are
   bounded per reported window by the run's own sends inside that
   window plus GGG's timing bucket, not by a cumulative total; the
   no-hold bound is `n ≤ 15` (the 10 s window), not 30; the ceiling can
   be overshot by one in-flight send (rail 1's caveat) and a
   `ceiling + 1` journal fails the cycle; the selector is
   case-insensitive like `account_matches`. Round 3 (same day, three
   findings, fixed): `all` defaults its freshness window to a day and
   the driver refuses a cycle whose over-estimated wire duration is not
   at most half the window (at 3600 s the hour-long cycle would
   re-stale its own facts and never close); the
   timing bucket goes by window position (5 s first, 60 s later —
   `bucket_for`), not by period; the selector lowercases in Unicode; the
   verifier is its own file, `tools/tracer-verify.py`, whose
   `--self-test` pins the nonzero-hit branches the mock cannot reach
   (checked in, green). Round 4 (same day, six findings, fixed): every
   wire phase must end with the daemon reporting the tripwire armed, the
   ceiling equal to the plan, that many sends counted, and a ceiling
   halt in force — a matching journal count alone is not evidence of
   the rail; the verifier accepts only 2xx, keys probe-first ordering by
   the account-qualified route, fails a probe with no state window or
   an active restriction (self-test mutations added); the evidence
   bundle holds only this run's journal slice plus a `verify.sh` that
   reproduces the verdict from byte 0, and a repeat run gets its own
   directory; the store-events readback covers the run's whole span;
   live preflight refuses working-tree changes to the rung's own files.
   Round 5 (same day, three evidence gaps, fixed): the daemon log is
   saved as this run's slice like the journal; the bundle carries its
   own copy of the verifier with checksums and `verify.sh` runs that
   copy, never the working tree's; the item-event readback sets an
   explicit limit and fails on reaching it. The run's outputs: a ledger
   row, the friction notes in the rung section, and then the two
   rulings below — "Binding-plan friction" and the method-test verdict.

Done = **pin the refresh Plan/quote/apply slice and the annotation API
it exercised** — a CLI tracer cannot close the whole GUI/MCP/TUI
frontier. The slice also tests the method itself: whether
pin-after-the-consumer survives product scope, where truth is the
owner's real use rather than a header; the verdict is recorded before
the pricing session and is its first input.

**Step 9 ran live 2026-09-01 and passed** (`LIVE-TESTING.md`, run
ledger: login `1/0/1`, cycle 1 `1/2/6` on an exact ceiling of 9, cycle
2 empty, no-op apply with no daemon; both probes 0 hits, zero non-2xx;
the plan quoted on the real account — the `/profile` discriminator
residual did not bite, so that accepted wart is closed). Rulings from
the run, both in Decisions: binding confirmed as written; a policy id
covers the tab and its children (the planner change is the follow-up
below).

**Method-test verdict (2026-09-01):** *pass on correctness, with the
owner-truth channel under-exercised.* The slice ran live first time with
no code change; every rail and check fired as designed, and the five
review rounds before the run did the catching rather than the run. But
the method's truth is the owner's experience, and the owner's one
friction note plus the remark that the driver's output was too dense to
read means the run was judged through the agent's reading, not the
owner's. Not a failure of pin-after-the-consumer — a caveat it carries
into the pricing session: budget for legibility (the parked output items
below) before that session's own live run, or say explicitly that agent
observations stand in for owner notes.

**Handle ruling built 2026-09-01:** `TabSelection::covers(tab)` (since step (3): `covers_tab` over the shared `Selection`) — own id
or parent's id; `SkipReason::EmptyStub` for a map/unique stub counting 0
with nothing held (a held item against a 0 count stays the disagreement
arm and fetches); `REFRESH_PLAN_SCHEMA` 3 → 4. Tests pin the three cases
(a named map tab plans its substashes the cycle after its first fetch,
with the empty stub skipped and the new kind surviving the strict parse;
a named folder plans its children at once; a child named directly is
still covered). The driver's readback now checks that every child of a
selected tab on record is fetched by the time the loop closes, or is an
`empty_stub` skip in the final plan. Mock rehearsal green in both
shapes: ids `cur1,dump,maps` = login, `1/1/1`, `1/1/3`, `1/1/4` (the
four substashes under `maps`, through their parent), empty; `all`
unchanged. The live rerun of the same five-id policy — the first
two-cycle discovery sample (64 substashes, ~15 min of holds) — is the
owner's, under the rung section.

### Characters in the refresh plan — next session (agreed 2026-09-02; design ruled 2026-09-02, before pricing)

Why now: tabs and characters are the only two paths items take into
the store, so this closes the ingest map ("the store holds what the C++
app shows" — evidence for ADR 0003 more than for pricing); the shape is
the same (a listing, one fetch per entity, a freshness window, binding
plan, apply parent, staleness gate, driver); both routes had first
contact 2026-08-30 with their policies recorded (N40 and the run
ledger: list `character-list-request-limit` `2:10:60,5:300:300`, fetch
`character-request-limit` `5:10:60,30:300:300`, both with a free HEAD);
the mock serves both under the real policy shapes; and it is cheap
evidence on the grammar question pricing will ask (a near-sibling is a
weak test, but it comes almost free, and strain here would be worth
knowing first).

**Rulings (owner, 2026-09-02 design session).** Evidence: the run-ledger
row `2026-09-02 characters sample` (`LIVE-TESTING.md`) and the official
API reference, read the same day (documented facts below are pending
ground-truth claims, master-side).

- **Identity is the character `id`; the name is the address.** `id` is
  a unique 64-hex string (documented; observed equal between list entry
  and fetched body). The fetch endpoint takes the name, so a plan action
  carries both: id for identity, coverage and reasons; name for the
  request, taken from the basis listing. Same shape as a substash
  fetched by `(parent, id)` from a cited basis: a name that moved fails
  its child honestly (404) or lands a different id (a recreated name) —
  the store records what the server said, keyed by the **body's** id,
  the intended character stays stale, and the next listing reconciles
  (D5a; fact drift does not refuse — the step-7 ruling). No expected-id
  check on the fetch: a 200 under a stale name is a true fact, and
  refusing it discards facts and wastes a paid send. Why the key must
  move — three failures of name-keying, only the first about renames:
  policy ids break on rename (intent references identity: the uuid
  precedent, first-consumer treatment); a deleted-and-recreated name
  inherits the old row's freshness and is never fetched (a planner
  hole); a rename moves every item (false events). Items locate at the
  character id.
- **Realm is a coordinate above league, everywhere.** PoE2 1.0 ships
  December 2026 (announced late August); league names collide across
  realms (Standard exists in both games). Policy becomes
  `realms.<R>.leagues.<L>.{tabs, characters, max_age_seconds}`
  (the realm nesting is **policy v2**, built; `characters` beside `tabs`
  makes **v3** — a new sibling field is a shape change, so a v2 reader
  says "newer version", never "malformed"; each older version upgrades
  on parse, v1 as realm pc); the
  plan envelope carries realm beside league; the snapshot is taken per
  `(realm, league)`; `tabs` and `characters` facts carry a realm column
  (existing rows pc); realm is an explicit param on all four data kinds
  (`stashes`, `stash`, `characters`, `character`), defaulted to pc only
  at the decode boundary (`Endpoint::from_job`) so persisted pre-realm
  jobs still decode. On the wire realm is a path segment and **pc is
  expressed by omission — `pc` is not a legal segment value**
  (documented), so pc URLs stay byte-identical to every live send so
  far: the realm step costs no live spend, and the mock rehearsal
  journal proves the URLs did not move.
- **Which realms each route accepts is declared in one place, never one
  shared list** (the `declare_route_knowledge` mold). Documented today:
  `/character` and `/character/{name}` take `xbox|sony|poe2`;
  `/stash…` and `/account/leagues` take `xbox|sony` and are titled
  "PoE1 only". A policy naming `tabs` under `poe2` is a structured parse
  error; no code path renders a stash URL with `poe2`. GGG is expected
  to extend PoE2 to the other endpoints (almost certainly the same
  segment): when it ships, the change is one row in that table plus a
  first-contact sample under the standing rule. Not pre-enabled — an
  unobserved URL shape is never sent.
- **The store's realm is the request's realm**, stamped from the params
  (the listing's or the fetch's), not the entry's `realm` field: the
  docs give that field as `pc|xbox|sony` while the endpoint accepts
  `poe2` (a contradiction, open until a PoE2 body is seen; the field
  stays verbatim in the json). Observed: a pc list's 59 entries all
  carry `realm: "pc"`. The address a plan renders is (request realm,
  listed name) — the one combination guaranteed to fetch. Whether a
  list spans realms is undocumented; the removal rule is realm-scoped
  (a realm-R listing retires only realm-R characters it did not stamp):
  under-retires if lists span realms, never over-retires.
- **Listed `deleted` or `expired` characters are skipped with a named
  reason, not fetched**, until evidence says otherwise. The docs define
  neither beyond "always `true` if present", and the invalid-request
  threshold (too many 4xx restricts the app, independent of rate
  limits) makes a 404 hunt a real cost. Observed: characters in ended
  leagues (Ancestors, Phrecia 2.0, an event) are listed with **no
  `expired` flag**, so `expired` does not mean "league ended", and
  **league names on characters are not restricted to
  `/account/leagues`** — the planner treats the league key as an opaque
  string. `Character.league` is optional (documented): a character with
  none is reported as uncovered, never a failure.
- **`skills` (PoE2) and `guardian` (PoE1: the inventory of an animate
  guardian — untradeable, still worth knowing) join the lifted arrays,
  and every item row records the array it came from** — a `container`
  ingest fact beside `location_kind`/`location_id`, not a derived
  column: it is not in the item's json, so `rebuild` cannot recompute
  it, exactly like location. Necessity, not convenience: the live
  guardian's five items carry `inventoryId` `Helm`/`BodyArmour`/
  `Gloves`/`Boots`/`Weapon` with `x`/`y` 0 — the character's own slot
  names — so the item alone cannot say which array it sits in, and
  `inventoryId` has no documented values at all. Location stays the
  character id (one removal pass per character); moving between arrays
  stays a `changed` event. All five guardian items carried ids
  (documented `Item.id` is optional; the store's id-less refusal
  stands — check the same on the first PoE2 `skills` body).
- **Drift tripwire at ingest.** GGG adds fields most leagues; a new item
  array on `Character` would go un-lifted silently. After the declared
  arrays are lifted, an array of item-shaped objects left in a character
  envelope is counted and surfaced in `store status` — never a failure.
- **`acq characters` and the MCP `characters` tool print the id beside
  the name** (full 64-hex: matching is exact, a prefix cannot be pasted
  into a policy). Name→id resolution at `policy set` is parked
  (trigger: authoring friction) — it would make the stored policy
  differ from what the human typed.
- **Freshness heuristic, owner's call for v1:** the list entry carries
  `experience` (observed on all 59), monotone for a character's life; a
  listing newer than our fetch reporting a different `experience` proves
  play since — the sibling of `ListedCountDisagrees`. A `league`
  disagreement (a Hardcore death landing in Standard) is the same arm.

Shape (falls out of the rulings):

- One policy, one plan, one loop: `characters: "all" | [ids]` beside
  `tabs` under `realms.<R>.leagues.<L>`. The character list is per realm
  (`list_characters { realm }`, no league — the envelope's league check
  treats realm-wide actions as in-envelope); a character is covered when
  its `(realm, league)` policy names it.
- Actions `list_characters { realm, reason }` and `fetch_character
  { realm, id, name, league, reason }` → `("characters", {realm})`,
  `("character", {realm, name})`; skip reasons `Fresh`,
  `AwaitingListing`, `Deleted`, `Expired`; separate `skipped_characters`
  / `unknown_characters` lists. **Plan schema 6** (realm took 5 on
  2026-09-02; a new action set is its own bump — one integer, and a
  realm-only plan file read by the characters build then says "newer
  schema", never "malformed"). The apply vocabulary widens to
  `characters` and `character`, both single-request; the daemon stays
  plan-blind.
- Store: `characters` mirrors `tabs` — `id` PK, `realm`, `name`,
  `league` (nullable), `json` (fetched), `listed_json` (a fetch never
  touches it), `listed_at`, `listed_response`, `fetched_at`,
  `removed_at`; membership stamped per listing response id. `tabs`
  gained `realm` (PK `(realm, league, id)`) at **facts v3**, the realm
  step; the character key is **facts v4**: its migration remaps through
  each row's json (`id` is there) and drops rows whose json lacks it
  (facts are refetchable), and remaps item locations from name to id.
- `Store::stash_snapshot` becomes the `(realm, league)` refresh
  snapshot: stash listing basis + tabs, character listing basis (per
  realm) + that league's characters, the policy row — one read
  transaction, nothing derived.
- Mock: realm paths on all four routes; one `poe2` character with a
  `skills` array; a `guardian` array on a pc character with slot-named
  `inventoryId`s; `frameTypeId` on items.
- Tests: coverage, freshness, disagreement, deleted/expired skips, the
  policy v1→v2 upgrade and strict round trip, the container column, the
  drift tripwire, and the process-level loop against the mock with tabs
  and characters in one policy.

Order of work: (1) realm as its own gate-green step — it touches the tab
slice already pinned live, which must stay green with pc throughout;
(2) the character key, columns, lifted arrays, container, tripwire;
(3) snapshot, planner, vocabulary, CLI/MCP; (4) live: one row under the
tracer rung's standing rule — the account's characters, list + fetches,
two probes (the tight list window, 2 per 10 s, is the limiter's, not the
plan's); (5) **PoE2 first contact**: `GET /character/poe2` is documented
live today — once the realm step exists, the owner creates a PoE2
character and one first-contact sample under the standing rule (fresh
daemon, ceiling 3: POST, HEAD, GET) answers what `Character.realm` says
for a PoE2 character, whether the pc list omits it, and what `skills`
items look like (ids present?).

**Realm step (1) built 2026-09-02** — three commits (`959e7fea` core,
`a0b2561b` store, and the planner/frontends), gate green, mock rehearsal
green in the `all`, ids, and `--realm xbox all` shapes; the pc journals
carry exactly the pre-realm routes (`stash-list@…`, `stash@…`) and URLs
(`/stash/Standard/…`), the xbox run probed `stash-list/xbox` first and
closed on the mock's empty console listing. Calls made inside the
ruling, revisable:

- **The realm table is one type in core** (`realm.rs`: `Realm`,
  `Family::accepts`), read by the daemon (rendering + admission), the
  mock (path classification), and the planner (policy parse) — the
  `declare_route_knowledge` mold, linkable by all three because the
  planner already depends on core. The store stays string-typed: it
  records the request's realm and never validates it.
- **A non-pc realm suffixes the limiter's route label**
  (`character-list/poe2`, `stash-list/xbox`) as well as the URL, so each
  realm's URL shape gets its own free HEAD before its first counted
  send; whether it shares the pc policy is learned from headers (N6
  already shares state by name). One extra HEAD per realm per lifetime.
- **Admission refuses a realm a family does not take** (`admit_realm`
  at submit and per apply tuple, before a job id exists) — the daemon
  side of "no code path renders a stash URL with poe2"; the planner
  side is the policy parse.
- **Items carry realm beside league** (stamped from the seam); the
  ad-hoc `refresh` kind forwards realm to its children; `leagues`
  stays realm-less (no consumer, not in the plan).
- **Mock consoles are truthful, not colliding**: an xbox/sony stash
  listing is empty and a tab fetch 404s (tab and item ids are
  GGG-unique, so pc's are never reused under another realm); the
  character list is per realm, with one poe2 character carrying a
  `skills` array (a hypothesis until PoE2 first contact).
- Policy v2 parses v1 in place; the stored value stays what was
  written (`policy show` shows what the human typed).

External review of the three commits (2026-09-02, Codex via the owner)
found four defects, fixed the same day before step (2): the handshake
compared the fixed package version (decision line above; a pre-realm
daemon would have sent a console job to pc — the one silent failure the
respawn mechanism exists to prevent); migrated items had a null realm
(now `NOT NULL DEFAULT 'pc'`, and the migration test carries an item);
the v2 policy parser had become an untagged enum and lost top-level
strictness (a v1 value with a stray field, or a v2 with both `realms`
and `leagues`, was stored half-honored — now the stamp dispatches into
one strict shape); and the version story was implicit (now: facts v4 and
policy v3 for characters, and plan schema 6 as the ruled stamp for
step (3) — written since step (3) landed the same day). One caution it stated, which
the order of work already implied and is now explicit: **until the
character key lands, cross-realm character ingestion is unsafe** —
character rows and item locations are name-keyed, so a PoE2 name that
collides with a pc name overwrites it; PoE2 first contact stays at (5),
after (2). It also asked for the realm boundary to be pinned at process
level, not only through the private `route_for`:
`acquisition-cli/tests/realm_wire.rs` proves, through the real binaries
and the send journal, that explicit pc and omitted realm are the same
route, that xbox work goes out on `stash-list/xbox` / `stash/xbox` with
their own probes first, that the mock's empty console listing lands no
pc tab under xbox, and that a poe2 stash job is refused with nothing
journaled; the tracer verifier checks every data route's realm suffix
against the run's `--realm`.

**Step (2) mechanism, agreed before building (2026-09-02):**

- `characters.league` is **listing-owned**: the coverage coordinate is
  what the basis listing said, so a fetch never overwrites it (the same
  rule as `listed_json` on tabs); a fetched body's league lives in its
  json and is the disagreement arm's other side. A character fetched
  directly, never listed, takes the body's league on insert only.
- **Container is compared explicitly at ingest**: a helm moving from the
  character's own `equipment` to its `guardian` has byte-identical json
  (`inventoryId` `Helm`, x/y 0), so "moving between arrays is a
  `changed` event" needs the column in the comparison, not only the json.
  A pre-v4 character item has no container on record (NULL — the value
  is not in the json, so no migration can recompute it); the first fetch
  after the migration sets it without an event.
- **Facts v4 migration**: `characters` is rebuilt keyed by `id`, taken
  from each row's json (list entries and fetched bodies both carry it);
  a row whose json lacks an id is dropped and its items retired (facts
  are refetchable); item locations move from `character/<name>` to
  `character/<id>` through the same json, so the first post-migration
  fetch produces no false `moved` events; `item_events` history keeps
  its old location strings (history is history). Stash items get
  container `items`.
- **Listing entries need `id` and `name`** (both documented required):
  `id` is the identity that makes retirement safe, `name` the address a
  plan renders; a fetched body without `id` is malformed too. Membership
  is stamped per listing response id and retired per realm, exactly as
  tabs are.

**Policy v3 shape (agreed 2026-09-02, for step (3)) — coverage per
facet:** `realms.<R>.leagues.<L>.{tabs?, characters?, max_age_seconds}`
where each facet is `"all"` or an id list and **absent means no coverage
of that facet** (an empty list is the same, explicitly); an entry naming
neither facet is malformed ("names no work"), never a silent no-op.
Validation is per facet against `Family::accepts`: `tabs` is refused
under a realm the stash family does not take (poe2), `characters` is
accepted under every realm — so a character-only PoE2 entry is the
ordinary v3 shape. v1 and v2 upgrade to their tab coverage plus no
character coverage; the stored value stays as written. "Names no work"
is judged after normalization: `tabs: []` with characters absent or
empty fails the same way. (Review finding 2026-09-02: v2's required
`tabs` and entry-level realm check could not express the ruled PoE2
policy.)

**Step (2) built 2026-09-02** (facts v4): `characters` keyed by `id`
with `listed_json`/`listed_response` and the realm-scoped, response-
stamped retirement tabs have; `items.container` (compared explicitly;
NULL for pre-v4 character items, `items` backfilled for stash rows);
`guardian` and `skills` lifted; the drift tripwire (`_unlifted` in the
envelope, `Ingest::unlifted`, `Status::unlifted_items`, printed by
`acq store status`); the v4 migration proven on a v0-shaped file with a
row that has an id (rekeyed, items relocated) and one that does not
(dropped, items retired). The mock's fetched character now carries the
list's id for its name (a fetch of an unlisted name mints a new one —
the recreated-character shape) and `StashHoarder` has a two-item
`guardian` with slot-named `inventoryId`s. `acq characters` already
prints the id beside the name (the list payload is verbatim) and the
MCP `characters` tool's rows carry `id`. Tests pin: rename keeps the row
and moves nothing; a recreated name is a never-fetched new row with the
old one retired; the listing owns `league`; a fetched body without an
id is malformed; the guardian swap is `changed`, not `moved`; the
tripwire counts and never fails.

Second review (same day, `953be323`..`e188c86f`) found three more
defects, fixed before step (3): a character or tab a listing dropped
left its items live (search showed a recreated character's inventory
beside the old one's — the very case the id key exists for; now the
retired location's items are retired with `removed` events, through the
same per-location removal); a fetch authorized under an old address
that landed after a newer listing rolled `name`/`class`/`level` back
(now listing-owned once a listing has named the row — the body's say
stays in `json`); and item removal keyed on `last_seen < at`, so two
fetches in one second left the second's omissions live (now
`items.seen_response`, membership per response like listings). Plus:
the drift count is per live character's latest fetch, not cumulative
(so it clears once a build that lifts the array has fetched); the v4
migration re-stamps character membership from the latest listing on
record per realm, so a basis a planner cites has its rows stamped to
it (pinned in the migration test); and the policy v3 shape above.

Third review (same day) found the same class once more, fixed: **a
fetch never revives a location a listing retired** — the row's
`removed_at` is listing-owned like its address, and a late fetch of a
retired character or tab records the body but *withholds* its item
facts (`Ingest::withheld`) until a listing names the location again,
when the next fetch lands them as a reappearance; **a location is its
full coordinate** (`Location`: realm, league for a stash, kind, id), so
item removal, counts, and move detection never collapse the same tab id
under two realms into one place; **a parent tab's fetch is its
substashes' listing** — a stub it no longer carries is retired with its
items, and a listing that retires the parent retires the substashes'
items while keeping their rows for the planner's orphan report; removal
events take their ids from the update itself (`RETURNING`), never from
a timestamp match; and a character's items take the row's listing-owned
league, not the body's.

Fourth review (same day): the withholding model's second-order gaps,
fixed (facts **v5**: `responses.withheld`). A withheld fetch no longer
touches the row at all — and, independently, **a listing that revives a
retired row clears its `fetched_at`**, because a plain drop-then-revive
by two listings (no late fetch anywhere) also left a live, empty
location the planner called fresh; pinned planner-side (`a revived tab
is planned as never fetched`). A substash's liveness includes its
parent's (a late substash fetch under a retired parent is withheld, and
a late fetch of the retired parent neither revives it nor rewrites its
children — the orphan report stays as the listing left it). A withheld
fetch keeps the *whole* body verbatim on its response row (arrays
included; nothing split off and lost), the count is on the row and in
the daemon log and `store status`. Event addresses carry the full
coordinate (`stash/<realm>/<league>/<id>`, `character/<realm>/<id>`).

Fifth review (same day): three edge cases, fixed (facts **v6**). A
listing that retires a parent clears its retained substashes'
`fetched_at` along with their facts, so when the parent returns the
plan fetches the parent *and* each substash again (pinned planner-side:
a revived parent replans its substashes). `responses.withheld` is
nullable — NULL is an ordinary response, so a withheld fetch of an
empty location is still marked (`Some(0)`) — and counts every item fact
the body carried, socketed gems included; `store status` reports both
withheld responses and items. The malformed-body contract holds for a
withheld body too: identity is checked before the store decides whether
a body lands or is withheld, so an id-less item or stub is refused and
nothing is written, whatever the location's liveness.

**Step (3) built 2026-09-02** (policy **v3**, plan schema **6**; calls
made inside the ruling, agreed with the owner before building):

- **The planner is facet-symmetric, not character-aware.** `Selection`
  (`all` | ids) serves both facets; `LeaguePolicy { tabs?, characters?,
  max_age_seconds }` with `None` meaning no coverage of that facet (an
  empty list normalizes to `None`); each facet compiles on its own —
  its listing verdict, its covered entities, its skips, its unknowns —
  and the actions are the tab facet's then the character facet's. The
  freshness rules are one function for both (`window_verdict`: never
  fetched, older than the window) plus each facet's own disagreement
  arm: `ListedCountDisagrees` for tabs; `ListedExperienceDisagrees`
  (judged only when the entry and the fetched body both carry
  `experience`) and `ListedLeagueDisagrees` (the row's listing-owned
  league against the body's — a Hardcore death) for characters, on the
  shared `FetchReason`. Skips are their own enum,
  `CharacterSkipReason { Fresh, AwaitingListing, NoLeague, Deleted,
  Expired }`, so a tab kind on a character skip is malformed.
- **"Names no work" applies after normalization to every version.** A
  stored v2 `tabs: []` now reads as malformed instead of compiling to an
  empty plan; nothing stored that (the MCP loop test's "empty coverage"
  fixture became an id the facts lack, which is the honest empty plan).
- **League-less characters are reported in every league plan of their
  realm** as a `no_league` skip (by id too — never as unknown, the facts
  know them): the ruling said "reported as uncovered", and a realm fact
  no league key reaches needs somewhere to appear. The snapshot carries
  them beside the league's own rows.
- **The store's liveness rule is consumed, not restated.**
  `CharacterSnapshot` offers exactly `fetched_at` (None for never
  fetched *or revived*), the listed entry verbatim, and the fetched
  envelope only while a fetch stands (`Null` after revival — the column
  still holds the disowned body). The planner reads those three facts
  and has no liveness logic. `Store::stash_snapshot` is
  `Store::refresh_snapshot`: `stash_listing` + `tabs`,
  `character_listing` (per realm) + `characters`, the policy row, one
  read transaction.
- **Envelope renames under the bump** (free now, never again):
  `basis.stash_listing` + `basis.character_listing` (both carried as
  the facts of the read, whichever facets the policy covers),
  `skipped_tabs` / `unknown_tabs` beside `skipped_characters` /
  `unknown_characters`; `list_characters { realm, reason }` carries no
  league and is in-envelope for any league of its realm
  (`RefreshAction::league()` is an `Option`); `fetch_character { realm,
  league, id, name, reason }` renders `("character", {realm, name})` —
  the listed name, no `class`/`level` (the ruled five fields).
- **A strictness hole the character skip test found, closed:** serde
  ignores extra fields beside an internally tagged *unit* variant's
  `kind` (`never_fetched`, `folder`, `deleted`…), `deny_unknown_fields`
  or not — "unknown fields at any depth are refused" held only for
  struct variants. `RefreshPlan::from_value` now requires the derived
  envelope (the quote aside — an observation with its own strict shape)
  to re-serialize to exactly what was read. The same shape as the
  store's five rounds: a rule honored on the main path, lost on one
  serde path.
- **Apply vocabulary**: `characters`, and `character` with a name; both
  single-request. `acq store characters` is new: the CLI had no
  store-side read of the character rows (only the MCP tool), and the
  driver's readback needs one. The mock's list entries and bodies carry
  `experience` (one constant: the mock account never plays).
- **Accepted residual:** the character list is realm-wide while plans
  are per league, so two league plans on one realm applied together
  each authorize their own `list_characters` — the same register as two
  realm plans listing separately; v1 lives with it.
- **Tests** (planner, each on the six-path checklist): v3 per-facet
  parse and every malformed shape; the character listing alone on a
  never-listed realm with no stash work; exact-id coverage and unknown
  ids; the experience arm forward-only (same-second and body-without-
  experience prove nothing) and the window; a league move fetching in
  the new league and leaving the old; deleted / expired / no-league
  skips, by `all` and by id, and in the realm's other league; the
  revived character as never fetched with a withheld late fetch in
  between; a recreated name planning the new id only; a row without a
  listed entry planning by the window; the mixed loop closing through
  `Endpoint::from_job`; realm-wide actions in-envelope and character
  skips parsing strictly. Store: the per-realm character basis with the
  league's rows plus league-less ones, revival clearing the body, two
  listings in one second, malformed stored json with its address.
  Process level: `acquisition-cli/tests/apply_loop.rs` now runs the
  mixed policy (two listings, then 7 tabs + 2 characters, then 7
  substashes, then empty, character rows fetched with items);
  `tests/characters_wire.rs` closes a character-only poe2 policy on
  `character-list/poe2` / `character/poe2` with probes first, nothing
  on any other data route, the `skills` item at the character under
  poe2 only, and a `tabs` facet under poe2 refused at `policy set`.
- **Driver and verifier**: `--characters all|id,...` and a `none` tab
  selection (a character-only run; how poe2 is driven), one probe per
  route the plan touches (four at most), pacing by the longer facet
  (the two policies run side by side), a character readback (every
  covered character fetched, or skipped for a reason that never
  fetches). Mock rehearsals green 2026-09-02: `all --characters all` =
  login, `1/2/2`, `1/2/9`, `1/1/7`, closed; `--realm poe2 --characters
  all none` = `1/1/1`, `1/1/1`, closed — that first cycle is exactly the
  standing rule's first-contact shape (ceiling 3: POST, HEAD, GET), so
  order-of-work (5), PoE2 first contact, can run as that invocation
  (`LIVE-TESTING.md`, "Characters rung").

Pending ground-truth claims (documented facts read 2026-09-02, to be
authored master-side and then cited here by number): realm segment
semantics per endpoint and pc-by-omission; PoE2 on the character
endpoints only; `Character.id` unique 64-hex, `Item.id` optional;
`inventoryId` undocumented; the invalid-request (4xx) threshold; the
`Character.realm` contradiction. Observed (this run): guardian slot
names; ended-league names without `expired`.

**Step (4) ran live 2026-09-02 and passed** (`LIVE-TESTING.md`, run
ledger `characters rung, pc` and the "Characters rung" section): the
tracer's five ids plus `--characters all` on pc — one 112-request cycle
(both listings, 5 tabs, 64 substashes, 41 Standard characters of 59
listed; the 18 in other leagues are outside a Standard policy), ceiling
117 exact, four probes at 0 hits, zero non-2xx, loop closed in 2 cycles.
The two facets paced independently (stash ~13 min, characters ~6.5 min;
the cycle is the longer). Facts: 10 of the 41 are stripped characters
(empty item arrays, `_split` 0/0/0 — the body's truth, not a lifting
gap); guardian lifted (4 characters, 18 items); 1081 `added` events at
`character/pc/<id>`; every body `metadata.version` `3.29.3`.

Exit: the loop closes on a policy naming tabs and characters together,
live, with the driver's checks green — **met 2026-09-02**; (5) PoE2
first contact waits on the owner creating a PoE2 character; then pricing.

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


## Open topics

- Priority levels: how many, and named or numeric? (Interactive > background is the intuition, *regardless of frontend* — an agent in a live conversation is interactive; the caller states its urgency, the frontend doesn't imply it.)

(2026-08-31: "delta/selection for refresh" and "user state on items" are resolved into the sync-policy / annotations / Plan decisions above; the tracer below builds them.)

## Explicitly deferred (do not build yet)

- ~~Multi-account build steps~~ — built 2026-08-30, steps (1)–(6) of "Multi-account design"; step (7)'s first contacts are in `LIVE-TESTING.md`'s run ledger (`/profile`, `/account/leagues`, `/character/{name}` all done 2026-08-30). GGG answered Q12 (2026-08-30): `/profile` is not rate limited at present (declaration confirmed, kept), and `/account/leagues`' counted HEAD will be corrected in a future release — its no-probe declaration stays until the free HEAD is observed live, then it goes and the probe returns.
- Queue-management UI (drag-to-reorder, per-job progress bars). v1.0 only guarantees the architecture makes this a rendering problem.
- ~~Agent/MCP traffic against GGG~~ — deferral lifted 2026-09-01 (decision "Agent traffic against GGG is allowed; the daemon is the single gate"): GGG permits agent use under the API rules; `acq-mcp` now spends through a running daemon in either mode and never spawns one in real mode.
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
  - Human-legible CLI output for the plan slice (the offline "no quote" note prints twice and carries a raw `os error 2`; the readback has no one-line "n tabs refetched, m items changed"; the folder child's `acq tabs` row is indented and truncated) → trigger: before the pricing session's live run, so the owner's friction notes — not the agent's reading — are that run's truth (method verdict above).
  - Fact-path migration to uuid naming → opportunistic, or never (facts are refetchable).
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
