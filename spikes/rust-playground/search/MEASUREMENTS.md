# The item search slice — the measurements

The measurements the build owes (`BUILD-PLAN.md`, "Measurements the
build owes"), each on the `.backup` copy the plan names, with the
command that produced it and its numbers as measured, never recalled.
Read by the block named — the measurement a step repeats, the number a
park's trigger or a review checks — and never whole: the verdicts, one
line each, are `SEARCH-SLICE.md`, "What the measurements taught", which
is what every reader of the record gets. A step that repeats a
measurement adds its row or its rerun here and its verdict there. The
story behind each — what a run first said and what was withdrawn, the
mutants tried, why the suites had passed over what an audit found — is
the record at `aeeba6d3`; what a property means and what it cannot
reach is its own header. Each block opens with its verdict.

**M2 — the deriver against the census (step 2, `ef720323`; rerun at
every change of the deriver since — 0 unread, 0 unexplained each time).** `python3
search/item-facts/scripts/m2-differential.py`, after `cargo build
--workspace`; input the live items of
`item-facts/raw/spike-GERWARIC_7694-2026-09-13.db` (22,721), 2026-09-19.
The script re-runs the census's own template rule over that one input,
hands the same rows to the deriver
(`crates/acquisition-search/examples/derive-census.rs`), applies each
departure below to the census's side with its own code, and compares
every (array, template) row: items, lines, flags, how many numbers, and
whether the ranged rule takes it (`ranged-split.py`'s cases A and B).

| | Census rule | Deriver |
| --- | ---: | ---: |
| templates / lines, all arrays | 6,476 / 88,128 | 6,606 / 86,450 |
| `explicitMods` | 5,304 / 72,868 | 5,278 / 72,506 |
| `implicitMods` | 446 / 6,050 | 440 / 6,050 |
| `utilityMods` | 26 / 2,911 | 23 / 2,911 |
| `ultimatumMods` | 34 / 3,567 | none: not lines |
| `hybrid.explicitMods` (D3) | never read | 199 / 2,251 |
| enchant, crucible, scourge, veiled, bonded, rune | 666 / 2,732 | the same |
| unexplained differences | | **0** |
| items with anything unread | | **0** of 22,721 |
| ranged (one `# to #`): rows / lines | | 201 / 3,708 — 186 the pair alone, 15 with a further number; 2 rows with two pairs, 387 with several numbers and no pair, 166 `(#-#)`, none of them ranged |

The departures, each a rule of `derive.rs` ("As built"), counted by the
script — lines whose template moved / census templates touched:

| Departure | Count |
| --- | ---: |
| `ultimatumMods` is no source of lines — a candidate ground-truth claim: on all 539 items `explicitMods` displays the same, with two lines more on 315 | 3,567 lines |
| a vaal gem's base skill is the source `hybrid` (D3; the census read top-level arrays only) | 2,251 lines added |
| an empty line displays nothing (an essence's spacer rows) | 362 lines |
| a row break `\r\n` is `\n` | 407 / 256 |
| `<style>{Display}` reduced, nested — a candidate ground-truth claim: markup the digest does not name, 821 tags on the copy | 478 / 359 |
| `[Tag\|Display]` and `[Display]` reduced (S4, C90) | 286 / 62 |
| `1,500` is one number (`#x Vivid Crystallised Lifeforce`) | 1 / 1 |

Deriving took 1.4 s for the 22,721 in a debug build, the parse included.

**M1 — the streaming body read (step 3, `c0a8f918`): 36 ms release and 8.7 MB resident over the whole corpus.** `/usr/bin/time -l
target/<profile>/examples/read-corpus
search/item-facts/raw/spike-GERWARIC_7694-2026-09-13.db <mode>`
(`crates/acquisition-store/examples/read-corpus.rs`), 2026-09-20: 22,721
items, 29.9 MB of bodies, 3,509 live locations, 4 listings, revision
1126 at facts v7; the file cache warm, the median of three runs, which
lie within 9 ms of each other.

| Mode | Release | Debug | Peak resident |
| --- | ---: | ---: | ---: |
| `stream`: each item dropped once counted | 36 ms | 94 ms | 8.7 MB |
| `hold`: every item kept as text to the end | 37 ms | 96 ms | 51.7 MB |
| `parse`: each body parsed as JSON, then dropped | 113 ms | 829 ms | 9.3 MB |

The header is 6 ms of each release read (16 ms debug): 4 ms with the
realms read from the locations alone, 6 with the listings and the
orphan exclusion. With M2's 1.4 s to derive in a debug build, an ask at
the seat's debug build is about 2 s before anything is evaluated; the
release build is what 500 ms is judged against (M3). The read's order —
location kind and id, then realm, then item id — was chosen on a
measurement: realm first sorted the whole corpus, bodies included,
43 MB resident against 10 MB and 0.13 s against 0.04 s in the `sqlite3`
shell.

**M3, M4 — a CLI ask, and `~` over all text (step 4, `28606ed4`): every release ask under 500 ms; `~` over every displayed string adds 9 ms.**
`python3 search/item-facts/scripts/m3-ask.py
search/item-facts/raw/spike-GERWARIC_7694-2026-09-13.db`, after the
three builds its header names, 2026-09-20. The script writes a fresh
`.backup` of the copy under `raw/m3/`, wraps it as a store directory
(`crates/acquisition-search/examples/copy-as-store.rs`) and asks through
`acq --json search --realm pc … > /dev/null` under `ACQ_PROVIDER=mock`
and `ACQ_STORE_DIR`: 22,623 live pc items of the copy's 22,721 (the rest
are poe2's), revision 1126, facts v7. Twice: the build at `28606ed4`,
first ask 418 ms, and the build the first two audits left (`ad5589c9`),
first ask 270 ms. Warm, the median of ten, in ms:

| Ask | Release at `28606ed4` | Release, audited | Debug, audited |
| --- | ---: | ---: | ---: |
| the empty query | 254 | 268 | 1,727 |
| OQ1 (`base:ring` for the class) | 267 | 287 | 2,192 |
| OQ1 refined by a fractured line | 262 | 283 | 1,998 |
| OQ2 `name="…"` | 254 | 269 | 1,733 |
| OQ3 a mod · a base | 259 · 253 | 276 · 270 | 1,892 · 1,732 |
| OQ4 a base and a phrase | 264 | 281 | 1,969 |
| OQ7 one line everywhere | 255 | 271 | 1,741 |
| AQ2 OQ1 and a `sum` over `template:resistance` | 271 | 297 | 2,427 |
| AQ5 life at 90, sorted | 256 | 284 | 1,818 |
| M4 a phrase · `text~"explo(de\|sion)s?"` · `text~"^adds [0-9]+ to [0-9]+"` | 261 · 262 · 260 | 277 · 277 · 275 | 1,893 · 1,895 · 1,817 |

Every release ask is under 500 ms, so the persisted-projection park's
trigger does not fire. An ask is its load: the empty query, which
evaluates nothing, is 268 ms, the dearest query adds 29 ms, and `~` over
every displayed string of every item adds 9 ms (M4). The debug build is
1.7 to 2.4 s, which is what the seat feels behind the README's alias.
What step 4's audit fixes cost, the two release binaries asked in turn
on the same copy in the same minutes, the median of eleven: 12 to 31 ms
an ask, five to twelve in a hundred — the deriver's flag checks in
every ask and the three-valued group in those that have one.

**The class table against the census (step 6, `eeaec66e`): 56 of 82 classes carried, 681 of 22,721 undecided, the buckets summing to the copy.**
`target/release/acq --json search --realm all --count class --limit 100`
over the M3 copy (`raw/m3/store`, 22,721 items, revision 1126, facts v7),
2026-09-23, after the table was regenerated with every release state
(G3). 56 classes carried, the six largest Skill Gems 3,596, Contracts
2,641, Utility Flasks 2,337, Support Gems 2,100, Rings 1,133, Jewels
1,035; `undecided` 681, its tally *base not in the table* 388, *base
under several classes* 195, *no table for the realm* 98; the buckets sum
to 22,721 (C105). The same ask before G3 was reversed had Blade Trap
among the 393 not in the table and 686 undecided. M2 rerun at the step:
0 unread, 0 unexplained; `reqlevel` reads a whole number on every item
of the copy that has a `Level` requirement.

**M6 — the totals coverage trial (step 7, `b5d62d92`; the park fired): every row carried by the corpus, the rows shipped unchanged.**
`python3 search/pseudo-stats/scripts/coverage-trial.py` over the
deriver's census of the copy (`item-facts/raw/m2/rust.json`: 22,721
items, 6,148 templates) against the shipped table (46 templates named by
a total), 2026-09-23. For each word, the templates holding it, the lines
displaying those templates, and how many a total counts — lines, since
the census counts items per (source, template) and an item carrying a
template twice, or two of them, is two incidences and one item (outside
audit, 2026-09-24); a word-based trial measures coverage, never
membership. Every row of the table is carried by the corpus.

| Word | Templates · lines | Counted by a total | Counted by none | What none counts |
| --- | ---: | ---: | ---: | --- |
| resist | 187 · 7,499 | 11 · 5,047 | 176 · 2,452 | maximum resistances (144, 133, 128), `Resistant Monsters` (123), players' and monsters' (119, 105, 105), per Alert Level (84–45), `Added Small Passive Skills also grant` (75–30), during flask effect (62, 7), minions' and allies' (47, 42, 32), exposure and penetration, conditional (`while on Low Life` 10, `while affected by Herald of …` 13, 9, 6; `when Socketed with a … Gem` 4 each), `(#-#)` descriptions |
| strength | 70 · 1,021 | 4 · 827 | 66 · 194 | `Added Small Passive Skills also grant: # to Strength` (41), `#% increased Strength` (26), requirements, per-Strength lines, `Other Item: (#-#) to Strength` (6) |
| dexterity | 54 · 1,049 | 4 · 846 | 50 · 203 | the same shapes (40, 35) |
| intelligence | 55 · 844 | 4 · 660 | 51 · 184 | the same shapes (39, 23) |
| attack speed | 70 · 1,465 | 1 · 404 | 69 · 1,061 | `Reinforcements have …` (357), monsters' (128), supported skills' (64, 49), allies' (37), during effect (34), `Grants …` (28); **`#% increased Attack and Cast Speed` (46), counted by neither speed total** |
| cast speed | 65 · 1,307 | 1 · 268 | 64 · 1,039 | the same shapes (357, 128, 71), and the 46 above |
| level of socketed | 29 · 395 | 22 · 346 | 7 · 49 | `# to Level of Socketed AoE Gems` (20) and `Duration Gems` (12): no C++ table, no total |

The verdict on the recipe's rows: every template the C++ rows name is
carried by the corpus, and the site's own answer on the ten fetched
items of q5 (`search/trade-query/data/fetch-census.json`) agrees with
`total_fire_res` on every one, worked by hand — 14 + 48 + 18 = 80,
18 + 48 + 15 = 81, 16 + 48 + 18 = 82, and the seven with one line — so
the rows shipped unchanged. What the trial found that no total counts —
the combined speed line, the AoE and Duration gem levels, the
conditional resistances — is coverage the rows never claimed; whether
any joins a recipe is the owner's (the plan's foot, T4).

**M3 at each later build** — the same command, the release build judged
against 500 ms; every release ask of every build below is under it but
one — the vocabulary whole at the audited step 7 build, measured beneath
the table, ruled at T5 and under again at `57ec6a9e`. What an ask over
budget fires is the owner's (2026-09-24): "M3 reports asks over 500 ms,
but only a load over budget triggers the projection experiment. An
evaluator cost over budget lands against the batch park." The debug
build is what the seat feels behind the README's alias. Step 4's per-ask
table is above; here the first ask after the copy is written, and the
range of the warm medians, in ms. This table is the standing benchmark
ledger until the first M3 rerun after the slice's close, which moves it
to its own append-only file in `RUN-LEDGER.md`'s mold, no budget.

| Build | Where | First ask | Warm, release | Warm, debug |
| --- | --- | ---: | ---: | ---: |
| `28606ed4` | step 4 as committed | 418 | 253–271 | — |
| `ad5589c9` | step 4, the first two audits' fixes | 270 | 268–297 | 1,727–2,427 |
| `36a9738a` | step 4b, part 2 (`group.rs`) | 445 | 270–299 | 1,739–2,438 |
| `3c7d31f3` | step 4b, after the fifth audit | 491 | 266–296 | 1,729–2,441 |
| `b4d7a7d0` | step 4b, the close | 486 | 269–304 | 1,762–2,489 |
| `31b5f09d` | step 5, with its six asks added to the script | 435 | 269–305 | 1,756–2,480 |
| `6f352523` | step 5, the audit's fixes | — | 269–306 | — |
| `99bd0962` | step 5, the review's fixes | — | 270–306 | — |
| `eeaec66e` | step 6, with its three asks added to the script | 291 | 275–309 | 1,789–2,514 |
| `b5d62d92` | step 7, with its four asks added to the script | 507 | 278–359 | 1,851–2,748 |
| `4237d2f3` | step 7, the two audits' fixes | 465 | 275–360, the vocabulary whole 954 | 1,852–2,748, the vocabulary whole 6,555 |
| `57ec6a9e` | step 7, T5 | — | the vocabulary whole 316 | the vocabulary whole 2,130 |
| `ec024ceb` | step 8, with its four asks added to the script | 448 | 276–370 | 1,862–2,762 |

The asks each step added to the script, at the step's build, warm
medians in ms; a step's empty query is its floor.

| Build | Ask | Release | Debug |
| --- | --- | ---: | ---: |
| `31b5f09d`, step 5 | AQ1 `--count tab,league,rarity` | 276 | — |
| | OQ7 `--count tab` | 274 | — |
| | `--cross league,tab` | 274 | — |
| | `--count base --sum stack` | 273 | — |
| | the vocabulary twice narrowed | 286 | — |
| | the vocabulary whole: every template of 22,623 items ranked, 6,113 rows, 20 listed | 302 | — |
| `eeaec66e`, step 6 | the empty query | 275 | — |
| | OQ1 as worded (`class:ring`) | 297 | — |
| | OQ5 by class and level | 277 | — |
| | `--count class` | 277 | — |
| `b5d62d92`, step 7 | the empty query | 278 | — |
| | AQ2 as worded, `pseudo.total_res>=60` over OQ1 | 359 | 2,748 |
| | the worked example whole | 356 | 2,405 |
| | `pseudo.dps>=100` sorted by it | 280 | 1,867 |
| | `--count class --sum pseudo.total_res` over the rares | 301 | 2,021 |
| `4237d2f3`, step 7 audited | the empty query | 275 | — |
| | the vocabulary whole | 954 | 6,555 |
| | the same binary, the computed-value scan skipped | 323 | — |
| | `line:total_res` (19 rows) · `line:total_fire_res` · `line:pdps` · `line:resist` (11 totals) | 356 · 315 · 279 · 524 | — |
| | the vocabulary twice narrowed | 306 | — |
| `57ec6a9e`, T5 | the vocabulary whole | 316 | 2,130 |
| `ec024ceb`, step 8 | the empty query | 276 | 1,862 |
| | the four socket asks (OQ3 by colour, OQ3 within one link group, `links` sorted, `--count links`) | 281–282 | 1,864–1,872 |

What the table says: a class table's or a totals table's parse is
within the noise of the empty query; a total asked of every item is the
dearest ask, its 19 rows each a group asked of each line, 80 ms over the
empty query; the vocabulary whole at `4237d2f3` was the computed-value
scan `line` alone gained at `e63c86a8` (an outside review's finding,
2026-09-24), costing by the table's rows and not its names, which no
limit can cut since the ranking needs every count first — T5 ruled it.
A run can have a slow half of unknown cause (the run before step 6's
row: 554–621 ms release for its first thirteen asks, 277–309 for the
last nine), so a run is claimed only when its halves agree, and a
slowdown's cause is never named before it is measured.

**The completion property's measured half: no counterexample, at step 4b and over step 8's generators** —
`cargo test -p acquisition-search --test generated_completion -- --ignored --nocapture`,
2,000 cases from a fixed seed. Run 2026-09-20 before part 2 and after it,
the same numbers both times; the figures here are the run of 2026-09-21,
over the generators as the fifth audit left them (decimal lines, a second
spelling, selectors that resolve to nothing), 174 of the cases authoring
errors. What was decided as stored and held under all fifteen
completions: 3,400 terms, 1,288 roots, 3,782 sort scalars;
counterexamples, none. What was undecided as stored:

| Undecided | Completions differed | Every one said no | Every one said yes |
| --- | ---: | ---: | ---: |
| a field, a flag or a phrase | 167 | 49 | 0 |
| a line's group | 419 | 535 | 0 |
| a sum's comparison | 110 | 164 | 99 |
| the root | 227 | 226 | 85 |
| a sort scalar | 953 | 888 with one value | |

The column that matters is the empty one: no undecided group, field,
flag or phrase was made true by every completion, which is where a
missed witness would show — with an unread slot read as a no again, 83
groups and 18 fields in 600 cases are, and the property refuses at once.
The rest are undecided rightly: a case the sampler cannot reach (a
template at one value, a selector nothing carries, a bound no generated
value breaks), a group no occurrence can satisfy (two quoted templates
in one and), and a comparison on an incomplete subtotal, undecided
whatever the subtotal already reaches. Rerun 2026-09-24 over the generators as step 8 left them: no counterexample; 16 groups and 29 fields were made true by every completion, each read — a decimal line's completions stay in tenths, a junk socket's group is drawn at random, a comparison on an incomplete subtotal is careful by rule — so the column is the sampler's reach now, not the evaluator's.

**The vocabulary against M2 (step 5, `31b5f09d`): 6,181 rows and 6,148 templates agree.** Over all realms the
vocabulary is 6,181 rows keyed by (realm, template); the deriver's
census rows of M2 (`item-facts/raw/m2/rust.json`, 6,606 by (source,
template)) hold 6,148 distinct templates, and 6,181 less the 33
templates the copy carries in both realms is 6,148.

**Step 8 — the sockets over the copy (`ec024ceb`): every bucket a value or `none`, `undecided` 0.** `--realm all
--count` of `sockets`, `links` and each colour: every bucket a value or
`none`, `undecided` 0, each table summing to 22,721; sockets on 3,456
items, 0 on the four `[]`, `links` on 3,452. The copy's socket shapes:
9,687 `{group, attr, sColour}`, 47 poe2 `{group, type}`; every group is
one contiguous run, so the C++ run rule (S22) and GGG's number agree
here; an abyssal socket (73 items) and a resonator's (7) is always a
group of its own.
