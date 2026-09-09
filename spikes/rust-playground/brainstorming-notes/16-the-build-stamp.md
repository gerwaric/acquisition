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
changes. The revision is a pure function of the listed files; nothing is claimed about the binary (corrected in 6.1).

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

## 6. Review round 1 (Codex, 2026-09-09) and the author's responses

The reviewer's text is with the owner; this section carries each point,
what was verified, and the response. Verified before answering:

- `acq daemon status` connects through `connect(false)` →
  `ConnectOptions::interactive(false)`: spawn off, **replace on**
  (`main.rs:36`, `:937`). The same is true of `jobs`, `status`, `result`,
  `auth status` and `cancel`/`set-priority` (`connect(false)` at 874–923).
- The 2026-09-08 ledger row records the trap: a `daemon status` typed in
  a second terminal without `ACQ_GGG` put a mock daemon on the default
  socket and the driver's cycle-1 daemon refused to start over it.
- The tracer driver is protected only because it exports
  `ACQ_NO_SPAWN=1` before its `status_json` probe (`tracer-rung.sh:198`,
  `:202`); a human at a terminal is not.
- `acquisition-plan/src/lib.rs:266` imports
  `acquisition_core::daemon::MAX_429_RETRIES`: the planner reaches into
  the daemon module, not only into `protocol` and `realm`.

### 6.1 Name it a runtime source revision — accepted

The value is not a binary identity and will not claim to be one. Its
definition: *the revision of the checked-in shared runtime sources this
process was compiled from, used conservatively to converge playground
clients and daemons.* Name: `RUNTIME_REVISION`; `--version` shape
`acq 0.0.1 (runtime 7ac91db821fd)`.

Inputs, as the reviewer lists them: the root `Cargo.toml`, `Cargo.lock`,
both crate manifests, the core and store source trees (`schema.sql` is
under `store/src` already), each file as `path NUL length NUL bytes` in
sorted path order, under a domain prefix that carries the format version
(`acq-runtime-revision/1`). `Cargo.lock` stays in: a false respawn on an
unrelated lock change is cheaper than an invisible dependency change.
Features, rustc, profile and target are out, and the doc comment says
so. `sha2` as a build-dependency is a real host-side unit; it is one
small compile per clean and it is accepted. (A dependency-free FNV-1a
would do for convergence; `sha2` is chosen so a shell one-liner can
recompute the same digest from a checkout if a future check wants to.)

Section 3.1's "reproducible: the same sources give the same id on any
machine" is corrected to: the *revision* is a pure function of the
listed files; nothing is claimed about the binary.

### 6.2 Replacement policy is too coarse — accepted, and it goes first

The finding stands on its own and has already cost a live run. The fix:
`ConnectOptions` gains an *observe* policy (spawn off, replace off) and
the observational verbs use it — `daemon status`, `jobs`, `status`,
`result`, `auth status`, the quote path (already autonomous). The
handshake's non-matching outcomes become distinct, reportable states
(absent, compatible, incompatible: `<their revision>`, wrong provider:
`<theirs>`) instead of one `Err` that `daemon status` folds into "not
running". `stop`, `reset-tripwire`, `cancel`, `set-priority` are
mutations with their own explicit policy (they act on the daemon that is
there, never replace it). *Use* verbs keep the interactive policy.

This is a C10 amendment ("replacing is the interactive CLI's policy" →
"replacing is a *use* verb's policy; observation never spawns or
replaces") and it is the owner's.

**Disagreement on landing:** the reviewer would land 6.1, 6.2 and 6.3
atomically. Preference here is three commits, each under the green gate,
in the order observe-policy → runtime revision → driver preflight. Each
is independently valuable; the driver is already protected by
`ACQ_NO_SPAWN=1`, so 6.2 is not a prerequisite for 6.3, but it is the fix
for a recorded trap and there is no reason to hold it behind the others.
Three small diffs are easier to review and to revert than one.

### 6.3 Driver preflight — accepted as specified

One `tools/preflight.sh` sourced by both live drivers, replacing the two
copies: (1) export no-spawn/no-replace before any binary runs; (2) refuse
dirty source and control files; (3) probe for any daemon, incompatible
ones included; (4) `cargo build --locked`; (5) probe again; (6) write
`provenance.json` — full HEAD, tree state, package version, runtime
revision, SHA-256 of the executable about to run, `rustc -Vv`,
`cargo -V` — before any wire phase, and include it in the evidence
bundle's checksum. `--locked` is the important detail: the build must
not rewrite the lockfile after the cleanliness check.

### 6.4 No second hash in the journal — agreed

One identity in the journal header (`runtime`); the executable hash
lives in `provenance.json`, where a specific path was chosen and run.
The frozen soak binary stays the one case where the artifact itself is
kept.

### 6.5 Name the version component; no `sed` over a human display — accepted

`--version` stays human. A structured surface for scripts:
`acq version --json` → `{"version":"0.0.1","runtime":"7ac91db821fd"}`
(one small verb; the reference regenerates). `soak-check.sh` is the only
script that parses today; it moves to the structured surface — or is
retired under P6 if the frozen-soak procedure is not coming back, which
is a question for the owner.

### 6.6 The larger question: a dedicated daemon and a protocol crate

The reviewer is right that "every frontend embeds `daemon run`" is an
implementation choice, not a ruling: C1 lists crates, C2 says clients
talk over IPC, C10 says respawn is the migration mechanism; none says
the daemon must be inside every binary. `current_exe()` was the cheapest
answer to "where is the daemon" for the first two consumers.

The author's view, in two parts:

**The crate split is a dependency-graph win on its own.** Today
`acquisition-core` holds the daemon implementation, the protocol, the
client and the shared types, so an edit to `daemon.rs` recompiles plan,
cli and mcp and relinks all 21 executables. A small protocol/client
crate (protocol enums, `Client`, `Realm`, the job model) that frontends
and the planner link, and a daemon crate that only the daemon binary
links, makes a daemon edit cost the daemon crate plus one link. The
planner's `MAX_429_RETRIES` import is the boundary not yet being there.
This split is worth doing whether or not `acqd` follows.

**A dedicated `acqd` changes what the identities are.** With one daemon
artifact, the two questions C10 answers with one value come apart:

- *Is the newest daemon implementation running?* — the client hashes
  the sibling `acqd` it would spawn and compares with what the running
  daemon reports about itself at startup. Runtime, no build script.
- *Is the wire compatible?* — the protocol crate's own source revision,
  which is the 6.1 build script moved to a tiny crate whose files change
  rarely. So the 6.1 machinery is not throwaway: it migrates and
  shrinks.

Costs the reviewer names, confirmed: executable location (a sibling of
`current_exe()` in the playground; packaging later); test discovery
(`CARGO_BIN_EXE_*` is per package, so either `acq` and `acqd` share a
package or the MCP's tests find the sibling under a `--workspace`
build); the MCP's spawn policy is unchanged. It is a C1 amendment and
the owner's call, and it should not ride on the stamp change.

**Recommendation on sequencing:** land 6.2, 6.1, 6.3 now (the git watch
costs every agent session today; three small commits), then hold the
split as its own design session with the owner.

### 6.7 The reviewer's three questions

1. *Is C10 protecting wire compatibility, the newest implementation, or
   both?* Both, with one value, which is why it needed a commit-grained
   stamp. Under 6.1 the runtime revision answers both conservatively;
   under the split they get separate identities (6.6).
2. *Must every frontend remain daemon-capable through `current_exe()`?*
   The owner's. Author's input: no ruling requires it; it was the
   cheapest location answer; with more binaries coming, a GUI carrying a
   hidden server mode is the odd shape, and the sibling-`acqd` shape is
   the clean one.
3. *Reconstructible source provenance, artifact attestation, or both?*
   Both, from different sources: HEAD and tree state from the driver
   (reconstructible), the executable's SHA-256 in `provenance.json`
   (attestation, useful when the artifact is kept, as the frozen soak
   binary is). The journal carries one identity and the run record maps
   it.

### 6.8 Open for the owner after this round

- The permanence question in 6.7(2) — this decides whether the split is
  scheduled.
- C10's amendment text (6.2 policy tiers; 6.1 identity).
- Whether `soak-check.sh` and the frozen-soak procedure are retired (6.5).

## 7. The owner's rulings on 6.8 (2026-09-09, verbatim)

Asked as three plain questions (must every frontend be able to become
the daemon; amend C10 on both counts; keep the frozen-soak check):

1. "omg yes--embedding the daemon sounds terrible compared to `acqd`.
   the path we took was an unplanned shortcut. time to grow up."
2. "yes, amend both."
3. "eliminate. there may be long-running tests in our future, be we will
   design that bridge appropriately when we come to it."

What follows from them:

- **A dedicated `acqd` and a protocol crate are scheduled**, as their
  own design session (C1 amendment, executable location, test
  discovery, packaging). Not part of the stamp change.
- **C10 is amended on both counts** — identity and replacement tiers.
  Draft below; lands with the observe-policy commit.
- **The frozen-soak procedure is gone**: `tools/soak-run.sh` and
  `tools/soak-check.sh` deleted in the commit that records this section;
  the ledger rows that cite the 2026-08 soaks stay as history. The
  `runs/soak/` evidence on disk is gitignored and untouched.
- Sequencing stands: observe policy → runtime revision → driver
  preflight, three commits; then the split. The runtime-revision build
  script migrates to the protocol crate at the split (6.6).

### 7.1 C10 amendment — agreed text (review round 2, Codex; approved at 794 bytes as one line, limit 800)

The reviewer's round-2 version replaces the author's draft: it states the
governing property and leaves both today's hash inputs and the future
`acqd` mechanism to the `client.rs` doc comment, so the split changes a
doc comment and not the ruling. "Uses" rather than "accepts": an observer
talks far enough to identify and report a daemon without using it.
Identity mismatch and provider mismatch are dimensions that can coexist,
not four exclusive states. The reviewer: "Beyond that wording adjustment,
I support the decision and would not reopen any of its architectural
substance."

> **C10 — Version handshake in the protocol; the protocol is
> single-version on purpose.** Kill-and-respawn is the entire migration
> mechanism. A client uses a daemon only when its provider and runtime
> identity match the runtime it would itself spawn. That identity changes
> automatically with the daemon and protocol implementation it governs;
> it never derives from Git state or a hand-maintained compatibility
> number. A use verb may replace a mismatch; observation never spawns or
> replaces and reports absence, identity mismatch, and provider mismatch
> distinctly; an autonomous client (MCP) never replaces. *Why:* a compat
> matrix is the reconciliation swamp; respawn is a one-line diff; an
> observer that replaced cost a live run (2026-09-08). *Details:*
> `client.rs` doc, C10. Amended 2026-09-09.

The *Details* doc comment says, today, "runtime revision over core, store,
manifests and lock"; after the split, "daemon artifact plus protocol
revision". **Approved by the owner 2026-09-09** ("I approve the text"), with the
trim that drops the closing "(history in git)": 794 bytes. It lands with
the observe-policy commit.

## 8. Landed

The three commits of §7 landed on 2026-09-09 as `0db28a8c` (observe
tier), `5c8d88cc` (runtime revision) and `2ead8172` (shared preflight,
`provenance.json`), with two external review rounds on each of the first
two fixed in the commits between; the range's `git log` is the story and
nothing restates it. C3's spawn sentence and the standing rule's "Build
before you run" bullet were approved by the owner the same day and say
so. The split is framed in `17-framing-the-daemon-split.md`.
