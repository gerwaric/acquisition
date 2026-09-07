# README as an index — diagnosis, form, routing

**Written 2026-09-07**, before the pricing close, at the owner's request
for a documentation review focused on the README's growth. Disposable
history like every note here, in 09's mold: what it proposes is real
only where it lands in `README.md`, `AGENTS.md`, a skill, or a test.
Nothing here touches a decision's content.

## Diagnosis

Measured on 2026-09-07 at `eed10233`:

| Measure | Value |
|---|---|
| `README.md` | 27.5 KB of a 30 KB gate (92%) |
| always-loaded path (AGENTS + README + CONTEXT) | 46.4 KB; README is 59% of it |
| "Try it" block | 9.7 KB: 40 command lines, 44 comment-only lines |
| README since the 2026-09-02 settle | 25.2 → 27.5 KB across six pricing steps, 200–700 B per step |
| prunes in three weeks | four (`2c5efcc7`, `9fa99459`, `53269ee5`, `2313d7bc`); growth resumed after each |
| `CLI-GUIDE.md` | 25 KB, declared frozen 2026-08-31, edited twice since (`13fead8d`, `10ef2374`) |

The bytes are the symptom. The disease is that **the README has no
form**. Every other document was given a unit and a limit — a registry
entry under 800 bytes, a ledger row, a findings row, a register row, a
procedure — and the unit is what makes routing mechanical: a fact that
does not fit the shape is the signal that it belongs elsewhere (a
narrative cannot fit a bullet, so the mechanism goes to the code). The
README's charter names topics ("what exists, how to run it, knobs, known
gaps"), not shapes, so nothing signals when a paragraph lands that does
not belong. The byte gate fires late and says "route something" without
saying what; each prune is by hand and the next slice refills it.

Two consequences follow, and both are the same shape 09 found for
`CONTEXT.md`.

**Rule density.** The owner's framing on 2026-09-02 was that bytes are a
proxy; the budget that governs an agent is the number of rules it must
hold before it knows its area. By that measure the README is the
always-loaded document with the *lowest* rule density and the *highest*
byte count: nearly everything in it is usage (whose home is `--help`) or
mechanism (whose home is a doc comment) or a restatement of a ruling
(whose home is the registry). It is paid every session and holds almost
no rule.

**The growth is per verb, not per session.** Look at where the README
grew: the last commit of each step — the record, the sweep, the session
close. The session-close routing table has a row for every kind of
*learning* and none for *usage*, so an agent that has just built a verb
and wants a durable home for "how to use it" has exactly one always-loaded
file the next agent will read, and writes a comment block there. That is
the incentive 09 diagnosed for CONTEXT ("commit messages feel ephemeral,
and CONTEXT is the only file every next session reads"), moved one
document over.

## What the README restates

Every copy below has a home already; the README is the second or third.

| In the README | Home |
|---|---|
| the limiter's mechanism (header-driven, timing bucket, one `reqwest::Client`, 429 re-queue, Cloudflare never retried) — the preamble | invariants 1–3; C17, C18, C21, C26; `ratelimit.rs`, `gate.rs` |
| the ladder's date and send count; "built 2026-08-29"; hashes of history | the run ledger; git |
| the document list under "Layout" (five files, out of step with `AGENTS.md`, which lists nine plus the area files and skills) | `AGENTS.md`, "Read before changing anything" |
| the store, planner and CLI bullets' module inventories | each crate's module doc, "As built" |
| the tour's comment blocks | the clap help strings (`acq shop render --help` already carries C74 and C72 and reads better than its README lines) |
| the scripted mock login (`/authorize?` → `/approve?&user=`) | the mock-session skill |
| probes, declared route knowledge, the 429 bound, per-account pacing — "Real GGG mode" | C20, C32, C26, C18, C31; `daemon.rs` |
| the macOS Keychain prompt | the live-run skill ("known costs, not stops") |
| two daemons in real mode; one daemon per store directory | C31, C6 |
| what the tripwire, ceiling and journal do — the rails bullets | `LIVE-TESTING.md` "Rails" (which says the README holds the *knobs*, and it does; but the bullets restate the rails) |
| the journal line format (`open` event, `shape`, `route` key, `headers`) | `TESTING-NOTES.md` "The journal is the contract surface"; `rails.rs` |
| lazy spawn and idle exit; account selection and refusal | C3, C51; `daemon.rs`; the `--account` help string |

The gate cannot see any of this. A duplication detector is the wrong
tool; the right one is a form that a restatement does not fit — the move
that stopped CONTEXT.

## The form

**The README is the index to what exists and how to reach it, the way
`CONTEXT.md` is the index to what was ruled.** Two always-loaded
documents, two indexes; everything either one names sits at a lower
rung. Five sections, each with its unit:

1. **Charter.** One paragraph, the owner's words (`CONTEXT.md` names it
   as the charter). No dates, no counts, no mechanism. The standing
   status sentence ("GGG side proven, frontend side the frontier") is
   the owner's to keep or revise — 07's question 7 (the self-description
   has drifted) is still open.
2. **What exists.** One bullet per crate: what it is, and where its
   "As built" doc is. No module inventories, no mechanism, no document
   list.
3. **The tour.** One line per verb, comment no longer than the line;
   grouped by what it needs (daemon, store only, nothing). **A verb's
   semantics is its `--help` string.** The tour shows the shape; the
   help is the reference. Pinned by a test (below).
4. **Knobs.** A table, one row per knob: name, default, effect in one
   clause, read at (file). Rails knobs point at `LIVE-TESTING.md`
   "Rails" for what the rail is; the journal row points at
   `TESTING-NOTES.md` for the line format. Pinned by a check (below).
5. **Known gaps.** As today: one bullet each, the workaround, the park
   it points at.

"Real GGG mode" collapses into the tour and the knob table: `ACQ_GGG=1`
selects the real provider under the existing registration (invariant 4);
mock and real never share a daemon (C10) or a keyring entry; the rule is
`LIVE-TESTING.md`, the procedure the live-run skill. Five lines.

Estimate after the edit: 10–12 KB; the always-loaded path 46 → ~31 KB.
The gate is the owner's number, by 09's rule (the measured floor plus
one slice's room).

## Two mechanical checks (rung 2)

The tour and the knob table are copies, and a copy is honest only when
something checks it. Both checks are cheap and match what the gate
already does.

- **`crates/acquisition-cli/tests/readme_tour.rs`.** Read the README's
  fenced block; for every line beginning `acq`, walk the verb path
  through the clap `Command` (stop at the first token that is a
  placeholder, a literal or a flag) and check that every `--flag` on the
  line — inside `[...]` included — is an argument of that subcommand or
  a global. About forty lines. The tour becomes the CLI's doctest: a
  renamed verb or a dropped flag fails `cargo test`, which the
  identifier scan cannot see today (`acq shop render` and `--covered-by`
  are not identifiers it checks).
- **`tools/docs-check.sh`, the reverse of the stale scan.** Every
  `ACQ_*` read in the crates' non-test code has a row in the knob table
  (exempt: `ACQ_BUILD`, a build-time value; `ACQ_UPDATE_FIXTURES`,
  test-only). Today the scan proves a README knob exists in the code;
  nothing proves a code knob exists in the README. All thirteen do; the
  check is for the fourteenth.

Where clap is thinner than the README, the text **moves**, it is not
deleted: `acq daemon status|stop` have no description; `acq submit
<KIND>` has none; `acq tabs --realm` says "pc, xbox, or sony" and not
that stashes refuse poe2 at admission; `acq characters --realm` should
say that pc is omitted on the wire. The jq one-liners for reading a plan
envelope go to `acq refresh --help` as after-help (the `plan_cmd.rs` doc
that points at the README for them then points at the help), or stay
as the tour's last lines — the owner's call.

## `CLI-GUIDE.md`

A frozen document is a rule ("not maintained as code changes"), and the
rule was broken twice within a week by agents who saw a command table
and completed it — the honest reflex, aimed at the wrong file. The
ladder says a rule that can be broken silently should become structure,
and the structure here is deletion: git holds the file at `10ef2374`,
`00-framing.md` cites it as a historical input, and a self-contained
snapshot for an outside conversation is cut from the record at the time
it is needed — cheap now that the record has a form, and `acq --help`
is 3 KB. One test cites "CLI-GUIDE §8" (`tests/reference_currency.rs`);
it should cite C11, which is on the uncited list.

**Owner, 2026-09-07, verbatim:** "my thought is that this is better
served by `acq --help` or automatically generated help/docs so an agent
doesn't need to run and build the executable just to see how to use
it." So: a generated `CLI-REFERENCE.md` — every subcommand's long help,
walked from the clap `Command`, plus the MCP tool list with its
descriptions — written by a test in the mold of the existing
`ACQ_UPDATE_FIXTURES` fixtures and failing when the committed file
differs. A golden file, not a frozen one: the difference between
CLI-GUIDE and this is that drift fails `cargo test`. The build cost
moves to the writer, who builds anyway; the reader (an agent, a Codex
session, an outside reviewer) reads the file. It is on-demand, not
always-loaded, and never in the identifier scan (it is code output).
The tour test above still stands: it pins the README's hand-written
lines, the golden file pins the reference.

## Routing rows to add

- `AGENTS.md` routing table and the session-close skill §2, one row:
  *how to use a verb, a flag or a knob* → its `--help` string (or the
  knob's read-site doc comment), and one line in the README's tour or
  one row in its knob table — never a comment block.
- `CONTEXT.md`, "CLI shape": the live verb list is `acq --help`; the
  README's tour shows the shape. One home named, not two.

## Small coherences met on the way

- Five registry entries cite "pattern 4", "pattern 8", "pattern 9",
  "pattern 10, 08 boundary 7", "pattern 3" — the patterns of notes 07
  and 08, which the project calls disposable and never a second
  authority. Name the note (`07 §4`): honest, and shorter than the
  current text under the 800-byte limit.
- `acquisition-store/src/lib.rs` and `acquisition-plan/src/lib.rs` say
  "moved here from the README on 2026-09-02" — history in a doc comment;
  trim at the next touch.
- The uncited list has 29 ids; C11 leaves it with the test citation
  above. The pricing ids on it (C73, C75, C76, C77) are the close's
  clause audit, as planned.
- `LIVE-TESTING.md`'s ~8 KB of rung sections are already flagged for
  the owner in the slice record; nothing new here.

## Why before the close, not after

The session-close procedure routes the always-loaded documents to budget.
Under the current README that is prune five, and the next slices are the
ones that stress exactly the tour: the explicit-selection door (trigger:
pricing closed) *changes* `refresh`'s verbs; currency totals adds one;
the GUI adds a crate and probably a knob. Under the form, each is a clap
change, one tour line, and a test that fails on the stale line. Done
first, the pricing close is a routing exercise under the new form and
every later slice inherits it.

## Questions for the owner

1. The charter paragraph: which sentences survive verbatim? The proposal
   keeps the first four (reference implementation; replaceable given a
   reason; may become the real one; judged by the same tests and ladder)
   and the status sentence, and drops the rest of the preamble to its
   homes above. 07's question 7 is yours to answer or leave.
2. `CLI-GUIDE.md`: delete, or keep and re-freeze?
3. The README gate after the edit.
4. The jq one-liners: `acq refresh --help`, or the tour?

## The charter (owner's ask, 2026-09-07)

Owner, verbatim: "The first sentence has been outgrown for sure. The
caveats about 'may be thrown away' were intended to avoid
over-commitment to ritual and process. The limiter also has a heavy
presence generally throughout this paragraph, which feels myopic now.
There's also some history notes that I don't think are worth their
bytes as narrative." And: some principles baked in have served well,
maybe not all, maybe new ones have emerged.

What each clause of the current paragraph does, and its fate:

| Clause | Doing | Fate |
|---|---|---|
| "reference implementation of the daemon and rate limiter" | names the branch by its two original components | outgrown: it is the system (store, planner, pricing, CLI, MCP); the label licensed disposability, which nobody believes of the store or 81 rulings |
| "find out what they need to be and pin that as tests and recorded decisions" | the purpose | **keep** — this sentence is the parent of the registry, the `c<n>_` tests and the routing rule |
| "replaceable given a reason … evidence, not a promotion … a fresh build may replace it" | anti-preciousness; keeps the freedom not to build process around protecting code | keep the license, drop the defensiveness; its mature form is *ceremony is earned by evidence* (below) |
| "judged by the same tests and the same live ladder" | the evidence standard | **keep**, updated: the ladder is closed; the standard is the suite, live runs under the standing rule, and the owner's real use (P2) |
| "the limiter's behavior is fully specified…" | the limiter as subject | one clause, as *foundation*: proven, and the reason every later caller is regulated by construction (00-framing, stance 1) |
| "GGG side proven … 2026-08-27 … ~1,450 sends … store built 2026-08-29 … protocol not yet pinned" | status and history | out: headers carry no status; the ledger and git hold the dates; "the frontend side is the frontier" moves every slice |
| "Tests pin behavior at those boundaries, never mechanisms" | a working-style rule | already in `CONTEXT.md` Working style; not repeated |
| second paragraph (mock by default; header-driven; choke point; 429) | mechanism | one clause survives (mock by default, a human sets `ACQ_GGG=1`); the rest is invariants 1–3, C17/C18/C21/C26 |

Principles that emerged since 2026-08-16 and have served, with where
each lives today:

- **Ceremony is earned by evidence and retired by it.** The ladder's
  paperwork retired when the limiter was proven (2026-08-30); a skill
  after a procedure has run twice and repeated a trap; a generalization
  after two consumers (P3); design sessions evidence-driven, never
  calendar-driven (P1). The owner's "avoid over-commitment to ritual and
  process" is this, stated once. Today it is four separate rules and no
  sentence.
- **Structure over discipline** — a rule that can be broken silently
  becomes a dependency edge, a lint, or a test; a document is the home
  of last resort (the choke point, C34/C39/C41 as edges the gate refuses,
  the 800-byte bullet, the golden file above). The ladder of six rungs
  that says this lives only in note 09 — a disposable note that
  `tools/docs-check.sh` cites by name. The same incoherence as the
  "pattern 4" citations.
- **The validating consumer is real use** (P2) — already a P line.
- **One authoritative home per fact** — `AGENTS.md`, Routing; the store's
  rule applied to the documents.
- **The single gate is the payoff, not the subject** — C14; every caller,
  human or agent, is regulated by construction, so a new caller is not
  a fresh risk.
- **The smallest slice that reaches real use, with the parts that will
  change kept cheap to change** — the owner's pricing re-scope framing
  (2026-09-04); P2 plus "simplicity over flexibility", not a new rule.

The charter is identity and stance, in prose, owner-held; the
citable rules stay in `CONTEXT.md` Working style. So the proposal is a
paragraph plus two Working-style lines, not a paragraph that restates
the rules.

Draft charter (~1,000 bytes; the current one is 2,519):

> This branch is the Rust implementation of Acquisition (`CONTEXT.md`,
> "Orientation"), built slice by slice against a mock provider — nothing
> here talks to GGG unless a human sets `ACQ_GGG=1`. Its purpose is to
> find out what the system needs to be and to pin that as tests at its
> boundaries and as recorded rulings, so the code stays replaceable
> given a reason. The work is judged by evidence — the offline suite,
> live runs under the standing rule, and the owner's real use of each
> slice — and never by ceremony: a rung, a rule, a generalization or a
> procedure is added when evidence asks for it and retired when the
> evidence is in. The rate limiter and the single gate are the proven
> foundation; their payoff is that every later caller, human or agent,
> is regulated by construction and is never a fresh risk. Whether this
> code ships is the owner's decision (ADR 0003) and nothing here
> anticipates it.

Candidate Working-style lines, for the owner:

- **P5.** Structure over discipline: a rule that can be broken silently
  becomes a dependency edge, a lint or a test; knowledge sits at the
  lowest rung that holds it (structure, test, script, on-demand file,
  always-loaded file, history); a boundary never sits below the
  always-loaded rung. (The ladder, from note 09, given a home.)
- **P6.** Ceremony is earned by evidence and retired by it: a procedure
  becomes a skill after it has run twice and repeated a trap; a
  generalization waits for two consumers (P3); a check or a rung is
  removed when what it guarded is proven.

Open: whether "reference implementation" is retired in
`CONTEXT.md`'s text too (`TESTING-NOTES.md` is a closed record and
keeps its wording as history).

## Outcome (2026-09-07, as run)

Commits `2f2f0f21..7b4d6e70`, five: the charter and P5/P6; the re-form
with the text carried into clap and `rails.rs`; the three checks and
the 15 KB gate; the two generated references and CLI-GUIDE's removal;
the citation coherences.

| Document | Before | After | Gate |
|---|---|---|---|
| `README.md` | 27.5 KB | 11.5 KB | 15 KB (was 30) |
| always-loaded path (AGENTS + README + CONTEXT) | 46.4 KB | 31.5 KB | |
| the tour | 40 verb lines, 44 comment lines | 30 verb lines, 0 continuations | `readme_tour.rs`, docs-check §4 |
| uncited decisions | 29 | 20 | help strings cite ids |
| `CLI-GUIDE.md` | 25 KB, frozen, edited twice | gone | `CLI-REFERENCE.md` 28 KB + `MCP-REFERENCE.md` 8 KB, golden |

The first generation of the MCP reference found the server's
`initialize` instructions still telling agents it refuses submissions
in real mode (lifted by C14 on 2026-09-01) — fixed in the same commit.
The owner's charter questions 1–3 were answered in conversation
(Rust named; "slice by slice" kept; the owner's judgement folded in as
"the verdict it returns"). Open: the owner has a second charter topic
for the next stopping point; the pattern citations in the *Why:* lines
are `07 §n` because the long form did not fit.
