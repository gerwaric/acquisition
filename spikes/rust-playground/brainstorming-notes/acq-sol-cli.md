The GUI maps cleanly to a CLI with roughly nine functional command groups. The key design choice is to make searches stateless and make every stash/character/item addressable by a stable typed ID.

## Proposed command tree

| Area | Commands | GUI functionality covered |
|---|---|---|
| Authentication | `auth login`, `auth status`, `auth logout` | OAuth authentication, re-authentication, remembered token |
| Session | `leagues list`, `context show`, `context set --realm --league` | Realm and public/private league selection |
| Inventory locations | `locations list`, `locations show`, `locations check`, `locations uncheck` | Stash/character tree and refresh checkboxes |
| Refresh | `refresh lists`, `refresh all`, `refresh checked`, `refresh selected` | All four current `TabSelection` modes |
| Items | `items search`, `items show`, `items pob`, `items tooltip` | Search form, table/tree, text/tooltip panel, PoB copy, Imgur upload |
| Pricing | `price get`, `price set`, `price clear`, `price legacy-plan`, `price legacy-apply` | Item/tab buyouts and legacy recovery |
| Forum shop | `shop threads`, `shop template`, `shop preview`, `shop publish`, `shop cookie`, `shop auto` | Complete Shop menu |
| Currency | `currency list`, `currency ratio`, `currency export` | Currency dialog, editable ratios, CSV history |
| Operations | `watch`, `status`, `ratelimits`, `logs`, `config`, `update check` | Automation, status bar, rate-limit dialog, event log, settings, updates |

The current four refresh modes are explicit in [util.h](/Users/tom/Development/GitHub/gerwaric/acquisition/src/util/util.h:39) and implemented in [itemsmanagerworker.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/itemsmanagerworker.cpp:329).

## Authentication and active context

```text
acq auth login [--reauthorize]
acq auth status
acq auth logout

acq leagues list [--realm pc|sony|xbox]
acq context show
acq context set --realm pc --league "Standard"
```

`auth login` would initiate the browser OAuth grant when necessary. Account identity should come from OAuth rather than a user-supplied account string. `--league` must accept values absent from the public list for private leagues.

Useful global context overrides:

```text
--data-dir PATH
--realm pc|sony|xbox
--league NAME
--account NAME
--log-level off|fatal|error|warn|info|debug|trace
```

The login code currently separates the live league request from browser authorization in [logindialog.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/ui/logindialog.cpp:201).

## Locations and refreshing

Use “location” as the shared term for stash tabs and characters:

```text
acq locations list [--kind stash|character] [--checked|--unchecked]
acq locations show stash:3a8…
acq locations check stash:3a8… character:MyCharacter
acq locations uncheck --all
```

Selectors should always be type-qualified—`stash:<id>` or `character:<id>`—while allowing names as a convenience only when they resolve uniquely.

```text
acq refresh lists
acq refresh all
acq refresh checked
acq refresh selected stash:3a8… character:MyCharacter
```

Important parity semantics:

- `lists` fetches stash and character metadata without contents.
- `checked` honors persisted check state.
- Game-priced locations remain refresh-locked and are included even if unchecked.
- `selected` accepts multiple locations.
- Map and Unique stash children need flags or settings:

```text
--map-children
--unique-children
```

- Emit per-location progress and a final result containing refreshed, skipped, and failed sources.
- Reads such as `items search` should use cached data by default; network access should remain explicit through `refresh`.

## Item search

The current catalog defines 38 filters in [filterspec.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/filters/filterspec.cpp:29). I would expose them as repeatable, composable flags:

```text
acq items search \
  --tab "dump" \
  --name "amethyst" \
  --type ring \
  --rarity rare \
  --ilvl 84.. \
  --mod "+#% total to Fire Resistance:40.." \
  --sort price \
  --group by-tab
```

Recommended range syntax is `MIN..MAX`, with either side optional and inclusive.

Filter families:

- Text: `--tab`, `--name`
- Choices: `--type`, `--rarity`
- Offense: `--crit`, `--dps`, `--pdps`, `--edps`, `--cdps`, `--aps`
- Defense: `--armour`, `--evasion`, `--energy-shield`, `--block`
- Sockets: `--sockets`, `--links`, `--colors R,G,B`, `--linked-colors R,G,B`
- Requirements: `--req-level`, `--req-str`, `--req-dex`, `--req-int`
- Miscellaneous: `--quality`, `--level`, `--map-tier`, `--ilvl`
- Flags: `--alt-art`, `--priced`, `--unidentified`, `--influenced`, `--crafted`, `--enchanted`, `--corrupted`, `--fractured`, `--split`, `--synthesized`, `--mutated`
- Mods: repeatable `--mod 'NAME[:MIN..MAX]'`

All filters are ANDed, including repeated mods. White sockets can satisfy requested red/green/blue counts, matching the GUI.

Presentation options:

```text
--group by-tab|by-item
--sort name|price|last-update|quality|stack|dps|pdps|edps|...
--order asc|desc
--columns name,price,ilvl,location
--format table|json|csv|tsv
--limit N
```

Discovery commands are important because the GUI currently supplies searchable dropdowns:

```text
acq catalog types
acq catalog rarities
acq catalog mods [QUERY]
acq catalog currencies
acq catalog columns
```

## Item inspection and export

```text
acq items show ITEM_ID [--format text|json]
acq items pob ITEM_ID
acq items tooltip ITEM_ID --output tooltip.png
acq items tooltip ITEM_ID --upload
acq items icon ITEM_ID --output icon.png
```

`items pob` prints the current Path of Building representation from [item.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/item.cpp:681). `tooltip --upload` prints the Imgur URL instead of copying it to the clipboard, covering the GUI upload action in [mainwindow.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/ui/mainwindow.cpp:2082).

## Pricing

```text
acq price get --item ITEM_ID
acq price get --location stash:3a8…

acq price set --item ITEM_ID \
  --type fixed --value 5 --currency chaos

acq price set --location stash:3a8… \
  --type buyout --value 1 --currency divine

acq price set --query QUERY.json --type ignore
acq price clear --item ITEM_ID
```

Buyout types should be:

```text
buyout | fixed | offer | no-price | ignore | inherit
```

Required behavior:

- `inherit` restores an item to its location’s price.
- Location prices propagate to inheriting items.
- Explicit item prices override location prices.
- Game-sourced prices are reported as locked and rejected by mutations.
- Results expose type, value, currency, source, inherited state, and last-update time.
- Bulk operations should preview the number of affected items unless given `--yes`.

The current GUI applies one price edit to mixed item/location selections and then propagates inheritance in [mainwindow.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/ui/mainwindow.cpp:667).

Legacy recovery naturally becomes:

```text
acq price legacy-plan OLD_DATABASE --output plan.xlsx
acq price legacy-apply plan.xlsx [--yes]
```

That preserves the existing plan/apply workflow documented in [legacy-buyout-import.md](/Users/tom/Development/GitHub/gerwaric/acquisition/docs/design/legacy-buyout-import.md:13).

## Forum shop

```text
acq shop threads list
acq shop threads set 1234567 2345678
acq shop threads clear

acq shop template show
acq shop template set --file template.txt
acq shop template set --stdin

acq shop preview [--output-dir shop-pages]
acq shop publish [--force|--if-changed]

acq shop cookie set --stdin
acq shop cookie status
acq shop cookie show --reveal
acq shop cookie clear

acq shop auto on|off
```

`preview` should always render fresh markup and emit every generated page, instead of reproducing the GUI clipboard limitation of only copying the first cached page.

Parity details from [shop.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/shop.cpp:222):

- Publishing requires configured thread IDs and a POESESSID.
- A manual GUI publication is forced even when content is unchanged.
- Automatic publication skips unchanged content.
- Items are grouped by buyout and split around the forum’s size limit.
- A rejected POESESSID is cleared and automatic publication is disabled.
- Automatic publishing occurs only after a clean refresh—never after failure or skipped sources.

## Currency

```text
acq currency list [--values chaos|exalted|both]
acq currency ratio list
acq currency ratio set chaos --currency alt --amount 12
acq currency ratio set exalted --currency divine --amount 8
acq currency export history.csv
```

`currency list` should include count, ratios, calculated values, and chaos/exalted/wisdom totals. Ratios are user-maintained, not market prices. Currency is recomputed from cached items in [currencymanager.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/currencymanager.cpp:58), while export writes historical snapshots in [currencymanager.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/currencymanager.cpp:230).

## Automation and diagnostics

A one-shot CLI needs a foreground long-running command to reproduce GUI timers:

```text
acq watch --refresh checked --every 30m --publish-shop
```

It should:

- Refresh checked locations periodically.
- Publish only after clean refreshes when shop automation is enabled.
- Print progress/events to stderr.
- Stop cleanly on SIGINT.

Diagnostics:

```text
acq status
acq ratelimits
acq ratelimits --watch
acq logs show
acq logs follow
acq update check
acq update ignore VERSION
acq config list|get|set
```

`ratelimits` should expose policy, queue depth, hits/limit, period, timeout, and status—the same fields as [ratelimitdialog.cpp](/Users/tom/Development/GitHub/gerwaric/acquisition/src/ratelimit/ratelimitdialog.cpp:83).

## GUI concepts that should not become commands

- Search-tab creation, rename, and deletion: each CLI invocation is already an independent search.
- Expand/collapse: replace with `--group` and output depth.
- Tooltip/Text/Hide tabs: replace with output formats.
- Clipboard actions: print to stdout, with an optional `--copy`.
- Theme and panel visibility: presentation-only; `--color`/`--no-color` is sufficient.
- Status dialogs: structured progress on stderr and machine-readable final results are more useful.

The highest-value initial slice would be `auth`, `context`, `refresh`, `locations`, `items search/show`, and read-only `price get`. That establishes the selector and output contracts that nearly every mutating command will reuse.