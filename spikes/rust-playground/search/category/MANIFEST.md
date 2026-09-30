# Evidence manifest — category

Every raw or clone input this track read (index rule 4). Nothing was fetched and nothing new was captured: each input is described by the manifest named in its row, and this file only points there. The clones sit beside `acquisition/` (`../../../../../<clone>` from `scripts/`); each script refuses a clone at another commit.

| Input | Bin | Described by | Read by | What was read |
| --- | --- | --- | --- | --- |
| `../trade-query/data/filters-2026-09-12.json` | data | `../trade-query/MANIFEST.md` | `common.site_filters` | `type_filters.category`: the ids and their texts |
| `../trade-query/data/items-2026-09-12.json` | data | `../trade-query/MANIFEST.md` | `common.site_coarse` | the 22 coarse categories and their base types, `monster` among them |
| `../trade-query/data/grammar.json` | data | `../trade-query/MANIFEST.md` | by hand | `item_categories`: the same 22 ids, labels and entry counts as `items-2026-09-12.json` |
| `../trade-query/raw/searches/q*-request.json`, `q*-fetch.json` | raw | `../trade-query/MANIFEST.md`, "The owner's searches" | `common.site_fetches` | the category option of each request; of each fetched item only `baseType` and the clipboard's `Item Class:` line — nothing about a seller |
| `../trade-query/raw/Trade - Path of Exile.html` and `…_files/*.js` | raw | `../trade-query/MANIFEST.md` | `categories.py` | grepped for the seven grouping ids (finding 2) |
| `poe1/data/base_items.json`, `item_classes.json`, `gems.json`, `mods.json` @ `e2bd511a` | clone | `../repoe/MANIFEST.md`, "The clones" and "The PoE1 export's files read" | `common.Export` | bases and their classes; the classes' display names; transfigured gems' names; the bestiary mods' names |
| `poe1/data/*.json` (the 30 top-level files) @ `e2bd511a` | clone | `../repoe/MANIFEST.md` | `beasts.py` | each searched for the copy's beast base names as strings (finding 4) |
| `../repoe/scripts/base-taxonomy.py` | committed | `../repoe/README.md`, F3 | `common.taxonomy` | imported, not copied: `CLASS_TO_TRADE`, `NAME_RULES`, `PARENTS`, `predict` |
| `crates/acquisition-search/reference/classes-v1.toml` | committed | its header; `tools/class-table.py` | `common.load_table` | which bases the shipped table carries |
| `crates/acquisition-search/src/class.rs` (lines 113–121, 370–460), `derive.rs` (line 1150) | committed | the crate | by hand, mirrored in `common.py` | the blight prefixes, the frame's pick, `shown` |
| `awakened-poe-trade/renderer/public/data/en/items.ndjson` @ `ce551eb7` | clone | `../prior-art/MANIFEST.md` | `common.apt` | `craftable.category` per base; the `CAPTURED_BEAST` namespace |
| `awakened-poe-trade/renderer/src/web/price-check/trade/pathofexile-trade.ts` (lines 13–56, 323–329) | clone | `../prior-art/MANIFEST.md` | `common.apt` | `CATEGORY_TO_TRADE_ID` |
| `awakened-poe-trade/renderer/src/parser/meta.ts`, `Parser.ts` (lines 184–185, 974–975), `web/price-check/filters/create-item-filters.ts` (lines 59–65), `public/data/en/client_strings.js` (line 57) | clone | `../prior-art/MANIFEST.md` | `common.apt`; by hand | the `ItemCategory` enum; how a captured beast is recognised and searched |
| `PathOfBuilding/src/Data/Bases/*.lua` @ `16de4b82` | clone | `../repoe/MANIFEST.md`, "The clones" | `common.pob` | every base's `type` and `subType` (22 files) |
| `PathOfBuilding/src/Classes/TradeHelpers.lua` (`M.getTradeCategory`, line 351) | clone | `../repoe/MANIFEST.md`, "The clones" | `common.pob` | its branches, parsed from the file |
| `../item-facts/raw/m3/store/mock/GERWARIC_7694.db` | raw | `../item-facts/MANIFEST.md` (the `spike-GERWARIC_7694-2026-09-13.db` row: the copy's sha256 `fbae8786…` is that row's); `../MEASUREMENTS.md`, M3 | `common.copy_items` (sqlite `mode=ro`) | every live `pc` item's `json` |
| `runs/seat-2026-09-26/REPORT.md` | local, gitignored | `../LEDGER.md`, the row "the first seat" | by hand | section 4, V1: the owner's words on beasts |
