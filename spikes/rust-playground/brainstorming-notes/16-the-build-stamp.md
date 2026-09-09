# The build stamp, and what the target tree is really made of

Deliberation, 2026-09-09. Disposable (P1). The question the owner asked:
`acquisition-core/build.rs` watches git and injects the commit into the
core crate; everything sits downstream of core; builds are slow and the
target tree is tens of gigabytes. Understand what the stamp was doing,
why, and whether there is a simpler and more idiomatic way to get the
same ends at this scale.

Everything below was measured on the owner's machine on 2026-09-09
(cargo 1.94.1, arm64 macOS, tree clean at `e5c3b46a`). Nothing was
changed in the repo; the tree was left clean and the binary at HEAD.

## 1. What was measured

| Experiment | Result |
| --- | --- |
| `touch .git/index`, nothing else changed, `cargo build` | core, plan, cli, mcp all recompiled: 23 s warm |
| append a comment to `core/src/lib.rs` (unstaged), `cargo build`, `acq --version` | `0.0.1 (e5c3b46a8eb0)` — no `-dirty` |
| `cargo test --workspace --all-targets --no-run` after that core change | 30 s warm; 21 executables relinked |
| `target/debug/deps` by extension | 600,127 `.o` files = 65 GB logical; 368 rlibs = 1.6 GB; 120 executables = 1.4 GB; rmeta 0.6 GB |
| `target/debug/incremental` | 57 `acquisition_core-*` and 64 `acq-*` variant directories, 8.7 GB |
| `.git/HEAD` mtime | Aug 31 — commits write `refs/heads/…`, never `HEAD` |

One measurement is contaminated and is not used: a first timed build
took 5 m 45 s, but a listing of the 600k-file deps directory ran at the
same time and starved it of I/O (5 % CPU). The warm figures above are
the honest ones.

### 1.1 Any index write rebuilds the workspace, stamp value or not

Cargo re-runs a build script when a `rerun-if-changed` path's mtime
moves, and it recompiles every dependent of a crate whose build script
re-ran — even when the script printed the same `rustc-env` line. The
index moves on every commit, `git add`, `git stash`, `git checkout`, and
on any `git status` that refreshes stat data. Agent sessions run git
constantly. The `HEAD` watch is inert for commits (its mtime is Aug 31);
the index watch does all the triggering.

So the cost is not "one rebuild per commit". It is one workspace
rebuild per git index write, which in an agent session is many times an
hour, and each one is followed by a test-set relink at the next
`cargo test`.

### 1.2 The `-dirty` marker is false exactly when it matters

The script only re-runs when the index moves. Editing a tracked source
file does not move the index. So the common case — edit `daemon.rs`,
`cargo build`, run — produces a binary that carries the *clean* commit
hash while containing uncommitted code. The live-run drivers
(`tracer-rung.sh`, `persist-check.sh`) refuse on `-dirty`; they would not
have refused that binary. What actually protects the runs today is the
drivers' separate `git status --porcelain` over the rung's own files
(`tracer-rung.sh:162`), which reads the checkout, not the binary.

The rung-8 class of mistake (a stale binary on a fresh checkout) is
caught by the stamp only in the *other* direction — a binary older than
HEAD — and only when the human remembers to compare.

### 1.3 The disk is not the stamp's doing

- **65 GB of `.o` files.** The dev profile on macOS defaults to
  `-C split-debuginfo=unpacked` (visible in every rustc command line).
  rustc leaves each codegen unit's object file beside the artifact so the
  debugger can find DWARF without a dSYM. They are orphaned with their
  variant and never deleted.
- **The variant directories cluster on `Cargo.lock` commits**, not on
  commits in general: the 12 lock-file commits (Aug 16, 20, 24, 29, 30,
  Sep 1 ×3, Sep 4, Sep 7 ×2) line up with the incremental clusters
  (4–5 directories at the same minute = one `cargo test --all-targets`
  producing lib, lib-test, and the feature variants below; then the
  check-mode set from clippy). The toolchain has not changed since March.
  Cargo never deletes an orphaned variant.
- **The whole workspace is built twice per configuration.** The store's
  `test-hooks` feature gates one five-line function (`break_for_tests`,
  `jobs.rs:122`, five callers in `daemon.rs` tests). Core's dev-dependency
  turns it on, so `cargo build` compiles store, core, plan, cli and mcp
  without it and `cargo test` / `clippy --all-features` compile all five
  again with it, under different metadata hashes. A side effect: the
  `acq` the integration tests run (`CARGO_BIN_EXE_acq`, `acq-be71bc…`) is
  a different build from `target/debug/acq` (`acq-297ce0…`).
- **14 integration-test files = 14 full-stack links** of ~34 MB each,
  run serially. Their total size (part of the 1.4 GB) is not the disk
  problem; their link time and serial execution are part of the
  iteration time.

## 2. What the stamp was for — four needs in one value

Reading the consumers (`lib.rs` `BUILD` / `VERSION_WITH_BUILD`,
`client.rs` handshake, `daemon.rs` `Hello` + startup line, `rails.rs`
journal header, `main.rs` `--version`, the three drivers, two skills,
the standing rule in `LIVE-TESTING.md`):

1. **Forensic provenance.** The journal header's `build` and the daemon's
   startup line say which code produced the evidence a ledger row cites.
   Born at rung 8 (`9704f7d6`, 2026-08-24).
2. **A procedural gate before spending GGG requests.** The drivers refuse
   a binary that is not clean HEAD. Also rung 8.
3. **Daemon/client compatibility (C10).** The handshake replaces a daemon
   from another build, because the package version is pinned at `0.0.1`
   and comparing it let a pre-realm daemon serve a newer client. Born at
   the realm review (`c9a07254`, 2026-09-02). It reused the stamp because
   the stamp was there.
4. **A human's "what am I running".** `acq --version`.

These want different identities:

- Needs 1 and 2 are about the **checkout**. Only a human or a driver is
  ever in a position to reconcile checkout with binary, and the drivers
  already read git.
- Need 3 is about **the daemon's code**. Git is a poor proxy for it: a
  docs-only commit forces a respawn; an uncommitted edit to `daemon.rs`
  does not.
- Need 4 is the **release version** in the shipped app; it needs a finer
  grain only in the playground, where the version never moves.

## 3. The cut: the binary never learns about git

### 3.1 Build identity from what cargo already tracks

Keep a build script in core, but have it hash the sources of the two
crates the daemon is made of — `acquisition-core/src`,
`acquisition-store/src`, both `Cargo.toml`s — plus `Cargo.lock`, and
declare `rerun-if-changed` on exactly those. Emit the first 12 hex of
the digest as `ACQ_BUILD`. Every one of those inputs already recompiles
core when it changes, so the stamp adds **zero** invalidation: a commit,
a stage, a `git status` cost nothing.

What this buys over "same commit":

- The identity changes when the daemon's code changes and only then. A
  dirty edit to the daemon triggers the C10 respawn (today it does not);
  an edit to the CLI or the planner does not (today it forces one).
- The "a `-dirty` stamp is the same for any dirty tree" residual is gone:
  two different dirty trees hash differently.
- `unknown` outside a checkout is gone; a tarball builds the same id.
- Both frontends built from one tree still agree, so the MCP's "is this
  my daemon" answer is unchanged. (`acq-mcp` must be able to accept a
  daemon it did not spawn — C6, C31 — which is why the identity has to be
  a property of the shared crates, not of the executable file.)
- The journal `build` field, `soak-check.sh` (which compares the frozen
  binary's `--version` to the journal's `build`), and the daemon log keep
  working unchanged. Only the string's *meaning* changes: content id, not
  commit.

Cost: `sha2` as a build-dependency (already in the graph; a small
host-side compile once), and ~40 files hashed when core or store
changes. Reproducible: the same sources give the same id on any machine.

Rejected on the way:

- *Exe identity at runtime, no build script at all* (hash or stat your
  own executable at startup, compare in the handshake). Cleanest
  possible, but `acq` and `acq-mcp` are different files, so the MCP
  would refuse every CLI-spawned daemon. It would only work if every
  frontend were one binary, and the owner ruled that out on 2026-09-09:
  "there will be at least one gui and/or tui — e.g. one for a classic
  search view like the c++ app, another for queue monitoring, and
  possibly others. it will all stay within the bounds of this repo, but
  we may have other binaries." So the identity must be a property of
  the shared crates, and 3.1 stands.
- *A hand-bumped protocol/schema version.* Discipline where P5 wants
  structure; forgetting the bump is the silent failure C10 exists to
  prevent.
- *Stamp in the binary crates instead of core.* Halves the damage (two
  bin recompiles per commit instead of everything) but keeps git in the
  build, keeps the index-write churn, keeps the dirty lie, and needs
  plumbing to push the value into core's handshake.

### 3.2 Provenance from the driver, by prevention rather than detection

The drivers already run `git rev-parse` and `git status`. Change the
preflight from *"the binary's stamp must equal HEAD and not be dirty"*
to:

1. refuse a dirty tree (the existing check over the rung's files, kept);
2. refuse a running daemon (existing);
3. `cargo build` — cheap when fresh, and after step 2 it is safe under
   the standing rule;
4. write HEAD, the tree state and `acq --version` (the build id) into
   the run directory and the ledger row.

A binary that predates the fix cannot then exist for a driven run, which
is stronger than detecting one. The hand procedure in the live-run skill
stays "cargo build, then run". The standing rule "verify the binary, not
the checkout" becomes "the driver builds and records; a journal's
`build` is the id the run record maps to HEAD".

### 3.3 Production: the identity is the package version

For a monolithically distributed app the daemon and every frontend ship
in one bundle at one version. A daemon from another release is replaced
on `CARGO_PKG_VERSION` alone — the idiomatic pattern, no build script.
The content id is the playground's stand-in while the version is
constant, and it costs nothing to keep beside the version later: it is
reproducible, so a release built from a tag has a stable id, and the
tag is the provenance record. Git in the build is the part not to carry
forward.

### 3.4 What is the owner's

- **C10's ruling** names "package version + git commit"; it would be
  amended to "the daemon's content id (core + store sources + lock)".
- **The standing rule** in `LIVE-TESTING.md` ("verify the binary, not
  the checkout") changes wording as in 3.2.

Everything else is internals under the quality gate.

## 4. The disk and the iteration time, separately

Independent of the stamp; each is one line or one move:

| Change | Effect |
| --- | --- |
| `[profile.dev] split-debuginfo = "packed"`, then one `cargo clean` — **landed 2026-09-09** | the 65 GB of `.o` files stop being produced; one `.dSYM` (~113 MB) per executable instead |
| retire the `test-hooks` feature; inject the queue failure at the boundary (make `daemon.db` unwritable from the test, or `#[doc(hidden)]` the hook unconditionally) | one artifact set per configuration instead of two; the tests run the same `acq` as `target/debug/acq` |
| one integration-test binary per crate (`tests/main.rs` + `mod` files) | 14 full-stack links become 2; cases run in parallel threads |
| periodic `cargo clean` until stable `cargo clean` gc lands | the lock-file orphans; far cheaper once the `.o` files are gone |

Measured after the clean, same machine, same commit: `cargo clean`
removed 668,902 files (83.9 GiB logical, 37 GB allocated); a full build
of all ~250 packages from nothing took 22 s and the `--all-targets`
test set 16 s, against 18–30 s for the *incremental* four-crate rebuilds
measured before the clean; `target/` is 3.0 GB with zero `.o` files and
23 `.dSYM`s. The bloated tree was itself a large part of the slowness;
which part (file count, unpacked object files, or machine load at the
time) was not isolated.

None of these changes what a test pins. The tests' own run time (process
tests spawning daemons and waiting on idle timers) is out of this note's
scope by the owner's call, 2026-09-09: a separate discussion.

## 5. Open for review

- Is `core/src + store/src + Cargo.toml ×2 + Cargo.lock` the right input
  set for "the daemon's code"? (The daemon links neither `plan` nor the
  frontends — C39 — so those are excluded on purpose.)
- Should the journal header also carry the exe's own content hash as a
  second forensic field, or is one id enough? (Lean: one; the run record
  carries the rest.)
- `--version` shape: `0.0.1 (a1b2c3d4e5f6)` as today, or name the kind
  (`build a1b2…`)? The drivers parse the parenthesised part.
- Order of landing: profile + clean first (mechanical, no ruling), then
  the stamp + driver change (needs the C10 amendment), then the feature
  and test-binary consolidation.
