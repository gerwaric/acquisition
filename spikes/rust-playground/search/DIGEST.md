# The search digest

Status: partial — item-facts — 2026-09-16

The slice's evidence in the shape a design can cite: one claim per
line, `S<n> · kind · weight · claim · pointer`, under the brief
(`brainstorming-notes/21-search-digest-brief.md`, d8502bfb). Ids are
stable and never reused; a pointer is a track README finding
(`item-facts F7`) or a data file in the track's `data/`. Reach a README only to verify a claim.

## item-facts

S1 · measure · — · 36,139 items from 1,800 tabs (Standard lists 3,347), 60 characters, 6 leagues, 2 realms; 44,083,581 JSON bytes, max 4,598 · F1, numbers.md
S2 · item · main · 13 fields on every item: `baseType, frameType, frameTypeId, h, icon, id, identified, ilvl, league, name, typeLine, verified, w`; `explicitMods` on 74.5 %, `properties` 74.4 %, `rarity` 70.2 %, `note` 0.5 % · F2, field-census.csv
S3 · ggg · main · `explicitMods`/`implicitMods` are `{description, flags}` objects from 2026-07-31, strings through 2026-07-23; no `hash` on 118,633 object lines · F3
S4 · ggg · main · `[Tag|Display]` markup in display text: pc 320 explicit lines, 18 property names, none pre-object; poe2 (98 items) 220 lines over 8 arrays · F3, numbers.md
S5 · measure · — · `explicitMods`: 26,930 items, 109,921 lines, 5,703 templates, 1,933 covering 90 %, 1,218 singletons; 4.08 lines per item, max 16 · F3, numbers.md
S6 · item · main · `properties`: 579 names over 66 type ids, 44,284 of 100,234 rows with no `type` (gem lines) · F4
S7 · ggg · main · `values[][1]` is the site's `ValueStyle` colour class (0 Default, 1 Augmented, 2 Unmet, 3–7 damage); only Augmented says anything about the value · F4
S8 · item · main · `baseType` joins a trade category for 28,313 items (78.3 %); unmatched: 7,512 maps, 272 transfigured gems (243 join by suffix), 98 poe2, 3 others · F5
S9 · item · main · PoE1 items carry no class field; poe2 items carry it as `properties[].type == 109` · F5
S10 · item · main · A map's `name`, `typeLine` and `baseType` all read `Map (Tier N)` (or `Ceremonial Map`, `Map of Miring`); none of 7,273 carries a `Map Tier` or any `Tier`-named property · F5, F7
S11 · measure · — · 20,132 of 20,228 same-id Standard items byte-identical 4–8 weeks apart; the 96 differ in `note`, `x`/`y`, `veiledMods`, one stack, never in mod text, icon URL or identity; 5,571 ids gone, 3 new · F6
S12 · limit · edge · `veiledMods` is a `Prefix#`/`Suffix#` placeholder re-rolled per response (11 of 12 veiled items; 55 carry it) · F3, F6
S13 · ggg · main · 85 of the reference's 105 top-level fields observed, none undocumented; `influences.{crusader,hunter,redeemer,warlord}` undocumented; 20 documented unobserved, 5 PoE2-only · F2, numbers.md
S14 · limit · main · A map's area exists only as icon art (178 names; 717 of 766 series-24 icons show none); no property, base type or trade base names it · F7
S15 · item · main · Ids survive a league's end: 40 Mirage items reappear under Standard with their ids · F1

## The acceptance set

Written when owner-seat and agent-seat merge.

## Limits register

Every `limit` claim, with what the search says when it meets it (an
unknown shown, never guessed, is the model).

- S12 — a veiled line is shown as the placeholder it is; a value query never matches it.
- S14 — a map's area is not a fact the search holds; a query for one gets an unknown, never the icon's art.

## Kill list

- item-facts: F7 `mn`/`mg`/`mi`/`mc` and the icon payload past the tier — unread; F2 `crucible.nodes`, `frameTypeId` composition — internals; Numbers' per-store and file-size tables — sizing.

## The question index

Written after the pilot's merge, from note 22's ranked questions.

## Convergence and contradiction

- item-facts F5 itemises the unmatched bases to 7,885 (7,512 + 272 + 98 + 3) while `data/numbers.md`'s `frameTypeId` table sums them to 7,826 (36,139 − 28,313); a 59-item gap the track does not explain. S8 carries F5's figures.
- item-facts F3 counts PoE2 markup over four arrays (192 lines); `data/numbers.md` lists eight (220). S4 carries the data file.
