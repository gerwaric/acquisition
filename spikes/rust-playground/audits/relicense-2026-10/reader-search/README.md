# reader-search: C++ → Rust contamination reading, search crate

Pair 1 (item.cpp, item.h, itemtooltiptext.cpp → derive/defences/describe/show, dps in pseudo.rs): CLEAN — FACT and IDEA only, no EXPRESSION.
Pair 2 (modlist.cpp, modlist.h, itemconstants.cpp → totals.rs, pseudo.rs): CLEAN — 0 of 419 totals rows come from modlist.cpp; every row traces to the owner's pseudomods.cpp (104) or a trade-site capture (315).
Pair 3 (search/bucket/column/itemcategories → category/class/counts/sockets/group/eval): CLEAN — vocabulary is GGG JSON keys plus trade-site ids, not the C++ filter or column labels.
No EXPRESSION rows. High-confidence IDEA rows: none (all IDEA rows are medium/low). Paths below: `S` = `spikes/rust-playground/crates/acquisition-search`.
One upstream note outside the scope: `src/pseudomods.cpp`'s 3-line header comment came from Ilya's 2014 commit 0406254c; the Rust side does not carry it.

Inputs: `findings.tsv` (23 rows), `tables.tsv` (5 rows). The counting scripts are in `work/` (`python3 -I work/cmp.py`, `work/cmp2.py`).

## Pair 1 — derived item properties

**Requirements row (FACT, high).** C++ `src/ui/itemtooltiptext.cpp:84-93`:
`text += first ? "Requires " : ", "; text += requirement.name + " " + value`.
Rust `S/src/derive.rs:543-547`: each requirement is rendered by `property()`.
GGG's `displayMode` decides the order (`S/src/derive.rs:1176`, `"{name} {joined}"`
for mode 0/2, `"{joined} {name}"` for mode 1). The parts are then joined with
`format!("Requires {}", each.join(", "))`. The row is what the game and the
trade site display. The owner ruled it from the site (`S/src/derive.rs:169-172`).
The comment at `S/src/derive.rs:172-174` that cites the C++ file records a
*difference*: C++ always puts the name first, and Rust follows displayMode
(`159 Str`). The comment is a comparison, not a sign that the code was copied.

**Property rendering (FACT, high).** For displayMode 3, C++
(`itemtooltiptext.cpp:40-49`) replaces `{%1}` with each value. Rust
(`S/src/derive.rs:1180-1192`) does the same with `{{{i}}}`. Rust also errors
when a non-empty value has no slot. GGG's API defines this format, and there
is no other way to render it. The `Name: v1, v2` form is the game's display.
C++ has two branches. Rust has six (modes 0–4, an empty name or empty values,
and requirements without a colon).

**Vaal gem / hybrid (IDEA, medium).** C++ (`src/item.cpp:108-121`) uses
`hybrid.isVaalGem` to choose between `typeLine` and `hybrid.baseTypeName`.
Rust (`S/src/derive.rs:563-575`) reads neither key. It reads
`hybrid.explicitMods` as the line source `hybrid` and `hybrid.properties` as
displayed strings, under an owner ruling. The two sides use the same GGG
field for different purposes. `isVaalGem`/`baseTypeName` appear in Rust only
inside a test fixture (`S/tests/derive.rs:470`).

**DPS (FACT, medium).** C++ (`src/item.cpp:571-626`): `DPS() = pDPS()+eDPS()+cDPS()`.
Each term returns 0 when a property is missing, and each is `aps * AverageDamage`.
Rust (`S/src/pseudo.rs:818-960`, doc 50-66) has one `inputs()` reader. It
returns "lacked" (never 0) when `Attacks per Second` or every damage property
is missing, uses exact decimal arithmetic, and gives reasons for anything unread.
It has no edps or cdps. The formula is the trade site's (filter ids
`dps`/`pdps`/`edps` in `spikes/rust-playground/search/trade-query/data/filters-2026-09-12.json`).
The decomposition is not C++'s.

**Unidentified / tooltip (NONE).** Rust has no tooltip. "unidentified" appears
only in prose (`S/src/derive.rs:19`). C++ category replace map (`src/item.cpp:22-39`),
mod-flag bucketing (`:296-485`), hash and PoB export (`:629-784`): no Rust
counterpart. `defences.rs` comes from RePoE through `tools/defence-table.py`,
and item.cpp has no quality-adjusted defence code to compare it with.

## Pair 2 — pseudo-mod totals

**The table (NONE).** `src/modlist.cpp:36-236` is a commented-out table,
`simple_sum`. git blame on `56a498e5^` shows rows by Ilya Zhuravlev (2014–15),
Daneel Trevize (2018), indiavenster (2019) and others. Its counts are in `tables.tsv`.
Rust `S/reference/totals-v6.toml` has 82 totals and 419 rows. `work/cmp2.py`
checks every row: 104 match `src/pseudomods.cpp` (owner, 2025;
`git log` 260cc74e, 56a498e5), 289 are `add row` and 26 are `twin` in
`spikes/rust-playground/search/pseudo-stats/data/table-changes.csv`, each
citing a capture. That leaves **0 rows unexplained**. 68 of simple_sum's 123
distinct templates also appear as Rust row templates. These are game mod
texts. Three divergences show the Rust table was built independently:
- Rust `increased_burning_damage` counts Burning, Fire and Elemental, but not
  simple_sum's `Damage over Time` or `increased Damage`.
- Rust counts unsuffixed `Adds # to # Fire Damage` under the spells totals as a
  twin (table-changes.csv:179). simple_sum does not.
- The resistance totals run cold, fire, lightning (pseudomods.cpp's order). simple_sum runs fire, cold, lightning.

The provenance line in `S/src/totals.rs:84-86` and in the toml header names
pseudomods.cpp and the site captures. It does not name modlist.cpp.

**"Adds" families (FACT, high).** The mod texts are the game's. Which lines
each total sums is the trade site's pseudo definition, with each Rust row
citing a capture round. This is not simple_sum's grouping.

**Templating and summing (IDEA, medium).** C++ (`src/modlist.cpp:359-360`,
Ilya 2014) replaces `[0-9.]+` with `#`, and the value is the mean of the
numbers. The summing block (`:374-392`) is owner-authored (blame 2025-12-21).
Rust (`S/src/template.rs:231`; rules at `S/src/totals.rs:12-37`) keeps the
sign inside the number, treats a range dash and `1,500` specially, and has
slots, weights, halves and ranged low/high sums. The idea is the same one the
trade site uses, and the expression is different.

**itemconstants.cpp (NONE).** `POS_MAP` (slot grid positions) has no Rust counterpart.

## Pair 3 — filters, categories, sockets, buckets

**Filter vocabulary (NONE, high).** C++ has 26 columns (`src/search.cpp:59-84`;
labels Q, PD, APS, Crit, Ar, Ev, ES, B, Lvl) and 38 filters. The filters live in
`src/filters/filterspec.cpp`, which is owner-authored in 2026 and outside the
scope list. The Rust `FIELDS` (`S/src/bind.rs:278-450`, 28 fields) are GGG
JSON keys (`typeline`, `base`, `frame`, `ilvl`, `stack`, `note`, `rarity`)
plus trade-site ids (`category`, `sockets`, `links`, `pdps`). No C++ label
appears. "38 vs 86" (commit 7677adcf) was a coverage census across three
namings. It was not a source.

**Class (IDEA, medium).** C++ `src/itemcategories.cpp:94-133` maps base → RePoE class.
It skips `unreleased` and `[DO NOT USE]`/`[UNUSED]`/`[DNT` names and lower-cases
the result. Rust `tools/class-table.py:18-32` uses the same public data with
different rules: it keeps unreleased bases on purpose (Blade Trap), keeps
display names, and adds `gems.json` for transfigured gems. C++ handles those
gems by cutting the name at `" of "` (`src/item.cpp:552-567`).
**Category**: 82 trade-site ids (`S/src/category.rs:1-40`), with nothing
in common with the C++ replace map.

**Sockets (IDEA, medium).** C++ `src/item.cpp:501-547` reads `attr` first and
falls back to `sColour`, maps S/D/I/G, and counts per group with a running
counter for links. Rust (`S/src/sockets.rs:12-27`, `S/src/derive.rs:126-148`)
reads only `sColour`. Commit ec024ceb states this difference. Rust counts are
intervals widened by unread sockets. The `linked( … )` rule quotes the
owner's description of the C++ Link Groups filter (`spikes/rust-playground/search/DIGEST.md:76`, S59).
That is an idea taken over, with no code. The layout `R-R-G B W`
(`S/src/sockets.rs:61-67`) is the game's display and PoB's convention.

**Buckets (NONE).** `src/bucket.cpp` is mostly the owner's M3 sort and residency
machinery. Rust "buckets" are the none/undecided rows of a count view
(`S/src/counts.rs`, `S/src/answer.rs:78-81`). The by-tab GUI model
(`src/search.cpp:425-510`) has no Rust counterpart. AND-with-early-exit is generic.

## Left out
- `src/filters/*` (owner, 2026) was not read beyond its headers.
- `S/src/eval.rs` and `S/src/group.rs` got only their docs and a grep, not a full read.
- `describe.rs` was read in part.
- Every C++ file in scope was read in full except `search.cpp` (lines 1-120 and 370-510 read; the rest is delta/model code), `bucket.cpp` (structure and comments) and `column.cpp` (grep).
