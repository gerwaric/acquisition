# Redesign Exploration

Status: living exploration for ADR 0003 (`docs/adr/0003-rewrite-vs-evolve.md`,
proposed August 2026). Nothing in this directory is frozen or
authoritative; documents here are speculation, proposals, and evidence
in progress. The frozen specs in `docs/design/` remain full authority
for all work on `master` until ADR 0003 is accepted.

This directory exists only on the `redesign` branch. `master` carries
the ADR 0003 stub and index pointers, nothing else. When the
exploration converges, the surviving analysis is distilled — into the
rewritten ADR 0003 and, where applicable, specs under `docs/design/` —
and squash-merged; the churn stays in branch history.

## Layout

- `README.md` — this file: conventions and the spike register.
- `inbox.md` — raw capture point for ideas from calls and chats.
- `topics/<topic>.md` — one living document per coherent thread
  (data model, UI architecture, network core, …). Each records
  current thinking, open questions, and rejected alternatives with
  reasons, and carries a status/provenance header like every other
  doc in this repo.

## Conventions

- **Distill, don't transcribe.** No raw chat or call transcripts.
  Ideas enter `inbox.md` as dated bullets with provenance ("from call
  with Auro, Aug 6"); inbox entries are consumed into a topic doc or
  explicitly dropped — the inbox must not accumulate.
- **Docs carry their own history.** Rejected alternatives and
  reversals are recorded in the topic docs themselves (as the frozen
  specs do), never only in commit messages — the eventual squash
  merge discards commit-level history.
- **Cite, don't restate.** Ground-truth claims stay in
  `docs/design/network-ground-truth.md` (N-numbers), correctness
  findings in `docs/cleanup/findings.md` (F-numbers). Redesign docs
  cite them by number. New evidence produced here that outlives the
  exploration should graduate into those ledgers, not live here.
- **Label the lane.** Quantitative and factual claims in topic docs
  are tagged **measured** (cite the benchmark or ledger entry),
  **estimated** (name the spike or measurement that would confirm
  it), or **inferred**. Unknowns are stated explicitly, not omitted —
  a doc that's silent on a question reads as if the question doesn't
  exist.
- **Cross-review gets IDs.** When a proposal is formally reviewed
  (by either collaborator or their tooling), findings get round-scoped
  IDs (`R1-*`, …) in the topic doc, matching the spec-review
  convention.

## Spike conventions

- Spikes are cut from `redesign` on `spike/<name>` branches and are
  **never merged**. A spike ends when its distilled result doc lands
  in this directory; the branch is then dead. Do not rebase spike
  branches — they are snapshots answering a question.
- Greenfield code (anything not building on the C++ tree) lives in a
  self-contained `spikes/<name>/` subdirectory on its spike branch
  (precedent: the retired phase-0 `spikes/qcoro/`), so it can be
  hoisted to its own repository later without surgery.
- Every spike gets a register row below when cut, and its result doc
  records the branch name.

## Spike register

Candidates are the de-risking questions a rewrite decision depends on;
they become real rows (with branch names) when cut.

| Spike | Question | Status | Result |
|---|---|---|---|
| rate-limit-core | Can a Rust client demonstrably honor the N-claims in `network-ground-truth.md` under burst load, as a single serialized gate? | cut: `spike/rate-limit-core`; terminal review passed; delivery-ready | [Yes for four OAuth policies at Known(5s/60s); conditional for backend-item at Assumed(60s/60s)](topics/rate-limit-core.md) |
| webview-scale | Does Tauri + TanStack Virtual over the windowed query protocol (`topics/migration-order.md`) stay responsive at 1M synthetic items, on both WebView2 (Windows) and WKWebView (macOS)? | candidate | — |
| credential-custody | Does the existing `acquisition` public-client registration accept the full PKCE flow from a non-Qt implementation (Rust `oauth2` crate, system browser, loopback listener on an arbitrary port, `client_secret` omitted)? The custody-model question was settled by research: `topics/credential-custody.md` §7. | candidate | — |
| data-migration | Can existing users' datastores (`src/legacy/`) migrate losslessly to a new core's persistence? | candidate | — |

## Inputs

- ADR 0002 and the cleanup it chose: the externalized-knowledge corpus
  that makes this exploration viable.
- PR #192 (`acquisitionctl`): a versioned local control contract over
  the current core — normalized item projection, revision-safe
  pagination, idempotent refresh operations. Treated as a draft of
  what any future core's interface could look like, independent of
  its C++ implementation's fate.
