The proposal has found the right architecture. I would continue with the four-layer model, a frontend-side planner, separate intent storage, daemon blindness, and refresh-with-plan as the tracer.

I would not accept the packet unchanged, however. Five concepts need sharper definitions before they become load-bearing:

1. A layer needs one authority, not necessarily one physical writer.
2. A Plan should be an immutable authorization envelope, not merely a forecast.
3. A quote is a time-stamped, non-reserving estimate—not a pure promise.
4. “Requests” and “budget” need separate logical-work and actual-wire-send meanings.
5. Irreplaceable annotations require stable account identity, concurrency control, and a real backup mechanism.

## Decision-by-decision rulings

| Decision | My ruling | Reason |
|---|---|---|
| A1 effects ledger readable | Amend | Bless a read-only effects-ledger API, not `JobDb` generally. A dead daemon proves no HTTP send is currently in flight, but persisted jobs may still be waiting or recorded as running. |
| A2 rails become product features | Amend | Reuse the machinery, but do not promote the existing rail semantics wholesale. The tripwire and lifetime ceiling are explicitly ladder-only today. |
| D1 four layers/one writer | Amend | Keep the layers, replace “one writer class” with “one authority and one sanctioned mutation API.” Recomputable derivations may be materialized. |
| D2 separate annotations | Amend | Strong direction, but it needs stable account identity, SQLite-safe backup, optimistic revisions, and orphan semantics. |
| D3 daemon never initiates traffic | Amend wording | Every root operation should have an external initiator. The daemon necessarily creates probes, child jobs, refreshes, and retries causally beneath it. |
| D4 sync policy first annotation | Accept with constraints | Define it as desired coverage/freshness, not a scheduler, and do not treat unchanged item counts as proof of unchanged contents. |
| D5 frontend-side Plan | Amend materially | Plan should exist offline from facts + annotations; quote is optional enrichment. Add provider, basis/revisions, operation kind/version, and bounded-action semantics. |
| D6 separate `quote` request | Accept verb, amend contract | Separate verb is right. “Pure,” scalar headroom, and single-route completeness are not. It is a non-mutating projection with an explicit basis and unknowns. |
| D7 remedy is a Plan | Accept narrowly | Use it for unmet freshness/planner preconditions, with stable structured error codes. Do not force ordinary stale reads to fail. |
| D8 per-operation budget | Amend substantially | The proposed descendant counter does not count probes or OAuth and discovers overruns too late. Define budget units first and prefer admission-time enforcement. |
| D9 all frontends share Rust | Accept | State it as “shared semantics live in Rust; every frontend has a Rust adapter.” Tauri’s webview remains presentation, not a second implementation. |
| D10 separate plan crate | Accept with responsibility shift | Policy compilation belongs in `acquisition-plan`; the store should expose facts and annotations, not contain half the planner. |
| D11 panic policy | Accept | This is especially important with persisted poison inputs. “Structured” should mean stable error kinds and context, not only an `anyhow` string. |
| Bucket C parking lot | Accept | Except that annotation row revisions are needed now; the full annotation event log can remain parked. |
| R1 no fused reads | Amend | Prohibit implicit network access in the store/read API. Do not prohibit explicit frontend workflows that refresh and then read. |
| R2 daemon blind to annotations | Accept | This is one of the strongest boundaries in the proposal. Its rationale should be architectural rather than dependent on current macOS cron behavior. |
| R3 no cached search service | Accept | The reopening triggers are good. |
| R4 schema internal | Accept | Add schema versions and compatibility errors; an accessible SQLite file is still not a supported SQL contract. |
| P1 evidence-driven sessions | Accept | Sound. |
| P2 owner’s real use | Amend | Owner use is strong evidence for this CLI tracer, not sufficient evidence for every GUI/MCP/TUI contract. |
| P3 generalize at second consumer | Amend | Good heuristic, not an absolute. Irreversible identity, durability, and concurrency choices sometimes need first-consumer treatment. |
| P4 lint/property/agent ownership | Amend wording | Good allocation rule, but design discussion is still useful before a property becomes lint or test. |

## The important amendments

### 1. D1 should describe authority, not physical writing

The four-layer model is excellent, but “nobody writes derivations” is already false in a productive way. Derived item columns are persisted and can be rebuilt by a frontend command through [`Store::rebuild`](</Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/crates/acquisition-store/src/lib.rs:586>). Imports also write fact-side state without being the daemon.

The durable property is:

> Facts, intent, and effects each have one authoritative mutation path. Derivations have no independent authority: they may be computed or materialized, but must be reproducible from their declared inputs.

That permits FTS indexes, extracted columns, summaries, and caches without confusing them with truth.

Similarly, A1 needs a read-only façade. The current public [`JobDb`](</Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/crates/acquisition-store/src/jobs.rs:82>) exposes `upsert`, deletion, and pruning as well as reads. A frontend-readable effects ledger should be a capability such as `JobLedgerReader`, not merely a convention that frontends promise not to call writer methods.

An offline state report should say something like:

- daemon: offline
- actual sends in flight: zero
- persisted work: 7 waiting, 1 previously recorded running
- ledger observed at: timestamp
- runtime state: unavailable/stale

“Daemon offline” must not collapse into “there is no outstanding work.”

### 2. D2 needs a durability design, not just a separate file

Separating annotations from facts is the correct choice. It eliminates daemon/frontend write contention and prevents fact-store replacement from accidentally erasing intent.

But two claims need correction.

First, “backup is a file copy” is not reliably true while SQLite WAL writers are active. Copying only the main `.db` can omit committed pages still in `-wal`. Backup/export should use SQLite’s backup API, `VACUUM INTO`, or a store-crate operation that takes a consistent snapshot.

Second, current account files are named from username, while the design explicitly accepts that a name change orphans the fact file. That is tolerable for refetchable facts; it is not tolerable for the only irreplaceable state. Current identity handling records UUID opportunistically but does not use it as a key ([`CONTEXT.md`](</Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/CONTEXT.md:77>)).

Before annotations land, introduce a stable local account key:

- Minted locally at first login, or based on UUID when securely known.
- `accounts.json` maps current username, aliases, UUID, and provider to it.
- Fact and annotation paths use that stable key.
- A rename updates the mapping rather than orphaning intent.

Finally, multiple frontends already exist. “Frontend is the writer class” does not prevent CLI and GUI from overwriting each other. The first annotation table should have an integer revision and compare-and-swap writes, or there should be an explicit decision that last-writer-wins is acceptable. Given the “irreplaceable” designation, I favor optimistic conflict detection. That does not require the full deferred annotation change log.

### 3. D5 and D6 should separate authorization from prediction

The strongest model for a Plan is:

> A Plan is a serializable, immutable statement of the effects the user has authorized, derived from a named snapshot of facts and intent.

It should contain at least:

- provider and stable account identity;
- operation kind and plan schema version;
- fact basis: response/listing IDs or timestamps;
- annotation revision;
- explicit actions or a declared upper bound;
- generated-at timestamp;
- freshness/completeness assumptions;
- optional quote with its own observation time and uncertainty.

Account alone is insufficient: the same account name can exist in mock and real providers, and a plan can become detached from the facts and policy revision that produced it.

Crucially, the Plan should still be computable when the daemon is down. Facts and intent determine what should happen. Live limiter state only estimates when it can happen. Therefore:

```text
Plan = authorized work + provenance
Quote = optional, time-sensitive forecast attached to that work
```

That is cleaner for offline CLI reads, a GUI opened before the daemon, and MCP inspection.

The proposal currently says request counts are exact while ETA may be unknown. “Request” is too overloaded for that claim. A refresh may induce:

- logical API GETs;
- HEAD probes;
- OAuth refresh POSTs;
- retries after 429;
- dynamic substash children;
- a listing GET under a different policy.

The daemon itself creates probes as daemon-submitted jobs ([`ensure_probe`](</Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/crates/acquisition-core/src/daemon.rs:908>)), so they are not naturally descendants of the user’s refresh parent. OAuth traffic is not a job at all.

I would expose at least two dimensions:

- `logical_requests`: exact or bounded work represented by the plan;
- `wire_sends`: projected range, with prerequisites and retries identified separately.

The quote must also admit that it is non-reserving. The existing limiter documentation already says `eta_for` is “an estimate, not a promise” ([`ratelimit.rs`](</Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/crates/acquisition-core/src/ratelimit.rs:583>)). Another client can submit work one millisecond after the quote. Headers may revise the prediction on the next response.

Consequently, `headroom_after` cannot honestly be one scalar. It is a projection per policy/window and account/IP scope, conditional on no intervening sends.

I would phrase D6 as:

> `quote` is a read-only, non-reserving projection over current daemon knowledge. It reports its observation time, basis, per-resource estimates, and unknown prerequisites; applying a plan may receive a different schedule.

A separate protocol request remains clearly better than a flag on `Submit`.

### 4. D8’s proposed enforcement unit is wrong

The proposal is right that per-operation enforcement is new machinery. But a counter on the descendant tree is not yet a meaningful request budget:

- probes may be separate root jobs;
- OAuth sends are not child jobs;
- retries add wire sends;
- one concurrent token refresh may serve several operations;
- dynamic deep fan-out discovers work only after earlier sends.

Nor should the normal behavior be “spend part of the budget, discover the excess mid-fan-out, then terminalize interrupted.” If the plan is explicit, reject it before spending. If fan-out becomes knowable after the parent’s listing response, check before submitting that fan-out.

For the tracer, I would implement a logical-work budget first:

> The submitted plan contains an explicit bounded action set. The daemon refuses it before child submission if its logical request bound exceeds `max_requests`.

An actual-wire-send budget is a different feature. It would require a causal operation ID propagated through children, probes, authentication, and retries, plus semantics for shared token refreshes. Keep that feature priced and deferred until a consumer truly needs it.

### 5. R1 should protect epistemics without constraining UX

The invariant worth keeping is:

> Store reads never initiate network traffic.

The stronger “no frontend operation may refresh and then read” does not follow. A GUI Refresh button naturally performs an explicit effect and then updates the view. A CLI command can explicitly orchestrate `apply → await → read` without weakening either surface.

Likewise, stale facts should normally remain visible with their freshness and completeness metadata. Only a caller that asserts a freshness requirement should receive an unmet-precondition error carrying a `RefreshPlan`.

That yields three honest operations:

- observe cached facts, including their age;
- assert a freshness condition and receive a Plan if unmet;
- explicitly apply and observe again.

It avoids making offline/stale data unnecessarily unusable.

## My recommendation for plan staleness

I would choose binding semantics for v1, but define “binding” more carefully than the proposal does:

> Applying a Plan may execute exactly the listed actions or a strict subset; it may never add an action that was not reviewed. New facts produce a new Plan.

This makes a Plan an authorization envelope rather than advice. It also preserves exact logical bounds and makes budget enforcement simple.

For the first tracer, I would exclude dynamic `--deep` fan-out. A refresh plan should list explicit tab IDs. If a tab vanished, that action fails or is reported as skipped; newly discovered tabs wait for the next plan. This is honest eventual reconciliation and does not fetch an unreviewed expansion.

The existing refresh implementation always re-lists and then creates children ([`daemon.rs`](</Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/crates/acquisition-core/src/daemon.rs:1700>)). Reusing it unchanged would make the reviewed plan advisory. The tracer should instead either:

- introduce an exact-tab batch parent that fans out the listed IDs without re-planning; or
- make the existing parent accept an explicit immutable action set and guarantee it never expands it.

If fresh listing is itself required, make it a plan action. Its result informs the next plan rather than silently changing the current one.

One related caution: `metadata.items` counts are useful evidence, but equal counts do not prove equal contents. One item can leave while another arrives, or an item can change in place. Counts may justify heuristics or detect definite differences; they cannot establish that a tab “cannot have changed” without a stronger GGG guarantee.

## Revised tracer order

I would keep refresh-with-plan, with this sequence:

1. Define the v1 semantics before schema work:

   - Plan is binding and serializable.
   - No dynamic deep fan-out.
   - Logical requests versus projected wire sends.
   - Reads return freshness; freshness assertions may return Plans.

2. Add stable local account identity, the annotation file, revisioned writes, orphan retention, and a store-managed backup/export operation.

3. Expose neutral store snapshots: sync policy, tab identities, freshness, listing basis, and metadata. Keep policy compilation in `acquisition-plan`.

4. Build `RefreshPlan` offline from facts and annotations. I would start with an operation-specific type rather than prematurely declaring one universal Plan grammar.

5. Add `quote` and attach its result as optional Plan enrichment. A single-route limiter primitive is fine internally, but do not describe the whole refresh quote as complete if the plan actually spans list, stash, OAuth, or probe traffic.

6. Expose `acq refresh --plan`; inspect both human and JSON forms.

7. Apply the exact plan action set through a parent job, with admission-time logical-budget checking.

8. Expose the same operation through MCP in mock mode and verify that MCP needs no parallel semantics.

9. Run the owner’s live use under the existing standing rule.

I would change the done criterion from “plausibly closing the frontend boundary” to:

> Pin the refresh Plan/quote/apply slice and the annotation API it exercised.

A CLI tracer cannot close the entire GUI/MCP/TUI frontier. A GUI may still expose requirements around pagination, event replay after disconnect, cancellation presentation, partial results, and long-lived subscriptions. Pinning one validated slice is fully consistent with the project’s method; declaring the whole frontier closed would not be.

## Process and parking lot

The parking lot is disciplined and should stand. Two nuances:

- Pricing will be the second plan-bearing consumer, but it may demonstrate that “Plan” is a family of operation-specific documents rather than one universal payload. Let that be evidence rather than forcing refresh and pricing into a common enum now.
- Shop publishing is correctly isolated. Because it is credentialed outward traffic outside the current API invariant, it should receive an equally structural ownership/rate/safety boundary before implementation.

For the working-style lines, P3 needs the largest correction:

> Generalize after two materially different consumers reveal the shared property, except where an early choice controls irreversible identity, durability, safety, or compatibility.

The proposal itself demonstrates the exception: stable account identity and annotation durability must be handled before the first intent row because repairing them later would put the irreplaceable state at risk.

## Bottom line

The proposal’s central synthesis is real:

```text
Facts + Intent ──frontend planner──> Plan
                                      │
                               optional Quote
                                      │
                                    Apply
                                      │
                              Effects → new Facts
```

The daemon remains a safe executor, not an application server. The store remains the home of observed and intended state. The shared Rust crates remain the semantic backbone for CLI, GUI, MCP, and TUI.

My strongest recommendation is to adopt that system while changing the meaning of Plan from “what I currently predict will happen” to “the bounded work I have authorized.” Once that is done, staleness, budgeting, concurrency, offline operation, and multi-frontend consistency all become substantially easier to reason about.
