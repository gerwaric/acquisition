<!-- hand-kept alongside bench/src/main.rs (the scan in `fn scan`, the SQL in `fn sql`); rows are the x1 counts from results.csv -->

# The timed queries

Twelve, each standing for a question someone actually asks: an owner-seat row
(`../owner-seat/data/questions.md`), a C++ filter (`../cpp-search/data/filters.toml`),
or a trade stat group (`../trade-query/data/grammar.json`, `stat_groups`). Every query
runs on every engine, at both identities, at all three scales, cold once and warm seven
times; the median warm time is reported. Both engines produce the matching set (the ids),
not just a count — `sqlite-count` rows repeat the SQL wrapped in `count(*)` so the index
work can be told from the row delivery.

Constants are three templates that the corpus actually carries in `explicitMods`:
`# to maximum Life` (2,791 lines), `#% to Cold Resistance` (1,444), `#% to Fire Resistance`
(1,418). Under the stat identity each is the trade stat id that
`../repoe/data/template-vs-translation.csv` gives it (`stat_3299347043`, `stat_4220027924`,
`stat_3372524247`).

| Query | Stands for | Predicate | Rows at 36,139 |
| --- | --- | --- | ---: |
| `q01-mod-value` | owner Q1/Q7, "a specific modifier value is often what is needed" | an `explicitMods` line whose identity is Cold Resistance and whose value ≥ 30 | 751 |
| `q02-mod-exists` | owner Q7, the sweep: does the item carry the mod at all | an `explicitMods` line whose identity is maximum Life | 2,775 |
| `q03-name-substring` | owner Q2/Q6; C++ filter 1 (Name) | lowercased `name + " " + typeLine` contains `forbidden` | 394 |
| `q04-base-with-mod` | owner's refinement: "I likely have a base in mind" | `baseType` = `Two-Stone Ring` and a Cold Resistance line ≥ 20 | 46 |
| `q05-rarity-ilvl` | C++ filters 3 (Rarity) and 18 (Item level) | `frameTypeId` = `Rare` and `ilvl` ≥ 84 | 2,204 |
| `q06-one-tab` | owner Q4/Q5, "where is it"; C++ filter 0 (Tab) | tab label = `Flasks` (exact, a coordinate lookup) | 848 |
| `q07-count-per-tab` | owner prompt 3, "counts and totals … might be useful" | count of items grouped by tab; rows are the tabs | 344 |
| `q08-count-group` | trade `count` group: at least 2 of 3 stats | ≥ 2 of {Life ≥ 60, Cold Res ≥ 25, Fire Res ≥ 25} | 626 |
| `q09-weight-sum` | trade `weight` group: a weighted sum over stats | 1·Life + 2·ColdRes + 2·FireRes ≥ 150 | 528 |
| `q10-boolean-or` | owner prompt 4: "(Armour > 1000) OR (Required Level < 80)" — the thing the trade site cannot express | `armour` > 1000 or `req_level` < 80 | 35,956 |
| `q11-cpp-mods-two-rows` | the C++ Mods filter as it is: two template rows ANDed | Life ≥ 50 and Fire Res ≥ 20, both `explicitMods` | 397 |
| `q12-leveling-set` | owner Q5, the leveling set by level bracket | `req_level` between 1 and 45, rarity in {Magic, Rare, Unique} | 3,880 |

Two absence rules carried over from the C++ app (`../cpp-search/README.md` F1) shape two of
the counts: a required stat the item does not carry reads as 0, so `q10`'s `req_level < 80`
is true for every item without requirements — 99.5 % of the corpus matches, which makes it
the deliberate worst case, a query whose whole cost is delivering rows. `q12` excludes them
by requiring `req_level ≥ 1`.

The SQL runs against a wide `items` table (18 columns) and a `lines(item, arr, line, value)`
table, with indexes on `lines(line, value)`, `lines(item)`, and on `items` `frame`, `tab`,
`ilvl`, `req_level`, `armour`, `base`; `ANALYZE` is run after the build. The scan runs over
one `Item` struct per item holding the same 18 fields plus a `Box<[Line]>`, with every
template, stat id, array name, tab and league interned to a `u32` at load.
