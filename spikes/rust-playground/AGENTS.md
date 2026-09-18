# Rust playground — agent entry point

You are in `spikes/rust-playground` on branch `spikes/rust-playground`:
a Cargo workspace implementing Acquisition (`README.md`, the charter
and crate index). The repository-level `AGENTS.md` describes the C++ app
on `master`; its build and Qt guidance does not apply here. ADR 0003
(rewrite vs. evolve) is the owner's call and needs nothing from you —
ignore it and `docs/redesign/`.

## Read before changing anything

Always, in this order:

1. `README.md` — what exists, how to run it, knobs, known gaps.
2. `CONTEXT.md` — invariants, cross-cutting decisions and parks, area
   decision index, working style.
   Owner (Tom) holds the boundaries (invariants, protocol, core API
   surface); agents own internals.

Then, only when the work touches it:

- Before running `acq` or driving `acq-mcp`: `CLI-REFERENCE.md` and
  `MCP-REFERENCE.md` (generated semantics, checked against the binaries),
  or `acq <verb> --help`. README's tour shows only the shape.
- Before anything that talks to the real GGG API: `LIVE-TESTING.md`
  (standing rule, rails), the tail of `RUN-LEDGER.md`, and the live-run
  procedure below. **The real provider is the default (C88): never start
  a real-mode daemon from an agent shell — the CLI refuses without a
  terminal, and the mock-session skill sets `ACQ_PROVIDER=mock`.**
  Live runs are human-run, from a terminal.
- Before touching tests or the harness: `TESTING-NOTES.md` — the send
  journal is the contract surface; tests pin boundaries, never mechanisms.
- Before reading, citing or fetching a surface GGG does not sanction
  (trade site, forums, feeds): `SURFACES.md` (C79); tooling fetches need
  the permission recorded in the surface's row.
- Before touching item search: `search/README.md` — one track per
  subdirectory, with the slice's rules in force.
- Before touching an area, read its decisions and parks:
  `decisions/daemon.md` (daemon, jobs, protocol, accounts),
  `decisions/network.md` (limiter, gate, rails, OAuth traffic),
  `decisions/store.md` (ingest, facts, realm, characters),
  `decisions/plans.md` (sync policy, planner, quote, apply),
  `decisions/pricing.md` (buyout intent, listing state, currency table,
  price plans, import, render), `decisions/frontends.md` (CLI, MCP,
  rendering), `decisions/search.md` (item search: the crate,
  the line, the query, the answer, the basis, the trade boundary).
  Each module doc names its decision file and ids at the top.

Also read the applicable closed slice record below; its findings table
is the review checklist. For the refresh and daemon split, also read the
crates' module docs (refresh: `src/lib.rs`, "As built").

| Before touching | Read |
| --- | --- |
| network layer | `NETWORK-CLEANUP.md` |
| pricing | `PRICING-SLICE.md` |
| store, planner or plan slice | `REFRESH-SLICE.md` |
| protocol, client, daemon lifecycle or world | `DAEMON-SPLIT-SLICE.md` |

Facts about GGG live in `../../docs/design/network-ground-truth.md`, cited
by claim number; new claims are authored on the master-side branch and
cherry-picked here, never the reverse. A slice's history is its commit
range — `git log` — and nothing restates it.

## Procedures (loaded when needed; one home each)

- Live run: `.claude/skills/live-run/SKILL.md`
- Mock session (isolation, scripted login, rehearsals):
  `.claude/skills/mock-session/SKILL.md`
- Session close (route what you learned; run before your last commit):
  `.claude/skills/session-close/SKILL.md`
- Research track (a subagent under a committed brief, then review and
  commit): `.claude/skills/research-track/SKILL.md`

## Quality gate, kept green by every change

```sh
cargo build --workspace
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
git diff --check
tools/docs-check.sh
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Build first, and again before any smoke or live run (C82). If contract
inputs change after building — including a formatting edit — build
again before testing or running. Never run the drivers while the gate
runs (C84). The artifact explanation is in
[session-close, "Why build order matters"](.claude/skills/session-close/SKILL.md#why-build-order-matters).

## Routing: one authoritative home per fact

Every fact has one source; a duplicate description rots. When a document
reaches its budget, route the information to its home. Registry formats
and park lifecycle are in `CONTEXT.md` ("Decisions", "Parked"): use
`decisions/<area>.md`, or `CONTEXT.md` only if every area must know it.

| Kind of fact | Home |
| --- | --- |
| ruling, invariant or boundary property | the registry; next stable `C<n>`, ruling verbatim, *Why:* and pointers |
| parked scope or question without a ruling | the area's "Parked", with its trigger |
| observation without a ruling | the open slice's "Observations still open"; after closure, an area park with a trigger |
| property pinned by a test | test name or comment cites the decision id; the entry's *Pinned:* names the file |
| review finding | slice's closed record: a row with its fix commit |
| build narrative | commit message |
| live run | one `RUN-LEDGER.md` row; journals in gitignored `runs/`, mock rehearsals in `runs/mock/` |
| fact about GGG | numbered ground-truth claim |
| mechanism | code doc comment under "Decisions as recorded" / "As built", headed by the decision id; until the code exists, a rule that decides an answer lives in the area's labelled contract-detail document the registry cites (`search/DESIGN.md`) |
| verb, flag or knob usage | clap help / MCP tool description (regenerate references with `ACQ_UPDATE_FIXTURES=1`), or knob read-site doc; README gets one tour line or knob row, never a comment block |
| procedure | skill file referenced here, when earned under P6 |
| deliberation | numbered `brainstorming-notes/` note while a ruling, tool, skill or crate doc cites its path; otherwise delete, citing history as "note NN at `<commit>`" |
| owner's verdict | verbatim from the conversation, marked as such |

`tools/docs-check.sh` checks budgets, stale identifiers, registry and note
citations, README form and dependency edges (C1); it reports uncited notes
and refuses paths to deleted ones. Rustdoc checks links in code docs.
Headers carry no status. Struck-through items are deleted. Session notes
are history, never a second authority.
