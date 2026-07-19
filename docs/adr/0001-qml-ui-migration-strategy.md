# ADR 0001: Incremental QML UI Migration Strategy

## Status

Superseded by ADR 0002 (July 2026). Originally proposed on the
`prepare-qml` branch; merged into the main tree July 2026 for the record,
with only this status section updated. Its companion implementation plan
(`docs/qml-migration-plan.md`) remains on that branch.

## Context

Acquisition is currently a C++23 Qt Widgets application. The project has a
substantial amount of reusable C++ application logic, including data storage,
Path of Exile API access, item parsing, buyout management, rate limiting, image
caching, shop updates, currency tools, logging, and update checks.

The UI layer is not a thin shell over that logic. Important workflow behavior is
implemented directly in widget code:

- `MainWindow` owns the main workflow orchestration, search tabs, item
  selection, buyout editing, menu actions, status widgets, tooltip commands, and
  service coordination.
- `Search` owns query state, filtered buckets, the item model, and direct
  `QTreeView` state such as model activation, sorting, expansion, and collapse.
- `Filter` subclasses construct `QWidget` controls directly and read/write form
  state from those widgets.
- Item tooltip rendering mutates generated `Ui::MainWindow` widgets and renders
  the tooltip widget for image upload.
- The current light/dark themes use qdarkstyle and application-wide QSS, which
  applies to Widgets but does not translate directly to QML.

The desired outcome is to move the UI to QML while preserving internals and the
existing UI/UX as much as practical, so user impact and regression risk stay low.

## Decision

Migrate incrementally. First extract stable C++ controller and view-model
boundaries from the existing Widgets UI, keep the Widgets UI working against
those boundaries, then introduce QML screens that use the same interfaces.

The QML implementation should be idiomatic QML rather than a mechanical clone of
the current `.ui` files. Business behavior should remain in C++ services,
controllers, and models. QML should own presentation, layout, and interaction
binding.

## Alternatives Considered

### Rebuild From Scratch Around QML

A rewrite could produce a cleaner first QML architecture and avoid carrying
forward some widget-era structure. However, it would also require rediscovering
many undocumented behaviors embedded in the current UI code, including search
tab behavior, item selection recovery after filtering, refresh-check state,
buyout editing rules, tooltip behavior, shop update flows, login settings, and
rate-limit status handling.

Because the goal is continuity rather than redesign, a scratch rebuild has high
regression risk and would likely duplicate business logic in QML or require the
same C++ extraction work anyway.

### Direct Mechanical Port From Widgets to QML

Recreating every widget and layout directly in QML would preserve some visual
structure, but it would also preserve the current coupling problems. The result
would likely be a QML UI that still depends on widget-shaped assumptions.

### Keep Qt Widgets Indefinitely

Keeping Widgets avoids migration cost and packaging changes. It also leaves the
current UI coupling in place and makes future UI modernization harder.

## Rationale

Incremental migration fits this codebase because the core services are already
mostly C++/Qt objects, while the UI behavior needs careful extraction. Keeping
the Widgets UI working during that extraction provides a behavioral reference
and lets the application remain shippable throughout the migration.

This approach also lets each boundary be validated before QML depends on it:

- Main workflow commands and state can be extracted from `MainWindow`.
- Search and filtering can be separated from `QTreeView` and widget controls.
- The existing `QAbstractItemModel` item tree can be adapted or wrapped for QML.
- Tooltip data can be exposed independently of the current generated widget UI.

## Consequences

Positive consequences:

- Lower user-facing regression risk.
- Existing internals can be preserved and tested through both UI implementations.
- The current Widgets UI remains a parity reference while QML is introduced.
- QML can be designed around clean C++ interfaces instead of generated widget
  structures.

Negative consequences:

- Widgets and QML may coexist for a period of time.
- Some work must happen before visible QML progress, especially around `Search`
  and `Filter` decoupling.
- QML will require separate theme work instead of relying on qdarkstyle/QSS.
- CMake and release packaging must be updated to include Qt QML/Quick modules
  and deployed QML imports/plugins.

## Follow-Up

Maintain the implementation details, phase checklist, risks, and validation
steps in `docs/qml-migration-plan.md` (on the `prepare-qml` branch).
