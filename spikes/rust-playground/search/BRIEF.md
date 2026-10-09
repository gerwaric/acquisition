# Item search — the brief for building 10a, why not (2026-10-09)

Piece a of step 10 builds `--explain`, `show <id> --against` and
`--context corpus`, and the zero block's offer of them
(`search/BUILD-PLAN.md`, "The steps left", the row 10; the cut into
four pieces and its rulings are the message of `060cd44f`). You are its
one builder. Its expected answers are below, worked by hand, and the
owner has read them: you write the tests from that table, then the
code, and you change no value in it. You commit nothing; the session
that launched you commits. After you an agent that has not read your
report looks at the build, then the owner.

The plan's rules bind this piece as they bound every step. This file
adds only what is this piece's.

One thing sets this piece apart. Nothing it answers is new arithmetic:
a forced node is the query with that node replaced by `true()` or
`false()`, both of which the language already says, binds and
evaluates, at the item's level and inside a group. So every number
this piece prints is a number the build already gives to a query a
user could type, and the piece's spine is that it stays so: one
evaluator, asked a transformed query (rule 10; the reference, *Outside
the first surface*, "alternatives for a node": "by the normal
evaluator; any optimisation must reproduce that"). The fault to expect
is the record's first shape, a meaning read off the syntax: a path
that names another node once parentheses are added, a substitution
that simplifies the tree it was put into.

## What the piece closes on

- Every cell of the tables below is answered by the build as the cell
  says, asked through the crate's boundary.
- For generated queries (`tests/common/generated.rs`), at every path
  the answer names: each of explain's two forced answers has the
  counts the build gives the transformed query it prints, asked as a
  request of its own; the printed text parses to the answered tree
  with that one node replaced and nothing else changed; and a
  respelling that changes no meaning (the equivalence property's ways)
  changes no explain count at the node that corresponds.
- AQ3 whole (`tests/acceptance.rs`): the why-not through
  `show --against`, and the zero answer offering what it now runs.
- Every command a zero block prints parses and is not refused (rule
  5), held where every printed command is run today.
- The refusal walk (`tests/refusal.rs`) passes with three entries
  gone from the list.
- The quality gate is green (`AGENTS.md`, "Quality gate").

## The expected answers

The fixture is `tests/answer.rs`'s `worked()`, the reference's seven
items, and the query its `WORKED`, canonical text
`league=Standard class=Rings rarity=rare line("# to maximum Life" arg1>=90) pseudo.total_res>=60`.
As given: 1 match (r1), 2 undecided (r5, r6). Cells are matched ·
undecided at the root, with the items that match.

| Path | Node | Forced true | Forced false |
| --- | --- | --- | --- |
| 0 | `league=Standard` | 1 · 2 — r1 | 0 · 0 |
| 1 | `class=Rings` | 2 · 1 — r1, r5 | 0 · 0 |
| 2 | `rarity=rare` | 2 · 2 — r1, r4 | 0 · 0 |
| 3 | the group whole | 3 · 2 — r1, r2, r7 | 0 · 0 |
| 3.0 | `"# to maximum Life"` in the group | 1 · 2 — r1 | 0 · 0 |
| 3.1 | `arg1>=90` in the group | 2 · 2 — r1, r2 | 0 · 0 |
| 4 | `pseudo.total_res>=60` | 3 · 1 — r1, r3, r6 | 0 · 0 |

Rows 3 and 3.1 are the reference's "it never creates a member": the
group forced true admits r7, which has no life line; its bound forced
true does not.

`show <id> --against` the same query:

| Item | Root | What decides it |
| --- | --- | --- |
| r1 | match | all five matched |
| r2 | no match | 3 failed: 20 and 75, neither reaching 90 |
| r3 | no match | 4 failed: 55 |
| r4 | no match | 2 failed: magic |
| r5 | undecided | 1 undecided: its base is not in the class table |
| r6 | undecided | 4 undecided: its implicit lines are unread |
| r7 | no match | 3 lacked: no life line |

`--context corpus` beside the same query: the answer's query is the
empty one, 7 matches, 0 undecided; with a count by `rarity`, rare 6
and magic 1; with a count by `class`, Rings 6 and undecided 1.

A zero answer, `rarity=magic pseudo.total_res<60`: 0 · 0, term 0
matching 1 and term 1 matching 1. It offers explain of path 0, then
of path 1, then the corpus context.

| Path | Forced true | Forced false |
| --- | --- | --- |
| 0 | 1 · 1 — r3 | 0 · 0 |
| 1 | 1 · 0 — r4 | 0 · 0 |

## Rules that bind you

- Read first, in this order: `AGENTS.md`, `README.md`, `CONTEXT.md`;
  the plan whole; `decisions/search.md`; in `search/DESIGN.md` the
  reference's *Two values*, *The request at a terminal*, *One worked
  example* from "When the total is zero", *Explain*, *Invariants of
  the surface* and *Outside the first surface*; `SEARCH-SLICE.md`
  whole — its shapes of fault are the checklist the look will hold you
  to; `TESTING-NOTES.md` before a test; the module doc of every module
  before you touch it, `answer.rs` and `bind.rs` first, then
  `group.rs`, `eval.rs`, `show.rs`, `describe.rs` and the CLI's
  `search_cmd.rs`; `tests/answer.rs` and
  `tests/generated_equivalence.rs`, how the worked example and the
  respellings are asked today.
- Write in `crates/` and the generated references
  (`CLI-REFERENCE.md`, `MCP-REFERENCE.md`, by `ACQ_UPDATE_FIXTURES=1`).
  Nowhere else: never `search/` — this file, the plan, the design, the
  ledger — nor `SEARCH-SLICE.md`, `decisions/`, `CONTEXT.md`,
  `README.md`. Those are the session's at the close.
- Never commit, push, stash, reset, check out, fetch or spawn. No
  network (C79). No daemon: any `acq` you run is run with
  `ACQ_PROVIDER=mock` and `ACQ_STORE_DIR` set, and the owner's store
  is never opened (the plan, rule 7).
- **No expected value changes.** A cell the rules as written do not
  give is a finding about the table or the reference: leave its test
  failing, report the cell and what the build answers, and never add a
  case to the build that exists to pass it.
- **A rule nobody stated stops you** (the plan, rule 4). Where the
  reference and the rulings do not say what one of the three answers
  of a query or an item, or the build would change what a user types
  beyond the list below, stop and report the question with its
  readings. A name, a key or a wording is yours to pick.
- No existing test's expected value changes but those that pin the
  three constructs as not built. Each that does is listed in your
  report.

## What is built

**Explain** (the reference, *Explain*, with the paragraph added at
`060cd44f`).

- *It travels in the request*, as the view does: an adapter's flag is
  an encoding of it, and the MCP at step 11 sends the same value.
- *A path* names a node of the returned, canonical query. The tree's
  nodes are numbered today (`bind.rs`, "Atomic terms are numbered by
  path"); a condition inside `line( … )` or `linked( … )` is numbered
  `<term>.<n>` by the same rule the tree's children follow. The
  paragraph names those two groups and no other: the conditions
  inside a `sum( … )` take no path in this piece. The root has none.
- *The substitution is literal*: the one node replaced, nothing
  reordered, flattened, merged or simplified around it (invariant 1),
  and the result bound and answered as any query is.
- *The answer* holds the root's matched and undecided counts three
  times — as given, with the node forced true, with it forced false —
  the two forced ones each beside its transformed query, text and
  tree. No rows. Its words are literal: forced true, forced false;
  never removed, never a contribution.
- *Beside `--count`, `--cross`, `--sort` or `--next`* it is an
  authoring error, said where the view's other conflicts are said
  (`View::of`).
- *A path the query lacks* is an authoring error whose readings list
  the query's nodes with their paths.
- *A substitution the binder refuses* is refused with the binder's own
  error. On the table's query none is: `line(true() arg1>=90)` binds
  today.

**`--context`.** `matches` is what every answer is today and changes
nothing. `corpus` takes any view: the answer's query is the empty one,
and the query typed beside it is carried in the answer as set aside,
printed on one line, counted nowhere. Whether a set-aside query that
does not bind is still an error is yours; say which in the report.

**The zero block.** In place of the two names it says are not built
(`Zero::not_built`), it offers what now runs, each made by the one
maker of a command and of a route (rule 10): explain of each child of
the root, fewest matched first and by path where two are equal, cut at
the block's bound with the rest counted (rule 9; invariant 5); then
the corpus context. An offer is a request and a command, never their
output: the block computes no forced count.

**`show <id> --against '<query>'`.** The item as `show` reads it, and
beside it the query's canonical text, the root's outcome on this item
— match, no match, undecided — and each node's outcome with what a
row shows of a matched term and what an undecided item says of its
reason, each from the maker that exists (`eval`'s evidence,
`eval::why`). Bounded as a row is. A query the binder refuses is that
error. It names the basis the item was read at and compares none:
`show --basis` stays unbuilt.

## The surface the piece adds

The session holds the diff of `--describe --json` against this list.
Anything a user can type or a consumer can read that is not here is a
stop.

- `acq search --explain <path>`, `acq search --context matches|corpus`,
  `acq show <id> --against '<query>'`, each with its help.
- In the request: the explained path and the context, keys yours.
- In the answer: the explain block; the set-aside query; in the zero
  block the offers, and `not_built` gone from it.
- In what `show` returns: the against block.
- `bind::NOT_BUILT` loses `--explain`, `--context` and
  `show --against`, and each refusal with it. The other seven entries
  and their "step 10" stay as they are: the close rewords them.
- At most two new `ErrorKind`s, if none that exists says a view
  conflict or a path the query lacks.
- No answer to anything typed today changes but the zero block's
  offers and those three refusals.

Not this piece: `--next`, `--request`, `--print-request`, `--rebind`,
`--fields`, `--view locations`, `show --basis`, `--membership all`;
the MCP; anything in `search/`.

## The work, in order

1. **The tests, before any code, each seen to fail for the reason you
   expect.** The tables, cell by cell, through the boundary; the
   generated property of "What the piece closes on"; explain beside
   each view it refuses; a path the query lacks; a node under a not,
   under an or and under `holds`, each worked by hand on the seven
   items; `--against` for the seven; the corpus context under rows and
   under both counts; the zero block's offers followed as requests;
   AQ3; one CLI `--json` smoke for each of the three flags, and every
   command the zero block prints run.
2. **The code.**
3. **The module docs**, citing the reference by section and taking no
   paragraph of it (rule 6); `--describe` saying what each construct
   does.
4. **Three breaks.** Break the build three times, one rule each — a
   substitution that simplifies around the node, a path inside a group
   numbered from the wrong child, an offer ordered by the query's
   order — and name the test that noticed. A break nothing notices is
   a finding, reported, not fixed in silence.
5. **The copy.** Over the copy as M3 asks it (`search/MEASUREMENTS.md`,
   the M3 block; the release build): one acceptance query's warm
   number, and the same query under `--explain` of one path. A
   measurement, reported with its command, never a test. The copy is
   local and may be absent: say so and go on.
6. **The gate**, all seven commands, in `AGENTS.md`'s order, after the
   last edit; build again after `cargo fmt` changes a file.

## The report

To the session, about 400 words and the lists: what was built, by
file; every flag a user can now type and every JSON key added; each
existing test whose expected value changed; each choice that was yours
— the keys, how a path inside a group is numbered where the group
nests, the set-aside query that does not bind, the error kinds; the
three breaks and what noticed; M3's two numbers; each gate command and
its result as it ran; what you left out; what you are least sure of.
If you stopped, the question comes first, with its readings and the
state of the tree.
