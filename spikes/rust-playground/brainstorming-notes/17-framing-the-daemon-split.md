# Framing for the daemon-split design session

**Written 2026-09-09, before the session**, in the mold of `00-framing.md`:
read this first, then `16-the-build-stamp.md` §6.6 and §7 (where the
split was ruled), then `../CONTEXT.md` and `../decisions/daemon.md`,
`../decisions/frontends.md`. It records the owner's stances, the settled
floor, what was measured, the seeds worth chasing, the gravity to resist,
and the shape the session's output takes. Disposable (P1): what it
proposes is real only as a ruling in the registry or as code under the
gate.

## The goal function

The owner's words, 2026-09-09, on whether every frontend must be able to
become the daemon: "omg yes — embedding the daemon sounds terrible
compared to `acqd`. the path we took was an unplanned shortcut. time to
grow up." And on what is coming: "there will be at least one gui and/or
tui — e.g. one for a classic search view like the c++ app, another for
queue monitoring, and possibly others. it will all stay within the bounds
of this repo, but we may have other binaries."

So the session is not "move `daemon run` into its own binary". It is:
**what is the daemon, as a thing every frontend shares, once it is its
own artifact?** The system is a daemon and thin frontends (C2); until
now the daemon was a hidden mode of each frontend and the boundary
between them was "whatever `acquisition-core` exports". The split makes
that boundary a crate and a wire, which is the one surface this project
has never pinned (`TESTING-NOTES.md`, "Before any fresh build": "Pin the
frontend boundary — after the consumer has validated the protocol"; the
plan slice is pinned at process level, "the protocol as a whole is
not"). The GUI, the TUI, the MCP server, an agent in a shell, and a cron
sync (C34: "scheduled syncs are small frontends") all arrive to whatever
this session draws. Draw it as the boundary of the product, not as the
leftover of a refactor.

## The settled floor (not up for debate here)

Naming what is settled is what makes it safe to be radical elsewhere.

- The five invariants; a daemon owns all GGG traffic; a frontend
  consumes exactly two surfaces — the daemon protocol and the store's
  read API — and no third door (C12; the 2026-08-31 stress test in 00).
- C10 as amended and built: kill-and-respawn is the whole migration
  mechanism, the protocol is single-version, a client uses only a daemon
  whose provider and runtime identity match what it would spawn, and
  observation never spawns or replaces. The three rulings of note 16
  §7: `acqd` is the shape; the identity comes apart into two values
  after the split; the runtime-revision build script migrates to the
  protocol crate rather than being thrown away.
- C3 as amended: lazy spawning is a use-verb policy; a frontend spawns
  only where its own policy permits (C13: the MCP never in real mode).
- C34's daemon blindness: the daemon reads no intent, runs no schedule,
  creates work only in service of client-submitted work.
- ADR 0003 is the owner's and is not this conversation; nothing here
  anticipates shipping. But P3 gives irreversible identity and
  compatibility choices first-consumer treatment, and a crate boundary
  that every later binary links is one.

## What was measured (2026-09-09, tree at `184ff276`)

The ground the session stands on, so it argues from the code:

- **Who imports what from core.** The planner: `realm::{Realm, Family}`,
  `protocol::{Quote, QuoteJob, QuoteScope}` — and `daemon::MAX_429_RETRIES`,
  the one reach into the daemon module (`acquisition-plan/src/lib.rs`).
  The CLI: `client`, `protocol`, `job`, `realm`, `provider::ggg_mode`,
  the version constants — and `daemon::{socket_path, log_path}`,
  `rails::RailsStatus`, `ratelimit::{PolicyStatus, SendRecord,
  DegradedEndpoint, WindowStatus}` for `daemon status` and `dash`. The
  MCP: `client`, `protocol`, `realm`, `provider::ggg_mode` — and
  `daemon::run`, the embedded daemon. So the frontend-facing surface is
  already small and mostly typed; what leaks is the daemon module's
  paths, one constant, and the status types the dashboard renders.
- **Where the daemon is embedded.** One spawn site
  (`client.rs::spawn_daemon`: `current_exe()` + `daemon run`); one
  handler per binary (`acq daemon run`; `acq-mcp` intercepts `daemon
  run` on its argv before `rmcp`); the tests start `daemon run` from
  `CARGO_BIN_EXE_acq` (CLI) and from the MCP binary (`plan_loop.rs`).
- **The build graph today.** An edit to `daemon.rs` recompiles core,
  plan, cli and mcp and relinks every test executable, because the
  daemon and the protocol are one crate (note 16 §6.6). The store's
  `test-hooks` feature doubles the workspace build; fourteen
  integration-test files are fourteen full-stack links (§4).
- **What C10 compares now.** One value, the runtime revision over core,
  store, manifests and lock, answering both "is the wire compatible" and
  "is the newest implementation running" conservatively.

## Seeds — where "unexpectedly better" might live

- **The protocol crate is the two-surface rule made structural.** Today
  door #1 is a convention over what core exports. A crate that holds
  only the wire (`Request`/`Response`, the job model, quotes, status
  types), the client (`connect`/`observe`/`stop_any`), and the daemon
  locator — linked by every frontend and the planner, never by the
  daemon's own crate for anything but the wire types — turns "a frontend
  links protocol + store + plan and nothing else" into an edge
  `tools/docs-check.sh` can refuse (P5: a rule that can be broken
  silently becomes a dependency edge). The MCP's rule "never in-process
  with the daemon" (C13) stops being a sentence and becomes the absence
  of a link.
- **The daemon's lifecycle becomes a product surface.** With one
  artifact, "which daemon is running, where did it come from, is it the
  one I would start" is a real question with one answer, reported the
  same way to a human at `acq daemon status`, a queue-monitoring TUI,
  the GUI, and an agent over MCP. The observe tier already reports the
  three states; the split gives it an artifact to name. Consider what a
  frontend that cannot spawn (a real-mode MCP, a cron sync) should be
  told, and by whom the daemon is started on a machine with a GUI in the
  dock — the answer may be "the GUI is the use verb".
- **Two identities, each from where it is cheap.** Note 16 §6.6: "is the
  newest implementation running" is answered at runtime — the client
  hashes the sibling `acqd` it would spawn and compares with what the
  running daemon reports about itself — with no build script; "is the
  wire compatible" is the protocol crate's own source revision, the
  present build script moved to a crate whose files change rarely. Ask
  whether the second is even needed once the first exists, and what
  each costs a frontend that did not spawn the daemon (C6, C31: `acq-mcp`
  must accept a daemon `acq` started).
- **Who links `Realm`.** It lives in core and both the planner and the
  store need it; it is not protocol and not daemon. Whether it goes to
  the protocol crate, the store, or a small model crate is a design
  act, and the answer says what the protocol crate *is*.
- **The build graph as a design outcome.** After the split an edit to
  the daemon costs the daemon crate plus one link; the `test-hooks`
  retirement and one test binary per crate (note 16 §4) ride along.
  Measure before and after, as 16 did, so the payoff is a number.
- **Tests find the daemon the way frontends do.** `CARGO_BIN_EXE_<name>`
  is per package; either `acq` and `acqd` share a package, or every test
  harness locates `acqd` through the same locator a frontend uses. The
  second is a property worth having anyway: one way to find the daemon,
  used by everything.

## Questions that are the owner's

1. The C1 amendment text: the crates after the split and the rule for
   what each may link.
2. Whether `acq` and `acqd` are two binaries of one package or two
   packages — packaging and test discovery follow from it.
3. The locator rule in the playground (a sibling of the calling
   executable; an env override for isolation; nothing on `PATH`?) and
   whether it is a ruling or an internal.
4. Whether the MCP keeps its mock-mode lazy spawn once the daemon is a
   separate file, or every non-CLI frontend only observes.
5. Whether one identity suffices after the split (the artifact) or the
   protocol revision stays beside it.

## Gravity warnings

- **Refactor gravity.** The split will be tempting to do as a file move
  with new crate names. It is the frontend boundary's first pinning; the
  crate's public surface is a contract from the day the GUI links it.
  Decide the surface, then move the files.
- **Rewrite gravity.** The daemon's internals are not this session.
  Note 16 §4 and TESTING-NOTES already say what a fresh daemon build
  would keep; nothing about `daemon.rs`'s mechanics changes here.
- **Compat gravity.** Two identities is not two protocol versions. C10
  stays single-version; the split changes what is compared, never
  whether a mismatch is tolerated.
- **Shortcut gravity.** "Keep the embedded mode for the tests" recreates
  the thing the owner called terrible, one door down. If the tests need
  a daemon, they find `acqd`.
- **Shipping gravity.** Bundle layouts, installers and signed binaries
  are ADR 0003's. Decide only what the playground needs and record the
  packaging question with its trigger.

## Output shape

Artifacts, not vibes, in 06's and 10's mold:

1. **Candidate decision lines** in registry form: the C1 amendment, the
   protocol crate's boundary as a property, the locator if it is a
   ruling, the identity model — for the owner to accept, amend, refuse.
2. **A commit sequence**, each step under the gate and rehearsed with
   both drivers in mock mode: the crate split (a dependency-graph win
   with no behavior change), the `acqd` binary and the frontends
   spawning it, the identity change; then the test-binary consolidation
   and the `test-hooks` retirement; a live run last, because the spawn
   path changes under the rails.
3. **A parking lot** with triggers: packaging, a GUI-hosted daemon,
   Windows named pipes, whatever else surfaces.
4. **The measurements**, before and after, so the payoff is recorded as
   a number in the record and not as a feeling.
