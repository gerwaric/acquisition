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

## Round 2 — external review (July 27, 2026)

Read-only design/code review of spec revision 2 (commit `5cc6db10`),
same external reviewer. Verdict: the revision closes the round-1
findings well; two new blocking interactions, two material gaps, four
minor corrections. Freeze gate stated: R2-1/R2-2 designed, R2-3 given
a concrete measurement gate, R2-4 mapped, and the S1-M2 spike
completed.

Every code claim was re-verified before acceptance (`shop.cpp`
submission chain, the inline fail-fast completion comment in
`RunUpdate`, `GetTab`'s id-only key, the `search.cpp` scale comment).
All four findings and all four corrections were accepted; none were
modified in substance. The round's pattern, recorded for future
rounds: both blockers were contracts that held at the signal layer
but broke one layer up (the shop's asynchronous submission pipeline)
or one nesting level down (synchronous reentrancy inside terminal
fan-out).

| ID | Group | Finding | Status |
|---|---|---|---|
| R2-1 | Shop / mid-refresh state | The clean-completion gate governs when submission *starts*, but `SubmitShopToForum` is asynchronous: it fetches the legacy stash index first (`shop.cpp:175-194`) and only the continuation reads live `ItemsManager::items()` and buyouts (`shop.cpp:289`). Update N+1, legally started after N's terminal event, can stream deltas into the state N's "clean" submission then reads — new under M2, since today `items()` changes only at final snapshots. | Resolved in D8: submission input is captured as an immutable **value** snapshot at request time (shared pointers are insufficient — N+1's successful rebase mutates shared `Item`s in place); the index is applied to the capture on arrival. Manual submission during a refresh captures and submits current published state, deliberately accepted. Pinned `shopSubmissionUsesCapturedSnapshot` (staged test). |
| R2-2 | Terminal fan-out reentrancy | D4's Idle-before-terminal invites a synchronous observer to start N+1 — but `RunUpdate` launches synchronously and fail-fast futures complete inline (`itemsmanagerworker.cpp:502-508`), so N+1's signals (even its terminal event, e.g. on a setup-cooldown fail-fast) can fire nested inside N's fan-out, reaching later observers before their `RefreshFinished(N)`. Defeats the ordering-is-identity contract that justified omitting an update ID; composes with R2-1. | Resolved in D4: a delivering-terminal guard around the `RefreshFinished` emit — an `Update()` in the window is accepted-and-deferred to the next event-loop turn (the deferral-while-initializing shape); a second request in the window is refused as if an update were active. Pinned `terminalFanOutDefersReentrantUpdate`. |
| R2-3 | Storage performance claim | Revision 2's "sub-millisecond pointer-chase at 100k" was asserted, not measured; the erase dereferences a heap object and compares type + `QString` per entry, M2 doubles the per-reply scans, and the code itself acknowledges users at the "hundreds of thousands or millions of items" scale (`search.cpp:243`). Worker parity establishes precedent, not a bound. | Resolved in D3: the flat-vector choice stands but now carries a **blocking implementation measurement** (M2-M2) with stated thresholds (combined erases < 2 ms at 100k, < 16 ms at 1m) on representative datasets; exceeding them makes the source-keyed fallback required, not discretionary. |
| R2-4 | First-error coverage | The missing-wrapper branches (a 200 whose parsed wrapper lacks its stash/character sub-object, `itemsmanagerworker.cpp:901/1033`) are terminal but hold no `FetchError`, so D4's first-error plumbing had undefined inputs; hook ordering and reset were also unstated. | Resolved in D5/D4 via the facade rather than synthesis: post-F62 the facade extracts the sub-object anyway (bytes capture), so an absent payload is reclassified as a facade-level `Parse` error and the worker branches are deleted — deliberately moving the case into D5's skip set. First-error storage precedes the fault hook and resets at the next accepted update. Pinned `missingStashWrapperSkipsTab` / `missingCharacterWrapperSkipsTab`. |

**Minor corrections, all adopted:** (1) revision 2's trailing D7
"renamed tab keyed by fresh metadata" transient was wrong —
`GetTab` keys on stable `location.id()` (`buyoutmanager.cpp:101`), so
per-delta inheritance is rename-proof and the paragraph is deleted;
the only real transient is the renamed-to-a-price tab awaiting the
final auto-tab pass, already described. (2) A skipped source is
"listed with stale contents", not F55's "listed-but-cold" — its
contents survive, and a successful final rebase still freshens their
embedded metadata. (3) `Application`, not `Shop`, connects to
`RefreshFinished` (the D8 criterion said otherwise). (4) The D9
throttle period must be injectable so `throttleDoesNotRearm` does not
wait wall-clock S.

## Round 3 — external review (July 27, 2026) — OPEN, unresolved

Read-only design review of spec revision 3 (**reviewed baseline:
commit `58d33480`**), same external reviewer. Verdict: revision 3
incorporates round 2 correctly but is not ready to proceed directly
to the spike — one new blocking shop-state issue, two material
state-machine gaps, one process contradiction, one wording
correction.

**Recorded, not verified.** This round was recorded by the outgoing
spec session as a handoff; per the practice of rounds 1–2, the
incorporating session must re-verify every code claim against the
codebase before accepting or resolving any finding, then fold the
resolutions into spec revision 4 and complete this table's Status
column.

**Recommended resolution order** (from the outgoing session, for the
incorporator): verify all four first; then R3-4 (a process decision
for Tom — it shapes how the spike is planned), then R3-1 (the largest
design work: a submission state machine), then R3-2 and R3-3
(contained amendments to D4 and the M2-M2 contract).

| ID | Group | Finding | Status |
|---|---|---|---|
| R3-1 | Shop submission coalescing | The immutable capture (R2-1) removes an *accidental coalescing* the live read provided: `Shop` refuses a submission while one is active (`shop.cpp:146` claimed), so if clean N+1 finishes while N's capture is still submitting, N+1's auto-submit is refused and `UpdateShopData()` completing N clears `m_shop_data_outdated` (`shop.cpp:355` claimed) — N+1 is neither posted nor left pending. Related: deltas deliberately do not call `ExpireShopData`, so a forced manual mid-refresh submission can find the outdated flag false and reuse cached shop data instead of rendering its fresh capture. Reviewer proposes an explicit submission state machine: every request captures; if busy, retain one pending latest auto snapshot; a monotonic shop-input revision replaces the boolean; completing N marks only N's revision clean; drain pending after the active submission ends; manual submission renders its capture regardless of the flag; centralized completion applies one pending policy on every exit; decide whether pending drains after a *failed* submission (probably not after auth failure, while staying outdated). Staged tests: N+1-completes-during-N, latest-wins coalescing, manual-after-delta with stale flag, newer expiry not cleared by older completion. | **Open** |
| R3-2 | Terminal-deferral reservation | The R2-2 guard protects the fan-out window only; between fan-out end and the queued turn, another `Update()` can observe an Idle worker and start first, so the promised deferred update is refused, delayed, or reordered. Reviewer proposes treating the deferred update as an active reservation until its Idle→Updating transition: later requests refused while reserved; selection arguments copied at reservation time; reservation cleared immediately before starting; worker destruction before the queued turn = "never accepted". Extend `terminalFanOutDefersReentrantUpdate` with a request arriving after fan-out but before the deferred turn. | **Open** |
| R3-3 | M2-M2 contract precision | The combined worker+manager threshold can fail in a way the named fallback cannot fix: the fallback re-indexes only `ItemsManager` storage, so if the worker's own scan alone exceeds the budget, the combined measurement still fails. Reviewer recommends measuring worker and manager separately, gating the M2 storage choice on the manager's *marginal* cost (combined reported as context), and recording a separate worker-index finding if the worker independently blows its budget; alternatively a combined threshold with a both-sides fallback, or measuring the whole synchronous delta path if the threshold means a frame budget. Also record build mode and measurement environment. | **Open** |
| R3-4 | Process: spike vs. freeze | The spec says implementation begins only after freeze (working rule 1) while S1-M2 needs a working throttle prototype and is the declared pre-freeze gate — a contradiction as written. Reviewer recommends a documented exception: a dedicated non-production spike branch/harness (discarded or left unmerged), result recorded in revision 4, then freeze, then production implementation; alternatively amend the working rule. | **Open** |

**Wording correction (open with the round):** D8's manual-submission
rationale says "what you see is what you post" — inaccurate, because
D9 deliberately lets the visible model lag the published state; the
capture is of the *published* state, not the rendered one.

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
- **Revision 3** (July 27, 2026): round-2 incorporation — R2-1 through
  R2-4 resolved as tabled above, plus the four minor corrections.
  Substantive changes: shop submission input captured by value at
  request time with the manual-submission policy stated (D8),
  terminal fan-out reentrancy guard with accept-and-defer semantics
  (D4), the linear-erase choice gated by the blocking M2-M2
  measurement with a mandatory fallback (D3), missing-wrapper
  payloads reclassified as facade `Parse` errors joining the skip set
  with first-error hook ordering and reset stated (D5/D4), and the
  incorrect renamed-tab transient deleted from D7.
