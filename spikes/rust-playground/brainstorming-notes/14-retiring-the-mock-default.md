# Retiring the mock default — is now the time?

**Written 2026-09-07**, at the owner's ask: "I think we are ready to
remove the ACQ_GGG training wheels." Disposable like every note here;
what it proposes is real only where it lands in code, the registry, a
skill or a test. No code was changed for this note.

## What the wheels were for

In August the limiter was unproven, agents ran commands freely, and the
fear was a bug flooding GGG. Mock-by-default made "an agent runs acq"
safe by structure (rung 1 of the ladder): no env var, no traffic. It
was the right wheel for that bicycle. It also shaped everything around
it: the live-run and mock-session skills open by unsetting it, three
drivers refuse a leftover export of it, six test harnesses scrub it,
`AGENTS.md` forbids setting it, and the charter written this morning
names it in its first sentence — thirty sites.

## What has changed since

- **The limiter is proven.** The ladder closed 2026-08-27 with zero
  429s across ~1,450 sends; every live run since (rung 11, persistence,
  four tracer and character runs, two readings) passed under it.
- **C14 (2026-09-01) already retired the premise.** Agent traffic is
  allowed because the daemon is the single gate; what protects the
  relationship is that every caller passes through it, not that callers
  are kept away. The mock default outlived the ruling that removed its
  reason.
- **The product surface is daily use now.** `policy`, `refresh --plan`,
  `price`, `shop render` are offline reads of the owner's real facts
  and intent, and they resolve the store directory from the same env
  var: a bare `acq price status` reads the *mock* store and reports no
  accounts. The wheel that kept traffic off GGG also keeps the owner's
  own data behind a flag on every offline command. Reading 1 was run
  that way; reading 2 would be too.
- **A process-wide mode leaks.** The env var is ambient: a shell that
  ran a live run still exports it, a mock session's isolation exports
  the other way, and the first line of both skills and three drivers is
  the cleanup. That trap class is the mechanism's, not the default's.

## What the wheels still protect, honestly

Not the relationship. The gate paces every send regardless of default;
what revokes an application is violations, not volume (N10), and a
runaway agent loop under the gate costs requests, never a 429. What the
default protects now is hygiene, three cases:

1. **Which store an offline command reads.** Flipping the default fixes
   the owner's case and moves the burden to mock sessions, which already
   set two isolation knobs and can set a third.
2. **An agent in a non-isolated session spawning a real daemon.** A
   bare `acq characters` from an agent shell with the owner's keychain
   available would lazy-spawn a real daemon and spend the owner's
   requests on the owner's account. Under the gate that is safe for GGG
   and wrong for the owner (probe hygiene: "hits > 0 means something
   else is on this account").
3. **A test written without the harness.** Every harness scrubs the env
   and would have to set the mock explicitly. The one path a test could
   take to GGG *outside* the choke point is `auth_start` returning a
   real authorize URL that the test's HTTP helper then GETs — the
   browser-owned navigation invariant 1 leaves outside the boundary.

Each of these has a structural answer that is better than the env var,
which is the real argument: the wheels are a rung-5 rule ("never set
it") standing where rung-1 and rung-2 structure can stand (P5).

## The proposal

**Flip the default and replace the wheel with structure.**

- `ACQ_PROVIDER=mock` opts into the mock; the default is `ggg`. The
  provider is already a named coordinate everywhere — the store
  directory, the keyring service, the handshake, the rails' persisted
  state — so the env var only chooses; nothing else moves. `ACQ_GGG` in
  the environment is refused at start with a one-line message naming
  the change, for as long as the owner wants: stale exports and old
  scripts fail loudly, never silently in the wrong mode.
- **A real daemon is started by a human.** Lazy spawn in real mode
  requires a terminal (stdin a tty); the CLI otherwise reports "no
  daemon running — start one from a terminal", the sentence the MCP
  server already says (C13, C14). Verified 2026-09-07: the agent shell
  this note was written from has no tty on stdin or stdout, so an
  agent's bare `acq characters` attaches to a running daemon or gets
  the refusal — case 2 closed by structure. Mock spawn stays free. A
  human's script keeps `acq daemon run` and `ACQ_NO_SPAWN` as today.
  Known limit: an agent harness that allocates a pty looks like a
  human; that agent is then acting as one, and C14 says its traffic is
  fine through the gate — the `AGENTS.md` rule ("never start a real
  daemon") stays as the discipline layer above the structure.
- **Real mode refuses `ACQ_NO_KEYRING`.** Every harness sets it; a real
  run wants the keyring (the live-run skill unsets it first). A daemon
  asked for both refuses to start naming the two — case 3's belt, and
  a kinder answer than today's silent memory-only real session. The
  tools (`acq-as.sh` sets both today, for the ladder) change to say
  what they mean.
- The harness and the drivers set `ACQ_PROVIDER` explicitly; the
  `ggg_refusal.rs` test flips its env; the mock-only kinds' refusal
  (rail 6) is unchanged.

**What must not change:** the standing first-contact rule (rails on,
ceiling 3, from a terminal); the rails off by default; C13 and C14;
one daemon per provider (the limiter's state keys carry the account,
not the provider — mixing is not safe); separate keyring services;
per-provider store directories; the mock as the test double and the
rehearsal stage.

**Not proposed:** the provider as a login-time property of the account
rather than a process choice. It is the cleaner shape — the store is
per provider already — but it needs a daemon serving two providers,
which C10 and the limiter's keying refuse, and no consumer asks for it.
Park it with its trigger: a consumer that needs mock and real in one
daemon.

## Blast radius

One bounded slice, about a day: `provider.rs` (`ggg_mode` → a
`provider()` reading `ACQ_PROVIDER`, the `ACQ_GGG` refusal),
`client.rs` (the tty rule in `interactive`), the daemon's start (the
keyring refusal), the two `provider()` helpers in the CLI and MCP, six
CLI test harnesses and the MCP harness, five tools, two skills,
`AGENTS.md`'s rule, `LIVE-TESTING.md`'s standing rule text, the README's
charter sentence and knob row (the knob check will insist on the row),
the `about` string, the MCP instructions, the generated references
(regenerated by their tests). A ruling in `decisions/network.md` — the
default and the two refusals as one entry — and a park in
`decisions/daemon.md` for the login-time shape.

Proof: the mock rehearsal (`tracer-rung.sh --mock` under the new knob),
then a first-contact-style live check under the standing rule — rails
on, ceiling 3, from a terminal, **no provider flag** — reading the
journal's `open` line for `provider: ggg`, the store directory, and
the keyring entry. One ledger row.

## Is now the time?

For:

- The evidence standard the charter names is met: the suite, the live
  runs, and the owner's use — reading 1 was the owner using the product
  with the flag in the way.
- C14 already made the change in principle; this is the mechanism
  catching up with the ruling, the same lag 09 found between "the
  property lands in CONTEXT" and the documents.
- The next consumers arrive to the default they will ship with. The
  GUI's dev loop would otherwise carry the flag into a Tauri process
  environment; the explicit-selection door and currency totals are
  owner-use slices.
- P6, exactly: a rung retired when what it guarded is proven. The mock
  default is the last piece of the ladder's paperwork; the tripwire
  stayed as code, and so should this — as the two refusals above.

Against, and the sequencing that answers it:

- Step 6 is unreviewed and reading 2 is pending; stacking a second
  concern on a slice under review is what `NETWORK-CLEANUP.md`'s
  process note warns against. So: the step-6 outside review first (it
  is owed anyway), then this slice, then reading 2 — which then runs as
  the first real use under the new default (P2: the validating
  consumer). The flip's failure modes are loud and immediate (no
  accounts known; no session; refused start), not the kind that would
  confound a forum paste.
- The 30 sites are a day, not an hour; a half-done flip is worse than
  either state. One commit for the mechanism, one for the sites, one
  for the record.

## Questions for the owner

1. `ACQ_PROVIDER=mock` (a coordinate, the store's own word) or
   `ACQ_MOCK=1` (a switch, symmetric with today)? The note assumes the
   former.
2. The tty rule: accept it as the structural answer for agents, with
   the pty case left to the `AGENTS.md` rule? Or require an explicit
   `acq daemon start` in real mode always, at the cost of the daemon's
   60 s idle exit turning into ceremony for the owner?
3. Real mode refusing `ACQ_NO_KEYRING`: is there a machine or a run
   where a real, keyring-less daemon is wanted (a Linux box without a
   secret service)? If so the belt becomes a warning in `daemon status`
   instead of a refusal.
4. Sequence: review of step 6 → this slice → reading 2, as proposed?
