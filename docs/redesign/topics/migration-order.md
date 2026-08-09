# Migration order and the UI/state boundary

Status: proposal, unreviewed
Provenance: distilled from Auro's multi-agent workflow output
(2026-08-08), based on his repository research and notes from the
August 2026 call. Measured claims cite the M3 benchmark docs and the
PR #192 control benchmark; the IPC/latency/RSS figures for the Tauri
path are engineering estimates pending the webview-scale spike.

## The central move

The three redesign tasks (decouple UI from state, structured search
engine, UI rewrite) collapse into one migration with a single early
decision: **define the UI/state boundary as a windowed query
protocol** — query in → (generation token, count, windowed rows) out,
plus a command path for buyout edits. This one artifact is
simultaneously the decoupling line, the search engine's API, the web
UI's wire format, and the future C++/Rust seam. It makes the Rust
question deferrable instead of blocking.

## Recommended order

1. **A0 — Freeze the boundary as a protocol, not a refactor.** Extend
   the PR #192 control layer (`ControlService` + view projection) into
   the canonical read-side contract. *Note: that layer exists only on
   the PR #192 branch — reviewing and merging PR #192 is the real
   step zero, and its contract deserves review rigor proportional to
   becoming the permanent seam.*
2. **A1 — Split `Search` into a non-Qt query core and a Qt view
   adapter.** The matchers are already widget-free (`src/filters/`),
   but `Search` is a fused query-engine + view-model owning
   `ItemsModel`, expansion/scroll state, and delta application;
   `MainWindow` owns the searches and routes deltas
   (`src/ui/mainwindow.cpp` `OnTabRefreshed` /
   `OnChildrenReconciled`). Highest-correctness-risk step — the code
   encodes spec-cited invariants (R1-7, D1–D4) — so do it while the
   existing engine and Qt tests still gate behavior.
3. **B — Build the structured query engine in Rust from day one**,
   behind the A0 boundary via a thin C ABI. AST with AND/OR/NOT and
   "like this item" templates, plus a compiled per-item predicate —
   non-negotiable because the streaming delta path evaluates filters
   per arriving item. The engine core (parser, AST, index) has zero
   UI coupling and can start immediately, overlapping A1; only
   integration waits. The open ~0.4 s full-refilter-at-1M finding
   lands here. *The C ABI seam is throwaway scaffolding (serialization
   at the seam, error mapping, MSVC+Rust packaging on three
   platforms); it is deleted once the Rust core becomes the host
   process — a bridge paid for twice, knowingly.*
4. **C-spike (parallel, ~2 days)** — prototype Tauri + TanStack
   Virtual against 1M synthetic items over the windowed protocol, on
   both WKWebView (macOS) and WebView2 (Windows). Cheaply falsifies
   or confirms the UI plan before anything is committed. Tracked in
   the spike register as `webview-scale`.
5. **Rust core expansion — strangler-fig, not big-bang.** Once B's
   engine is load-bearing, grow Rust outward: item store/index, then
   per-tab hydration. Lazy loading of recursive map tabs lands here
   as lazily-materialized index segments — not decided earlier,
   because whether unloaded tabs are searchable is the engine's
   storage decision.
6. **C — UI rewrite last, as a thin windowed view.** By then the
   boundary is proven, the engine is fast, and the Qt UI has been
   running against the same protocol — a frontend swap, not an
   architecture change, retirable feature-by-feature.

## The Tauri question: conditional yes

Tauri + TS + TanStack (Effect undecided, see open questions) can beat
the current app's *perceived* performance, because the measured
bottleneck is the data/model layer, not Qt's renderer: ~13 s to move
975k items across a process boundary, ~0.9 s broad refilter at 1M
(measured). With the query engine resident in the core and only
~10–50 KB visible windows crossing IPC, keystroke-to-rows is estimated
well under 50 ms (estimate). Conditions:

- **No endpoint may ever return all matching items across IPC.** One
  careless API reintroduces the 13-second path. This should become a
  written standing constraint on the protocol, not a convention.
- It will not beat a well-rebuilt Qt UI on the same Rust core in
  absolute terms (~200–400 MB webview overhead, slightly higher
  latency floor — estimates), but that gap is below the
  feels-instant threshold, and the web stack wins decisively on
  iteration speed and agent-authored UI.
- Tauri over Electron: with a Rust core, Electron's advantages are
  irrelevant and it adds ~100–200 MB baseline. Tauri's real cost is
  webview inconsistency across platforms — hence the two-platform
  spike.

## Supporting findings

- **1M items fits fully resident today** — the M3 benchmarks measured
  223.9 MB key overhead against a 300 MB budget. Lazy loading is an
  optimization, not survival. If "millions" means 3–5M, measure
  first.
- **The delta pipeline (M1–M3) is done** and merged to `master`
  (PR #188; verified 2026-08-08). A wholesale rewrite would discard
  just-finished, measurement-gated C++; the strangler approach around
  the A0 protocol preserves it. AGENTS.md's "in progress" wording is
  stale.
- **The PR #192 control layer is the natural seed of the whole
  architecture** — already the projection/protocol prototype this
  plan builds on (consistent with ADR 0003's Inputs section).

## Open questions

- ~~Shop/forum write path: its coupling to `Search` was not traced by
  the original analysis; the plan is read-path-heavy and the write
  side is unexplored.~~ Answered 2026-08-08: no `Search` coupling at
  all; the write side is a small A0 addendum, not a second protocol.
  See `topics/shop-write-path.md`.
- Credential custody (OAuth + POESESSID) under a Tauri-style security
  model: unexamined here; covered by the `credential-custody` spike
  candidate.
- Effect (v3/v4): not load-bearing in this plan; keep on trial until
  a spike shows what it buys a two-person project.
- ~~Whether the A0 protocol needs a formal versioning/compatibility
  story before the Qt UI and a spike UI consume it concurrently.~~
  Answered 2026-08-08 by the step-zero review
  (`topics/control-contract.md`): the versioning story exists —
  exact-match envelope version, additive-evolution rule, verified
  rejection of unsupported versions (R1-8). The sharper remaining
  question is the notification path: v1 is strictly poll-based
  request/response, which a live UI consumer cannot use as-is
  (R1-7). That design item, plus a windowed-read command (R1-1),
  is what A0 still owes before two consumers exist.

## Verification notes

Cross-checked against the working tree on 2026-08-08 (Claude,
gerwaric's session): the `MainWindow` delta-routing description and
invariant citations match the code; M3's merge to master is confirmed
(the original analysis had left it unverified); `src/control/` does
not exist on `master`/`redesign` — it lives only on the PR #192
branch, hence the step-zero note under A0.
