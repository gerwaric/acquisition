# Legacy Buyout Import — Design

Status: tracer bullet implemented, August 13, 2026. Deliberately minimal —
this is not a frozen spec. Evidence backing every claim here:
`legacy-buyout-import-investigation.md`.

## Goal

Let a user recover buyouts from a pre-0.16 database file into the current
`item_buyouts` / `location_buyouts` store, keyed on GGG ids.

## Tracer bullet scope

A menu-driven, single-file import, end to end:

1. **UI**: a "Buyouts → Import legacy buyouts…" menu action opening a
   `QFileDialog` on the acquisition data directory. The user picks the old
   database file themselves (backups included — they're just files).
2. **Read**: `LegacyDataStore` parses the file, fixed to be lenient:
   unknown JSON keys skipped, bad rows/missing keys logged and skipped
   instead of invalidating the store.
3. **Match**: `LegacyItem::hash()` per stored item → map legacy hash →
   all matching (item id, tab id, location type). Tab buyouts:
   `"stash:<label>"` → every stash id with that label from the old tabs
   table (`"character:<name>"` → the character).
4. **Write**: upsert through `BuyoutRepo` via a new raw-id save overload.
   Ambiguous hashes → write to all matching ids (replicates old
   behavior). Existing rows are **not overwritten** (skip-existing — the
   simplest safe rule).
5. **Report**: a message box with counts — imported / ambiguous /
   orphaned / skipped — and the same detail at `info` in the log.

Simplifying assumptions, on purpose:

- **GGG ids are persistent.** Validated at 1170/1171 over 14 months.
- **db_version 4 and 5 only** (5 is master's harmless re-stamp of a v4
  file — see R1-1 in `legacy-buyout-import-reviews.md`). Files at `<4`
  are detected and refused with a clear message (add the prefixed-hash
  variant later if anyone hits it).
- Import is idempotent by construction (skip-existing upserts), so no
  imported-state tracking is needed.

## Deferred (iterate after the tracer bullet works)

- Automatic candidate discovery across `data/`, `data-backup-*/`,
  `m_data_save*/`, with league/count/mtime shown per candidate.
- Smarter conflict policy (newest `last_update` wins; MANUAL protection
  à la `MigrateItem`) and merging multiple copies of the same file.
- db_version <4 hash variant.
- A startup nudge when unimported legacy buyouts are detected.
- Deleting the superseded live-hash migration machinery
  (`ItemsManager::MigrateBuyouts`, `Item::hash_v4`/`old_hash`,
  `ItemLocation::GetLegacyHash`) — closes F54, retires the F51
  constraint. Do this only once the importer has proven itself in a
  release.
