My judgment: make one small documentation-compaction pass, then build pricing as the second vertical tracer. After pricing semantics survive CLI and MCP use, build the first real GUI slice around refresh, search, and pricing.

Do not begin with a general GUI shell, broad code review, test-suite campaign, or more networking work. Those would improve or expose the existing system without answering its most important remaining question: can this architecture carry real product intent as cleanly as it carries network effects?

## Where the project actually stands

This is no longer a spike in engineering maturity. It is still a reference implementation in product completeness.

What is proven:

- The GGG boundary is unusually well understood. The ladder closed after roughly 1,450 live sends with zero 429s; it also exercised transient failures, token expiry across sleep, per-account counters, crash recovery, and PoE2 first contact. [LIVE-TESTING.md](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/LIVE-TESTING.md:796)
- Network safety is structural: one choke point, one send-lifetime gate, header-driven correction, bounded retries, visible probes, persistent jobs, and evidence-preserving journals.
- The fact store has survived real data rather than only fixtures: tabs, substashes, characters, multiple realms, item movement, retirement, late responses, malformed bodies, and PoE2’s id-less granted skills.
- The refresh loop is now complete: intent → snapshot → immutable plan → quote → exact application → new facts → empty replan.
- CLI and MCP already share the planner, store, and daemon client rather than reimplementing semantics.
- The workspace is healthy now: clean tree, 246 tests passing, Clippy clean with warnings denied, formatting clean, and `git diff --check` clean.

What is not yet proven:

- Pricing and effective-price semantics.
- Advanced item discovery and filtering.
- Shop derivation and publishing.
- Currency accounting.
- A real graphical frontend.
- GUI-specific contract needs such as pagination, event replay, long-lived subscriptions, and presentation of partial failures.
- Packaging and Windows transport.

So the project has surpassed the C++ application in architectural integrity, failure honesty, observability, multi-account identity, and agent ergonomics. It has not yet surpassed it as an everyday inventory-management product. That distinction should govern the next phase.

## What the history teaches

The productive pattern was remarkably consistent:

1. Establish a small structural invariant.
2. Drive a real consumer through it.
3. Observe failures rather than predict all of them.
4. Promote the surviving property into a test or decision.
5. Discard or replace the mechanism when the consumer disproves it.

That sequence gave the project:

- The journal as the contract surface, after tests and rails proved unable to identify which binary had actually run.
- The shared store, after `acq pull` demonstrated that frontend-owned snapshots would duplicate traffic and discard useful partial results.
- Persistent jobs, after a live halt showed that unsent work and completed work needed to survive the daemon.
- UUID-bound annotations, after reasoning from the old buyout migration failures.
- Binding plans, after a real refresh run showed that reviewed work could remain exact without causing practical pain.
- Evidence-preserving refusals, after PoE2 returned a legitimate shape the store did not understand.

The project’s greatest asset is therefore not just the daemon or the store. It is the method by which uncertainty becomes evidence and then becomes structure. [TESTING-NOTES.md](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/TESTING-NOTES.md:35)

## The architectural achievement

The four-layer model is the project’s real center:

- Facts: what GGG reported.
- Intent: what the user wants.
- Derivations: what follows from facts and intent.
- Effects: work that changes or contacts the outside world.

Each has one authoritative mutation path. The daemon remains blind to intent; store reads remain network-free; derivations remain reproducible; plans authorize effects without becoming effects themselves. [CONTEXT.md](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/CONTEXT.md:56)

This is stronger than ordinary modularity. The modules constrain one another in useful ways:

- The daemon cannot quietly become an application server.
- A refresh cannot erase user intent.
- A read cannot unexpectedly spend requests.
- A planner cannot invent entities absent from its cited facts.
- An autonomous client cannot replace a human’s live real-GGG daemon.
- A malformed response remains evidence without becoming truth.
- A frontend needing new semantics must put them through one of the two shared doors.

That is the synthetic coherence you were aiming for. The parts do not merely coexist; their boundaries make the other parts safer.

## The next move: pricing as a semantic tracer

The existing documents already nominate pricing as the second plan-bearing consumer. [CONTEXT.md](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/CONTEXT.md:1115) I agree, but not because it happens to be next in the parking lot.

Pricing is the first materially different test of the architecture:

- Refresh reconciles facts by spending network budget.
- Pricing edits local, irreplaceable intent.
- Refresh plans are executed by daemon jobs.
- Pricing plans should be applied locally through the annotation store.
- Refresh plans are dominated by freshness and request cost.
- Pricing plans are dominated by inheritance, precedence, conflicts, and reversibility.

If both fit the same high-level rhythm without being forced into the same payload or executor, then “Plan” has been validated as a family of operation-specific authorization documents. If they do not, pricing will reveal precisely where the grammar should stop generalizing.

I would impose these boundaries on the pricing tracer:

1. Store only explicit assertions.

   Do not reproduce the C++ mechanism of materializing inherited prices onto every item. Store a tab or character price and explicit item overrides; compute effective item prices as a derivation.

2. Treat in-game prices as facts.

   A price parsed from a tab name or item note belongs to the fact layer. It should remain read-only, with its source and precedence explainable. A manual mutation against a game-locked target should be reported as a non-action, not stored as a shadow value that can never take effect. The legacy rules are summarized in [pricing.md](/Users/tom/Development/GitHub/gerwaric/acquisition/docs/user/pricing.md:31).

3. Give pricing a strict domain type.

   The generic annotation row is excellent storage infrastructure, but `{scope, key, kind, value}` must not become the domain API. Define a versioned, strictly parsed `Buyout` type with legal price kinds, value rules, currency identifiers, and target kinds.

4. Make effective price explainable.

   Every result should say not only “5 chaos” but why: game item note, explicit item assertion, inherited location assertion, unpriced, ignored, or otherwise non-postable. This is where the new system can become plainly better than the C++ application.

5. Keep `PricingPlan` local.

   It should cite the account UUID, relevant fact identities, current annotation revisions, explicit mutations, no-ops, and refusals. Applying it should use the annotation store’s CAS machinery. It should not contact or spawn the daemon, request a quote, allocate a network job, or pretend local intent editing has a wire cost.

6. Keep accepted mutations atomic.

   The plan can report game-locked, missing, or malformed targets individually. But the explicit mutations it authorizes should apply in one transaction and fail together if a cited revision has moved. That preserves the meaning of reviewing a document before applying it.

7. Do not mutate the sync policy implicitly.

   Pricing an item should not silently edit a second annotation. Freshness requirements should instead appear when a consumer needs them—especially shop rendering or publishing—as a visible precondition with a refresh remedy. That converts the C++ “priced tabs are secretly locked into refresh” coupling into an explicit relationship between two kinds of intent.

8. Start with handles, not a grand query language.

   Exercise one location price, one inherited item, one item override, clear-to-inherit, a game-locked price, a removed/orphaned item, and a stale-revision conflict. Advanced filters can wait until a real batch-pricing workflow demands them.

9. Expose the same semantics through CLI and MCP.

   That is enough to validate sharing. A GUI should consume the validated result later rather than helping invent it.

There is one question I expect pricing to reopen: annotation history. Current revisions prevent clobbering, but they cannot answer “what was repriced since my last session?” Do not build a generic event system before the tracer. Make that question part of the tracer’s done criterion. If current rows plus plan artifacts cannot answer it adequately, pricing is the legitimate trigger for an atomic annotation event log.

## Why not the other directions first?

| Direction | Assessment |
|---|---|
| Pricing tracer | Highest leverage: validates the missing intent/derivation half and adds direct user value. |
| New GUI | Important immediately after pricing; premature now because it would stabilize presentation around incomplete product semantics. |
| Documentation cleanup | Necessary as a short enabling pass, not as the next product phase. |
| Broad code review | Likely to optimize mechanisms already reviewed heavily; use targeted seam review during pricing instead. |
| Test-suite review | The project already identified diminishing returns here. Add tests from a new consumer rather than auditing tests in the abstract. |
| More networking work | The GGG boundary is proven and now offers sharply diminishing returns. |
| Shop publishing | Valuable later, but introduces a second credential and a wholly different outward-traffic safety boundary. |
| Advanced search | Build when pricing or the GUI produces concrete selector and latency needs; avoid designing 38 filters from the C++ interface by inheritance. |

## The small cleanup I would do first

The agent control plane is beginning to accrete faster than it is compressing. The required entry documents now total about 2,800 lines, and `CONTEXT.md` says “current state only” while carrying long implementation and live-run histories. That is a genuine threat to the project’s agent-intuitive character.

I would make a tightly bounded compaction pass before pricing implementation:

- Keep `CONTEXT.md` to active invariants, decisions, open questions, and working style.
- Move closed tracer/character/legibility narratives into explicitly historical records or rely on Git where the evidence already lives.
- Turn the enormous README store/planner description into links to focused current-state documents.
- Keep `LIVE-TESTING.md` authoritative, but separate its active standing rule from the closed ladder narrative.
- Leave the intentionally frozen [CLI-GUIDE.md](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/CLI-GUIDE.md:3) clearly historical.
- Do not rewrite or reconsider settled decisions during this pass.

This is not cosmetic documentation work. Reducing the fixed reading cost protects the method that made the code coherent.

## The sequence I would follow

1. Compact the active documentation without changing architecture.
2. Hold a pricing crystallization session based on product laws, not C++ widgets.
3. Build the narrow pricing plan/apply/effective-price tracer through CLI and MCP.
4. Use it against realistic stored data, entirely offline.
5. Record what pricing teaches about plan families, annotation history, batch atomicity, and freshness coupling.
6. Then build the first GUI vertical slice: account/realm/league selection, item search, effective-price explanation/editing, and refresh plan/apply progress.
7. Let that GUI—not speculation—surface pagination, subscription, cancellation, and presentation requirements.
8. Approach shop publishing only after pricing and GUI semantics are stable.

The deepest directional shift is this: the infrastructure phase is over. The project should now become domain-led without surrendering its structural discipline. Pricing is the right next step because it converts the architecture from an exceptionally safe data-acquisition system into the beginning of a better Acquisition.