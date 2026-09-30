# Item search — the build plan (stage 6)

The brief for the build, read whole before any search step. What each
step met — the audits and their fixes, the measurements, the holes ruled
and the builder's observations — is `SEARCH-SLICE.md`, the record; the
measurements' tables are `search/MEASUREMENTS.md`, read by the block a
step repeats; which steps are built is the ledger, `search/LEDGER.md`,
never this file. A built step's row leaves this file at the step's close: what
each was to close on is this file at `880d9387`. What this file held
before the record was opened — every audit round, the step 4b brief, the
first showing of the store's read — is this file at `2d25cb03`.

Ruling (owner, 2026-09-21): SEARCH-SLICE.md is opened now, at step 5, in
PRICING-SLICE.md's mold, rather than at the slice's close as
search/BUILD-PLAN.md says. The plan stays the brief.

What a step reads (2026-09-23, after a measured read of the slice's
documents: 308 KB before any code, seven times what a step needs):
this file whole; `decisions/search.md`; what its row's *Reads* names;
the record; `search/LEDGER.md` by the row — a hole's id before one is
brought; `search/MEASUREMENTS.md` by the block it will repeat; a
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
   foot of this file; a ruled one is a line in `search/LEDGER.md`,
   "Holes ruled", its evidence the ruling commit's message.
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

## The first seat — after step 9, an agent's with the owner in the loop

The seat is an agent's, the owner in the loop (owner, 2026-09-25; P2):
the owner will not type queries himself. He talks to an agent that
drives `acq search` from a terminal on his real store, reacts to what
comes back, and collects the agent's own feedback on the contract. The
owner's own seat is the GUI's, on its own stream. An agent's feedback
is cheap and repeatable, the owner's reaction needs him once per
surface. If the store holds more than one realm every search names
`--realm` (C96); an agent states it each time, so the default realm's
trigger is the GUI's (`decisions/frontends.md`, "Parked").

Sat 2026-09-26 on the release build at `94a3d18c` (`search/LEDGER.md`, the
row "the first seat"): ten faults, F1–F10, and ten verdicts,
V1–V10, its "Holes ruled". The owner's order for what follows
(2026-09-26, V10 and after): a fix session against the seat's hash,
then the trade site's computed values as a research track, then
percentile, category and step 10. Why the seat came after step 9, what
it could ask and what it refused by name: this file at `880d9387`.

## The steps left — provisional, his to reorder

| Step | Builds | Needs | Reads | Closes on |
| --- | --- | --- | --- | --- |
| 9d · percentile and category | `pseudo.defence_pct` as the site's Base Percentile (V5), by the site's own rule, pinned at 9c; the ranged family's 21 totals, whose rows mean one of two stats a category tells apart — a weapon's own line or not (owner, 2026-09-26: "agree with 9d"); category as a grouping above class (V1: the beasts are its first case), from the trade site's categories and/or RePoE under C106's admission test (owner, 2026-09-26), computed from class and reviewed base rules in one place (rule 10), never a second classifier | 9c | C101, C106; `class.rs`, `totals.rs`; the grouping park's entry | OQ5 by category; a beast found by its category; `Adds # to # Fire Damage` on a weapon counted toward attacks alone |
| 10 · continuing and exchanging | in the order the seat wanted them (the sitter, 2026-09-26): `--explain` and `show --against` first — a zero answer was where they were missed; `--next` and `--request` for the MCP, an agent pages and replays; `--view locations`, `--fields`, `--rebind` and `show --basis` had no consumer at the seat and wait for one. `--next` refused across a changed basis; `--membership all` (the reference, *Membership*): the read hands removed items over with `removed_at`, every such row marked, the scope block counting each, an `id:` term that matches nothing live naming the removed id and its route, `acq show` on a removed item; `--print-request`, `--request`, `--rebind`; `show <id> --against`; `--explain`; `--context corpus`; `--view locations`; `--fields` | 4 | C98, C100, C104, C108; the reference, *Membership*, *Explain*, *Outside the first surface*; `answer.rs`, `corpus.rs`; gap 3 (the record) | AQ3 whole; AQ4 across a refresh that removed the item; M5 |
| 11 · the close | the MCP tool over the same request, with an agent's seat as its own consumer (P2), `--membership all` among what it validates; `acq items search` retired (below); the closed record, and whether its rows fold back into it from `search/LEDGER.md` or stay a file looked up (the owner's); this file deleted; `search/` shrunk; `DESIGN.md`'s paragraphs into module docs | all | everything above; `decisions/frontends.md`; the kill list (`DIGEST.md`); `MCP-REFERENCE.md`; "What happens to `acq items search`" below | AQ1–AQ5 driven through the MCP; the gate; the kill-list recheck |

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
10 (ruled 2026-09-23; `search/LEDGER.md`, "Holes ruled", gap 3).

## Parks whose triggers the build fires

| Park | Fired by | Then |
| --- | --- | --- |
| the totals coverage trial | step 7, before the first recipe was accepted — fired 2026-09-23 | `search/pseudo-stats/scripts/coverage-trial.py` over the deriver's census; the rows stood (M6, `search/MEASUREMENTS.md`) |
| a grouping above class | step 6 reaching OQ5 — fired 2026-09-23; the seat — fired 2026-09-26 (V1: the beasts follow the trade site's categories; OQ5 took 18 terms) | brought to the owner; kept parked, OQ5 pinned from his words, the trigger now the seat (`decisions/search.md`) |
| a unique's variant as a field (C107) | step 4 reaching OQ2 | the trigger reopens the question, not the build: brought to the owner with his Ashes of the Stars as the first test |
| the digest's kill list | step 11, the last acceptance test written | recheck against the tests and the reference before any removal |
| a persisted projection; the store's search-at-scale park | M3, only if the load exceeds 500 ms — an evaluator cost over budget is the totals batch park's (owner, 2026-09-24, `decisions/search.md`) | reported with the numbers; it opens the experiment among the candidates, never persistence by default. The seat goes ahead: slow is not wrong |
| the totals batch (`decisions/search.md`) | M3, a totals ask over 500 ms; or a consumer that holds the corpus asking — fired at step 9 by the letter | V9 (2026-09-26): stays parked, the spikes measured first; revisited at step 11 with a corpus's join time fixed |
| a tab's type as a field (`decisions/search.md`) | the first seat behind it — fired 2026-09-26 | V10: built at 9b (`tab.type`), the entry deleted |
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
line in `search/LEDGER.md`, "Holes ruled", its rule in `search/DESIGN.md`.
J1–J7 are what the two tracks run before 9d surfaced (2026-09-30;
`search/category/README.md`, `search/pseudo-stats/README.md`, "The
percentile's round", "The ranged family on the copy"), brought before
the build so that none is met mid-diff; a J the owner cannot rule from
what he knows is a search of the sitting he runs next.

| # | Hole | Built meanwhile | Recommendation |
| --- | --- | --- | --- |
| T3 | `pseudo.defence_pct` and the ranged totals: each evidenced at 9c (`search/pseudo-stats/README.md`), built at 9d. Ruled 2026-09-26: a ranged total asked with no slot word is its `avg`, the one number the site shows ("yes, bare means average") | refused by name until 9d | — |
| J1 | a beast's leaf: nothing on this machine says which of `monster.beast`, `monster.yellowbeast`, `monster.redbeast` a captured beast is under; all 158 on the copy are frame Rare; three readings cited to sources split them 158/0/0 (one category), 0/158/0 (the frame), 0/122/36 (a lure word on the base); the game's data gives a beast no base and no class (category F3, F4) | every beast `undecided(category)` with that reason; class as today | if the owner knows what the site's colours mean in the game, his word is the reading, tested by the six searches of category open question 1; if not, the searches first |
| J2 | a parent's own items: `map` holds 1,724 of the copy's items directly (Inscribed Ultimatums, other Misc Map Items), so a parent is not only the union of its leaves, which K7 assumed (category F2) | — | a term names the site's id as spelled; a parent matches what the site lists at it or under it by the dot; open question 3 asks the site whether `map` holds those items |
| J3 | invitations: the export's name rule puts all 165 of the copy's invitations under `map.invitation`, whose text is "Maven's Invitation", and none of them is Maven's (category F1, F5) | those items `undecided(category)`, the rule's reason | the site's answer (open question 4) before the rule ships |
| J4 | the groupings: seven ids (`weapon.one`, `weapon.dagger`, …) whose members no captured file names; one fetch shows `weapon.onesword` holds base swords and rapiers both (category F2) | a grouping refused by name (rule 1) | open question 2's searches, one per member in doubt; a grouping is then a term over the members the site answered |
| J5 | the percentile at an exact half: the stated rule reproduces 541 of 543 captured items, the two misses display at an exact half; reading the half as either integer reproduces all 543, each still one value; no site filter selects such an item, so no search asks it; the copy holds 9 (pseudo-stats, "The percentile's round") | — | adopt: a display at an exact half admits both integers as rolls, and the definition prints that as its one departure from the site's tip |
| J6 | the shapes round i asks (three defence types, the quality enchant, a roll over the maximum, a defence line under its global twin, ward): before it is sat, does the build answer them by the stated rule or refuse the item? | — | the stated rule, each shape listed in the record as unpinned until the round is read; a refusal would make the field lacked on every ward base for a reason C93 does not have |
| J7 | the ranged limit: 573 of the copy's 703 unsuffixed lines sit on weapons; neither the export nor the site calls any weapon line global; 36 weapon lines on uniques the text join cannot place, 19 of them uncaptured, which i06–i08 ask (pseudo-stats, "The ranged family on the copy") | — | the class decides, a weapon's line is its own; every ranged total prints the limit: a global line on a weapon is read as the weapon's own, and i06–i08 say how often the site disagrees |
