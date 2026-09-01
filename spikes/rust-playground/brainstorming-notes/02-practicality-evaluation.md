# Practicality evaluation of the first-round synthesis

**Written 2026-08-31.** The owner is inclined to accept all of
`01-first-round-brainstorming.txt`'s recommendations. This note evaluates
whether that path leads to something buildable and runnable — simpler and
better, supporting CLI, GUI, MCP, and TUI — by checking the synthesis's
claims against the actual code rather than against the documents it was
written from. Verdict first, then the evidence, then the honest list of
places where the synthesis underestimates, then rulings-to-make.

## Verdict

**Accept, with five amendments.** The synthesis's load-bearing "the spike
already has this" claims verify against the code — several are *more* true
than it claimed. The proposed path adds remarkably little new surface: one
store-crate write API (annotations), one small protocol addition (quote),
one shared Plan type. No new processes, no new wire contracts, no third
door. The tracer choice (refresh-with-plan) is right, and — importantly —
it is right *because* it lands on exactly the spots where the synthesis
underestimates, so the unknowns get paid for one slice at a time instead
of surface-wide. The amendments below are not objections; they are the
difference between "one dry-run flag and a crate" and what building it
will actually feel like.

## Claims checked against code

- **Effects = the job model, facts = the store.** Confirmed. `schema.sql`
  has `fetched_at` on tabs and characters, `item_events` with
  added/moved/changed/removed, items keyed on stable GGG id. The essays'
  observed plane is running today.
- **"The daemon can predict" is literal.** `eta_for(route, ahead, now)`
  in `ratelimit.rs` simulates the pacing rule forward — there is a test
  named `eta_simulates_the_pacing_rule_forward`. For the refresh tracer's
  shape (one list + N stash fetches, essentially one policy), `quote` is
  a modest extension of a function that exists, not new science. The
  synthesis's cheapness claim survives *for the tracer*; cross-policy
  quotes are more work and the tracer doesn't need them.
- **"`daemon.db` is blessed as frontend-readable" is already
  structurally true.** The persisted queue lives *in the store crate*
  (`acquisition-store/src/jobs.rs`), with WAL and a busy timeout, comment
  reading "the daemon writes while any number of frontends read." The
  effects-ledger-behind-door-2 part of the ledger symmetry is not a
  proposal; it is a description. `acq state` degrading gracefully with
  the daemon down is buildable today.
- **"Built once, every frontend inherits" holds — for a specific
  reason worth recording.** All four frontends are Rust linking the same
  crates: CLI (clap), MCP (`rmcp`), GUI (Tauri — its backend is Rust, so
  Tauri commands wrap the shared crates directly), TUI (`acq dash`
  already exists inside the CLI binary). If the GUI were pure
  JS-over-a-wire this claim would collapse into "built once plus
  reimplemented once"; Tauri is what saves it. The four-frontend question
  the owner asked is answered by *two surfaces plus shared Rust crates*,
  and that answer is only as strong as the all-Rust-frontends premise —
  worth a decision line so it doesn't erode silently.
- **WAL concurrency story extends to annotations.** The store already
  runs multi-process (daemon writer, N readers). Frontends writing an
  annotations file adds frontend-vs-frontend write contention; SQLite
  WAL + busy timeout handles it, and a separate annotations file means
  the daemon's write path and the frontends' never contend at all —
  the separate-file choice is right on engineering grounds, not only on
  the legacy-buyout disaster-recovery lesson.

## Where the synthesis underestimates (bounded, and the tracer hits all of them)

1. **Quote has an honesty problem the Plan type must carry from day
   one.** The limiter starts every daemon lifetime empty; before an
   endpoint's probe, its policy is unknown; after a restart, headroom is
   stale until the first response. So a quote is sometimes "N requests,
   ETA unknown until probe" — the request *count* is always exact, the
   cost/ETA half is sometimes a confession. This is fine — it is the
   essays' own epistemics stance applied to the plan itself — but it
   means the Plan type needs its uncertainty field in v1, not as a later
   refinement. A Plan that pretends to know the ETA it doesn't have
   would violate the design's own first principle.
2. **Plan-staleness is a real semantic decision, not a detail.** A
   refresh plan is computed from the *stored* tab listing; the world can
   move between plan and apply, and the refresh parent re-lists. Binding
   or advisory? (a) Apply submits exactly the planned children, no
   relist — cheap, but fetches into a possibly-moved world; (b) apply
   runs the parent with the plan as filter + budget, recomputing at
   fan-out — honest, but the plan you reviewed is not exactly what runs.
   This is the C++ clean-refresh gate resurfacing one level down, and it
   is the single most valuable thing the tracer will teach. It should be
   left open *on purpose* going in, and pinned by what the tracer
   learns.
3. **Per-operation budget is new daemon machinery, not a promotion.**
   `ACQ_MAX_SENDS` is one lifetime-global counter at the send boundary.
   `--max-requests` on a plan needs a counter scoped to a parent's
   descendant tree and a terminal semantics for tripping mid-fan-out.
   The semantics has a template — the mid-fan-out restart rule already
   established "never success over a partial set" — but it is a real
   feature with edge cases, not a rename. Budget as *visibility* (the
   quote) is nearly free; budget as *enforcement* is the priced item.
4. **"An error's remedy is a plan" is a door-2 idiom, not a system-wide
   law.** The daemon cannot compute plans — it is blind to facts and
   annotations by design, and that blindness is the architecture's best
   property. So stale-read refusals and planner errors carry Plans;
   daemon protocol errors keep their current shapes. Record the
   unification scoped to where it can actually live, or the first
   daemon-side error that "should" carry a plan becomes pressure to make
   the daemon read the store.
5. **Dangling intent needs a day-one answer in the annotations schema.**
   Annotations keyed on GGG item id outlive their items (sold, moved out
   of a tracked tab). The store already has `removed_at` and removal
   events, so this is cheap — but the schema should decide up front that
   intent on a removed item is *kept and surfaceable as orphaned*, never
   dropped. That is the legacy-buyout lesson stated as a property:
   annotations are the only irreplaceable state, so no fact-side event
   may cascade into deleting one.

## The synthesis's own flagged weak point resolves cleanly

§1's "daemon permanently blind to annotations" was offered as the claim
most worth stressing, with daemon-resident scheduled syncs as the
breaking scenario. The scenario is already foreclosed by two standing
facts: the daemon is reactive by design (it never initiates a job — every
send has a submitting client), and on macOS a cron-spawned daemon has no
keychain anyway (`ACQ_NO_SPAWN` exists precisely because of this), so
scheduled work needs a user-session process regardless of who executes
the policy. A scheduled sync is therefore a small frontend — cron or a
login-session agent running plan/apply — not a daemon capability, and
daemon blindness costs nothing to keep. Suggested companion decision
line: **the daemon never initiates GGG traffic; every job has a
submitting client.** That line is what makes permanent blindness safe to
pin, and it is true today.

## Rulings the owner is asked for (with this note's leanings)

These are the brainstorm's §7 questions, restated with a
practicality-informed lean:

1. **Annotations write path through the store crate: yes.** Separate
   file per account (facts refetchable, intent irreplaceable; zero
   daemon/frontend write contention; backup is a file copy; a fact-store
   migration or rewrite cannot touch it). This amends the "frontends
   read the file directly" line to "frontends read facts and read/write
   intent, all through the store crate" — one writer class per layer.
2. **Scope taxonomy: accept as direction, build only per-account now.**
   The sync policy is per-account — the easy first row. Create `user.db`
   when the first genuinely user-scoped kind arrives (currency ratios,
   saved searches); record the taxonomy so the schema is designed for
   it, per pin-after-the-consumer.
3. **Quote: lean separate protocol request, not a flag on `Submit`.**
   Mechanically tiny either way, but `Submit`'s contract is heavily
   loaded (id allocation, persistence, rollback-on-write-failure); a
   dry-run that allocates no id and persists nothing is a different verb
   wearing a flag. A `Quote` request keeps `Submit`'s teeth undiluted.
   Owner's boundary; taste only.
4. **Pinning: agree with the brainstorm.** Direction lines for the
   grammar and the Plan unification; pin only what the refresh tracer
   validates; pricing and shop in the parking lot with named landings.

One addition not in the brainstorm's list: **where the planner lives.**
`acquisition-core` already depends on `acquisition-store` (the daemon
writes through it), so a planner in core would compile — but "the daemon
never reads the store" is currently near-structural, and a fact-reading
planner inside the daemon's own crate erodes that to discipline. Lean: a
separate `acquisition-plan` crate (depends on core's client/protocol
types + store), linked by frontends only. Cheap now, awkward later.

## Multi-account through the tracer

The three-accounts test, applied to the tracer itself: the sync policy is
per-account (first annotation row — the easy case, as the brainstorm
says); a plan is per-account by construction because quotes are priced
against per-account limiter keys; `acq refresh --plan` with several
sessions live should refuse-and-list exactly like every other job
command (stateless selection precedent). Cross-account planning is a
frontend loop over per-account plans, matching the existing "no
cross-account `refresh --all`" line. Nothing new needed — but the Plan
type should carry `account` from v1 so a GUI showing three plans side by
side never has to guess.

## Simplicity audit

The direct check of "simpler and better": count what the accepted path
adds against what the essays imagined. Ops log → already exists
(daemon.db + journal). Event tail → already exists (subscribe +
item_events). Snapshot diff → already exists (item_events). Budget
visibility → quote (small). Budget enforcement → per-op counter
(moderate, priced above). Desired plane → one table + one write API.
Plan grammar → one shared crate + one protocol request. Everything else
is exposure and naming. The essays' cathedral compiles down to roughly
three bounded pieces of work plus a tracer — that is the strongest
practical evidence that the synthesis found the real system rather than
decorating one.

## What would falsify this evaluation

Worth naming so the tracer is read honestly when it runs:

- If the quote turns out to need cross-policy simulation even for plain
  refresh (e.g. probe jobs interleaving in ways `eta_for` can't
  express), the "modest extension" claim was wrong — reassess before
  pinning the protocol shape.
- If plan-staleness (amendment 2) can't be resolved without the daemon
  reading intent, the blindness line fails its first real test — that
  would be a finding worth a session of its own, not a quiet workaround.
- If writing the annotations API surfaces frontend-vs-frontend conflict
  cases that last-writer-wins can't honestly serve, the deferred
  annotation change-log (the brainstorm's §3 prediction) moves from
  "noted" to "next" — the schema should already be shaped for it.
