# prior-art — how a maintained item-to-trade tool gives a line its identity

Status: first pass complete — 2026-09-13

- A displayed line's identity is **its own English text**, hashed (fnv1a-32)
  into a byte offset in a 2.5 MB ndjson; the entry it lands on carries the
  trade ids. There is no mod id, no hash of the game's data, no join.
- The text is *not* stable: across 55 data commits, **2436 times** a trade id
  survived while the text it displays changed — 526 of them in one patch
  (`data/cadence.csv`). The trade id is the spine; the line is the moving part.
- One entry never carries two ids of the same kind (**0** of 9278). Two trade
  ids behind one line are held as a two-stat `StatGroup` with a named
  resolver, and the second id is merged in *at match time* — 57 of the 380
  known collisions are handled this way, 83 stay separate entries, 240 are
  carried by at most one id (`data/coverage.csv`).
- Its **stash search is a passthrough**: the tool types a saved string into
  the game's own Ctrl+F. It holds no stash, calls no stash endpoint, and
  spends everything it knows about an item on a trade query (`data/stash-search.md`).
- What it needs is only what a clipboard item shows: it carries 12507 of the
  capture's 17958 trade ids and drops four whole categories (crucible,
  scourge, delve, ultimatum) untouched.

## Question

Awakened PoE Trade turns an item's clipboard text into a trade query and has
lived for years with what the repoe track measured: local/global twins,
several trade ids for one line, renames across patches. How does it represent
a stat, where does the dataset come from, and what happens when the game
moves? Read at `ce551eb7a9b704fbdcc2478eebb26be8f91786c7` (`MANIFEST.md`);
paths below are relative to that clone. Nothing here is from memory of the tool.

## Findings

**F1 — Identity is the displayed line, keyed by a hash of its own text.**
`STAT_BY_MATCH_STR_V2(matchStr)` takes fnv1a-32 of the *matcher string*,
binary-searches a `Uint32Array` of (hash, byte offset) pairs, and parses the
one ndjson line at that offset
(`renderer/src/assets/data/index.ts:123-137`, index built by
`renderer/src/assets/make-index-files.mjs:29-38`). Hash collisions are caught
by re-checking the string against the parsed entry's matchers
(`index.ts:130-135`). Lookup is therefore O(log n) with no in-memory map: the
2.5 MB file is held as one string and sliced. `ref` is **English in every
language file** (`ru/stats.ndjson` line 1 has the English `ref` and Russian
matchers); the localized text lives only in `matchers[].string`. So the tool
already separates identity (a stable English key) from display — but the key
is itself a sentence, and F5 shows what that costs.

**F2 — Numbers are removed by trying every placement.** A stat line is turned
into candidate templates by `_statPlaceholderGenerator`: every numeric run is
captured, `(min-max)` advanced-description bounds are peeled off, and up to 11
combinations of "which numbers stay literal, which become `#`" are yielded in
order, most-placeholders-first, with the raw text last
(`renderer/src/parser/stat-translations.ts:67-133`). Each candidate is looked
up until one hits and the modifier type is present in its `trade.ids`
(`:140-147`). That is how `1 Added Passive Skill is a Jewel Socket`
(`matchers[1].value = 1`) and `# Added Passive Skills are Jewel Sockets`
reach the same entry (`data/stat-model.md`, line 1). Reversed wordings carry
`negate: true` (1653 matchers) and flip the roll's sign (`:154-162`); the
game's "advanced mod description" wording is a second matcher key,
`advanced` (1420 matchers), indexed instead of `string` when present
(`make-index-files.mjs:32-36`).

**F3 — Ambiguity is a two-stat group with a named resolver.** 100 of 9178
lines are `StatGroup`s; every one holds exactly two stats. `resolve.strat` is
one of four (`interfaces.ts:42-54`, `stat-translations.ts:262-316`):

| Strat | n | What it does |
| --- | --- | --- |
| `trivial-merge` | 43 | keeps the first, pushes the other's id into its list |
| `select` | 41 | picks by the item's category (`WEAPON`/`ARMOUR`/a name) — the local/global twin |
| `percent-merge` | 11 | a 100 % roll means the flag stat; merges with `{div_by_100}` or `{empty_if_100}` |
| `flag-merge` | 5 | a specific roll value means the flag stat; merges with `{empty}` |

The local/global twin is exactly the "same line, two ids" case:
`#% increased Armour` appears twice with *identical* refs and *identical*
matchers, told apart only by `resolve.test: ["ARMOUR", null]`
(`data/stat-model.md`, line 489). Merging writes into the id list at match
time (`_mergeTradeIdsInto`, `:318-330`), and a filter with several ids
becomes a trade `count` group with `value.min = 1` — an OR
(`renderer/src/web/price-check/trade/pathofexile-trade.ts:895-909`); the
`{…}` prefixes are stripped and interpreted there (`:911-934`).

**F4 — Unknown lines are kept, never guessed.** `linesToStatStrings` walks a
modifier's lines and, for each start, tries growing multi-line joins until the
caller says "parsed"; anything left over is pushed to `item.unknownModifiers`
with its modifier type (`Parser.ts:1221-1236`, `stat-translations.ts:30-65`)
and rendered as an orange "Not recognized modifier"
(`filters/UnknownModifier.vue:1-14`). Reminder text in parentheses is skipped
(`:37-47`). There is no fuzzy match on mod text anywhere; the only Levenshtein
in the app is on OCR'd *gem names* (`item-search/WidgetItemSearch.vue:158-185`).
PoE1 text has no `[Tag|Display]` markup — 0 lines of `stats.ndjson` and 0 of
`items.ndjson` match it — but the game's conditional markup is handled, and
only on the name plate: `<<set:…>>` is dropped and the first `<if:…>{…}`
branch always taken (`Parser.ts:1187-1200`, called at `:363` and `:369`).

**F5 — Coverage: it carries what a clipboard shows, and nothing else.** Its
12507 trade ids are all known to the 2026-09-12 capture (**0** unknown), and
cover 12507 of the capture's 17958. The 5451 it lacks are four categories it
ignores outright (crucible 2492, scourge 409, delve 81, ultimatum 63), most of
sanctum (187 of 240), 944 enchants and 1026 explicits — sampled, those are
map-device and atlas stats (`Area contains The Feared`, `Monster Level: #`)
that never appear on a stash item's text. All 88 option-bearing capture ids
are carried. Of the 380 collisions the trade-query track found: 57 resolved by
a group (30 trivial-merge, 24 select, 3 percent-merge), 83 kept as separate
ndjson entries (Q2), 143 with only one of the ids
carried, 97 with neither — and **0** as several ids in one entry.

**F6 — Update handling: a generator it does not ship, and two failure modes.**
The dataset generator is **not in this repository** — no script writes
`stats.ndjson`; the only build step over it is `make-index-files.mjs`, which
regenerates the four `*.index.bin` files, run at build start and on every
`.ndjson` change (`renderer/src/assets/vite-plugin-make-indexes.ts:10-22`). `DEVELOPING.md` and
CI (`.github/workflows/main.yml`) describe only lint/build/package. Data
arrives as opaque "update data" commits: 55 of them at this path over 4.5
years, median 12 days apart (max 205), the file growing 6139 → 9198 refs
(`data/cadence.csv`). The moves are not small: +1402 refs in one commit
(`d876a14`, 2026-07-27), −930 in another (`9a012b5`, "update stats for
3.26.0.15", which also dropped 1434 trade ids and re-texted 526). 31 of 54
diffs removed at least one ref. Against that, 210 distinct refs are hard-coded
as string literals in 10 source files via a `stat()` marker
(`renderer/src/parser/calc-q20.ts:5-50`, eight `filters/pseudo/*` files, and
`trade/pathofexile-trade.ts`);
every one is asserted at startup and a miss throws `Cannot find stat: X`
(`assets/data/index.ts:187-190`, `:199-215`). So a rename fails **loudly** for the 210
refs the code names, and **silently** — one orange line — for the other ~9000.

**F7 — Its "stash search" does no matching.** A saved string is put on the
clipboard and Ctrl+F / Ctrl+V / Enter are synthesised into the game window
(`main/src/shortcuts/text-box.ts:53-65`); the editor warns past 250 characters,
the game's field limit (`stash-search/stash-search-editor.vue:16`). "Search
similar" JSON-quotes the parsed item's *name* and sends it down the same path
(`item-check/hotkeyable-actions.ts:42-48`). The separate "item search" widget
is a substring lookup over two slices of `items.ndjson` (alt-quality gems,
Replica uniques), capped at 5 results, for Heist targets
(`item-search/WidgetItemSearch.vue:132-156`, `:262-276`). Full table and the
grep that rules out a stash endpoint: `data/stash-search.md`.

**F8 — What it does not attempt.** No corpus, no persistence of items, no
ranking, no saved queries beyond a string. No mod-level identity: a modifier's
tier comes from the item's own advanced description line
(`advanced-mod-desc.ts:88-90`, read at `filters/FilterModifierTiers.vue:42-57`),
not from a mod database; `modFamily` (187 entries) is used only for timeless
jewels and mercenary gems. No derivation of an unknown line. No PoE2.

## Numbers

| Measure | Value |
| --- | --- |
| ndjson lines / `StatGroup` lines / stat entries | 9178 / 100 / 9278 |
| matchers (1 / 2 / 3 / 4 / 8 per entry) | 11539 (7050 / 2205 / 17 / 5 / 1) |
| matchers with `value` / `advanced` / `negate` | 505 / 1420 / 1653 |
| entries with several ids in one category | 0 |
| entries with ids in several categories | 2294 |
| `better` +1 / 0 / −1 | 7779 / 1352 / 147 |
| entries with `trade.option` / `trade.inverted` | 85 / 96 |
| resolvers: trivial-merge / select / percent-merge / flag-merge | 43 / 41 / 11 / 5 |
| other keys: anointments / mercenary / fromAreaMods / modFamily / dp / jewelleryQuality | 562 / 532 / 234 / 187 / 151 / 12 |
| distinct trade ids / unknown to the capture | 12507 / 0 |
| capture ids carried / lacked (of 17958) | 12507 / 5451 |
| collisions (380): group / separate / partial / none / several-ids | 57 / 83 / 143 / 97 / 0 |
| data commits at this path; span; median gap | 55; 2022-03-24…2026-09-09; 12 d |
| refs first → last; commits removing a ref | 6139 → 9198; 31 of 54 |
| surviving ids re-texted, total / worst commit | 2436 / 526 |
| refs hard-coded in code and asserted at init | 210 |

## Open questions

| Q | What closes it |
| --- | --- |
| Q1. What generates `stats.ndjson`, and from what — the trade endpoint, the game files, or both? Nothing in this repository writes it. | the owner clones the sibling project (`MANIFEST.md`, row 2); until then the generator's inputs are unknown and F6 rests only on the commits. |
| Q2. Are the 83 "separate entries" collisions genuinely unambiguous (different displayed text), or silently unreachable? | join `data/coverage.csv`'s `apt_ref` per collision id against `../repoe/data/trade-stat-map.csv`'s `english` column and count the pairs whose matcher strings are equal. |
| Q3. Do the 2436 re-texts include cases where a *code-named* ref changed, i.e. did the app ever ship broken? | re-run `scripts/cadence.py` diffing each revision's refs against the 210 `stat()` literals at that same commit. |
| Q4. This history is 55 commits at the current path; `--follow` finds 69 through two renames (`289047f`, `21f0017`). Does the pre-2022 stretch change the cadence? | re-run `scripts/cadence.py` with `--follow` and `-M`; the brief pinned the 55. |

## Provenance

| Output | Script | Inputs |
| --- | --- | --- |
| `data/stat-model.md`, `data/coverage.csv` | `scripts/stat-model.py` | clone `renderer/public/data/en/stats.ndjson`; `../trade-query/data/stats-2026-09-12.json`, `.../stat-collisions.csv`; `../repoe/data/trade-stat-map.csv` |
| `data/cadence.csv` | `scripts/cadence.py` | `git log`/`git show` over the clone (read-only) |
| `data/stash-search.md` | hand-written, no script | the widget, IPC and main-process files it cites |

Both scripts refuse to run unless the clone's HEAD is
`ce551eb7a9b704fbdcc2478eebb26be8f91786c7`. `MANIFEST.md` holds the access row.

## Review

| # | Finding | Fix |
| --- | --- | --- |
