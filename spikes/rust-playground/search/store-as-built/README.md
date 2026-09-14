# store-as-built — what a search consumer gets from the store today

Status: first pass complete — 2026-09-13

- The whole item surface is **four server-side predicates**: a case-insensitive
  substring over `name`/`type_line`/`base_type`, realm, league, and
  live-or-removed. Not even `rarity` — a column — can be filtered on.
- **The one indexed dimension has no read.** `items_location` is what every
  per-container count seeks on, and no public read answers "the items in this
  tab / on this character / in this container".
- **`items_names` serves nothing.** Maintained on every ingest, unusable by
  the only search read (`LIKE '%x%'`), so `acq items search` is a full scan of
  `items` plus a `serde_json` parse of each returned body.
- **9 of `items`' 21 columns are lifted from the body; 224 census paths are
  not** — seven of them on every item (`frameType`, `frameTypeId`, `icon`,
  `identified`, `ilvl`, `league`, `verified`).
- The first change is a **read**, not a schema change: the index it needs
  already exists. Column, then derived table, are the next two rungs (F8).

## Question

What does the store crate expose to a frontend today — the facts tables and
their columns, the neutral snapshots, `item_events`, liveness and freshness,
the realm and container coordinates — what of an item is a column against
what is verbatim JSON, and where the first store change would fall. Read from
the code at `193b44c4`, so the design session starts from what is.

## Findings

**F1 — The read surface is 17 reads, and the item half of it is thin.**
`data/read-surface.md` has the table: what each returns, what it is keyed on,
its cost, and which frontend calls it. Of the item reads, `Store::search` is
the only one that takes a predicate, and its parameters are `text`, `realm`,
`league`, `include_removed`, `limit` — nothing else. `Store::item` takes an
id. There is no read that takes a rarity, a frame type, a container, an
`ilvl`, a mod, or a location. `Store::orphaned_item_annotations` exists and
**no frontend calls it**; `Annotations::get_as`, the typed intent read (C66),
is called only by the plan crate's tests — production reads `get`/`list` and
parses with `check_value`.

**F2 — The one indexed dimension has no read (the first gap).**
`items_location (location_kind, location_id)` is the index `Store::tabs`,
`Store::characters` and `read_items` all seek on
(`data/query-plans.txt`), and it is the coordinate C29/C54 make authoritative —
yet nothing public is keyed on it. `TabRow.item_count` gives a tab's live
count and no way to see the items behind it. The only routes to "this tab's
items" today are `pricing_snapshot` — which returns the *whole league*, needs
an `Annotations` handle and refuses unless the file's account uuid pairs — or
`Store::search` with a substring that happens to match. C48 shuts the third
door: raw SQL is not a surface.

**F3 — `items_names` cannot serve the only search read.** The index over
`(name, type_line, base_type)` is written on every ingest and every
`rebuild`. `Store::search` builds `%{text}%`, so SQLite's LIKE optimization
never applies; the plan is `SCAN items USING INDEX items_location` plus
`USE TEMP B-TREE FOR LAST 2 TERMS OF ORDER BY`. Two counterfactuals in
`data/query-plans.txt` pin why: an *anchored* `LIKE 'Kaom%'` also scans,
because LIKE is case-insensitive by default and the index collates BINARY;
under `case_sensitive_like=ON` the same query becomes
`SEARCH items USING INDEX items_names`. The index would serve `name = ?`, and
no read in the crate does that. Every match then costs a second time:
`ItemRow` carries `json: Value`, so each returned row is a full body parse.

**F4 — Columns against JSON.** `data/columns-vs-json.csv`, one row per column
and per census path. Of `items`' 21 columns: 9 derived from the row's own json
(`Columns::of`), 6 ingest facts the body does not carry (realm, league,
location_kind, location_id, container, socketed_in), 4 bookkeeping, the id,
and the body. `w` and `h` are columns that appear in **no read type**; `x`/`y`
appear only in `ItemSnapshot`, so `acq items search` sorts by grid position
and does not return it. Two paths are lifted at read time with no column and
no index — `$.note` and `$.inventoryId`, both in `read_items` only.

**F5 — Two reads mean different things by "in this league".**
`Store::search` filters `items.league`: for a stash item that is the request's
league, for a character item the character row's listing-owned league *as it
stood at ingest* (`lib.rs`, the `row_league` lookup). `read_items` ignores
`items.league` for character items and joins the character's **current**
league, deliberately carrying league-less characters too (C61). So a
league-filtered search silently drops a league-less character's items and can
answer from a stamp the last listing has already moved.

**F6 — Freshness, events, grouping.** Per item the store gives `first_seen`,
`last_seen`, `removed_at` and `seen_response` (that last only through
`ItemSnapshot`); membership is per response, never per clock (C54), so "what
this location holds" is exact rather than a timestamp guess. Change history is
`events_since(since, limit)` — keyed on time alone: no filter by kind,
location or item, and the name/type come from a LEFT JOIN to the live row.
Grouping does not exist: the only aggregates on the surface are the per-tab
and per-character `item_count` and the 12 counts in `Status`.

**F7 — What the module doc already rules** (so this README points instead of
restating): `crates/acquisition-store/src/lib.rs`, "As built" and "Decisions
as recorded" — C29 (bodies verbatim except at the item seams; derived columns
re-extracted, never refetched), C54 (liveness, membership per response), C55
(the character id is identity), C56 (`container`, and the drift tripwire), C57
(a granted skill is a property of its host, never a row), C60 (the store's
realm is the request's), C30 (a refused body is evidence). Cross-cutting: C12
(two surfaces), C34 (four layers; derivations reproducible), C48 (raw SQL is
not a surface).

**F8 — Where the first change falls.**

| Class | What it would be | Schema change | Ruling that governs it |
| --- | --- | --- | --- |
| a new read | `items_at(location)`; a container/location filter on `search`; an events filter | none — `items_location` already serves it | C48 (the API must be expressive enough that going around it is never worth it) |
| a derived column | `frame_type`, `identified`, `ilvl`, `corrupted`, `note` | a column + `rebuild` re-extracts it from each row's own json | C29 (a wrong extraction is repaired by re-extracting, never a refetch) |
| a derived table | a mod/stat index, FTS at ingest | a new table, reproducible from `items.json` | C34 + `decisions/store.md` "Parked: search-at-scale", whose trigger is *a real consumer with a measured latency or duplication case* — item search is that consumer, and engine-bench is the measurement |

**What a search consumer can do today, without any store change.** Pull the
corpus and filter in the frontend: `Store::search("")` matches every row whose
extracted columns are non-NULL (the current extractor writes `""`, not NULL)
and returns each with its whole body, so every predicate in the census is
reachable — at one full scan, one temp sort, and one `serde_json` parse per
item. Locate and group are then the consumer's too: `ItemRow` carries realm,
league, `location_kind`, `location_id`, `container` and `socketed_in`, so the
full coordinate is in hand; `Store::tabs` and `Store::characters` name the
containers. Freshness comes free (F6). What it cannot do is push any of it
down.

## Numbers

| Measure | Value | Source |
| --- | --- | --- |
| public reads on the surface | 17 (1 with no caller) | `data/read-surface.md` |
| server-side item predicates | 4 (substring, realm, league, live/removed) | `Store::search` signature |
| tables / indexes in the schema | 8 / 4 | `schema.sql` |
| columns across `items`, `tabs`, `characters`, `item_events` | 53 | `data/columns-vs-json.csv` |
| `items` columns: derived / ingest fact / bookkeeping / id / body | 9 / 6 / 4 / 1 / 1 | same |
| `items` columns in no read type | 2 (`w`, `h`) | same |
| read-time `json_extract`s with no column | 2 (`note`, `inventoryId`) | same |
| census paths with no column | 224 (73 top-level) | same |
| top-level paths on **every** item with no column | 7 | same |
| queries planned (9 from the crate, 3 counterfactuals) | 12 | `data/query-plans.txt` |
| of them planned as a full table scan | `search`, `characters`, `stash_basis`, and 2 counterfactuals | same |
| corpus the shares are measured over | 36,139 items | `../item-facts/data/field-census.csv` |

## Open questions

| # | Question | The one read or experiment that closes it |
| --- | --- | --- |
| Q1 | What does the full scan plus a body parse per row actually cost at league scale? The plans say what SQLite does, never how long. | engine-bench: time `Store::search` with an empty pattern against a real facts file, against the same corpus in a derived schema. |
| Q2 | At a few hundred thousand rows, does `items_location` suffice for a location-keyed read, or does it want `(realm, league, location_kind, location_id, removed_at)`? | engine-bench, same run. |
| Q3 | Is F5's stale `items.league` observable — does a character's league move between listings often enough to matter? | one store test over a listing that moves a character's league, or one live refresh across a league migration. |
| Q4 | Which of the seven universal JSON paths does a real question actually filter on, and which of them deserve a column rather than a parse? | owner-seat's questions beside item-filter's predicate list, at the design session. |
| Q5 | Must a search result carry the whole body, or a projection? `ItemRow` gives the body; `ItemSnapshot` shows what a projection looks like. | agent-seat and owner-seat: what a result row must show. |
| Q6 | The plans here are SQLite 3.53.4 (python); the shipped `acq` links 3.46.0 (bundled by rusqlite 0.32). | re-take the plans from the crate itself if a plan ever decides something. |

## Provenance

| Source | Access | Supplied by |
| --- | --- | --- |
| `crates/acquisition-store/src/{schema.sql,lib.rs,snapshot.rs,annotations.rs,index.rs,world.rs}` at `193b44c4` | the repository | — |
| `crates/acquisition-{cli,mcp,plan,daemon}/src` — callers, grepped by `scripts/read-surface.py` | the repository | — |
| `CLI-REFERENCE.md`, `MCP-REFERENCE.md` (generated from the binaries' own help) — verb and tool names | the repository | — |
| `decisions/store.md`; `CONTEXT.md` C12, C34, C48 | the repository | — |
| `../item-facts/data/field-census.csv` (36,139 items) | this directory | the item-facts track |
| SQLite version of the shipped binary: `strings target/debug/acq` → `3.46.0`; the plans were taken under python's 3.53.4 | this machine | — |

Outputs: `data/read-surface.md` (`scripts/read-surface.py`),
`data/columns-vs-json.csv` (`scripts/columns-vs-json.py`),
`data/query-plans.txt` (`scripts/query-plans.py`). Each script fails rather
than print a stale table if a symbol it annotates has moved. No `raw/`: this
track read only the repository.

## Review

| # | Finding | Fix |
| --- | --- | --- |
