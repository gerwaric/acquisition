# Account Store Redesign

**Status: PROPOSED for review, August 1, 2026.** This document records the
decisions approved during the investigation of `SqliteDataStore`. It is not
yet frozen and authorizes no implementation by itself. The companion
execution plan is `account-store-plan.md`.

## Purpose

Retire the legacy `DataStore`/`SqliteDataStore` abstraction now that stash and
character persistence has moved to `UserStore`. The replacement must give
every durable value a typed owner and an explicit lifetime and scope. It must
also preserve old files as forensic evidence rather than converting them in
place.

This redesign covers account database identity and layout; realm/league
scoping; stash, character, buyout, shop, currency, and location-UI
persistence; the remaining global `SqliteDataStore` values; import of the old
account-name and account/league stores; and backup and recovery expectations.

It does not redesign the item refresh pipeline or forum submission protocol.
Those consumers retain their current behavior except where persistence scope
is corrected.

## Current state

Two kinds of `SqliteDataStore` are opened today:

1. A global hashed file stores the OAuth token and supplies an old application
   version fallback.
2. One hashed file per `(account display name, league)` stores shop threads,
   shop template, last shop hash, currency configuration and history,
   refresh-checked state, and a buyout-key migration marker.

Each old database has a generic `data(key, value)` table and a currency
time-series table. Older copies may also contain ignored `tabs` and `items`
tables. `SqliteDataStore` has no schema version and runs `VACUUM` when opened.
Its thread-local connection machinery remains from the former item-storage
path; the remaining consumers do not require that abstraction.

`UserStore` is newer and already owns typed stash, character, and buyout
repositories. It uses one file per account display name, SQLite
`user_version`, transactional forward migrations, and connection pragmas.
Stash and character rows carry realm and league information, but buyout rows
and the database identity do not yet carry the full approved scope.

Relevant existing findings are:

- F22: refresh-checked state is split from repository-backed buyouts.
- F54: the v4-to-v5 buyout-key migration changes memory but not the buyout
  repository.
- F66: legacy location keys use bare IDs without a location type.

## Decisions

### D1. One database per stable account identity

Each Path of Exile account has one database:

```text
data/account-<stable-account-id>.db
```

The stable ID comes from an identity whose API contract says it survives
display-name and discriminator changes. The OAuth `sub` and account-profile
`uuid` are candidates, but implementation must verify which is canonical and
whether they are interchangeable. A display name is never a database
identity.

The filename component must use a canonical, validated representation. The
database also records the stable ID and current display name as metadata so
logs, diagnostics, and account selection remain understandable.

The `data/` directory is dedicated to account databases and preserved legacy
files. The `account-` prefix remains because it makes files recognizable in
logs, backups, and support instructions.

### D2. Realm and league are mandatory domain scope

All stash, character, buyout, shop, currency, and location-UI records are
scoped by both realm and league. A repository operation that handles these
records receives an explicit context, conceptually:

```cpp
struct LeagueContext
{
    QString realm;
    QString league;
};
```

No repository infers this scope from the active UI selection or a filename.
The account ID need not be repeated in every table because the database file
itself is the account boundary.

Character records with no league in an API response still need a deliberate
storage representation. The schema/API design must not silently substitute
the currently selected league. This case is an implementation-design hold
point.

### D3. Shop state is realm-and-league local

Forum thread IDs, the effective shop template, and the last successful
submission fingerprint belong to an exact `(realm, league)` context. The
forum web server does not make this boundary safe on Acquisition's behalf;
posting an item under the wrong league may be accepted with undefined
behavior.

`ShopRepo` replaces the `shop`, `shop_template`, and `shop_hash` magic keys.
It stores structured thread IDs rather than semicolon-delimited text. The
persisted hash becomes a typed successful-submission record, including enough
input identity to establish that it applies to the same context, threads, and
template. A future shared/default-template feature must be explicit rather
than emerge through missing scope.

### D4. Buyouts are realm-and-league local and type-qualified

Item buyouts are keyed by:

```text
(realm, league, item_id)
```

Location buyouts are keyed by:

```text
(realm, league, location_type, location_id)
```

This removes reliance on undocumented global uniqueness and closes F66 for
new storage. A transfer from a temporary league into a permanent league does
not inherit a buyout accidentally. If preserving buyouts across a transfer is
desired, it must be a separately specified copy/move policy triggered by
observed domain evidence.

Buyouts are irreplaceable user-authored data. No schema migration may drop or
rebuild them as though they were a cache.

### D5. Typed repositories replace the key/value datastore

The account database is exposed through repositories with domain-specific
types and operations, not a generic string-key interface. The intended
ownership is:

- `StashRepo`: refetchable stash metadata and payload cache.
- `CharacterRepo`: refetchable character metadata and payload cache.
- `BuyoutRepo`: user-authored item and location buyouts.
- `ShopRepo`: shop configuration and successful-submission state.
- `CurrencyRepo`: authored ratios/configuration and historical snapshots.
- `LocationUiStateRepo`: refresh selection and similar durable location UI
  state.
- `AccountMetadataRepo` or equivalent store-owned methods: stable identity,
  current display name, and migration provenance.

`DataStore`, `Set`, `Get`, `SetInt`, and `GetInt` disappear after migration
compatibility no longer requires them. Serialization errors, missing values,
and database failures must not all collapse into the same default-value
result.

### D6. Currency data is separated by meaning

User-authored conversion ratios/configuration are distinct from counts
derived from the published item snapshot. Derived counts are reconstructed
where practical rather than treated as authored durable state.

Historical currency snapshots remain in SQLite. Their identity is not a
seconds-resolution timestamp primary key; the schema permits multiple samples
at the same recorded second and orders them deterministically. The repository
also needs an explicit retention decision before implementation is complete:
unbounded retention may be retained deliberately, but not accidentally.

The legacy `currency_base` value is importer input only and is not reproduced
in the target schema.

### D7. Credentials are outside account-domain databases

OAuth refresh tokens and POESESSID values are needed before an account
session database is opened. They do not belong in an account repository.

The preferred target is the operating system's credential facility, with a
small account-selection record mapping stable ID to display name and a
credential reference. Cross-platform availability, packaging impact, and a
fallback must be established before choosing an implementation. Any fallback
must be explicitly named credential storage, not another generic data bag.

The application-version value belongs in application settings. The global
database's `version` is only a legacy import/fallback concern.

### D8. Legacy files are immutable forensic sources

Acquisition never renames, modifies, vacuums, deletes, or replaces an existing
legacy data file. This remains true after a successful import. Old files may
contain unknown tables, malformed records, or evidence useful to future
forensic tools.

The importer therefore:

- opens sources read-only through a dedicated reader, never
  `SqliteDataStore`;
- writes only to the target account database;
- imports each source in an atomic target-side transaction;
- records source path/name, detected format/schema, a content fingerprint,
  importer version, import time, outcome, and useful diagnostics;
- is idempotent for the same source fingerprint and importer version;
- does not mark partial or failed work complete;
- leaves unsupported and unknown source data untouched;
- does not embed a redundant copy of the source file in the target database.

Source fingerprinting and SQLite sidecar handling must be specified before
implementation. A fingerprint cannot be declared reliable if an associated
journal or WAL could change the logical source view.

### D9. Target migrations distinguish caches from authored data

The account store retains transactional, monotonically versioned schema
migrations. A migration may invalidate or rebuild refetchable stash and
character payload caches when necessary. It may not discard buyouts, shop
configuration, currency ratios/history, or other user-authored data.

Migration code must validate actual schema shape where historical releases
are known to have stamped an incorrect version, following the lesson in F64.
Every DDL change increments the schema version in the same change.

The old `db_version` key is a buyout-key migration checkpoint, not a target
schema version. F54 must be resolved or explicitly rendered unreachable by a
tested importer before that marker is retired.

### D10. Backups must be SQLite-consistent

Copying database files and possible WAL sidecars as ordinary files is not an
accepted backup protocol. The redesign must use the SQLite backup mechanism,
a known-closed connection boundary, or another documented consistent-snapshot
procedure.

Backups identify their account database and schema version and report failed
copies. Legacy forensic sources remain outside cleanup even if they are also
included in a broader user-requested archive.

## Preliminary schema shape

Exact column types and foreign-key policy remain subject to implementation
review. The following keys are architectural requirements rather than final
DDL:

| Data | Required identity/scope |
|---|---|
| Account metadata | stable account ID |
| Stash | realm, league, stash ID |
| Character | realm, league representation, character ID |
| Item buyout | realm, league, item ID |
| Location buyout | realm, league, location type, location ID |
| Shop profile | realm, league |
| Shop thread | realm, league, stable ordering/thread ID |
| Successful shop submission | realm, league, submission/input identity |
| Currency configuration | realm, league, currency identity |
| Currency snapshot | realm, league, generated row ID, observation time |
| Location UI state | realm, league, location type, location ID |
| Import provenance | source fingerprint, importer version |

Whether stash and character API IDs are sufficiently unique to remain the
physical primary key is a schema-review question; repository lookups and
uniqueness constraints must still honor realm and league even if a surrogate
key is selected.

## Failure and recovery model

- Failure to open or migrate the target database prevents writes and is
  surfaced distinctly from an empty account.
- A failed import rolls back its target transaction and preserves diagnostics.
- Re-running an incomplete import is safe.
- A successfully imported source remains available for manual comparison.
- Cache corruption may be healed by refetching only after user-authored rows
  are protected.
- No automatic recovery path deletes a database or legacy source.

## Rejected directions

### A generic key/value table inside `UserStore`

Rejected because it relocates rather than fixes hidden typing, ownership,
scope, error, and migration contracts.

### One database per league

Not selected. Physical league isolation makes archival simple but multiplies
files and migration/connection work, splits the existing account cache, and
complicates account-wide operations. One database per stable account with
explicit realm/league keys retains logical isolation without accumulating a
file for every temporary or private league.

### Moving all remaining values to `QSettings` or JSON files

Not selected. Application preferences remain in `QSettings`, but dynamic
domain collections, authored data, submission records, and time series need
typed migrations and atomic updates. SQLite is already required and provides
those properties.

### In-place conversion or cleanup of legacy files

Rejected. It would destroy forensic evidence and make importer mistakes
irreversible.

## Open design items

The following must be resolved before this spec is frozen:

1. Verify the stable identity contract and choose OAuth `sub`, profile `uuid`,
   or a documented canonical mapping.
2. Define the representation and query behavior for characters without a
   league.
3. Decide currency-history retention.
4. Choose the credential backend and cross-platform fallback.
5. Specify source fingerprinting for SQLite files with journals/WALs.
6. Write final DDL and repository error/result contracts.
7. Define conflicts when both an existing `UserStore` and one or more old
   league stores contain related authored data.
8. Decide when import is attempted relative to authentication, database open,
   and initial cached publication.

## Acceptance criteria

The redesign is complete when:

- account renames and discriminator changes reuse the same database;
- two accounts cannot share a database accidentally;
- identical league names in different realms cannot share domain state;
- shop state and buyouts cannot cross realm/league boundaries;
- stash and character location IDs cannot collide in location-keyed state;
- every former live key has a typed owner or an explicit retirement path;
- imports are atomic, idempotent, observable, and leave every source byte
  untouched;
- authored data survives every supported migration and induced failure;
- consistent backup and restore are exercised by tests;
- production code no longer constructs `SqliteDataStore` or depends on
  `DataStore`.

