# Account Store Redesign: Implementation Plan

**Status: DRAFT, revision 5, August 1, 2026.** Revision 5 makes failure
diagnostics independent of the target database through the design's
migration-run collector (D11). Revision 4 tracked the
design's revised D11 boundary (continuable ambiguity versus source-level
failures that stop cutover), evidenced item-buyout scope through stored
location joins (D12), deferred Sentry diagnostics in favor of the local
report, and scoped the bundled league asset to historical ID strings known
to Acquisition. This plan
sequences implementation of `account-store.md` and must not outrun unresolved
hold points. Unlike the design, it may be retired after execution; durable
decisions remain in the design and findings register.

## Working rules

- Make no production persistence change until the target operation has a
  failure-path test.
- Never write to a legacy source, including in tests intended to model
  production behavior.
- Treat stash/character payloads as refetchable caches and buyouts, shop
  configuration, currency configuration/history, and UI choices as authored
  or user-valuable data.
- Every DDL change and its schema-version increment land together.
- Each phase is independently buildable and testable. Phases 1-5 are
  preparatory-only until the single cutover gate after Phase 5: an existing
  install must never activate an empty or partially populated account store.
- Keep the old reader path available until all supported imports have run, but
  do not keep dual writers.
- Prefer design D11's detect-default-record-report over per-edge resolution
  machinery. Continuable ambiguity (row-level defaults, unattributed files,
  cache-only sources) never blocks cutover, destroys evidence, or resolves
  silently. An attributed source that may contain user-authored or
  user-valuable data but cannot be read consistently, and any target-side
  creation, copy-forward, migration, or commit failure, stops cutover safely
  per D11's boundary.

## Phase 0: Evidence and design closure

### 0.1 Stable identity

- Verify in authoritative API/OAuth documentation what OAuth `sub` and account
  profile `uuid` identify, their stability, realm behavior, and relationship.
- Capture representative authenticated payloads without committing secrets.
- Select the canonical stable ID and define validation/canonicalization.
- Pin the rule that a remembered token lacking the canonical identity claim
  must complete one online refresh before the account store opens.

**Gate:** the application can determine one canonical stable ID before it
chooses an account database; no deferred-open half-state is introduced.

### 0.2 Legacy population and schema inventory

- Enumerate every historical filename form. Include the global
  `MakeFilename("", "")` file (MD5 input `"|"`), pre-discriminator
  account/league hashes, and discriminator-suffixed files whose hash input
  uses only the base account name.
- Create the bundled known-league asset: a reviewed list of historical
  league ID strings known to Acquisition, maintained in the repository.
  Historical login code populated its league selection from the API's league
  `id` and persisted the selected text, so ID strings are the candidate
  form; add display-name variants only if Phase 0 finds concrete historical
  evidence for them. Do not claim completeness over every public league.
  Discovery unions the asset with the current API league IDs and the
  remembered league setting; files matching no candidate are classified
  present-but-unattributed (D11). Fixtures pin important historical
  examples.
- Inventory known table shapes and key encodings from repository history.
- Build fixture databases for each supported shape, plus unknown tables,
  malformed rows, duplicate/conflicting values, journal/WAL sidecars, and
  failed reads. Sidecar fixtures prove both sides of the D11 boundary: an
  unattributed sidecar-bearing file is reported and skipped, while an
  attributed source that may hold user-valuable data stops cutover safely.
  Edges handled by D11
  default-and-report need one fixture proving detection and recording, not
  exhaustive per-variant fixtures.
- Confirm reachability and required handling of F54's `db_version` states.
- Classify hash-keyed legacy buyouts: translate only with sufficient source
  item/location payloads and an unambiguous hash mapping; exercise absent and
  incomplete `items` tables plus F51's unnamed-tab ambiguity. Select either an
  explicitly legacy-keyed holding table or provenance diagnostics for rows
  that cannot be translated. Never guess silently (D11).

**Gate:** every importer branch is justified by a fixture or deliberately
classified unsupported while preserving its source.

### 0.3 Final contracts

- Resolve every open item in `account-store.md`.
- Write final DDL, repository result/error types, and transaction ownership.
- Record deliberately unbounded currency-history retention; do not design
  retention-policy machinery.
- Finalize canonical logical-content serialization for importer fingerprints;
  a rollback journal or WAL sidecar classifies the source unsupported for
  that run with a diagnostic, and the D11 boundary decides whether that
  source's status stops cutover.
- Specify per-type import precedence without using "last writer wins" as an
  implicit default; where evidence is absent, D11's default-and-report
  applies instead of bespoke resolution rules. Start from the design's
  open-item-3 baseline: copied-forward `UserStore` buyouts win existing
  scoped keys, translatable legacy buyouts fill missing keys, hashed stores
  stay authoritative for shop/currency/refresh state, and same-owner
  conflicts are recorded rather than merged.
- Review the target against F22, F54, F64, and F66.

**Gate:** freeze the reviewed design before schema implementation.

### 0.4 Known-closed version backup

- Move version-change backup before creation of any database connection.
  The pre-0.16 version fallback currently reads the global legacy store
  (`Application::SaveDataOnNewVersion`); the preferred rule is to treat a
  missing settings version as backup-required unconditionally, which is
  simpler than opening the global database before the boundary. A read-only
  open-check-close remains the fallback only if unconditional backup proves
  impractical.
- At that known-closed boundary, copy database files and any sidecars and
  report every failed copy.
- Do not delete or relocate legacy sources.
- Test ordering (no connection exists), sidecar inclusion, and copy failure.
- Do not add a manifest, restore workflow, or backup-rotation subsystem; those
  require a separately approved user-facing feature.

**Gate:** an integration test proves version backup occurs before any database
opens and produces a complete copy of its known-closed inputs. This lands
before Phase 1 creates or copies an account database.

## Phase 1: Account identity and store foundation

- Introduce a typed account identity value and `LeagueContext`.
- Add account-selection metadata sufficient to map a remembered login to a
  stable ID without using a display name as identity.
- Rename/evolve `UserStore` into the account-store boundary.
- If `data/account-<stable-id>.db` does not exist, locate the source
  `userstore-<name>.db` by the remembered `account` setting — the name that
  created today's file — never the freshly authenticated display name, and
  copy the closed source once through SQLite's backup API using a read-only
  source connection. Never rename or modify the source. Record in account
  metadata whether the store was created fresh or copied, and from which
  file.
- Open the copied or fresh target with a unique Qt connection name and the
  existing connection pragmas, then continue the `user_version` ladder only
  in that target.
- Add account metadata and migration-provenance schema.
- Make open/migration failures explicit to the caller.
- Add tests for canonical filenames, display-name changes, distinct accounts,
  invalid IDs, schema creation, migration rollback, concurrent open
  behavior actually required by the application, and copy-forward attempted
  while a second instance holds the source open. For the concurrent-hold
  case, success means obtaining a transactionally consistent snapshot; the
  test must not expect the source's bytes or modification time to remain
  unchanged if the other process is actively writing.

This phase does not parse hashed legacy `SqliteDataStore` data. The current
`UserStore` is a direct copy-forward ancestor, not an importer source.

**Phase gate:** a new account can open a correctly identified empty target,
and a current account can copy forward all `UserStore` rows, without changing
either source. This is preparatory-only until the cutover gate after Phase 5.

## Phase 2: Realm/league-scoped existing repositories

- Change stash and character repository contracts to require
  `LeagueContext` where applicable.
- Redesign item and location buyout keys to include realm and league; location
  keys additionally include location type.
- Backfill scope for copied-forward buyout rows per design D12: join both
  item and location buyouts through their stored `location_type` and
  `location_id` — `stash` rows to `stashes.id`, `character` rows to
  `characters.id`. Mark a row evidenced when the join produces one
  realm/league. Apply the remembered realm/league only when no unique join
  exists, flagged attributed-not-evidenced. Record ambiguous joins instead
  of choosing one silently.
- Preserve all copied-forward buyouts through an ordinary transactional
  `user_version` migration in the target database.
- Add negative tests proving cross-realm, cross-league, and cross-location-type
  reads cannot occur.
- Add migration-failure injection proving authored rows survive rollback.
- Keep F54 out of this schema step: it concerns hash-keyed buyouts in old
  `SqliteDataStore` files and is resolved by Phase 4 classification. The
  runtime migration is deleted in Phase 6 rather than fixed.

**Phase gate:** existing typed repositories operate against the target scope,
and no copied-forward authored buyout is lost under supported schema
histories. This remains preparatory-only.

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
- Retain history without a limit, deliberately.
- Move CSV export to the typed query.
- Test two snapshots in one second, deterministic ordering, that history is
  never pruned, and serialization/schema upgrades.

### 3.3 LocationUiStateRepo

- Move refresh-checked state out of `BuyoutManager` persistence.
- Key it by realm, league, location type, and location ID.
- Keep `BuyoutManager` responsible only for buyout behavior, closing F22 as a
  consequence of clearer ownership.
- Test identical IDs across location types and contexts.

**Phase gate:** all session-scoped live string keys have typed target owners.
This remains preparatory-only until their old values can be imported.

## Phase 4: Read-only legacy importer

- Implement a dedicated read-only SQLite source reader with no schema setup,
  pragmas that mutate state, cleanup, or `VACUUM`.
- Detect supported source formats without altering them.
- Fingerprint a canonical serialization of extracted logical rows, not source
  bytes, so the old runtime's `VACUUM` does not defeat idempotency. If a
  rollback journal or WAL sidecar is present, classify the source unsupported
  for this run with a diagnostic, then apply the D11 boundary: skip an
  unattributed file, but stop cutover safely for an attributed source that
  may contain user-valuable data.
- Discover hashed account/league stores by forward-hashing candidates from
  the known-league union (bundled asset, current API leagues, remembered
  league) against remembered account names. Report files matching no
  candidate as present-but-unattributed (D11); never guess ownership. Do not
  parse the current account-name `UserStore` here.
- Assign scope to imported rows per design D12, recording evidenced versus
  attributed origins; realm is always attributed for hashed sources.
- Import each source into one target-side transaction.
- Record provenance and outcome only according to the frozen idempotency
  contract.
- Import old caches where useful, hash-keyed buyouts where translatable, shops,
  currency data/history, refresh state, and migration metadata according to
  explicit per-type conflict rules. Record untranslatable/ambiguous buyouts in
  the approved holding area or provenance diagnostics.
- Preserve unknown keys/tables only in the source; report them in diagnostics
  without pretending they were imported.
- Build the user-facing diagnostic surface (D11): a migration-run collector
  that accumulates diagnostics in memory across discovery, copy-forward,
  migration, and import; persistence of applicable records to target
  provenance on commit; a notice when defaults or attributions were
  applied; and an "account migration diagnostic report" export rendered
  from the in-memory run diagnostics plus previously committed provenance,
  saveable even when the target database is unavailable. Sanitized
  aggregate telemetry is
  deliberately out of scope for v1; it may be added later as a separate
  change with its own consent and privacy review.
- Compare source bytes and relevant filesystem metadata before and after every
  importer integration test.

**Failure matrix:** exercise open failure, malformed schema, malformed value,
constraint conflict, target write failure, commit failure where injectable,
process interruption/restart, repeated import, changed source content, a
new importer version, and diagnostic-report export while the target cannot
be created, opened, or committed.

**Phase gate:** supported real-world fixtures import exactly once, failures
are recoverable, the D11 boundary is enforced — unattributed and cache-only
sources are reported and skipped, while an attributed source with
potentially user-valuable data that cannot be read consistently stops
cutover safely — and all source files remain byte-for-byte untouched. The
new store still does not activate until Phase 5 removes remaining old
writers.

## Phase 5: Credential and global-state extraction

- Move the OAuth refresh token and POESESSID into a small typed credential
  store with explicit serialization and file-permission behavior. OS keychain
  integration is a separate future project, not a cutover dependency.
- Store non-secret stable-ID/display-name account-selection metadata beside
  it without recreating a generic key/value database.
- Preserve "remember me" semantics: clearing credentials does not erase
  realm/league preferences or account-domain data.
- Move the application version fully to application settings.
- Read the global legacy datastore only through the immutable importer during
  the compatibility period.
- Test first login, remembered login, token refresh, account switching,
  display-name change, a pre-`sub` token requiring online refresh,
  user-requested credential clear, and upgrade from each supported
  global-store state.

**Cutover gate:** normal startup no longer opens any writable legacy
`SqliteDataStore`; current `UserStore` copy-forward and every supported,
attributed hashed-store import complete before the target becomes active;
unattributed and cache-only sources are reported and skipped without
blocking, while an attributed source that may contain user-valuable data but
cannot be read consistently stops cutover safely (D11); and any cutover
failure stops safely with a report — rendered from the migration-run
collector when the target database is unavailable — rather than
presenting an empty store or falling back to the legacy write path — the
untouched legacy files are the recovery guarantee, and the atomic idempotent
import makes a retry safe. Phases 1-5 ship together for existing installs
unless an earlier release leaves the old runtime path fully active and treats
new code as dark, preparatory infrastructure.

## Phase 6: Retire the legacy abstraction

- Remove remaining `DataStore` parameters from managers and UI construction.
- Remove `SqliteDataStore`, `DataStore`, `CurrencyUpdate`'s legacy placement,
  hashed filename generation, startup `VACUUM`, and thread-local legacy
  connections.
- Delete `ItemsManager::MigrateBuyouts`; Phase 4 owns the only supported
  hash-to-ID translation and F54 is not repaired in the runtime path.
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
| Import | first run; repeat; partial failure; changed source; unsupported source; unattributed file; attributed-scope row; attributed-but-unreadable source stopping cutover |
| Durability | transaction rollback; restart; SQLite copy-forward; known-closed backup |
| Credentials | remembered; cleared; expired; pre-`sub` refresh |

## Release strategy

Prefer additive commits and dark preparatory infrastructure before one visible
cutover:

1. Build and test the target store, copy-forward, typed repositories, importer,
   and credential extraction without activating an incomplete path for
   existing users.
2. Cut over only when current `UserStore` data and all supported hashed-store
   user data are available in the target during the same release.
3. Observe user-filed diagnostic reports on GitHub across at least one
   compatibility release, and let observed frequency decide which edges
   deserve further investment. If reports prove insufficient, add sanitized
   aggregate telemetry as a separate change.
4. Remove legacy runtime code and the generic abstraction after the announced
   compatibility boundary.
5. Retain source files indefinitely unless a future, separately approved user
   action manages them. No automatic cleanup milestone is planned.

Diagnostics must identify files safely without logging tokens or serialized
credential contents. Any telemetry or uploaded diagnostics requires separate
privacy review and user consent; deferred aggregate telemetry falls under
that requirement, while the user-reviewed "account migration diagnostic
report" export does not upload anything on its own.
