# Rust playground — agent entry point

You are in `spikes/rust-playground` on branch `spikes/rust-playground`: a
Cargo workspace (`acquisition-core`, `acquisition-store`,
`acquisition-plan`, `acquisition-cli`, `acquisition-mcp`) that is the
**reference implementation** of the Rust daemon and rate limiter. The
repository-level `AGENTS.md` describes the C++ app on `master`; its build
and Qt guidance does not apply here. ADR 0003 (rewrite vs. evolve) is the
owner's call and needs nothing from you — ignore it and `docs/redesign/`.

## Read before changing anything

Always, in this order:

1. `README.md` — what exists, how to run it, knobs, known gaps.
2. `CONTEXT.md` — invariants, the cross-cutting decisions, the index of
   the area decision files, the cross-cutting parked items, and the
   working style.
   Owner (Tom) holds the boundaries (invariants, protocol, core API
   surface); agents own internals.

Then, only when the work touches it:

- Before anything that talks to the real GGG API: `LIVE-TESTING.md` (the
  standing rule, the rails, the run ledger) and the live-run procedure
  below. **Nothing here talks to GGG unless `ACQ_GGG=1` is set; never set
  it outside that procedure.** Live runs are human-run, from a terminal.
- Before touching tests or the harness: `TESTING-NOTES.md` — the send
  journal is the contract surface; tests pin boundaries, never mechanisms.
- Before touching the network layer: `NETWORK-CLEANUP.md` (closed record).
- Before touching an area, its decisions and what it has parked: `decisions/daemon.md` (daemon,
  jobs, protocol, accounts), `decisions/network.md` (limiter, gate, rails,
  OAuth traffic), `decisions/store.md` (ingest, facts, realm, characters),
  `decisions/plans.md` (sync policy, planner, quote, apply),
  `decisions/pricing.md` (buyout intent, listing state, currency table,
  price plans, import, render), `decisions/frontends.md` (CLI, MCP,
  rendering). Each module's doc names its file and ids at the top.
- Before touching the store, planner, or plan slice: `REFRESH-SLICE.md`
  (closed record; its findings table is the review checklist) and the
  crates' module docs (`src/lib.rs`, "As built").

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

A procedure becomes a skill after it has run twice and repeated a trap,
not before.

## Quality gate, kept green by every change

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
git diff --check
tools/docs-check.sh      # byte budgets on the always-loaded documents; stale identifiers
```

`cargo test` and `cargo clippy` do not rebuild `target/debug/acq`; run
`cargo build` and check `acq --version` before any smoke or live run.

## Routing: one authoritative home per fact

The store's rule applies to the documents: every fact has one source, and
a copy elsewhere is a parallel description that rots. When a document's
budget trips, something landed where it does not belong.

| Kind of fact | Home |
| --- | --- |
| a ruling, invariant, or boundary property | the registry: one entry, a stable `C<n>` id, the ruling verbatim, *Why:*, and pointers (under the check's length limit) — in `decisions/<area>.md`, or in `CONTEXT.md` only if every area must know it |
| parked scope, or a question with no ruling yet | the area's `decisions/<area>.md`, "Parked", one entry with its trigger — in `CONTEXT.md` only if it crosses every area; a fired trigger deletes the entry |
| a property pinned by a test | the test cites the decision id (`c6_…`, or a comment); the entry's *Pinned:* names the file |
| a review finding | a row in the slice's closed record, with its fix commit |
| a build step's narrative | the commit message |
| a live run | one run-ledger row; journals in `runs/` (gitignored; mock rehearsals under `runs/mock/`) |
| a fact about GGG | a numbered ground-truth claim |
| how a mechanism works | a doc comment on the code, under "Decisions as recorded" / "As built", headed by the id |
| a procedure | its skill file, referenced here |
| deliberation | a numbered note in `brainstorming-notes/`, disposable |
| the owner's verdict | recorded verbatim from the conversation, marked as such |

Headers carry no status. Struck-through items are deleted. Session notes
are history, never a second authority.
