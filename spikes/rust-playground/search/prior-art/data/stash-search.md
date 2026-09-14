hand-written (no script) from the reads listed in each row, all at awakened-poe-trade@ce551eb7a9b704fbdcc2478eebb26be8f91786c7

# What Awakened PoE Trade's "stash search" and "item search" actually do

Neither is a search over a corpus of the player's items. The tool holds no
stash: the only GGG endpoints it calls are `/api/trade/search` and
`/api/trade/fetch` (`renderer/src/web/price-check/trade/pathofexile-trade.ts:773`,
`:806`) plus poe.ninja for prices (`renderer/src/web/background/Prices.ts:54`);
no stash endpoint appears anywhere in `renderer/src`, `main/src` or `ipc/`.
The only item the tool ever has is the one the player copied to the clipboard.

## The two features

| | **stash search** | **item search** |
| --- | --- | --- |
| Where | `renderer/src/web/stash-search/` | `renderer/src/web/item-search/` |
| What it is | a pad of saved strings, each a button with an optional hotkey (`widget.ts:6-11`) | a name lookup over the shipped base-type dataset (`WidgetItemSearch.vue:132-156`) |
| Corpus searched | none — the *game client's* own stash search box | `items.ndjson`, restricted to two slices: alt-quality gem names and Replica unique names (`WidgetItemSearch.vue:262-276`; the slices are built by substring scan at load, `assets/data/index.ts:105-106`) |
| Query model | an opaque string; the tool never parses it (`WidgetStashSearch.vue:107-112`) | free text, lowercased, split on whitespace (`WidgetItemSearch.vue:138`) |
| How a match is decided | by the game: the string is put on the clipboard, then Ctrl+F, Ctrl/Cmd+V, Enter are synthesised into the game window (`main/src/shortcuts/text-box.ts:53-65`) | every whitespace-separated part must be a substring of the item name, **and** some word of the name must start with the longest part (`WidgetItemSearch.vue:146-149`) |
| Limits | the editor turns the input red past 250 characters, the game's field length (`stash-search-editor.vue:16`) | needs ≥ 3 characters; more than 5 hits returns `false` and the UI says "too many" (`WidgetItemSearch.vue:140-152`, `:56-57`) |
| Result | the game highlights the matching stash cells; the tool learns nothing | up to 5 rows, icon + name + a "select" button; a selected item is priced from poe.ninja and can be sent to the price-check window as a synthetic item (`WidgetItemSearch.vue:232-260`, `:289-311`) |
| Ranking | none | none — insertion order, capped at 5. OCR input only is ranked, by Levenshtein distance over the alt-quality gem names (`WidgetItemSearch.vue:158-185`) |
| Defaults shipped | four map-rolling strings and six dump-sorting strings, e.g. `"Pack Size: +3"`, `"Map Device" "Rarity: Normal"` (`WidgetStashSearch.vue:44-70`) | its stated purpose is Heist target gems and Replica uniques (`WidgetItemSearch.vue:28-36`) |

## The third path: "search similar"

A hotkey over the item under the cursor. It takes the parsed item's *name*,
JSON-quotes it, and sends it down the same `stash-search` IPC action — so it
too is a string typed into the game's Ctrl+F, not a query
(`renderer/src/web/item-check/hotkeyable-actions.ts:42-48`; the action is
declared in `ipc/types.ts:26-28` and `:174-177`, dispatched in
`main/src/shortcuts/Shortcuts.ts:60-64` and `:154-155`).

## What this means for a stash search of our own

The tool inherits the game's search grammar wholesale and therefore inherits
its ceiling: substring matching over rendered item text, one line of it, 250
characters, no ranking, no numeric comparison, no result set the tool can
read back. Everything the tool knows how to do with an item — parse the
lines, give each a stat identity, compute a Q20 property, build a filter with
a numeric range — it can only do for the *one* item on the clipboard, and it
spends that knowledge on a trade-site query, never on the player's own stash.
The prior art for identity is deep here (see `stat-model.md`); the prior art
for search is a passthrough.
