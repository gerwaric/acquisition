# The item search slice — the measurements

The measurements the build owes (`BUILD-PLAN.md`, "Measurements the
build owes"), each on the `.backup` copy the plan names, with the
command that produced it and its numbers as measured, never recalled.
Read by the block named — the measurement a step repeats, the number a
park's trigger or a review checks — and never whole: the verdicts, one
line each, are `SEARCH-SLICE.md`, "What the measurements taught", which
is what every reader of the record gets. A step that repeats a
measurement adds its row or its rerun here and revises its verdict
there. The
story behind each — what a run first said and what was withdrawn, the
mutants tried, why the suites had passed over what an audit found — is
the record at `aeeba6d3`; what a property means and what it cannot
reach is its own header. Each block opens with its verdict; M3's
moves with every build and is the record's line.

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
to 22,721 (C105). Of the 388 not in the table, 230 are blighted and
blight-ravaged maps (`Blighted Map (Tier N)` is no export base, though
`Blighted Map` is) and the rest itemised beasts foremost, 71 distinct
names in all; the 195 under several classes are the Eldritch and
Maven's invitations foremost; no gem is among them, `gems.json` having
supplied every transfigured gem of the copy, 272 items step 2's census
could not join by base. The same ask before G3 was reversed had Blade Trap
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
any joins a recipe is the owner's (V6: `search/LEDGER.md`, "Holes
ruled").

**M3 at each later build** — the same command, the release build judged
against 500 ms. The verdict moves with every build and is the record's
line, revised in place (`SEARCH-SLICE.md`, "What the measurements
taught"); what each rerun showed is beneath the tables. The vocabulary
whole at the audited step 7 build, measured beneath the table, was
ruled at T5 and is under again at `57ec6a9e`. What an ask over
budget fires is the owner's (2026-09-24): "M3 reports asks over 500 ms,
but only a load over budget triggers the projection experiment. An
evaluator cost over budget lands against the batch park." The first
seat ran the release build (the owner's choice, 2026-09-26), so what it
felt is the number the budget judges; its wall times are the seat block
at the foot. Step 4's per-ask
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
| `57c2f78b` | step 9, with its four asks added to the script | 622 | 443–529: two over, AQ2 as worded 529 and the worked example whole 528 | 2,282–3,184 |
| `1e0d85c6` | step 9, the review's fixes (one two-path extract for the note and the slot) | 1,113 | 443–528, the same two over; the series' empty query 902 with the copy's cache still cold, 446–449 asked alone afterwards | 2,285–3,191 |
| `7ff3c947` | step 9, the second look's fixes (the extract read by a flat scanner) | — | three asks alone: the empty query 442, `has:priced` 443, the worked example whole 522 | — |
| `94a3d18c` | the seat's hash, again with the six asks the seat saw at 695–715 ms added to the script (V9) | 587 | 427–530, the same two over: AQ2 as worded 530 and the worked example whole 528 | 2,275–3,211 |
| `0e4c91ad` | step 9b, the seat's fixes (the `-p acquisition-cli` form, sha256 `c49086ed…`) | 635 | 441–529, the same two over: AQ2 as worded 529 and the worked example whole 523; the empty query 448 | 2,294–3,206 |
| `e81cc8ca` | step 9c, totals v2: 155 rows where v1 held 104 (the `-p acquisition-cli` form, sha256 `c2766a44…`) | 629 | 445–579, the same two over and further: AQ2 as worded 579 and the worked example whole 578, 50 and 55 ms more than at 9b; every other ask 445–488, the empty query 446 | 2,348–3,614 |
| `9842d0c9` | step 9c2, totals v3: 60 totals and 283 rows where v2 held 36 and 155, with its three asks added to the script (the `-p acquisition-cli` form, sha256 `13fa9955…`) | 615 | 452–508 but the two totals asks of before, three over: AQ2 as worded 588 and the worked example whole 587, 9 more each than at 9c as the empty query is 7 more, and the type's damage with attack skills 508; every other ask 452–497 | 2,421–3,167 but the totals' three: 3,689, 3,347 and 2,788 |
| `c28c476a` | step 9c3, totals v5: four readings of other totals, with their four asks added to the script (the `-p acquisition-cli` form, sha256 `ed3b02fa…`) | 458 | 450–496 but seven over: the three totals asks of before at 593, 584 and 506; three of the readings' four, 640, 611 and 522; and the vocabulary twice narrowed at 514, whose narrowing `resist` three readings' definitions match — 499 at `23991fe1` and 520 here, the two binaries asked in turn | 2,432–3,162 but the seven: 3,708, 3,356, 2,799, 3,806, 3,543, 2,886 and 2,894 |
| `bd4679e1` | step 9c4, a mod's rows kept by the deriver, with its two asks added to the script (the `-p acquisition-cli` form, sha256 `cb391165…`) | 475 | 464–499 but twelve over, the empty query 467 and an ask 12 to 22 ms above what `7435874e` asks in turn (the step's block): the seven of before at 606, 608, 527, 668, 631, 538 and the vocabulary twice narrowed 530; four the floor carried over, the `sum` over `template:resistance` 510, the vocabulary whole 509, `--count class --sum pseudo.total_res` 514 and `pseudo.total_mana` 501; and the step's own, two readings in one ask, 817 | 2,537–3,012 but the twelve: 4,036, 3,709, 3,011, 4,274, 3,942, 3,106, 3,070, 3,289, 2,843, 2,908, 2,789 and 5,657 |
| `0962588b` | step 9c4, the review's fix: a slot read of a part that may hold, never of every line (the `-p acquisition-cli` form, sha256 `71f97b38…`) | 569 | 472–498 but eight over: the totals' asks 64 to 223 ms under `bd4679e1`'s — 542, 541, 546, 531 and the two readings in one ask 594 — and every other ask some 8 above it, the empty query 475, which carries the `sum` over `template:resistance` 511 and the vocabulary's two, 510 and 519 | 2,659–3,152 but the eight: 3,879, 3,555, 3,928, 3,679, 4,999, 3,401, 3,087 and 2,976 |
| `ca9f53bc` | step 9c4, a group's quoted tests compiled once (the `-p acquisition-cli` form, sha256 `cfdec97b…`) | 599 | 455–494 but six over, the empty query 461: the two totals asks of step 7 at 521 and 522, two readings at 528 and 522, the two readings in one ask 578, and the vocabulary whole 507 | 2,438–3,178 but the six: 3,666, 3,340, 3,708, 3,467, 4,792 and 2,751 |

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
| `57c2f78b`, step 9 | the empty query | 445 | 2,285 |
| | `has:priced` · OQ6 (`name="Ashes of the Stars" has:priced`) · `--count price.currency --sum price.amount` over `has:priced` | 443 · 446 · 447 | 2,282 · 2,287 · 2,283 |
| | `price.amount>=1` sorted by it | 487 | 2,285 |
| `94a3d18c`, the seat's six (V9) | seat 3 the empty query · 24 `frame=quest --count base,container` · 49 `class=rings --limit 0` · 53 OQ2 `--count line:reservation,attributes` · 60 `--count line~^adds # to # cold` · 62 `"Kaom"` | 457 · 451 · 446 · 459 · 458 · 452 | 2,303 · 2,322 · 2,304 · 2,401 · 2,387 · 2,472 |
| `0e4c91ad`, 9b | the seat's six again | 443 · 453 · 450 · 454 · 452 · 447 | 2,294 · 2,313 · 2,299 · 2,392 · 2,377 · 2,466 |
| `e81cc8ca`, 9c | the two totals asks (AQ2 as worded · the worked example whole) · `--count class --sum pseudo.total_res` | 579 · 578 · 488 | 3,614 · 3,273 · 2,660 |
| `9842d0c9`, 9c2 | `pseudo.total_mana>=100` (5 rows) · `pseudo.total_attack_speed<0` (4) · `pseudo.increased_lightning_attack_damage>=30` (10) | 479 · 475 · 508 | 2,610 · 2,575 · 2,788 |
| `c28c476a`, 9c3 | `pseudo.count_res>=3` (37 rows of four totals) · `pseudo.count_ele_res=3` (30 of three) · `pseudo.total_all_ele_res>=30` (the same 30) · `pseudo.total_all_attributes>=10` (12 of three) | 640 · 611 · 522 · 481 | 3,806 · 3,543 · 2,886 · 2,611 |
| `bd4679e1`, 9c4 | `"#% to all Elemental Resistances"<0`, a mod named by a row · `name="Thread of Hope" pseudo.count_res=3 pseudo.count_ele_res=3` (two readings, 67 rows of the totals they read) | 487 · 817 | 2,667 · 5,657 |
| `ca9f53bc`, 9c4 after its review | the same two · the four readings of 9c3 (`pseudo.count_res>=3` · `pseudo.count_ele_res=3` · `pseudo.total_all_ele_res>=30` · `pseudo.total_all_attributes>=10`) | 477 · 578 · 528 · 522 · 484 · 470 | 2,576 · 4,792 · 3,708 · 3,467 · 2,858 · 2,607 |

**The price join's cost (step 9, `57c2f78b`): the floor rose from 276
to 445 ms release, and two totals asks crossed 500 ms with it.** The
same copy, the same script, the same minutes: the empty query 276 → 445
ms release and 1,862 → 2,285 debug, every earlier ask up by the same
150–180 ms, and the two asks whose evaluator cost is the totals' — AQ2
as worded 359 → 529, the worked example whole 356 → 528, their own cost
84 ms at step 7 and 84 now — over the budget on the higher floor. The
join is one pricing snapshot and one `resolve` per (realm, league) the
scope names; the copy's pc realm holds two, Standard (21,311 items) and
Allflame (1,312), and `target/release/acq --json price status --realm pc
--league <l>` over the copy — process start to exit, the median of ten
— is 119 ms for Standard and 13 for Allflame, which is the join's cost
within the noise. What the 119 ms is made of — the snapshot's
`json_extract` over 21,311 stored bodies against `resolve`'s 21,311
listings — is unmeasured; the review's fix, which reads both body
fields in one two-path extract, changed no ask by more than the noise
(`1e0d85c6`). The load, the empty query, stands at 445 ms:
55 under the budget, so the projection park's trigger does not fire by
the owner's rule (2026-09-24), and the two asks over it are an
evaluator cost on a floor the join raised — a shape the ruling did not
name, brought to the owner (`SEARCH-SLICE.md`, "Observations still
open", step 9). Over the copy's fresh intent file the game alone prices
what `has:priced` finds; the count is the answer's, not recorded here.

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

What each rerun showed, in the order of the builds:

- M3, M4 (step 4, again at every build since): every release ask under
  500 ms, so the projection park does not fire; the one ask over it, the
  vocabulary whole at `4237d2f3`, was ruled at T5 and is under again;
  `~` over all text adds 9 ms; the debug build is 1.7 to 2.8 s.
- M3 (step 9): the price join raised every release ask 150–180 ms (the
  empty query 276 → 445); the load under 500, so the projection park
  does not fire; two totals asks over 500 on the higher floor, ruled at
  the seat (V9).
- M3 again at the seat's hash (9b, V9): the six asks the seat saw at
  695–715 ms are 446–459 on the copy — the spikes did not recur, their
  cause unmeasured; the totals batch stays parked. M3 at 9b's hash
  (`0e4c91ad`): the empty query 448, every ask 441–529, the same two
  totals asks over 500; the fixes moved no ask past the noise.
- M3 at totals v2 (`e81cc8ca`): every ask 445–488 but the totals' two,
  579 and 578, 50 and 55 more than at 9b: a total's cost is its rows,
  155 where there were 104.
- M3 at totals v3 (`9842d0c9`): the two are 588 and 587, moved with the
  floor and not by the table, whose 128 new rows are other totals';
  a third ask is over, a total of 10 rows at 508. An ask pays for the
  rows of the total it names, near 6 ms each over a floor of 452.
- M3 at totals v5 (`c28c476a`): a reading pays for the rows of every
  total it reads — the two counts 640 and 611, near 5 ms a row over a
  floor of 457 — but a least, which stops at the first total that is
  nothing: 522 and 481 over 30 rows and 12. Seven asks are over, the
  vocabulary twice narrowed among them, 499 before and 514: its
  narrowing matches three readings.

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
whatever the subtotal already reaches. Over step 7's third-look generators (`ecb83b65`), 9 groups failed as stored and lacked under a completion in 2,000 cases, against 19 and 21 with step 4b's. Rerun 2026-09-24 over the generators as step 8 left them: no counterexample; 16 groups and 29 fields were made true by every completion, each read — a decimal line's completions stay in tenths, a junk socket's group is drawn at random, a comparison on an incomplete subtotal is careful by rule — so the column is the sampler's reach now, not the evaluator's.

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

**The first seat's wall times (`94a3d18c`, release, 2026-09-26): 48
search asks, median 445 ms; eight over 500, six of them cheap asks at
695–715 ms of unknown cause.** The wrapper's wall time
(`runs/seat-2026-09-26/ask.sh`), process start to exit, on the owner's
store (22,623 pc items), not the copy; one binary throughout. The 48
successful search asks (`--describe` and `--realm all` among them):
median 445, least 17 (`--describe`). Over 500: the 18-term OQ5 (ask 52,
509), the totals ask (56, 531), and six at 695–715 — asks 3 (the first
empty query), 24 (`frame=quest`), 49 (`class=rings --limit 0`), 53 and
60 (vocabulary counts), 62 (a bare phrase) — whose spread is larger
than the totals' own cost and whose cause is unmeasured. poe2 (98
items) 41 ms; `show` 127–152; a refusal 8–21. The disposition (V9,
owner): the totals batch stays parked; the spikes are measured first —
an M3 rerun at this hash, on the copy, with the six asks in the script.

**M3 again at the seat's hash (`94a3d18c`, 2026-09-26, before 9b changed the floor): the six asks the seat saw at 695–715 ms land at 446–459 ms on the copy — the spikes did not recur, and their cause is unmeasured.** The
same script with the six added, the release form the script builds
(`cargo build --release -p acquisition-cli`, sha256 `aefa167c…`); the
seat's binary was `--workspace --release`, `9ecf385d…` — the same source
under another feature unification, which cargo uplifts by turns: a
no-op build swapped the hash. Every one of the six is within the noise
of the asks beside it (the empty query 427, the six 446–459), and the
two totals asks are over 500 as they were at step 9 (530, 528). On the
copy the six cost what their neighbours cost; what cost them 250 ms
more on the owner's machine — the owner's store rather than the copy,
a cold cache, another process, or the asks there — did not recur here
and is unmeasured (a review of 9b corrected the first wording, which
named the machine); the disposition
is V9's: the totals batch stays parked, revisited at step 11.

**Step 9c4 — a mod displayed over several rows, over the copy (`bd4679e1`, 2026-09-27): 2,345 lines of 532 templates; one of them feeds a total by a row, Thread of Hope's, and its 12 items answer as the site shows; the step costs the load 3 to 5 ms, and an ask of a total pays some 2 ms a row where it paid 5 to 6.**
`python3 search/pseudo-stats/scripts/rows-of-a-mod.py`, over the
deriver's census of the copy (`item-facts/raw/m2/rust.json`, written by
the M2 rerun at the step: 0 unread, 0 unexplained, 22,721 items, 86,450
lines, 6,148 templates), the trade site's stats
(`trade-query/data/stats-2026-09-12.json`) and the shipped table:

| | The copy | The trade site's stats |
| --- | ---: | ---: |
| mods, or texts, of several rows | 2,345 lines of 532 templates | 2,026 of 18,187 stats, 1,984 distinct texts |
| by how many rows | 2 rows 1,863 · 3 rows 446 · 4 rows 23 · 5 rows 3 · 6 rows 6 · 8 rows 4 | — |
| their rows, as templates | 901, of which 353 carry a number | — |
| rows that are also some mod's whole text, which the vocabulary lists | 45 | — |
| rows beginning lower-case, a sentence wrapped | 81 templates in 264 lines | — |
| one template displayed by two rows of one mod | 0 | 0 |
| rows a total's row names | 1: `#% to all Elemental Resistances`, 12 lines, under five totals | 0 |
| a stat of the site's with another's row between its rows | 1 mod, 12 lines: Thread of Hope's, `Passive Skills in Radius can be Allocated without being connected to your tree\nPassage` around the resistance | — |

So a row named reaches every mod of the copy that a total could read
and had not, and listing every row in the vocabulary would add 856
rows to its 6,148 (L2, `search/LEDGER.md`). The store's own bodies give Thread of
Hope's mod as the trade site's capture does, three rows in one
description: what the review of 9c3 could not verify without the copy.

The copy's 12 Threads of Hope, `target/release/acq --json search --realm
pc` over the M3 copy: `name="Thread of Hope"` 12; with
`pseudo.count_res=3 pseudo.count_ele_res=3` 12; with
`pseudo.total_all_ele_res<0` 12; with `"#% to all Elemental
Resistances"<0` 12; with `-has:pseudo.count_res` 0 and with
`undecided(pseudo.count_res)` 0; sorted by `pseudo.total_fire_res`,
-20, -17, -17, -15, -14, -13, -13, -11, -11 and -10 three times. Over
the whole realm the row's term matches 33, fails 651 and lacks 21,939;
among the jewels the vocabulary's row of the template counts 46, the
12 among them, its route returning 46, and the mod's own row 12.

The text, read on the copy: a row of the answer shows the mod whole,
its rows on one line; `show` prints each row as a name with its own
numbers; a wrapped sentence's row finds its mod (`line("to a maximum of
#%")`, 8 items). A sum and the undecided block of a reading print as a
total's do (`--count rarity --sum pseudo.count_res` over the unique
rings: 399 over 363, 140 lacking); no count of the copy is open.

What the step costs, the release binaries of four commits asked in
turn over the copy, the median of eleven, in ms (`7435874e`, sha256
`d486b814…`, the build before the step; `bd4679e1`, `cb391165…`, the
step as closed; `0962588b`, `71f97b38…`, the fix of its review's
finding; `ca9f53bc`, `cfdec97b…`); the same binary under two names
differs by 1:

| Ask | `7435874e` | `bd4679e1` | `0962588b` | `ca9f53bc` |
| --- | ---: | ---: | ---: | ---: |
| the empty query | 456 | 472 | 476 | 461 |
| OQ7, one line everywhere | 464 | 475 | 482 | 467 |
| the vocabulary whole | 495 | 511 | 520 | 502 |
| the vocabulary twice narrowed | 519 | 536 | 512 | 497 |
| AQ2 as worded | 590 | 612 | 538 | 523 |
| `pseudo.count_res>=3` | 650 | 670 | 549 | 534 |
| a mod named by a row | 473 | 494 | 495 | 479 |
| two alternatives in one group | 490 | 506 | 510 | 496 |

Two causes, each measured by taking it out or putting it back:

- **A total's row compiled again at every ask.** The table is read at
  every ask, 284 rows, each a line's group. The step's first build
  compiled each row's quoted template once more than the build before
  it, and the review's fix twice more; `ca9f53bc` takes the tests from
  the bound tree, where they were compiled. It was most of what the
  step cost the load: 16 ms as closed, 20 after the fix, 5 now — 3 in
  another turn of the same four. The debug build pays some 400 µs for
  each: its empty query is 2,547 as closed, 2,665 after the fix and
  2,449 now, where M3's debug asks began at 2,432 before the step.
- **A slot read of every line an ask weighed.** Until `0962588b` a sum
  read the slot's number of each line beside whether the line held,
  and a line that did not hold had its template's numbers counted all
  the same. The fix reads a slot of a part that may hold. With the
  read put back by hand in that build, `pseudo.count_res>=3` is 684 ms
  beside 544 without it and 662 before the fix, and AQ2 as worded 618
  beside 532 and 605. So an ask of a total pays 1.7 to 2.0 ms for each
  row it reads (`ca9f53bc`: 67 ms over 37 rows, 61 over 30, 117 over
  67), where it paid 5 to 6.

Every one of M3's 49 asks is answered the same, whole, by
`bd4679e1`, `0962588b` and `ca9f53bc`. The deriver alone, release, over
the copy's bodies (`target/release/examples/derive-census`, five runs
in turn), is 195 to 199 ms at `7435874e` and 206 to 215 at `bd4679e1`:
a difference the asks do not show, the whole step being 3 to 5 on the
load, and that the close took for the step's cost before the first
cause was found.
