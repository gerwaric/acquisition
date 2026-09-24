---
name: session-close
description: End a working session on the Rust playground — route what was learned to its one home, keep the always-loaded documents at budget, commit with the story in the message. Use before the last commit of any session, and whenever a slice closes.
---

# Session close

The always-loaded documents accrete when a session ends by writing its
story into `CONTEXT.md`. This procedure gives the story a home instead.
The rule is one authoritative home per fact (`AGENTS.md`, "Routing");
the ladder behind it is note 09 at `decda84e`.

## 1. Gate

Run the quality gate in `AGENTS.md`, "Quality gate".

Run each command bare and read its exit status. A check piped through
`grep` or `tail` reports the filter's exit, not the check's: on
2026-09-03 an over-length registry entry was committed that way.

### Why build order matters

`cargo test --all-targets` never writes `target/debug/acqd` (no test in
the daemon crate asks for it) and rewrites `target/debug/acq-mcp` in its
all-targets form — a different artifact: proptest, the plan crate's
dev-dependency, unifies `num-traits/std` into the chrono that rmcp links
(measured 2026-09-11; `acq` and `acqd` have one form each since the
store's `test-hooks` feature and its dev-only `hooks` feature went);
`cargo clippy` rewrites none. So run the gate's `cargo build --workspace`
before any smoke or live run, and never run the drivers while the gate
runs — their preflight build replaces `acqd` under a live daemon (an
artifact mismatch, C84). The contract revision hashes the protocol
crate's sources, so an edit or a `cargo fmt` there after the build
leaves `acqd` on the old revision and every daemon test failing "another
contract": build again first (2026-09-12).

The full contract-input set is documented in
[the revision build script](../../../crates/acquisition-protocol/build.rs).

## 2. Route each thing learned

Route every item through `AGENTS.md`, "Routing: one authoritative home
per fact"; the registry format and park lifecycle are in `CONTEXT.md`.
`REFRESH-SLICE.md` is the shape for a finding row; `LIVE-TESTING.md`
defines the run-ledger row. Usage references regenerate from their
sources (commands in each reference's header); `tests/readme_tour.rs`
and `tools/docs-check.sh` hold README's form.

Do not write "built on <date>", "step N done", or a list of what a test
covers into `CONTEXT.md`: git holds the first two, the test the third.
`tools/docs-check.sh` reports decisions nothing cites; when you touch the
code behind one, name it in a test or a doc comment so the report shrinks.
A help string is a doc comment the user reads: cite the id there too.

## 3. When a slice closes

Cut its `CONTEXT.md` section to rulings, properties and pointers; cite
the last full-text commit in the section; give the slice a closed
record in the mold of `NETWORK-CLEANUP.md` (step ledger, the shapes of
fault its reviews taught, what the runs taught, observations still
open). Strike-through items in
a "Parked" list are deleted, not kept; a park whose trigger fired goes with them.

## 4. Commit

One commit per concern. The message carries the narrative — what
changed, what the review found, what the run showed — because that is
where the next session will read it (`git log`, the slice's range). A
size stated in the message is measured with `wc -c` after the last
edit, never recalled (four amended messages, 2026-09-16).
