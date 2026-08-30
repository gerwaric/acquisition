# Rust playground — agent entry point

You are in `spikes/rust-playground` on branch `spikes/rust-playground`: a
Cargo workspace (`acquisition-core`, `acquisition-cli`) that is the
**reference implementation** of the Rust daemon and rate limiter. The
repository-level `AGENTS.md` describes the C++ app on `master`; its build
and Qt guidance does not apply here.
ADR 0003 (rewrite vs. evolve) is the owner's call and needs nothing from
you — ignore it and `docs/redesign/`.

Read, in this order, before changing anything:

1. `README.md` — what exists, how to run it, knobs, known gaps.
2. `CONTEXT.md` — invariants, decisions with rationale, open topics,
   deferred work, and the working style. Owner (Tom) holds the boundaries
   (invariants, protocol, core API surface); agents own internals.
3. `LIVE-TESTING.md` — the control document for anything that touches the
   real GGG API. **Nothing here talks to GGG unless `ACQ_GGG=1` is set;
   never set it without the ladder rules in that file.** Live runs are
   human-run, from a terminal (a cron-spawned daemon has no keychain), and
   the ladder is closed: a new live run needs a new hypothesis written
   there first.
4. `TESTING-NOTES.md` — how the project checks its own work; the send
   journal is the contract surface.

Quality gate, kept green by every change (`NETWORK-CLEANUP.md`):
`cargo test --workspace --all-targets`, `cargo clippy --workspace
--all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`,
`git diff --check`.

Working against the mock daemon in a session: use `ACQ_SOCKET=<short
path>` and `ACQ_NO_KEYRING=1` to stay isolated, `ACQ_STORE_DIR=<scratch>`
so the shared store does not land under `~/.local/share`, and `acq daemon
stop` when done. A scripted mock login is `acq auth --no-browser` in the
background, then `curl` the printed URL with `/authorize?` replaced by
`/approve?`. macOS has no `timeout`; long holds (up to 300 s + 60 s) are
the limiter working, not a hang.

Facts about GGG live in `../../docs/design/network-ground-truth.md` and are
cited by claim number; new claims are authored on the master-side branch
and cherry-picked here, never the reverse.
