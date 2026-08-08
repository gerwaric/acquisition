# ADR 0003: Rewrite or Evolve the Core and UI

## Status

Proposed (August 2026). Under active exploration on the `redesign`
branch, in `docs/redesign/`; that directory holds the living option
analysis, spike register, and spike results, and is deliberately kept
off `master` while it churns. This document records only the question,
the option space, and what remains authoritative in the meantime. When
a decision is reached, this file will be rewritten with the decision
and its rationale, and the surviving analysis will land with it.

Until this ADR is accepted, nothing changes: the items-pipeline and
network-redesign specs in `docs/design/` remain full authority for all
work on `master`, and maintenance and releases of the current
application continue normally.

## Context

ADR 0002 deferred the QML migration and chose an interior design
cleanup instead. That cleanup and the milestone work that followed
(items pipeline M1–M3, network redesign phases 1–5) had a side effect
that changes the option space: most of Acquisition's hard-won knowledge
now lives outside the code, in citable documents —

- `docs/design/network-ground-truth.md`: numbered, evidence-cited
  claims about how the Path of Exile API actually limits requests;
- `docs/cleanup/findings.md`: the project-wide register of design and
  correctness findings, open and resolved;
- the frozen specs and their review histories, which record accepted
  decisions alongside rejected alternatives and why.

The classic argument against rewrites — that the old code encodes
years of invisible fixes a new implementation would rediscover the
hard way (ADR 0001 made exactly this argument against a scratch QML
rebuild) — is therefore weaker here than it usually is. A new
implementation in any language inherits every ground-truth claim and
finding unchanged.

Two further inputs prompted reopening the question ADR 0002 deferred:

- PR #192 (August 2026) proposes `acquisitionctl`, a versioned local
  control plane over the current core, making Acquisition drivable by
  scripts and coding agents. Whatever happens to its C++
  implementation, its contract — a normalized item projection,
  revision-safe pagination, and an idempotent refresh-operation
  lifecycle — is a draft of what any future core's interface could
  look like, and treats agent access as a first-class surface.
- The maintainer and the PR author agreed (call, August 2026) that
  both evolving the current C++/Qt codebase and replacing it outright
  are acceptable outcomes, provided the externalized knowledge carries
  forward.

## Question

What architecture, and consequently what stack, should carry
Acquisition forward: the existing C++23/Qt Widgets application evolved
in place, or a new implementation that inherits the documented
knowledge but not the code?

Options under exploration (details and current standings live in
`docs/redesign/` on the `redesign` branch):

1. Evolve in place — keep the C++/Qt codebase and continue
   incremental redesign under the existing spec process.
2. Ground-up rewrite — new core and UI on a new stack (for example a
   Rust core with a webview UI; illustrative, not a commitment).
3. Hybrid — a new core exposing a contract shaped like PR #192's,
   with the UI decision made separately against that contract.

Decision drivers include: single-gate rate limiting per the
ground-truth claims; UI performance at 100k–1M items; OAuth and
POESESSID custody; migration of existing users' data; multi-platform
packaging and updates; the cost of a dual-track period in which the
current application still needs maintenance while a replacement
gestates; and agent accessibility as a first-class requirement rather
than an afterthought.

## Consequences

Deferred until a decision is recorded. The immediate consequences of
proposing this ADR are procedural: exploration happens on the
`redesign` branch; spikes are cut from it on `spike/*` branches and
are never merged, with only distilled result documents landing back
in `docs/redesign/`; and `master` is unaffected apart from this
document and its index entries. Branch and spike conventions are
recorded in `docs/redesign/README.md` on the `redesign` branch.

## Follow-Up

- Living exploration: `docs/redesign/` on the `redesign` branch.
- Control-plane contract input: PR #192 and
  `docs/design/local-control.md` on its branch.
- Predecessor decisions: ADR 0001 (superseded), ADR 0002 (accepted;
  its cleanup outputs are what make this ADR's option space viable).
