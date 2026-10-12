# reader-store: items worker / item location / SQLite datastore vs. acquisition-store + acquisition-plan

**Headline (from findings.tsv, tables.tsv)**
- Pair 1 (itemsmanagerworker/itemsmanager → store lib.rs, jobs.rs, plan lib.rs): CLEAN. The shared vocabulary is GGG facts (tab types, endpoint shapes, item arrays); two reconciliation ideas are shared, expressed with no common structure.
- Pair 2 (itemlocation → store, plan game_side.rs): CLEAN. `(Remove-only)` and forum `linkItem` markup are game text; location identity, hashing and ordering are not reproduced.
- Pair 3 (sqlitedatastore/datastore.h → store DDL): CLEAN. No column set, key, pragma or file-naming rule in common.
- No EXPRESSION rows. No high-confidence IDEA rows.

C++ paths are relative to the repository root; Rust paths are relative to `spikes/rust-playground/crates/`.

## Pair 1: items worker and planner

**Why the lexical scan scored itemsmanagerworker.cpp highest.** The file is 1389 lines, mostly long English comments. Its overlap words (`outstanding`, `truthful`, `indistinguishable`, `substring`, `recognize`) are ordinary English. They match Rust files outside this pair's scope (ratelimit.rs, realm.rs, tests). The shared vocabulary is shared facts and shared English, not shared structure.

FACT rows:
- **Map/Unique substashes.** C++ `special` = `stash.parent && (type == "UniqueStash" || type == "MapStash")` (src/itemsmanagerworker.cpp:168) hides those children from the tab list. Rust `matches!(row.r#type, "MapStash" | "UniqueStash")` (acquisition-plan/src/lib.rs:1798) picks `FetchSubstash` over `FetchTab`. The two type strings are GGG data. The substash URL `/stash/{league}/{id}/{sub}` is ground-truth Appendix A (docs/design/network-ground-truth.md:1081). The uses differ.
- **Where children arrive.** Folder children come in the top-level list; Map/Unique children come only in the parent's reply. C++ re-emits list children (cpp:757) and also accepts children from a Folder reply (cpp:1014). Rust upserts list children with the folder as parent (store lib.rs:1261) and never fetches folders (`SkipReason::Folder`, plan lib.rs:797).
- **Request addresses** (realm, league, id, optional sub; a character by name): cpp:689 and `Endpoint::Stash { sub }` at store lib.rs:484. These are endpoint shapes.
- **Character item arrays.** See tables.tsv. C++ has 4 (equipment, inventory, rucksack, jewels; cpp:307). Rust has 6 (inventory, equipment, jewels, rucksack, guardian, skills; store lib.rs:455). The order differs. All six are documented fields.
- **socketedItems.** C++ recurses and gives a gem its host's location (cpp:1294). Rust lifts each gem to its own row with a `socketed_in` column (schema.sql:99).

IDEA rows (none high):
- **The parent reply is authoritative for its substashes** (medium). C++ (cpp:1052): "The parent's reply is authoritative for its children: drop cached items that display under this tab but were fetched from a child it no longer lists". It does this with `EraseSourcesIf` over `FetchSourceKey` plus a `stashChildrenReplaced` signal limited to Map/Unique. Rust (store lib.rs:1474): "The parent's fetch is its substashes' listing: a stub it no longer carries is retired, with its items". It does this with an SQL `UPDATE tabs SET removed_at … AND (listed_response IS NULL OR listed_response <> ?5)`. The API forces the rule, because substashes are visible only in the parent reply. The mechanisms share nothing.
- **A fresh listing is authoritative for membership** (medium). C++ rebuilds `m_tabs` and erases buckets (cpp:792). Rust stamps `listed_response`, then retires unstamped rows with `removed_at` (store lib.rs:1439). Rust also keeps orphaned substash rows, writes `item_events` and withholds fetches of retired locations. None of that has a C++ analogue.
- **New tabs wait** (low). C++ F55 leaves a newly discovered tab unfetched in a partial refresh (cpp:748). Rust plans are binding, so "newly discovered tabs wait for the next plan" (plan lib.rs:144). The reasons differ.

NONE (the decomposition differs): the C++ update is selection-driven, with `TabSelection` All/Checked/Selected/TabsOnly (cpp:399), buyout-driven "checked" tabs, needed/received counters, stop tokens, Parse-skip, `get_map_stashes`/`get_unique_stashes` settings, a 10-character stash-id warning, and a realm/league filter on characters. None of this appears in Rust. The Rust planner (plan lib.rs:1571–1820) compiles a sync policy into a binding plan. It uses a freshness window, a listed-count disagreement arm, an experience/league disagreement arm for characters, deleted/expired skips, an `EmptyStub` guard and `OrphanedParent`. None of these exist in C++. jobs.rs (a persisted job queue, jobs.rs:54) has no C++ counterpart. itemsmanager.cpp (buyout passes, timers, db_version 4→5 migration) has no counterpart in these Rust files.

## Pair 2: item location

- **`(Remove-only)`** (FACT, high). C++: `m_removeonly{stash.name.endsWith("(Remove-only)")}` (src/itemlocation.cpp:28), used to skip refresh locks (itemsmanager.cpp PropagateTabBuyouts). Rust has no remove-only flag. game_side.rs:51 says "A tab name carries the game's own `(Remove-only)` … tolerated, not read" (T11), and the string appears in Rust only in tests and fixtures.
- **Forum `linkItem`** (FACT, medium; this belongs to the pricing pair and is noted here for completeness). C++ writes `location="Stash%1" league x y realm` with n = API index + 1 (itemlocation.cpp:121). Rust writes `realm location="Stash{n}" league x y` with n = the tab's rank among website-listed tabs (T24; plan shop.rs:690). Both the attribute order and the meaning of n differ.
- **Substash attributed to its parent** (IDEA, low). C++ sets `id()` = parent and `fetch_id` = child (itemlocation.h:61). Rust keeps the substash as its own `tabs` row with `parent`, and the game side reads the parent's name (C80, game_side.rs:68).
- NONE: `GetLegacyHash` ("stash:"+label), `operator<` (stash before character, then tab index or locale-lowercased name), `GetRect`/QuadStash and `rebaseTabMetadata` are absent from Rust. Rust identity is (realm, league, kind, id) (schema.sql:91, store lib.rs:484). Rust orders characters by realm, league, level desc, name (store lib.rs:1681) and tabs by `TAB_ORDER_KEYS` (store lib.rs:695).

## Pair 3: SQLite datastore

- C++ creates `data(key TEXT PRIMARY KEY, value BLOB)` and `currency(timestamp INTEGER PRIMARY KEY, value TEXT)` (src/datastore/sqlitedatastore.cpp:40-41). Historically it also created `tabs(type INT PK, value BLOB)` and `items(loc TEXT PK, value BLOB)`, removed in git 55aa77f1. It sets no pragmas, runs `VACUUM` on open, and names files MD5(user|league)[-discriminator].
- Rust (acquisition-store/src/schema.sql:10-128) has `responses`, `refused`, `leagues`, `account`, `characters`, `tabs` (realm, league, id composite key), `items` (keyed by GGG item id), `item_events`, `jobs` (jobs.rs:54), and `annotations` and `meta` (annotations.rs:125, 138). It uses WAL, `synchronous`, `user_version` and `VACUUM INTO` backups, and keys files by provider and account uuid.
- The only things in common are the generic names `tabs`/`items` (the C++ ones are historical) and a key/value table (`meta`). Neither is distinctive. Verdict: NONE.

## Commits read
fa74c5ef (store crate; mentions C++ only as "never requested by the C++ app"), ee7d7bd7 and 25f99e4f (game_side; they cite the C++ `~c/o` prefix and tab-name substitution as things *not* done). The other C++-mentioning commits in cpp-commits.txt do not touch these files.
