# Legacy Buyout Import — Investigation Results

Status: evidence record, August 13, 2026. Facts only; the design lives in
`legacy-buyout-import.md`.

## The problem

Acquisition versions before 0.16 keyed buyouts on a bespoke MD5 item hash
because the 2014 API had no item ids. The 3.29 API changes broke that hash
function (and any future API change can break it again), and the current
codebase keys buyouts on the official GGG item ids instead — but has no way
to read the old hash-keyed buyouts. At least one user has reported losing
buyouts to this. GitHub context: discussion #88.

## Validation result (the headline)

The recovery approach — recompute the legacy hash from the *old stored
item JSON* (not from live API data), then use the GGG `id` stored in that
same JSON to rekey the buyout — was validated end-to-end on a real
pre-0.16 database (GERWARIC, Standard league, v0.15.0 backup, db_version 4,
19,570 items):

| metric | result |
|---|---|
| item buyouts matched | **1137 / 1137** |
| tab buyouts matched | **12 / 12** |
| orphaned buyout hashes | 0 |
| ambiguous hashes (>1 item shares the hash) | 83 |
| matched item ids still present in the *current* stash data, 14 months later | 1170 / 1171 |

The ambiguous hashes are identical items (same mods/properties, e.g.
identical stacks) and duplicate/unnamed tab labels (the F51 story). Old
acquisition could not distinguish these either — the one buyout displayed on
every matching item — so applying an imported buyout to all matching ids
reproduces the old behavior exactly.

The validation script is `legacy-buyout-hash-check.py` in this directory:
a Python replica of `LegacyItem::hash()` run directly against an old
sqlite file. Run it as `python3 legacy-buyout-hash-check.py <old-db-file>`.

## The old data format (≤0.15.x)

One sqlite file per account+league at
`data/<md5(account "|" league)>[-<discriminator>]` (see
`SqliteDataStore::MakeFilename`, unchanged since then). Contents:

- `data` table (`key TEXT PRIMARY KEY, value BLOB`):
  - `buyouts` — JSON map, legacy MD5 hash → buyout
    (`{value, last_update, type, currency, source, inherited}`)
  - `tab_buyouts` — JSON map, `"stash:<label>"` or `"character:<name>"`
    → buyout
  - `db_version` — hash generation: `4` = no `<<set:…>>` prefix (any
    install that ran a 0.9.x+ client is at 4); `<4` = the
    `<<set:MS>><<set:M>><<set:S>>`-prefixed variant
  - also `version`, `currency_items`, `refresh_checked_state`, etc.
- `items` table (`loc TEXT PRIMARY KEY, value BLOB`): **keyed by the GGG
  stash tab id** (10-hex) for stashes, by character name for characters.
  Values are JSON arrays of the raw old item JSON, which includes the GGG
  `id` plus fields old acquisition injected: `_tab_label` / `_character`
  (used by the hash), and also `_tab`, `_type`, `_removeonly`,
  `_socketed`.
- `tabs` table (`type INT PRIMARY KEY, value BLOB`): type 0 = stash list
  JSON (with GGG ids and labels), type 1 = character list.

Because both the hash inputs and the GGG id come from the same stored
snapshot, the join is exact and fully offline — no network, no dependence
on the current API wire format. This is why the approach is immune to the
3.29 breakage: nothing is recomputed from live data.

## The new data format (master)

`data/userstore-<account>.db`:

- `item_buyouts` (`buyoutrepo.cpp`): PRIMARY KEY `item_id` (GGG id), with
  `location_id`, `location_type`, and the buyout fields. No league/realm
  column — ids are globally unique in practice.
- `location_buyouts`: PRIMARY KEY `location_id` (GGG stash id or
  character name).

`BuyoutManager::Load()` reads **only** these tables. There is currently no
code path from a legacy file into the new store.

## Existing bridge code and its defects

`src/legacy/` on master (compiled, but nothing outside the directory calls
it) already contains most of the machinery, written for the v0.15.x buyout
validator campaign:

- `LegacyDataStore` — reads a legacy sqlite file into typed structs.
- `LegacyItem::hash()` — replicates the db_version-4 hash from stored
  JSON. Validated correct by the experiment above.
- `LegacyBuyoutValidator` — the old diagnostic dialog; **not** in
  CMakeLists.txt on master.

Commit 8fb80be6 (January 18, 2026, "Add skeletal code for buyout import")
added a Buyouts menu with an import action; the menu was later stripped
from `mainwindow.cpp`, but `BuyoutRepo` survives from that commit.

Defects found (must fix before use):

1. **The items parse fails on real data.** `LegacyDataStore` parses with
   `error_on_unknown_keys = true`, but `LegacyItem` models only the ~10
   hash-relevant fields; real stored items carry `ilvl`, `frameType`,
   `icon`, `league`, `descrText`, `_removeonly`, `_tab`, `_type`, and
   more. Verified against the real 0.15.0 file: every non-empty items row
   would be rejected.
2. **All-or-nothing error handling.** One bad row or one missing `data`
   key (`return` inside the row loop; `ok &=` chain) invalidates the
   whole store. Very old files may lack keys the loader requires.
3. **No raw-id write path.** `BuyoutRepo::saveItemBuyout` requires an
   `Item &`; the importer has only ids and old JSON.

## Where old files survive

- The live `data/` file may be **stripped**: on the investigated machine
  the live Standard file has empty `items`/`tabs` tables and no
  `buyouts`/`tab_buyouts` keys (db_version stamped 5) — 0.16.x-era churn.
- Full copies survive in the automatic upgrade backups:
  `data-backup-<old-version>[-n]/` (created by
  `Application::SaveDataOnNewVersion` since f05444a6, December 31, 2025)
  and the pre-0.16 clients' own `m_data_save<version>/` directories. The
  validated 26.9 MB file came from `m_data_save0.15.0/`.
- Users upgrading directly from ≤0.15 to current master keep a full live
  file: master's `SqliteDataStore` only ignores the old tables, it never
  deletes them (and `BuyoutRepo::resetRepo` deliberately never drops).

## Semantics worth remembering

- **Buyout `source`**: `"game"` buyouts are parsed from in-game note text
  and regenerate on refresh while the note exists; `"manual"` buyouts
  exist only in acquisition's database and are the genuinely
  unrecoverable data users lose. (The validated account's buyouts were
  all `game`; `manual` and `character:`-located items are untested
  corners — worth checking against another user's file.)
- **Conflict precedent**: `BuyoutManager::MigrateItem` never overwrites a
  target whose source is MANUAL.
- **Dead machinery this supersedes**: `ItemsManager::MigrateBuyouts` +
  `Item::hash_v4`/`old_hash` + `ItemLocation::GetLegacyHash` recompute
  legacy hashes from live API data — exactly what 3.29 broke — and the
  v4→v5 step never persisted anyway (F54). Once the importer lands, that
  machinery can be deleted, resolving F54 by removal and retiring F51's
  "don't rekey GetLegacyHash" constraint.
