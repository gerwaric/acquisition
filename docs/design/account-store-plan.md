# Account Store Redesign: Implementation Plan

**Status: DRAFT, August 1, 2026.** This plan sequences implementation of
`account-store.md`. It must be revised as that proposed design is reviewed and
must not outrun unresolved hold points. Unlike the design, this document may be
retired after execution; durable decisions remain in the design and findings
register.

## Working rules

- Make no production persistence change until the target operation has a
  failure-path test.
- Never write to a legacy source, including in tests intended to model
  production behavior.
- Treat stash/character payloads as refetchable caches and buyouts, shop
  configuration, currency configuration/history, and UI choices as authored
  or user-valuable data.
- Every DDL change and its schema-version increment land together.
- Each milestone is independently buildable, testable, and safe to ship unless
  it is explicitly marked preparatory-only.
- Keep the old reader path available until all supported imports have run, but
  do not keep dual writers.

## Phase 0: Evidence and design closure

### 0.1 Stable identity

- Verify in authoritative API/OAuth documentation what OAuth `sub` and account
  profile `uuid` identify, their stability, realm behavior, and relationship.
- Capture representative authenticated payloads without committing secrets.
- Select the canonical stable ID and define validation/canonicalization.
- Specify behavior when a remembered legacy credential lacks that ID or the
  profile lookup is unavailable.

**Gate:** the application can determine one canonical stable ID before it
chooses an account database, or has an explicit non-destructive deferred-open
flow.

### 0.2 Legacy population and schema inventory

- Enumerate every historical filename form, including pre-discriminator and
  discriminator-suffixed account/league hashes.
- Inventory known table shapes and key encodings from repository history.
- Build fixture databases for each supported shape, plus unknown tables,
  malformed rows, duplicate/conflicting values, journal/WAL cases, and failed
  reads.
- Confirm reachability and required handling of F54's `db_version` states.

**Gate:** every importer branch is justified by a fixture or deliberately
classified unsupported while preserving its source.

### 0.3 Final contracts

- Resolve every open item in `account-store.md`.
- Write final DDL, repository result/error types, and transaction ownership.
- Decide currency-history retention.
- Specify import conflict precedence without using "last writer wins" as an
  implicit default.
- Review the target against F22, F54, F64, and F66.

**Gate:** freeze the reviewed design before schema implementation.

## Phase 1: Account identity and store foundation

- Introduce a typed account identity value and `LeagueContext`.
- Add account-selection metadata sufficient to map a remembered login to a
  stable ID without using a display name as identity.
- Rename/evolve `UserStore` into the account-store boundary.
- Open `data/account-<stable-id>.db` with a unique Qt connection name and the
  existing connection pragmas.
- Add account metadata and migration-provenance schema.
- Make open/migration failures explicit to the caller.
- Add tests for canonical filenames, display-name changes, distinct accounts,
  invalid IDs, schema creation, migration rollback, and concurrent open
  behavior actually required by the application.

This phase does not import or write legacy domain data.

**Ship gate:** a new account can open a correctly identified empty target
store without changing any legacy file.

## Phase 2: Realm/league-scoped existing repositories

- Change stash and character repository contracts to require
  `LeagueContext` where applicable.
- Redesign item and location buyout keys to include realm and league; location
  keys additionally include location type.
- Preserve all existing buyouts through a transactional migration.
- Add negative tests proving cross-realm, cross-league, and cross-location-type
  reads cannot occur.
- Add migration-failure injection proving authored rows survive rollback.
- Resolve F54 in the new write-through/import path before retiring
  `db_version` semantics.

**Ship gate:** existing typed repositories operate against the target scope,
and no authored buyout is lost under supported schema histories.

## Phase 3: New typed repositories

### 3.1 ShopRepo

- Store ordered thread IDs structurally.
- Store the effective template for an exact realm/league.
- Store successful-submission identity with context, relevant input identity,
  and timestamp.
- Move `Shop` off `shop`, `shop_template`, and `shop_hash`.
- Test that identical rendered content in different realms/leagues never
  suppresses a submission incorrectly.

### 3.2 CurrencyRepo

- Separate authored currency ratios/configuration from derived counts.
- Store snapshots with a generated identity and observation timestamp.
- Implement the approved retention policy.
- Move CSV export to the typed query.
- Test two snapshots in one second, deterministic ordering, retention, and
  serialization/schema upgrades.

### 3.3 LocationUiStateRepo

- Move refresh-checked state out of `BuyoutManager` persistence.
- Key it by realm, league, location type, and location ID.
- Keep `BuyoutManager` responsible only for buyout behavior, closing F22 as a
  consequence of clearer ownership.
- Test identical IDs across location types and contexts.

**Ship gate:** all session-scoped live string keys have typed target owners.

## Phase 4: Read-only legacy importer

- Implement a dedicated read-only SQLite source reader with no schema setup,
  pragmas that mutate state, cleanup, or `VACUUM`.
- Detect supported source formats without altering them.
- Establish a stable logical fingerprint procedure, including journal/WAL
  rules decided in Phase 0.
- Discover the account-name `UserStore` and hashed account/league stores that
  belong to the authenticated identity. Do not guess ownership where the hash
  cannot be tied to known account/league inputs.
- Import each source into one target-side transaction.
- Record provenance and outcome only according to the frozen idempotency
  contract.
- Import caches, buyouts, shops, currency data/history, refresh state, and
  migration metadata according to explicit per-type conflict rules.
- Preserve unknown keys/tables only in the source; report them in diagnostics
  without pretending they were imported.
- Compare source bytes and relevant filesystem metadata before and after every
  importer integration test.

**Failure matrix:** exercise open failure, malformed schema, malformed value,
constraint conflict, target write failure, commit failure where injectable,
process interruption/restart, repeated import, changed source content, and a
new importer version.

**Ship gate:** supported real-world fixtures import exactly once, failures are
recoverable, and all source files remain byte-for-byte untouched.

## Phase 5: Credential and global-state extraction

- Move the OAuth refresh token and POESESSID into the selected credential
  backend/fallback.
- Store only the credential reference and non-secret account-selection
  metadata outside the credential backend.
- Preserve "remember me" semantics: clearing credentials does not erase
  realm/league preferences or account-domain data.
- Move the application version fully to application settings.
- Read the global legacy datastore only through the immutable importer during
  the compatibility period.
- Test first login, remembered login, token refresh, account switching,
  display-name change, backend unavailable, user-requested credential clear,
  and upgrade from each supported global-store state.

**Ship gate:** normal startup no longer opens a writable global
`SqliteDataStore`.

## Phase 6: Consistent backup and recovery

- Replace ordinary copying of live SQLite files with the approved consistent
  snapshot mechanism.
- Define backup manifest information: application version, schema version,
  stable account ID, creation time, and included files.
- Do not delete or relocate legacy sources during backup rotation.
- Add restore validation before replacing or opening a target.
- Test backup under WAL activity, injected write failure, incomplete backup,
  restore to a clean data directory, and schema-version mismatch.

**Ship gate:** an integration test can create, mutate, back up, restore, and
verify authored data without relying on timing or loose sidecar copying.

## Phase 7: Retire the legacy abstraction

- Remove remaining `DataStore` parameters from managers and UI construction.
- Remove `SqliteDataStore`, `DataStore`, `CurrencyUpdate`'s legacy placement,
  hashed filename generation, startup `VACUUM`, and thread-local legacy
  connections.
- Keep the dedicated immutable importer for the announced compatibility
  period; it is not the old runtime abstraction.
- Update tests and fixtures to use typed repositories or narrow fakes.
- Update `AGENTS.md`, `BUILD.md` if setup changed, and the documentation map.
- Record resolved findings in `docs/cleanup/findings.md`; do not renumber them.

**Completion gate:** the full build and Qt Test suite pass; a source search
shows no production `DataStore`/`SqliteDataStore` use; upgrade fixtures retain
all authored data; and legacy sources remain present and unchanged.

## Cross-cutting verification matrix

Each relevant phase must cover these dimensions rather than relying only on a
happy-path migration test:

| Dimension | Required cases |
|---|---|
| Account identity | same account renamed; two accounts with similar names; invalid/missing ID |
| Scope | same league across realms; two leagues in one realm; private league |
| Location identity | stash/character ID collision; same location ID across contexts |
| Data class | refetchable cache; authored row; derived value; history |
| Upgrade | fresh database; every supported schema; historically mis-versioned schema |
| Import | first run; repeat; partial failure; changed source; unsupported source |
| Durability | transaction rollback; restart; WAL/backup; restore |
| Credentials | remembered; cleared; expired; backend unavailable |

## Release strategy

Prefer additive releases before removal:

1. Ship the target store and typed repositories with tests.
2. Ship read-only import while legacy sources remain accepted inputs.
3. Observe migration diagnostics across at least one compatibility release.
4. Remove legacy runtime writes and the generic abstraction.
5. Retain source files indefinitely unless a future, separately approved user
   action manages them. No automatic cleanup milestone is planned.

Diagnostics must identify files safely without logging tokens or serialized
credential contents. Any telemetry or uploaded diagnostics requires separate
privacy review and user consent.

