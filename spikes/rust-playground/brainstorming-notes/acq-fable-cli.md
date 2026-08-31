# Acquisition CLI — command surface

## What the GUI actually does (the verbs behind the chrome)

Stripping away widgets, the app has seven real capabilities: **authenticate** (OAuth for the API, POESESSID for the forums), **refresh** items from the API (four selection modes: all / checked / selected / tabs-list-only), **search** items with ~38 filters, **price** items and tabs (buyouts with inheritance), **publish** a forum shop (render priced items into BBCode, post to threads), **track currency** wealth over time, and **import legacy buyouts** via an auditable xlsx plan. Everything else is either presentation (theme, tooltips, expand/collapse) or plumbing surfaced to the user (rate-limit status, event log, update checker).

## Proposed command map

**Global flags** (replace login-dialog and settings-menu state): `--data-dir <path>` (profiles), `--league <id>`, `--realm pc|sony|xbox`, `--account <name>`, `--format table|json|csv`, `--log-level <lvl>`, `--quiet`.

### `auth` — identity
| Command                      | Replaces in GUI                                                         |
| ---------------------------- | ----------------------------------------------------------------------- |
| `auth login`                 | OAuth "Authenticate" button — open browser, local callback, store token |
| `auth status`                | "You are authenticated as X" label; show token expiry, account, scopes  |
| `auth refresh`               | silent startup token refresh                                            |
| `auth logout`                | *(gap — GUI has no logout, only "Remember me" uncheck)*                 |
| `auth sessid set/show/clear` | POESESSID dialog (needed only for forum posting)                        |
| `leagues`                    | login-dialog league combo (accepts arbitrary private-league ids)        |

### `tabs` — locations and what gets refreshed
The GUI's per-tab refresh checkboxes are really a persisted "tracked set" (`refresh_checked_state`), with priced tabs force-locked into it. That deserves first-class commands:

| Command                              | Replaces                                                                   |
| ------------------------------------ | -------------------------------------------------------------------------- |
| `tabs list [--stale] [--characters]` | tree bucket rows; "Fetch tabs list" (with `--remote` to re-fetch metadata) |
| `tabs track / untrack <selector…>`   | tick/untick checkboxes, Check/Uncheck All/Selected                         |
| `tabs tracked`                       | visual check state (should show which are buyout-locked and why)           |

### `refresh` — the core fetch
| Command                                                  | Replaces                                                                   |
| -------------------------------------------------------- | -------------------------------------------------------------------------- |
| `refresh`                                                | "Refresh checked tabs" (tracked set — the sensible default)                |
| `refresh --all`                                          | "Refresh all tabs"                                                         |
| `refresh <selector…>`                                    | tree context menu "Refresh Selected" (tabs by name/id, characters by name) |
| `refresh --lists-only`                                   | "Fetch tabs list" (TabsOnly mode)                                          |
| `refresh --include-maps --include-uniques`               | the two "Get map/unique stashes" toggles (or config keys)                  |
| `refresh --watch [--interval 30m]` or a `daemon` command | auto-refresh timer + automatic shop update (see shop)                      |

Semantics worth preserving: refuse/queue concurrent refreshes; report per-tab progress and a final outcome (completed / completed-with-skips / failed) — the shop auto-publish gate depends on "clean" completion.

### `items` — search and inspect
The GUI's search tabs are per-session filter sets over the local cache; a CLI does this better with flags plus *persisted* named searches (a straight upgrade — the GUI loses them on exit).

| Command                              | Replaces                                                              |
| ------------------------------------ | --------------------------------------------------------------------- |
| `items search [filters…]`            | the whole left filter panel                                           |
| `items show <item-id>`               | detail pane (Text-tab form: properties, requirements, mods, location) |
| `items pob <item-id>`                | "Copy for Path of Building" → stdout                                  |
| `items icon <item-id> -o file.png`   | composited item image (imgur upload could be `--upload`, or dropped)  |
| `search save/list/run/delete <name>` | search tabs (+, rename, middle-click delete)                          |

Filter flags, mirroring the catalog: `--tab`, `--name`, `--category`, `--rarity` (incl. non-unique), min/max pairs as `--sockets 5..6`-style ranges for damage/APS/crit/DPS/pDPS/eDPS/cDPS, armour/evasion/ES/block, sockets/links, req level/str/dex/int, quality/level/map-tier/ilvl; `--socket-colors 2R1G`, `--linked-colors`; booleans `--corrupted --unidentified --influenced --crafted --enchanted --fractured --split --synthesized --mutated --alt-art --priced`; repeatable `--mod "name[:min[:max]]"` (ANDed). Plus output controls replacing view/sort UI: `--group-by tab|none` (By Tab / By Item), `--sort <column>`, `--columns`.

### `price` — buyouts
| Command                                                                            | Replaces                                                                                       |
| ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `price set <selector> --type bo\|fixed\|co\|no-price [--value N --currency chaos]` | buyout combo/currency/value row (works on items *and* tabs)                                    |
| `price clear <selector>`                                                           | setting type back to `[Inherit]`                                                               |
| `price show <selector>`                                                            | Price column / buyout widgets (must distinguish manual / inherited / game-set)                 |
| `price import-legacy <datafile> [--plan-only -o plan.xlsx]`                        | "Recover legacy buyouts…" wizard (plan → review → apply maps perfectly to a two-step CLI flow) |
| `price apply-plan <plan.xlsx>`                                                     | "Import buyout plan…"                                                                          |

Rules the CLI must keep: game-set buyouts (from item notes / tab names) are read-only and never posted; items inherit tab buyouts; any savable buyout locks its tab into the tracked/refresh set.

### `shop` — forum publishing
| Command                                       | Replaces                                                                                                   |
| --------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `shop threads set 123,456 / show / clear`     | "Forum shop thread…" dialog                                                                                |
| `shop template set [-f file \| stdin] / show` | "Edit Shop Template" (`[items]` placeholder)                                                               |
| `shop render`                                 | "Copy shop data to clipboard" → stdout (handles the multi-page 50k-char split honestly instead of warning) |
| `shop publish [--force]`                      | "Update forum shop(s)" (hash-suppression means unchanged shops are no-ops unless forced)                   |
| `shop status`                                 | staleness, last hash, thread count vs pages needed                                                         |
| `shop auto on/off`                            | "Automatically update shop" (only meaningful with the daemon/watch mode)                                   |

### `currency` — wealth tracking
| Command                                         | Replaces                                                  |
| ----------------------------------------------- | --------------------------------------------------------- |
| `currency list [--ratios]`                      | currency dialog grid + totals (chaos/exalt/wisdom)        |
| `currency ratio set <name> --chaos N --exalt N` | editable ratio spinboxes                                  |
| `currency history [--csv -o file]`              | "Export to CSV…" plus on-screen history the GUI never had |

### `status` / `config` / misc
| Command                               | Replaces                                                                                                               |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `status`                              | status bar (Ready/Busy/Waiting + message), last refresh outcome, RePoE data readiness                                  |
| `ratelimit`                           | rate-limit dialog (policy/rule/hits/limit/period/status table) and the countdown button                                |
| `config get/set/list`                 | settings menus and advanced login options: league, realm, proxy, crash reporting, refresh interval, map/unique toggles |
| `update check`                        | update-checker dialog (skip-version becomes irrelevant; just print)                                                    |
| `data export [--json\|--csv\|--xlsx]` | *(gap — GUI has no items export at all; near-free for a CLI and probably its biggest immediate value)*                 |

## What drops out, what changes shape

**Drops entirely:** theme, tooltip rendering/fonts, minimap, expand/collapse, column resizing, quit confirmation, event-log panel (stderr + log file already cover it), the imgur upload (arguably).

**Changes shape rather than disappearing:**
- *Clipboard verbs → stdout.* "Copy shop data", "Copy for PoB" become pipeable output.
- *Checkboxes → a tracked set.* The refresh-checked state stops being incidental UI and becomes an explicit, inspectable resource.
- *Timers → a daemon or `--watch`.* Auto-refresh and auto-shop-update are the one piece of GUI behavior that's inherently long-running; a one-shot CLI needs either a `daemon`/`watch` subcommand or an external scheduler story (`refresh && shop publish` in cron covers most of it).
- *Modal dialogs → flags + exit codes.* The legacy-import wizard's plan/confirm/apply steps map to `--plan-only` / `apply-plan`; destructive confirmations become `--yes`.
- *Session-only searches → persisted named searches* (an improvement the GUI can't claim).

**Two-credential wrinkle to keep visible:** OAuth covers everything except forum posting, which needs POESESSID; the CLI should make `shop publish` fail with the same clear "forums don't support OAuth" message rather than burying it.