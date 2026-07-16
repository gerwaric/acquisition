# ADR 0002: Defer QML Migration in Favor of Interior Design Cleanup

## Status

Accepted (July 2026)

## Context

ADR 0001 (on the unmerged `prepare-qml` branch, together with
`docs/qml-migration-plan.md`) proposed an incremental migration of the UI from
Qt Widgets to QML. A code investigation performed while evaluating that plan
found that the preparatory refactoring it required — decoupling `Search` from
`QTreeView`, separating filter logic from widget construction, fixing
model/view signal discipline — is valuable independent of QML, while the
QML-specific work has significant costs for this application:

- Acquisition is a dense desktop power tool (tree tables, menus, dialogs),
  which is Qt Widgets' strongest terrain and Qt Quick's weakest. Recreating
  `QTreeView` behavior (26-column tree, header sorting, checkboxes, extended
  selection, context menus) in QML is the largest single cost of the migration
  and delivers little user-visible benefit.
- The investigation surfaced latent correctness problems in the core
  (threading in `ItemsManagerWorker`, model/view signal hygiene, layering
  inversions) that would undermine a QML port and should be fixed first in any
  case.
- The project has no automated test suite, making any large UI rewrite risky.

## Decision

Defer the QML migration indefinitely. Keep the current Qt Widgets UI and UX.
Invest instead in an interior design cleanup that fixes correctness and
structure problems in the existing code, documented in `docs/cleanup/plan.md`
with detailed per-phase implementation documents in `docs/cleanup/`.

This decision supersedes ADR 0001. The QML plan documents remain on the
`prepare-qml` branch for reference; much of their analysis (coupling points,
validation checklist) is carried forward into the cleanup plan.

## Consequences

Positive:

- Correctness bugs (threading, model signaling) get fixed directly instead of
  being carried into a new UI stack.
- Each cleanup phase is independently shippable; the application remains
  releasable throughout.
- The decoupling work (Search/view, filters-as-data) preserves the option to
  revisit QML later at a lower cost, but does not depend on that ever
  happening.
- No packaging, theming, or dependency changes are required.

Negative:

- The UI remains Widgets-based; any modernization of the visual layer is
  postponed.
- Some cleanup work (e.g. controller extraction from `MainWindow`) has a lower
  payoff without a second UI consumer, and is therefore scoped down.

## Follow-Up

- Top-level plan: `docs/cleanup/plan.md` (completed July 2026 and retired;
  see git history)
- Investigation findings that motivated this decision:
  `docs/cleanup/findings.md` (still live, trimmed to open findings plus a
  resolved ledger)
- Successor effort: `docs/design/items-pipeline.md`
