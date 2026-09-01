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
- **Version handshake in the protocol; the protocol is single-version on purpose.** Kill-and-respawn is the entire migration mechanism — no deprecation, no compat matrix. Replacing is the *interactive CLI's* policy only (`ConnectOptions::interactive` — the caller is the human expressing intent); an autonomous client (MCP) never kills or replaces a daemon: the mismatch it sees may be a human's live GGG run, so it reports and stops (`ConnectOptions::autonomous`, `client.rs`). Known caveat, accepted: two frontends built from different commits would thrash by respawning each other's daemons — theoretical in a one-workspace playground, recorded so it isn't relearned live. Rationale: CLI and running daemon may be from different builds; three frontends with a compat matrix is the reconciliation swamp, three frontends with respawn is a one-line diff. Amended 2026-08-30 (autonomous policy).
- **CLI emits structured output** (`--json` on every command, the error path included: failures are `{"error": …}` on stdout with exit 1, and a failed job's outcome exits 1 in both output modes). Rationale: the CLI is itself an API; makes MCP and agent use nearly free. Trued up 2026-08-30 (four commands ignored the flag; `--json` mode exited 0 on a failed job).
- **A frontend consumes exactly two surfaces: the daemon protocol and the store crate's read API — no third door.** This pins the boundary's *location*, not its content: the verbs stay revisable until a consumer validates them (TESTING-NOTES.md, "pin after the consumer"). A frontend that wants a third channel is a protocol or store change, recorded here first. Rationale: bespoke per-frontend channels are what turns three frontends into a review burden; two shared surfaces keep the contract in exactly two places, enforced by what frontends link against rather than by process. Decided 2026-08-30.
- **The MCP server is a fourth thin client (`acquisition-mcp`, binary `acq-mcp`, official `rmcp` SDK over stdio), never in-process with the daemon.** Same reasoning that moved reads to the store: daemon-hosted queries make the daemon an application server. The binary embeds `daemon run` like `acq` (lazy spawn execs `current_exe`). Two structural rules in the rail-6 mold: it never kills or replaces a daemon (autonomous connect policy above), and it refuses `submit_job` in real-GGG mode until the agent-traffic deferral lifts — store reads and observing a live daemon are allowed in either mode, they send nothing. It lazy-spawns only in mock mode; login stays human, via the CLI. The tracer is the consumer that validates the protocol: when it has proven the shape, the protocol gets pinned — the GUI arrives to a pinned boundary and proposes changes against it, rather than reopening the question. Decided 2026-08-30.
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
- **Bodies are stored verbatim except at the item seams; `items` is the only place to look for an item.** Every item array (tab `items`, character `inventory`/`equipment`/`jewels`/`rucksack`, each `socketedItems`) is lifted into `items`, one row per GGG item id (stable across moves), keyed by location `(kind, id)`; the envelope keeps the counts under `_split`, so envelope + rows is the response exactly. Derived columns come from the row's own JSON (`rebuild` re-extracts; never a refetch). Ingest compares with the previous state and records `item_events` — this replaces `pull`'s snapshot diff. Rationale: raw-plus-parsed duplicated every body (a league spans 1000× in size); raw-only made every query a body scan and gave user state (buyouts, notes) no key. Decided 2026-08-29; the real-snapshot replay (322 tabs, 19,210 rows, 2.3 s, zero false changes 8 h apart) is the evidence.
- **Multi-account is one daemon holding many sessions, never one daemon per account.** The Cloudflare bound (`SendGate`, 2 live sends) is a per-IP property (P-B, ground truth §1) held as per-process state; two daemons on one machine make it a 4-wide burst that neither sees, with separate tripwires. Rung 11 (2026-08-30) showed the other half: `Account` rules count per account on GGG's side, so two accounts never contend on layer 2 — the only thing they share is layer 1 and the `Ip`-scoped token endpoint, which is exactly what the single gate exists for. Built in two halves with different blast radii (option C): **account as first-class identity now** (store path, job field, keyring key — leaves), **many live sessions later** (a refactor confined to the session layer). Limiter and probe scope keying — `(account, policy)` for `Account` rules, policy alone for `Ip` rules, scope learned from `X-Rate-Limit-Rules` — is a **precondition of the session map, not an optimization**: with two live sessions on one policy each response would overwrite shared state with a different account's counters, and the next send from the other account floods (a 429 path; the "over-waits, never floods" reading only held for rung 11's sequential switch). Decided 2026-08-29, amended 2026-08-30 after review across sessions; design below in "Multi-account design"; built 2026-08-30 through step (6) — step (7)'s live samples are in `LIVE-TESTING.md`'s run ledger.
- **Per-route knowledge about GGG that headers cannot teach lives in one place (`Daemon::declare_route_knowledge`), and strict observation is the default everywhere else.** `GET /profile` (first contact 2026-08-30) answers 200 with no `X-Rate-Limit-*` headers at all and 403 to HEAD, which strict observation ("every endpoint has a policy", post-N33) classed as a protocol failure and discarded. Now: a route *declared* policyless accepts a 2xx with **no** rate-limit header (a partial set is still a failure; a policy that later appears is learned strictly), becomes `EndpointState::Policyless`, and is paced by nothing but the send gate; a declared no-probe route goes straight to its GET. Only `/profile` is declared, and it is called at most once per login. Not generalised on purpose: "any headerless 2xx is fine" reopens the blind spot strict observation closed. Owner decision 2026-08-30; GGG confirmed the same day (Q12/N38): `/profile` is not rate limited at present, so the declaration is confirmed and stays until headers ever appear — strict observation covers that arm.
- **Rate limiter spec will be expressed as test tables, not prose.** `docs/design/network-ground-truth.md` (the claims registry; it indexes the deeper spike evidence) is the input; "given these headers, wait N seconds" tests are the permanent, enforced spec.
- **The system is four layers — facts, intent (annotations), derivations, effects — each with one authoritative mutation path, not one physical writer.** Facts mutate only through the store crate's ingest surface (daemon `record`; `store import`); intent only through the store crate's annotation write API (frontends); the effects ledger only through the daemon; derivations have no independent authority — computed or materialized, always reproducible from declared inputs (`rebuild` is their maintenance, not fact ingestion). The daemon is permanently blind to intent, and it creates work only in causal service of client-submitted work (probes, children, retries) — never spontaneously: no schedules, no policy execution, no annotation reads; scheduled syncs are small frontends. Rationale: "a sync can never clobber intent" becomes structural, the way the choke point made rate-limit discipline structural — and blindness is safe exactly because the daemon never initiates. Decided 2026-08-31 (brainstorming-notes 06, ruled).
- **Annotations are the only irreplaceable local state.** A separate per-account file named by the account uuid (identity decision in "Multi-account design"), keyed on stable GGG ids, written only through the store crate with integer-revision compare-and-swap; no fact-side event ever deletes intent — an annotation whose item is removed is kept and surfaceable as orphaned; export/backup is a store-managed consistent snapshot (`VACUUM INTO` / SQLite backup API — a raw file copy under WAL is not a backup). Rationale: facts are refetchable at the cost of requests; intent has no server to refetch from — the C++ legacy-buyout saga is the full price of getting this wrong. Decided 2026-08-31.
- **The sync policy is the first annotation: a per-account, inspectable declaration of desired coverage and freshness — not a scheduler — compiled by the frontend-side planner into minimal requests.** `metadata.items` counts are heuristic evidence: they can prove a tab changed, never that it didn't. Rationale: C++ tracked-set/clean-refresh semantics, the old delta/selection topic, and both redesign essays independently describe this one object. Decided 2026-08-31.
- **A Plan is a serializable, immutable authorization envelope, and plans are binding.** Derived from a named snapshot of facts + intent, computable with the daemon down; it carries provider + account uuid, operation kind + plan schema version, fact basis (response/listing ids or timestamps), annotation revision, the explicit action set (or a declared upper bound), generated-at, freshness assumptions, and optionally a quote with its own observation time. Work has two dimensions: `logical_requests` (exact or bounded) and `wire_sends` (a coarse range plus named prerequisites — probe, token refresh, possible 429 retries — never a precise accounting). Applying a Plan executes exactly the listed actions or a strict subset, never an unreviewed addition; new facts produce a new Plan; v1 excludes dynamic `--deep` fan-out (a vanished tab fails or is reported skipped; newly discovered tabs wait for the next plan). Operation-specific types first (`RefreshPlan`); a universal grammar waits for the second plan-bearing consumer. Binding is revisable on tracer evidence (the owner's live-use friction notes are the data). Decided 2026-08-31.
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

### Annotations & plans — the refresh tracer (decided 2026-08-31; steps 1–8 built, step 9 prepared — owner run pending)

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
3. Neutral store snapshots (2026-08-31, `Store::stash_snapshot` in
   `acquisition-store/src/snapshot.rs`) — with the malformed-2xx
   ruling: a malformed body is a typed refusal that writes nothing and
   fails the *job*; only `acq store import` keeps the legacy tolerance,
   at its own boundary.
4. `RefreshPlan` compiled offline in `acquisition-plan` (2026-09-01).
   `REFRESH_PLAN_SCHEMA` is **3**; any shape change anywhere in the
   envelope — the embedded `Quote` included — is a schema bump, so an
   older reader reports "newer schema", never "malformed".
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
   ggg-mode quote enrichment attempts no connection, returning the plan
   with a note naming the open topic below. `apply_plan` is refused in
   ggg mode alongside `submit_job`. Pinned at process level (review
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
- Whether `quote` is allowed over MCP in real-GGG mode — it sends nothing, but it is a daemon interaction in ggg mode; owner call. Step 8 built the conservative default (no connection attempted; the plan comes back with a note naming this topic), so the ruling stays open with nothing blocked on it.
- Binding-plan friction: D-line "plans are binding" is revisable on tracer evidence — the owner's live-use friction notes against subset-only reconciliation are the data; re-ruled (or confirmed) after the live rung.

(2026-08-31: "delta/selection for refresh" and "user state on items" are resolved into the sync-policy / annotations / Plan decisions above; the tracer below builds them.)

## Explicitly deferred (do not build yet)

- ~~Multi-account build steps~~ — built 2026-08-30, steps (1)–(6) of "Multi-account design"; step (7)'s first contacts are in `LIVE-TESTING.md`'s run ledger (`/profile`, `/account/leagues` done; `/character/{name}` pending). GGG answered Q12 (2026-08-30): `/profile` is not rate limited at present (declaration confirmed, kept), and `/account/leagues`' counted HEAD will be corrected in a future release — its no-probe declaration stays until the free HEAD is observed live, then it goes and the probe returns.
- Queue-management UI (drag-to-reorder, per-job progress bars). v1.0 only guarantees the architecture makes this a rendering problem.
- Agent/MCP traffic against GGG — blocked on verifying GGG's policy stance on agent traffic before the MCP path ships. The refusal is structural since 2026-08-30: `acq-mcp` refuses `submit_job` in ggg mode. (Owner-driven live baseline testing of the daemon against GGG is not deferred; it has its own control document.)
- **Parking lot (2026-08-31, each with its trigger so deferral never needs re-arguing):**
  - Pricing-as-document → lands on the annotations layer + plan/apply after the tracer; the second plan-bearing consumer, and the test of whether Plan is one grammar or a family of operation-specific documents.
  - Legacy buyout import → a patch generator into the ordinary annotation plan/apply path; the wizard dissolves.
  - Shop / forum publishing → outward credentialed traffic (POESESSID) **outside the API choke-point invariant**; requires its own equally structural ownership/rate/safety boundary session before any implementation.
  - User-scoped annotations home (`user.db`) + scope taxonomy → trigger: the first user-scoped kind (currency ratios, saved searches).
  - Annotation event log → trigger: `diff --since` needs "what got repriced," or conflicts need history (row revisions exist from day one; the schema is shaped so the log is an addition, not a migration).
  - Wire-send budget → trigger: a consumer that needs enforcement over actual sends, not logical work.
  - Universal Plan grammar / five-verb surface → direction only; evidence at pricing.
  - Dynamic `--deep` fan-out under plans → trigger: tracer evidence that two-cycle reconciliation genuinely hurts.
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
