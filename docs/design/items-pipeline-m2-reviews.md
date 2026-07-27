# Items Pipeline M2 Spec — Review History

**Companion to `items-pipeline-m2.md`** (split out per the convention
`network-redesign.md` arrived at when its review appendix outgrew it —
here from round 1, before any appendix accumulated). This file
preserves the M2 spec's decision history: review-round finding tables
(`R1-*`, `R2-*`, …), round narratives, and the revision log. The spec
cites these IDs inline and records only current decisions; new review
rounds and revision-log entries append here, with the spec's status
line updated in the same commit. Where a later round supersedes an
earlier resolution, both are recorded.

`R<round>-<n>` means a finding from that numbered review round of this
spec — deliberately distinct from the cleanup register's permanent
`F` numbers and from the network redesign's `ER`/`IR`/`R4`+ series.

## Round 1 — external review (July 27, 2026)

Read-only design/code review of spec revision 1 (commit `aaa70f1e`) by
an external reviewer (different model). Overall verdict: direction
sound — the two shaping decisions (D1 F62-first sequencing, D2
fetch-source atomic unit) were not challenged — but not freezable:
several contracts would have either violated M2's own constraints or
forced design choices during coding.

Every code-level claim was re-verified against `d995840b` before
acceptance (anchors noted per finding below). All nine findings were
accepted. Eight were resolved as the reviewer recommended; R1-2 was
resolved by the alternative the reviewer offered (relax-and-bound)
rather than their preferred option (indexed representation), with the
reasoning recorded.

| ID | Group | Finding | Status |
|---|---|---|---|
| R1-1 | Failure semantics / shop | D5's completed-with-skips outcome reaches the unchanged D8 final cascade, so `Application` auto-submits the forum shop containing stale contents for the skipped tabs — and `RefreshFinished` (ordered after `ItemsRefreshed`) arrives too late to suppress it. Today a parse failure aborts without posting. | Resolved in D8/D4: automatic submission moves to `RefreshFinished`, gated on clean completion. Skips get a user-visible surface (D5). Pinned `shopSubmitsOnlyOnCleanCompletion`. |
| R1-2 | Published-state storage | The spec claims O(delta) work in `ItemsManager`, but its only storage is a flat `Items` vector, so finding a replacement's old items is O(all items) per delta — an unacknowledged contradiction with D8 and the acceptance criteria. | Resolved in D3 by the reviewer's second option: the flat vector stays and one linear predicate-only erase pass per delta is explicitly permitted, bounded by worker parity (`RemoveItemsFetchedBy` is the identical per-reply pass, shipped in M1). The source-keyed map with a lazily rebuilt flat vector is the named fallback if measurement disagrees, and the natural M3 direction. |
| R1-3 | Replacement key | The M2 contract keys replacements by (type, fetch id) but the worker's `RemoveItemsFetchedBy` erases by fetch id alone (`itemsmanagerworker.cpp:451`); a cross-type id collision would make worker and published state diverge, breaking the snapshot-plus-deltas invariant. | Resolved in D3: a named `FetchSourceKey{type, fetch_id}` keys application in both the worker and `ItemsManager`; the worker's erase predicate gains the type field as part of M2. |
| R1-4 | Scoped pricing | D7's safety argument ("final pass overwrites divergence") only holds on success; D4 deliberately allows deltas followed by failure, where no final pass runs. Tab-name auto-pricing would then have mutated persistent tab-buyout state keyed by unpublished metadata, and the lock exclusion means a game-priced item can remain unlocked and drop out of the next checked refresh (locks feed `GetRefreshChecked`, `buyoutmanager.cpp:193`, which drives the worker's Checked selection, `itemsmanagerworker.cpp:398`). | Resolved by rewriting D7 to the reviewer's policy: scoped pass = note-based pricing + inheritance from published tab buyouts only; tab-name auto-pricing stays final-only; locks add monotonically per delta and are cleared only by the successful final pass. Pinned `scopedPricingIsFailSafeAcrossFailedUpdate`. |
| R1-5 | UI state machine | D9's two-tier wording marks only non-intersecting-branch searches dirty (every delta invalidates every search), and has a transition hole: an intersecting delta's pending tick can be orphaned by a tab switch, because `Search::FilterItems` skips refiltering on `TabChanged` unless a dirty flag is set (`search.cpp:212`). "Flush pending" was unspecified. | Resolved by replacing the two tiers with the reviewer's five-rule state machine (all searches dirty on every delta; timer owned by the current search; refilter clears own flag; tab switch/delete cancels the timer, the flag preserves the work; final snapshot cancels and clears through the full path) and adding five automated `MainWindow` acceptance tests against the existing fixture (`mainwindowfixture.h` — verified present). The S1-M2 spike narrows to UX-feel judgment. |
| R1-6 | Terminal event shape | `RefreshOutcome` permitted invalid states (both `skipped` and `error` always present), `SkippedTab` had no fields, `AbortUpdate()` receives no error today so first-error preservation was unstated, `Canceled` was omitted from D5's classification, and "accepted update" was undefined. | Resolved in D4: sum type `std::variant<CompletedRefresh, FailedRefresh>`; skipped entries reuse `FetchSourceKey`; first-error capture at `StopUpdateForFailure` with explicit mappings for catch-alls and unstopped-token `Canceled`; "accepted" defined as the Idle→Updating transition. |
| R1-7 | Terminal event ordering | `FinishUpdate` sets Idle *after* emitting (`itemsmanagerworker.cpp:1279`); a terminal event inserted before that assignment means a synchronous observer reacting to "finished" by starting an update is refused as still updating. | Resolved in D4: pinned ordering — final `ItemsRefreshed` → Idle → `RefreshFinished` on success; Idle → `RefreshFinished` on failure. |
| R1-8 | Internal contradiction | D3 allows a parent reply to emit its replacement delta plus empty ghost-drop deltas, while the first acceptance criterion demanded "exactly one" `TabRefreshed` per accepted reply. | Resolved: criterion reworded to one primary replacement delta followed by zero or more empty reconciliation deltas, order pinned. |
| R1-9 | F46 | The spec touches `ItemsManager::OnItemsRefreshed`'s debug scan ("stays on the final snapshot") without resolving or explicitly deferring F46, whose register entry asks that M2/M3 work on this function absorb it. | Resolved in D8: M2 absorbs F46 — the scan is gated behind `spdlog::should_log` (or deleted) as part of the `OnItemsRefreshed` rework; the register's F46 entry moves to the resolved ledger when that lands. |

**Round-1 narrative.** The two heaviest findings were R1-1 and R1-4,
both the same failure of the draft: revision 1 argued safety from the
success path (the final pass, the final emit) in a design whose whole
point (D4's no-rollback) is that updates can end without one. R1-1's
nuance, recorded for fairness: partial refreshes already submit with
stale *unselected* tabs today, so posting stale data is not itself
new — the regression was that a tab the user explicitly selected could
go silently stale into a post, where today's behavior is to not post
at all. R1-2 was accepted as a real internal inconsistency but
resolved toward simplicity: the erase the reviewer flagged is the
same operation the worker itself performs per reply, so M2 legitimizes
it with a stated bound instead of building an indexed store one
milestone early. R1-5's state machine replaced a vaguer two-tier
description outright — the reviewer's version is both simpler and
closes two holes the draft's wording created.

## Revision log

- **Revision 1** (July 27, 2026, commit `aaa70f1e`): initial draft —
  staleness preamble against `d995840b`, D1–D11, acceptance criteria,
  input-traceability table covering the full M2 inbox from
  `items-pipeline.md`.
- **Revision 2** (July 27, 2026): round-1 incorporation — R1-1 through
  R1-9 resolved as tabled above. Substantive changes: forum submission
  re-gated on clean completion (D8), `FetchSourceKey` unifying both
  erase predicates (D3), the flat-vector linear erase legitimized with
  the worker-parity bound (D3), D7's scoped pricing rewritten to be
  fail-safe across a failed update (monotone locks, tab-name pricing
  final-only), D9 rewritten as an explicit five-rule state machine
  with automated acceptance tests, `RefreshOutcome` as a sum type with
  first-error preservation and pinned Idle-before-terminal ordering,
  and F46 absorbed. The shaping decisions D1/D2 are unchanged. This
  file split out in the same commit.
