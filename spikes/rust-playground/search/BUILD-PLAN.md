# Item search — the build plan (stage 6)

The brief for the build, read whole before any search step. What each
step met — the audits and their fixes, the measurements, the holes ruled
and the builder's observations — is `SEARCH-SLICE.md`, the record; the
measurements' tables are `search/MEASUREMENTS.md`, read by the block a
step repeats; which steps are built is the record's ledger, never this
file. What this file held
before the record was opened — every audit round, the step 4b brief, the
first showing of the store's read — is this file at `2d25cb03`.

Ruling (owner, 2026-09-21): SEARCH-SLICE.md is opened now, at step 5, in
PRICING-SLICE.md's mold, rather than at the slice's close as
search/BUILD-PLAN.md says. The plan stays the brief.

What a step reads (2026-09-23, after a measured read of the slice's
documents: 308 KB before any code, seven times what a step needs):
this file whole; `decisions/search.md`; what its row's *Reads* names;
the record; `search/MEASUREMENTS.md` by the block it will repeat; a
module doc before touching
its module; the reference by the section a row names. `search/DIGEST.md`
is looked up by `S<n>` and never loaded; a closed track by citation.

Authorities, not restated here: `decisions/search.md` (C89–C107),
`search/DESIGN.md` (the language reference and the contract detail),
`search/DIGEST.md` (the acceptance set, the limits register), note 23
(row 6, decision 15). Under decision 15 everything after the first seat
is provisional, this plan's later steps included. This file is deleted
at the slice's close and cited by hash.

## How a partial build stays honest

1. **The parser knows the whole language from step 1; the evaluator
   knows part of it.** What is not built lives in one closed list in the
   crate, each entry a construct of the reference and the step that
   builds it. A request that uses one is refused before anything is
   evaluated — `not built: pseudo.* (step 7)` — an error of its own kind,
   never the unknown-name error, never undecided, never an empty answer.
   `--describe` prints the same list. One test walks it: every construct
   of the reference either evaluates or refuses by that name, and the
   list the test walks is the list `--describe` prints.
2. **Scope, basis and undecided are in the first answer or there is no
   answer.** The first surface prints the scope block, the basis, the
   four counts per term and the undecided items with their reasons. A
   reason whose source is not built yet (a base the class table lacks, a
   price unresolved) cannot occur, because the construct that needs it
   is refused.
3. **Tests pin the crate's boundary** — a request in, an answer out, as
   JSON — and the CLI's `--json`; text is a function of JSON (C53).
   Every count in a fixture answer is worked by hand first, as the
   reference's worked example is.
4. **A rule the reference does not state stops the step** and comes to
   the owner if it changes what a user types; otherwise it is a row in
   the slice record's observations. A name the reference hands to the
   builder — a flag, a key, a field's spelling — is not a gap: the
   builder picks it and `--describe` or the help prints it. What a name
   *means* — which items it matches — is. The unruled ones are at the
   foot of this file; a ruled one is a line in `SEARCH-SLICE.md`, "Holes
   ruled", its evidence the ruling commit's message.
5. **An answer never prints a command the same build refuses.** A
   block the reference bounds with a route to the whole (invariant 5)
   prints that route only when it runs; until then it prints the count
   left out and names the unbuilt construct. The route property's test
   covers it: every command an answer prints parses and is not refused.
6. **The language reference stays whole in `search/DESIGN.md` until the
   first seat has ruled.** Module docs cite it by section and take no
   paragraph of it before then: the seat may reopen that page, and two
   copies would rot. Owner, 2026-09-19: "accepted". The contract detail
   below it leaves as its header says, when a module doc carries the
   paragraph — the delay was withdrawn for the detail 2026-09-23, since
   a seat reads the tool and not the page (five blind seats opened
   neither), and the detail had grown to three copies.
7. **Whose store.** The agent tests on fixtures built through the
   store's own ingest, and measures on a sqlite `.backup` copy under a
   track's `raw/`, always with `ACQ_STORE_DIR` set. The owner's store is
   opened only by the owner, at a terminal. No GGG traffic anywhere in
   this plan; the one daemon is M5's, a mock session.

Rules 8 to 10 are what step 4b's audits taught (`SEARCH-SLICE.md`,
"Findings"), each a fault that came back until it was named;
the owner approved their meaning from a plain account of each and left
the wording to the builder (2026-09-22: "the words that get written down
are for you"; "If so, go ahead"). Each names what already holds it: the
rule is for the moment before the code is written.

8. **Unknown is said once, where the evidence is read, and of no more
   than was lost.** What the deriver cannot read — a flag that is no yes
   or no, a number it does not read, an array that is no array — is
   unread there, at that grain: the flag, the slot, the array. Too small
   and it reads as absent, and a not makes a witness of it; too large and
   terms that never needed it are left open (C93). Nothing downstream
   has a fallback or a second way to fail. *Held by:* the completion
   property; `derive::Slot`, `Line::flags_unknown`, `Unread::line`.
9. **A cut follows the data, never the query.** A block that shows part
   of something orders it by the item or by the counts, so that two
   spellings of one question show the same part (invariant 7), chooses
   what is relevant before it cuts, and counts what it left out in the
   unit it cut — rows, values, unread parts. *Held by:* the equivalence
   property, where a generated or fixed case crosses the bound: a new
   bound needs a case past it.
10. **One maker for each kind of thing an answer holds.** What a group
    means (`group.rs`, refused elsewhere by `tools/docs-check.sh` §7), a
    command (`answer::command`: rule 5 is this rule for commands), why
    an item is undecided (`eval::why`), a sum (`exact.rs`). A second
    maker is how two parts of one answer come to disagree: look for the
    first before writing one. *Held by:* §7; the CLI test that runs
    every command printed; the cross-checks.

## Steps to the first seat

| Step | Builds | Closes on |
| --- | --- | --- |
| 1 · the language | the `acquisition-search` crate (C89's edges in `tools/docs-check.sh` §5, each with a breaker; C47's lints); the tree and its JSON form; the parser over the whole query language; lowering; the canonical printer; every error the grammar defines, with its readings | the round-trip test; the corpus table; the holes table |
| 2 · the derivation | a pure function, body and ingest columns in, the item out (C103): fields, displayed strings, lines as (kind, template, numbers) with slots and the ranged rule, and what could not be read, by collection | fixture tests; M2 against the census |
| 3 · the store's read | C103's read, the revision, the coverage rows — **shown to the owner first** (the showing: this file at `2d25cb03`; the read as built: `corpus.rs`'s doc, C108) | store tests, `REFRESH-SLICE.md`'s findings table as the checklist; the one-snapshot test; M1; C103's second decision given its own entry (C108) |
| 4 · the first surface | the binder (names, near names, closed sets); three-valued evaluation with witnesses (C93); occurrence binding, the sort scalar, the together count (C92); the answer (C100): query, scope, basis, terms, total, rows, the zero-total block, a route per term count; `--describe`; `--json`; the verbs `acq search` and `acq show <id>` (the item as the deriver sees it: fields, place, every line with its kind, template and numbers, what was unread; `--json` the same, structured; the stored body on request by a flag, which still works when deriving fails — the reference's `acq show`), their README tour lines, `CLI-REFERENCE.md` regenerated | the worked example reduced to what is built, hand-counted; the acceptance rows marked 4; the refusal walk; the route property; M3, M4 |
| 4b · the properties | what four audits of step 4 found by generating, put in the gate as properties; then a group's meaning computed once, so that no second reading of it can disagree (its brief: this file at `2d25cb03`) — no new surface and no rule changed | the three properties green in the gate, each shown able to fail; the structural rule with its breaker; step 4's regression tests untouched and green; M3 again; one more outside audit, its yield reported beside 6, 5, 2, 3 |
| 5 · counts and the vocabulary | `--count`, `--cross`, `--sum` (C95); `none` and `undecided` buckets with routes (C105); `--count line[:text,…]` (C97) | AQ1; C105's two invariants pinned on a one-value key; C95's sum over a fixture of three kinds — an item with the value, one lacking it (adds nothing, counted as lacking), one whose value is unread (the subtotal marked incomplete, never a total); the vocabulary's pasted term selects its row; M3 again; then the seat |

**The reference was brought level with what was ruled here before step
1** (2026-09-19, the owner approving the wording), since step 1's corpus
is complete against that page and this file is deleted at the close. A
hole ruled keeps its rule in `search/DESIGN.md` and one line in
`SEARCH-SLICE.md`.

## The first seat — after step 9, an agent's with the owner in the loop

After 5, not 4 (owner, 2026-09-19: "I'm ok delaying the seat for a good
reason, especially if it's related to discovering the design and
implementation"): the vocabulary is how a template is found without
knowing it, it is the route to everything a `:` or `~` selector resolved
to, and with `--sum` the seat exercises all six of the lines it revisits
first. Then after 9, not 5 (owner, 2026-09-25, approving the builder's
wording): steps 6 to 8 were built before the seat; 9 neither depends on
the answer's shape nor changes it; 10 and 11 are shaped by what the
seat shows, so they wait for it.

The seat is an agent's, the owner in the loop (owner, 2026-09-25; P2):
the owner will not type queries himself. He talks to an agent that
drives `acq search` from a terminal on his real store, reacts to what
comes back, and collects the agent's own feedback on the contract. The
owner's own seat is the GUI's, on its own stream. The five blind seats
at step 5 (the record, "Findings") are the prototype: an agent's
feedback is cheap and repeatable, the owner's reaction needs him once
per surface. `cargo build --workspace`, then `acq search`; it reads
the store directly, as `acq tabs` does — no daemon is started or
spoken to. If the store holds more than one realm every search names
`--realm` (C96); an agent states it each time, so the default realm's
trigger is the GUI's (`decisions/frontends.md`, "Parked").

What the seat can ask:

```
acq search --realm pc '"# to maximum Life">=90' --sort 'line("# to maximum Life").arg1' --desc --limit 10
acq search --realm pc 'rarity=rare base:ring line(template:resistance is:fractured)'
acq search --realm pc 'name="Ashes of the Stars"'
acq search --realm pc 'rarity=rare base:ring "+#% to Cold Resistance" -"+#% to Chaos Resistance"'
acq search --realm pc 'line(template:explode) -is:corrupted tab:dump'
acq search --realm pc 'line("Adds # to # Cold Damage" low>=15 high<=45)'
acq search --realm pc '(rarity=rare base:ring) sum("# to maximum Life")>=90'
acq search --realm pc 'undecided("# to maximum Life">=90)'
acq search --realm all 'id:<a handle an answer printed>'
acq show <an id an answer printed>
acq search --realm pc 'rarity=rare base:ring' --count line:resist,life
acq search --realm pc --count tab,league,rarity
acq search --realm pc 'rarity=unique' --cross league,tab
acq search --realm pc 'frame=currency' --count base --sum stack
acq search --describe
```

Composition whole (`and or not - ( ) holds undecided`); phrases and
`text:`; `name` and `base` with `:` `=` `~`; `rarity`, `frame`, `class`
(step 6: the game's names, `--describe class`), `ilvl`, `reqlevel`,
`is:`, `has:`; place (`league: tab: character: container:`); `id:`;
the price (step 9: `has:priced`, `price.amount`, `price.currency`,
`price.lot`); `line(…)` whole, the shorthand, `sum(…)`; `--sort --desc
--limit --json`; `--count`, `--cross`, `--sum`, and the vocabulary.

A row shows the lines the query touched (C100), so an item's other
lines are read through `acq show <id>`, or by naming them in the query;
`--fields` comes later. What is bounded with no route yet, each printed
as a count and the unbuilt construct's name (rule 5): rows
past `--limit` (a larger `--limit` serves; `--next` is step 10); the
coverage list (`acq tabs` and `acq store characters` exist today;
`--view locations` is step 10).

What it refuses by name: `pseudo.defence_pct` and a ranged total,
`pseudo.<name>.<slot>` (no step: the plan's foot), `--fields`, `--next`,
`--explain`, `--context`, `--view locations`, `--print-request`
`--request` `--rebind`, `show --against` (10).

It exercises all six of the lines the seat revisits first
(`decisions/search.md`, "Standing"): C91's ambiguity error, C92's binder
and sort scalar, C93's undecided route, C95's sum rule, C98's basis as
printed, C104's text.

Sat 2026-09-26 on the release build at `94a3d18c` (the record, the
ledger row "the first seat"): ten faults, F1–F10, and ten verdicts,
V1–V10, the record's "Holes ruled". The owner's order for what follows
(2026-09-26, V10 and after): a fix session against the seat's hash,
then the trade site's computed values as a research track, then
percentile, category and step 10.

## After step 5 — provisional, his to reorder; 10 and 11 wait for the seat

| Step | Builds | Needs | Reads | Closes on |
| --- | --- | --- | --- | --- |
| 6 · class | the class table as reference data, its source chosen under C106's admission test (`item-facts/data/class-evidence.csv` is where the read starts); `class:`; the reason *base not in the class table* | 4 | built | OQ1's slot, OQ5; C105's test as worded (ten rare items by class) |
| 7 · computed values | the totals table (C94) after the coverage trial; `pseudo.total_res`; a weight may be a fraction (the site's own `+94.5 total maximum Life` over `+90` life and `+9` Strength, `pseudo-stats/README.md`), and whether a total is ever rounded is shown to the owner first (gap 6); the sum-status table; then `pseudo.dps`, `pseudo.pdps` (C101) | 6 for the worked example | C94, C95; the contract detail, C94 and C101; the reference, *Values* and *A sum's status*; `counts.rs`, `eval.rs`, `exact.rs`; `search/pseudo-stats/README.md`, `cpp-search/data/pseudomods.toml`; gap 6 and E2 (the record, "Holes ruled"); the coverage-trial park | AQ2; the reference's worked example whole, every count as printed there |
| 8 · sockets | `sockets`, `links`, `sockets.<colour>`, `linked(…)` (C101); undecoded shapes counted unread (S16) | 4 | C101; the contract detail, C101; `derive.rs`; S16, S58, S59; `item-facts/README.md`, the socket shapes | OQ3's socket reading |
| 9 · price | the effective price joined read-only (C81, C100); the crate links `acquisition-plan`; the basis gains the intent revision; the reason *price unresolved* | 4 | C81 (`decisions/pricing.md`), C98, C100; `corpus.rs`, `answer.rs`; the planner's effective-price read | OQ6: the item found with the owner's own price; a valuation asked for is a stated limit (C102) |
| 9b · the seat's fixes | F1–F10 as tests named for the fault, against the seat's hash, the 65 asks replayed (`runs/seat-2026-09-26/replay.sh`) and every diff read and explained in the commit; the class reading (V1 blighted maps past the API's prefix, V2 invitations by frame, F2 a term false under every candidate); the undecided display (V8); the tab type as a field (V10, the park fired); the examples teaching `class:ring` where they taught `base:ring` | 9 | the report (`runs/seat-2026-09-26/REPORT.md`, gitignored); `class.rs`, `eval.rs`, `answer.rs`, `bind.rs`; the contract detail, C93 and C106 | every F held by a test; the replay's diffs each explained; M3 at the fix's hash, the six spiking asks in the script (V9) |
| 9c · the trade site's computed values | a research track (`.claude/skills/research-track/SKILL.md`): what each pseudo of the site counts — the ranged `adds # to # <type> damage [to attacks|spells]` family, `total_life`, T4's lines, Base Percentile's formula (V5) — settled by human-run trade searches under `SURFACES.md`; what cannot be mimicked (affix counts, the owner's caveat on V6) listed as out of reach | 9b | C94, C99, C102; `search/pseudo-stats/README.md`; the trade-site row in `SURFACES.md` | a committed brief; a row per pseudo with its evidence; the totals table's changes as rows, none applied without one |
| 9d · percentile and category | `pseudo.defence_pct` as the site's Base Percentile (V5); category as a grouping above class (V1: the beasts are its first case), from the trade site's categories and/or RePoE under C106's admission test (owner, 2026-09-26), computed from class and reviewed base rules in one place (rule 10), never a second classifier | 9c | C101, C106; `class.rs`, `totals.rs`; the grouping park's entry | OQ5 by category; a beast found by its category |
| 10 · continuing and exchanging | in the order the seat wanted them (the sitter, 2026-09-26): `--explain` and `show --against` first — a zero answer was where they were missed; `--next` and `--request` for the MCP, an agent pages and replays; `--view locations`, `--fields`, `--rebind` and `show --basis` had no consumer at the seat and wait for one. `--next` refused across a changed basis; `--membership all` (the reference, *Membership*): the read hands removed items over with `removed_at`, every such row marked, the scope block counting each, an `id:` term that matches nothing live naming the removed id and its route, `acq show` on a removed item; `--print-request`, `--request`, `--rebind`; `show <id> --against`; `--explain`; `--context corpus`; `--view locations`; `--fields` | 4 | C98, C100, C104, C108; the reference, *Membership*, *Explain*, *Outside the first surface*; `answer.rs`, `corpus.rs`; gap 3 (the record) | AQ3 whole; AQ4 across a refresh that removed the item; M5 |
| 11 · the close | the MCP tool over the same request, with an agent's seat as its own consumer (P2), `--membership all` among what it validates; `acq items search` retired (below); the closed record; this file deleted; `search/` shrunk; `DESIGN.md`'s paragraphs into module docs | all | everything above; `decisions/frontends.md`; the kill list (`DIGEST.md`); `MCP-REFERENCE.md`; "What happens to `acq items search`" below | AQ1–AQ5 driven through the MCP; the gate; the kill-list recheck |

Not in this plan: the trade translation (C99, headed "Direction"). No
acceptance question needs it; whether it is a step here or the next
slice is the owner's call, best made after a seat.

## The acceptance set as tests

One test per question over a fixture store, through the crate's
boundary, plus one CLI `--json` smoke each. The queries enter step 1's
corpus, so that every question can be written in the language is
evidence at step 1. Names are illustrative, as in the reference.

| Id | The test asks | Green at |
| --- | --- | --- |
| OQ1 | `class:ring rarity=rare (line(template:resistance) or line(template:strength))`: the query names the lines it wants to see, so the rows show them; refined: `line("T" is:fractured)`. The fixture holds a ring with neither line, which must not appear | with `base:ring` 4 · as worded 6 |
| OQ2 | `name="<a unique>"`: one item or a zero answer naming the scope, and its place; then `acq show <id>` for every line with its values, which is what the owner reads the version from; nothing says legacy (C107) | 4 — and it fires the variant park |
| OQ3 | a mod, a base, a unique, socket colours, each its own query | 4 · sockets 8 |
| OQ4 | a staff by what is remembered of it (`class:`, `base:`, a phrase), item and tab; `--describe league` says place, never origin (S177) | `base:` 4 · `class:` 6 |
| OQ5 | from the owner's words (2026-09-23): items that can be equipped at a low character level with basic resistance, life and damage modifiers — `(class:… or class:…) (reqlevel=..30 or -has:reqlevel) (line(template:resistance) or line(template:life) or line(template:damage))`, the wearable classes spelled out while the grouping above class stays parked. The fixture holds distractors the bracket alone admits — a gem and a map with a damage line, a currency stack with no level requirement at all — and none may appear | 6 (ruled 2026-09-23: "yes, lets keep this parked") |
| OQ6 | the item found, `has:priced`, the price as the owner set it; no valuation | 9 |
| OQ7 | one line across every tab and character, item and tab on each row; `--count tab` | rows 4 · count 5 |
| AQ1 | `--count tab,league,rarity`: three tables, no rows | 5 |
| AQ2 | `(<the OQ1 query>) pseudo.total_res>=60`: the old query parenthesised and one new term. The fixture holds an OQ1 match under 60, and the test fails unless it leaves | mechanism 4 (the new term a `sum`) · as worded 7 |
| AQ3 | each row names what matched; a zero answer names the scope searched; `show --against` | 4 · why-not 10 |
| AQ4 | `id:<handle>`, every printed id accepted back (S198) | 4 |
| AQ5 | `"# to maximum Life">=90` sorted: a count and a sorted list across every mod array | 4 |

## What happens to `acq items search`

Nothing, until step 11. It stays as built while the design can still
cycle — the old verb is also a control the owner can run beside the new
one. Step 11 retires, in one commit: the verb (`acq items show` is
already answered by `acq show <id>` with its raw flag, there since step
4, so no inspection is lost), the MCP's substring tool, `Store::search`, and the
`items_names` index it never used (S124, S136) — the README tour line,
`CLI-REFERENCE.md` and `MCP-REFERENCE.md` regenerated. Retiring it needs
the new verb to answer what `--removed` answers: `--membership all`, step
10 (ruled 2026-09-23; `SEARCH-SLICE.md`, "Holes ruled", gap 3).

## Parks whose triggers the build fires

| Park | Fired by | Then |
| --- | --- | --- |
| the totals coverage trial | step 7, before the first recipe was accepted — fired 2026-09-23 | `search/pseudo-stats/scripts/coverage-trial.py` over the deriver's census; the rows stood (M6, `search/MEASUREMENTS.md`) |
| a grouping above class | step 6 reaching OQ5 — fired 2026-09-23; the seat — fired 2026-09-26 (V1: the beasts follow the trade site's categories; OQ5 took 18 terms) | brought to the owner; kept parked, OQ5 pinned from his words, the trigger now the seat (`decisions/search.md`) |
| a unique's variant as a field (C107) | step 4 reaching OQ2 | the trigger reopens the question, not the build: brought to the owner with his Ashes of the Stars as the first test |
| the digest's kill list | step 11, the last acceptance test written | recheck against the tests and the reference before any removal |
| a persisted projection; the store's search-at-scale park | M3, only if the load exceeds 500 ms — an evaluator cost over budget is the totals batch park's (owner, 2026-09-24, `decisions/search.md`) | reported with the numbers; it opens the experiment among the candidates, never persistence by default. The seat goes ahead: slow is not wrong |
| the totals batch (`decisions/search.md`) | M3, a totals ask over 500 ms; or a consumer that holds the corpus asking — fired at step 9 by the letter | V9 (2026-09-26): stays parked, the spikes measured first; revisited at step 11 with a corpus's join time fixed |
| a tab's type as a field (`decisions/search.md`) | the first seat behind it — fired 2026-09-26 | V10: built in 9b |
| pricing on the MCP (`decisions/frontends.md`) | step 11, the read model landed | the owner's call whether it is built then |
| the sticky default realm | the GUI's realm control (2026-09-25), no step of this slice | the GUI stream's |

Every other park's trigger is a recorded question or an ask, which a
seat may produce and a step cannot.

## Measurements the build owes

All on a `.backup` copy, recorded in `search/MEASUREMENTS.md` with the
command that produced them, the verdict one line in `SEARCH-SLICE.md`.

| # | What | When |
| --- | --- | --- |
| M1 | the streaming body read: wall time and peak memory over the whole corpus | step 3 |
| M2 | the deriver against the census: templates, counts, the ranged split, unread shapes | step 2 |
| M3 | a CLI ask, process start to exit, `--json` to `/dev/null`, over the backup copy (its item count recorded), for every acceptance query built so far. Warm: the median of ten consecutive asks. First: the first ask after the copy is written, reported as seen — this machine's file cache is not controlled, so it is never the budget's number. Both profiles: the release build is judged against 500 ms, the debug build is what the seat feels | steps 4 and 5, before the seat; again at 7 and 9 as their queries land |
| M4 | `~` over all displayed text (the reference: "its cost … is unmeasured") | step 4 |
| M5 | a re-derive while a refresh is writing — a mock session over a fixture store; the reload's cost per committed tab, which a resident consumer pays | step 10 |
| M6 | the totals coverage trial | step 7 |
| M3 again | at `94a3d18c`, the six asks the seat saw at 695–715 ms in the script, on the copy: whether the spikes are the ask's or the machine's (V9) | before 9b changes the floor |
| M7 | the resident memory of a held corpus — the derived items and their lines, which a GUI holds all day (M1 measured the streaming read alone, 8.7 MB) | step 10, beside M5 |

Not owed by this build: whether job events reach a client that did not
submit the job (C98) — a resident client's question, the GUI's.

## Holes not yet ruled — the owner's, at the seat

Each under rule 4, with the builder's recommendation, which is no
ruling (an outside audit's finding, 2026-09-20); a ruled one becomes a
line in `SEARCH-SLICE.md`, "Holes ruled", its rule in `search/DESIGN.md`.

| # | Hole | Built meanwhile | Recommendation |
| --- | --- | --- | --- |
| T3 | `pseudo.defence_pct` — V5 (2026-09-26): the site's Base Percentile, to be built (9d), its formula 9c's to evidence. The ranged total: 9c evidences its lines (owner, 2026-09-26: "there's a planned research session that will get us what we need for this") | refused by name until 9d | admitted the day a trade search evidences its lines (9c) |
| T4 | the trial's uncounted lines (M6) — V6 (2026-09-26): every total counts what the trade site's pseudo of that name counts, settled by a human-run trade search where unknown (9c); the owner's caveat: some, such as affix counts, may not be mimicable | the C++ app's rows, unchanged until 9c evidences a change | rule per line from 9c's rows |
