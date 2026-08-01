# Account Store Redesign

**Status: PROPOSED for review, revision 5, August 1, 2026.** Revision 5
makes failure diagnostics independent of the target database through an
in-memory migration-run collector (D11) and records reviewer preferences
against open items 2 and 3. Revision 4 drew the boundary between continuable
ambiguity and source-level failures that stop cutover (D11), evidenced
item-buyout scope through stored location joins (D12), narrowed the
re-import claim to forensic repair, deferred Sentry diagnostics in favor of
the local report, and scoped the bundled league list to historical ID
strings known to Acquisition.
This document is not yet frozen and authorizes no implementation by itself.
The companion execution plan is `account-store-plan.md`.

## Purpose

Retire the legacy `DataStore`/`SqliteDataStore` abstraction now that stash and
character persistence has moved to `UserStore`. The replacement must give
every durable value a typed owner and an explicit lifetime and scope. It must
also preserve old files as forensic evidence rather than converting them in
place.

This redesign covers account database identity and layout; copy-forward of the
current `UserStore`; realm/league
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

The current `userstore-<display-name>.db` has one classification in this
design: it is the direct ancestor of the target account database. At cutover,
Acquisition copies it once to the stable-ID filename through SQLite's backup
API, leaves the original untouched, and continues the existing
`user_version` migration ladder only in the copy. The source is located by
the remembered `account` setting — the name that created today's file — never
the freshly authenticated display name, because a rename between sessions
would otherwise silently miss the file and forever forgo copy-forward. The
target's account metadata records whether the store was created fresh or
copied, and from which source. It is not also parsed by the legacy key/value
importer. The hashed `SqliteDataStore` files are separate legacy sources and
require that importer.

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

OAuth is the login path and the serialized token already carries `sub`, so no
general deferred-open state is planned. Once Phase 0 verifies the identity
contract, a stored token from before `sub` was persisted must complete one
online token refresh before Acquisition chooses or opens an account database.

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

Character league is nullable, matching the API model and the existing
realm-wide character-list reconciliation. League-scoped queries do not treat
`NULL` as the currently selected league. Realm-wide reconciliation may see
and retain those rows; they enter a league-specific item/shop surface only
after the API supplies that league.

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
  state. Its initial API stays limited to the load/set/clear operations the
  current refresh-checked feature needs; it is not a general preferences repo.
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

Historical currency snapshots remain in SQLite with deliberately unbounded
retention. Their identity is not a seconds-resolution timestamp primary key;
the schema permits multiple samples at the same recorded second and orders
them deterministically. One small row per refresh does not justify retention
policy machinery now; a future limit would be a separate product decision.

The legacy `currency_base` value is importer input only and is not reproduced
in the target schema.

### D7. Credentials are outside account-domain databases

OAuth refresh tokens and POESESSID values are needed before an account
session database is opened. They do not belong in an account repository.

The first target is a small typed credential store with a non-secret
account-selection record mapping stable ID to display name. This removes
credentials from the generic SQLite bag without making account-store
retirement depend on a new cross-platform keychain integration. Its security
posture must be no worse than today's plaintext `QSettings` POESESSID and
SQLite OAuth token, and its file permissions and serialization contract must
be explicit. An operating-system credential facility is a desirable, separate
future security project rather than a requirement of this redesign.

The application-version value belongs in application settings. The global
database's `version` is only a legacy import/fallback concern.

### D8. Legacy files are immutable forensic sources

Acquisition never renames, modifies, vacuums, deletes, or replaces an existing
legacy data file. This remains true after a successful import. Old files may
contain unknown tables, malformed records, or evidence useful to future
forensic tools.

The hashed-`SqliteDataStore` importer therefore:

- opens sources read-only through a dedicated reader, never
  `SqliteDataStore`;
- writes only to the target account database;
- imports each source in an atomic target-side transaction;
- records source path/name, detected format/schema, a logical-content fingerprint,
  importer version, import time, outcome, and useful diagnostics;
- is idempotent for the same source fingerprint and importer version;
- does not mark partial or failed work complete;
- leaves unsupported and unknown source data untouched;
- does not embed a redundant copy of the source file in the target database.

The fingerprint is a canonical serialization of the typed rows the importer
actually extracts, not source-file bytes. This remains stable across the old
runtime's startup `VACUUM` and ignores unknown evidence without claiming to
have imported it. The old `SqliteDataStore` uses rollback-journal mode; if a
journal or WAL sidecar is present, the importer classifies that source as
unsupported for this run, records a diagnostic, and never guesses at a
potentially interrupted logical state. What happens next follows D11's
boundary: an unattributed sidecar-bearing file is reported and skipped, but
an attributed, recognized store that cannot be read consistently may contain
the user's only shop configuration, currency history, refresh state, or
legacy buyouts, so it stops cutover safely rather than activating without
that data. Recovering such a source by copying the database and sidecar into
temporary space and letting SQLite recover the copy is possible but is extra
machinery, deferred until the edge appears in practice.

Discovery of hashed sources works forward from candidates: Acquisition
bundles a reviewed list of historical league ID strings known to
Acquisition, unions it with the current league IDs from the API and the
remembered league setting, crosses the result with remembered account names,
and hashes each candidate pair. Historical login code populated its league
selection from the API's league `id` and persisted the selected text, so ID
strings are the correct candidate form; no display-name variants are needed
unless concrete historical evidence surfaces. The bundled list does not
claim completeness over every public league. A file matching no candidate is
reported as present-but-unattributed per D11 — never silently skipped and
never guessed at. Private leagues and any missed public league degrade to
that report path, not to breakage.

The current `UserStore` copy-forward obeys the same immutability outcome but
is not an importer operation: a read-only connection to the closed source is
copied with SQLite's backup API, the original is retained, and all schema
migrations run against the new copy.

### D9. Target migrations distinguish caches from authored data

The account store retains transactional, monotonically versioned schema
migrations. A migration may invalidate or rebuild refetchable stash and
character payload caches when necessary. It may not discard buyouts, shop
configuration, currency ratios/history, or other user-authored data.

Migration code must validate actual schema shape where historical releases
are known to have stamped an incorrect version, following the lesson in F64.
Every DDL change increments the schema version in the same change.

The old `db_version` key is a buyout-key migration checkpoint, not a target
schema version. Legacy hash-keyed buyouts can be translated only when the
source contains enough item/location payload data to recompute the relevant
hash and the mapping is unambiguous. The importer never guesses silently
(D11): translatable rows become scoped target buyouts; untranslatable or
F51-ambiguous rows are
recorded as unimported diagnostics or retained in an explicitly legacy-keyed
holding table selected in the final DDL review. Either outcome preserves the
source. Once this import behavior is tested, the defective runtime migration
from F54 is deleted rather than repaired.

### D10. Backups must be SQLite-consistent

Version-change backup runs at a known-closed boundary before any database
connection opens. When the old application version cannot be determined from
settings, the preferred rule is to treat backup as required unconditionally
rather than open the global legacy database before the boundary. At that
boundary ordinary copying, including any associated
sidecars, is acceptable and must report failures. The one-time current
`UserStore` copy-forward uses SQLite's backup API. This redesign does not add
an automatic restore feature, manifest format, or backup-rotation subsystem.
Legacy forensic sources remain outside cleanup.

### D11. Edge cases use the simplest safe default; source-level failures stop cutover

Acquisition's user base is small, and most import/migration edge cases may
never occur in the field. Rather than specifying resolution machinery for
each one, the standing policy is: detect the edge, apply the simplest safe
default, record what was done in provenance, and surface it to the user.
This is sound only because legacy sources are immutable (D8) and every
applied default is recorded. Immutable sources and recorded defaults
preserve enough evidence for a later importer or forensic repair tool to
reconsider the decision; automatic corrective re-import is not promised,
because after the target has accumulated new edits a re-import may conflict
with changed buyouts, shop configuration, or currency history and would need
its own precedence rules.

Not every edge is continuable. The boundary between defaults that proceed
and failures that stop is:

- **Row-level ambiguity** (untranslatable buyout, ambiguous join, conflicting
  value): default, record, report, continue.
- **Unattributed file** (matches no known account/league candidate):
  record, report, continue. Acquisition cannot establish that it belongs to
  the authenticated account.
- **Attributed source containing only replaceable caches:** record, report,
  continue.
- **Attributed source that may contain user-authored or user-valuable data
  but cannot be read consistently** (sidecar present, read failure): stop
  cutover safely and report. Activating without it would recreate the
  invisible-data problem this design exists to prevent.
- **Target creation, copy-forward, migration, or commit failure:** stop
  safely.

Reporting must not depend on the component that failed. Several stop-safely
cases leave no usable target database: the target cannot be created or
opened, copy-forward fails before metadata exists, a migration transaction
rolls back its provenance, or the target commit fails. Diagnostics are
therefore accumulated by a migration-run collector that is independent of
the target database: it gathers records in memory throughout discovery,
copy-forward, migration, and import; applicable records are persisted to
target provenance when the target transaction commits; and the user-facing
report renders from the in-memory run diagnostics plus any previously
committed provenance. When the target is unavailable, the current run's
report can still be saved. This is not a second persistent logging system —
nothing is durable outside target provenance except the report the user
explicitly exports.

The user-facing export ("account migration diagnostic report") may include
filenames and league names because the user reviews it before pasting it
into a GitHub issue. Cutover additionally requires provenance recording and
a visible notice when defaults or attributions were applied. Sanitized
aggregate telemetry per edge class may be added later as a separate change
with its own consent and privacy review; it is not a requirement of this
redesign, and GitHub reports from the small user population decide whether
that investment is worthwhile.

### D12. Imported rows receive explicit, recorded scope attribution

Every buyout, shop, currency, and location-UI row entering the target needs
a realm and league, but the legacy sources do not carry them internally.
Scope is assigned per row from the best available evidence, and the origin
is recorded:

- **Evidenced:** a league recovered from a hashed filename whose MD5
  preimage matched a known `(account, league)` candidate; a realm/league
  joined from stash or character rows via a stored location. Both item and
  location buyout rows already carry `location_id` and `location_type`, so
  both join the same way: a `stash` location joins to `stashes.id`, a
  `character` location joins to `characters.id`. A row is evidenced when
  that join produces exactly one realm/league.
- **Attributed:** the remembered realm (hashed sources never encoded a
  realm) and, for rows with no unique join, the remembered league.
  Ambiguous joins are recorded rather than resolved by silently choosing
  one. Attributed rows are flagged attributed-not-evidenced in provenance
  per D11.

This is a deliberate, recorded exception to D2's no-inference rule, confined
to one-time import/migration attribution. The D2–D4 scope guarantees apply
fully to evidenced rows; attributed rows are best-effort, and their flag
makes a misplacement diagnosable if a user ever reports one.

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
- Diagnostic reporting survives target-side failure: the migration-run
  collector holds the run's diagnostics in memory, so a report can be saved
  even when the target database cannot be created, opened, or committed
  (D11).
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
2. Write final DDL and repository error/result contracts, including whether
   untranslatable legacy buyouts use a holding table or provenance-only
   diagnostics. Review preference: provenance-only diagnostics, unless a
   concrete recovery workflow requires the table — the immutable source
   already preserves the complete row, and duplicating unresolved legacy
   records into the target creates another schema and lifecycle to
   maintain.
3. Define per-type precedence for conflicts between copy-forward `UserStore`
   data and old hashed account/league sources, without treating the former
   as another importer input; where evidence is absent, D11's
   default-and-report applies rather than bespoke resolution rules. Review
   baseline: copied-forward `UserStore` buyouts win when the same scoped
   target key already exists; translatable legacy buyouts fill missing
   target keys; hashed stores remain authoritative for shop, currency, and
   refresh state because the current `UserStore` never owned those values;
   any genuine same-owner conflict is recorded rather than resolved through
   elaborate merging.
4. Pin the exact cutover ordering relative to authentication and initial
   cached publication; activation is forbidden until all user-valuable data
   paths are available in one release, and a cutover failure stops safely
   with a report rather than falling back to the legacy write path.

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
- every applied edge default and scope attribution is recorded in provenance
  and reportable by the user, and no source is silently skipped;
- an attributed source that may contain user-valuable data but cannot be
  read consistently stops cutover rather than activating without it;
- a stopped cutover still produces a saveable diagnostic report even when
  the target database cannot be created, opened, or committed;
- authored data survives every supported migration and induced failure;
- known-closed version backup and SQLite copy-forward are exercised by tests;
- production code no longer constructs `SqliteDataStore` or depends on
  `DataStore`.
