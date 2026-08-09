# Synthetic Data

Status: living topic doc for ADR 0003 exploration. Tooling lives in
`tools/synthdata/`; this document records the design, its rationale,
and rejected alternatives. Provenance: distilled from the synthetic
data brainstorm (Auro, Aug 8, 2026) following the maintainer's Aug 8
Discord note (synthetic data, UX instrumentation, rust CLI).

## Principle

**Commit generators, not data.** The repo carries tooling, statistics,
and a small fixture slice of public RePoE data; every dataset is
produced locally, parameterized by scale and seed. The privacy
guarantee is deliberately narrow: **no raw item records, item or stash
ids, item names, tab names, league names, or directly identifying
account data are ever committed.** A profile *is* derived from a real
account — it is that account's shape as counts and quantiles — so
committing one is a consent decision, not an anonymity proof: profiles
are committed only with the account owner's explicit consent, and
`fresh-account.json` is the contributing author's own account,
committed by its owner. Aggregation is not a mathematical anonymity
guarantee (a very small account's quantiles closely reproduce its
exact shape); owner consent is the rule precisely because of that.
Veteran profiles from other contributors follow the same rule: the
owner runs `profile.py`, reviews the output, and commits it
themselves or explicitly hands it over.

## Inputs

Two:

- **RePoE** (`repoe_data.py`) — the same `https://repoe-fork.github.io`
  source the application fetches (`src/repoe/repoe.cpp`), plus
  `mods.min.json`, which the app does not consume but the generator
  needs for legal mod pools, tier value ranges, and display templates.
  Cached under `tools/synthdata/.cache/<version>/` (gitignored), keyed
  by upstream `version.txt` so a game update invalidates naturally.
  A deterministic subset (`make_fixtures.py`) is committed under
  `tools/synthdata/fixtures/` — a few bases per emittable item class,
  their spawn-legal mods, and the referenced stat translations — so
  the test lane runs offline (`generate.py --repoe-dir fixtures`).
  `.github/workflows/synthdata-freshness.yml` checks weekly (a single
  `version.txt` fetch) whether upstream moved, re-extracts on
  mismatch, and warns — distinguishing "version bumped, subset
  unchanged" from "the shapes the generator relies on drifted" —
  without ever failing a build.
- **A profile** (`profile.py`) — statistics extracted from a real
  userstore DB: tab-type mix, items-per-tab quantiles, league shares
  (recorded by rank, never by name), character levels, buyout and
  public-tab density. Counts and histograms only — no ids, names,
  items, or mod values — but still derived from a real account and
  committed only under the owner-consent policy stated under
  Principle. `fresh-account.json` (committed) is the contributing
  author's own small new-player account; a veteran-account profile
  can be added later if the maintainer wants his account's shape
  represented, on the same terms.

## Generation (`generate.py`)

- **Mods use RePoE-derived candidates and ranges**, subject to the
  approximations documented under Known limits: prefix/suffix pools
  filtered by spawn-weight tags against the base item's tags, group
  exclusivity respected, values rolled inside real tier ranges,
  display text instantiated from the mod's own template. Emitted in
  the 3.29 wire shape: `ItemMod{description, flags}` objects for
  implicit/explicit (multi-line text stays one description with
  `\r\n`), plain strings for enchant/utility/rune arrays, per
  `src/poe/types/item.h`.
- **Structure matches the application's currently modeled payload
  shapes** for the structures it exercises: Map/Unique stashes are
  parent rows plus child rows, with `children` metadata in the parent
  payload (F49); items shelf-pack into real grid bounds; stackables
  use base `stack_size` and carry the `Stack Size` display property
  the app's count actually reads; icons use the
  `web.poecdn.com/image/<dds path>.png` form of RePoE
  `visual_identity`, which resolves publicly.
- **`--coverage`** appends a tab covering the optional fields
  enumerated by `coverage_axes()` — currently each influence singly
  and paired, corruption states, fracture/synthesis (as mod flags),
  foil, relic, scourge, crucible, and the PoE2-only parse fields
  (`runeMods`, `gemSockets`, `sanctified`, `doubleCorrupted`,
  `unidentifiedTier`, `desecrated`) — a list that tracks `item.h` by
  maintenance, not by construction. PoE2 fidelity deliberately
  matches the app: parse-level only, realm flows unchanged (F63's
  deferral stands).
- **`--items` is an approximate minimum for stash items**, not an
  exact cardinality: generation stops after the tab that reaches the
  target, then characters and (with `--coverage`) probe items are
  added on top. The CLI reports the populations separately
  (`N items (a stash, b character, c probe)`); counts are top-level
  items only, socketed sub-items excluded. Benchmark sweeps that need
  exact-cardinality comparisons should compare reported totals, not
  the `--items` argument.
- **Deterministic** per the full input tuple: seed, item target,
  profile bytes, league list, `--now` timestamp, RePoE input bytes,
  and quirks-registry bytes. Every generated database gets a sidecar
  `<db>.manifest.json` whose `repro` block records all of these
  (content-hashed), so a dataset stays identifiable after the
  repository and upstream RePoE have moved on; the ctest lane
  enforces two-run equality on every run (`selftest.py`). Measured:
  100k items ≈ 43 s / 65 MB; ~2M extrapolates to ≈14 min / 1.3 GB
  (estimated). Scale is a parameter, so the webview-scale spike's
  target is a sweep, not a decision.

## Validation

The regular suite exercises the generator on every ctest run — no env
vars, no network:

- **`synthdata_generate`** (a CTest fixture, registered when a Python
  interpreter exists; without one it is not registered and
  `tst_synthdata` skips with a message) runs
  `tools/synthdata/selftest.py`: generates a small store twice from
  the checked-in RePoE fixtures with identical inputs, fails on any
  row-level difference (the determinism contract, enforced rather
  than claimed), and hands the result to `tst_synthdata`.
- **`tests/tst_synthdata.cpp`** then makes three independent checks:
  1. *Parse*: every stash and character payload loads through the
     application's own `UserStore`/repo layer into the typed `poe::`
     structs. Precisely: this exercises the repository/deserialization
     layer that `ItemsManagerWorker::ParseCachedItems` also uses — it
     does not invoke the worker itself; worker, model, search, and
     tooltip behavior are covered by their own tests, not this one.
  2. *Probes*: the generator's manifest names every coverage axis and
     every registry quirk with the exact serialized bytes of the item
     it emitted; the test asserts each probe appears in the raw
     database exactly once. A coverage entry that silently stops
     being emitted, gets overwritten, or loses its unusual
     representation (an explicit `null`, a deliberately deleted key)
     is a named failure — "probe 'crlf-in-mod-text' found 0 times" —
     not a still-parseable tab.
  3. *Schema*: the generator necessarily duplicates the `UserStore`
     DDL (it is Python; the app schema lives in C++). The hazard is
     drift — the generator stamping the current `user_version` onto
     an outdated shape, which would silently bypass migrations. So
     the test creates a fresh store through `UserStore` itself and
     compares schemas: every object's whitespace-normalized DDL,
     every table's `table_info` rows, and `user_version`. Any app
     schema change that the generator misses fails the suite.

The same binary validates big external datasets: point
`ACQ_SYNTH_DATA_DIR`/`ACQ_SYNTH_ACCOUNT` at any generated store to
override the ctest default. The parse check caught two wire-format
facts on its first run (mods-as-objects; crafted/fractured as flags
rather than arrays), which is the argument for the direction of
authority: the generator tracks the app's parse layer, not the other
way around.

## Wire idiosyncrasies (maintainer feedback, Aug 8)

Real accounts carry a decade of historical shapes the developer docs do
not describe: colour strings that are not 6 characters, crucible mod
trees surviving only on Standard, fields that predate the docs. The
response is two-sided:

- **Reproduce what is evidenced.** A survey of a real 3.10-era Standard
  account (43k items) informed the generator: capitalized
  `frameTypeId` vocabulary with a small share of legacy items omitting
  the field entirely; `rarity` strings alongside `frameType`; socketed
  gems carrying old-style `socket`/`colour` fields; GGG's
  empty-array `flags` encoding on mods; a deliberately 3-character
  stash colour on the coverage tab. The `--coverage` sweep covers the
  optional fields enumerated by `coverage_axes()` — a list maintained
  against `item.h`, the app's accumulated catalog of everything GGG
  has ever sent — including Standard-only survivals (crucible,
  scourge, logbooks, ultimatum, race rewards) and PoE2 parse fields.
- **Curate what is known.** `quirks-registry.json` is the injection
  point for hand-picked idiosyncrasies: each entry carries a status
  (verified-in-data / documented / reported / community) plus
  provenance under explicit rules — `source` holds durable references
  only (archived URLs with dates, issue numbers, forum thread ids,
  files in this repository); evidence that cannot be linked or
  committed gets an `inspected` record (who looked, when, the exact
  observation, whether it can be rechecked); and entries probing
  tolerance with a shape we hold no captured bytes for are flagged
  `stress_case`, preserving the line between "the application should
  tolerate this" and "GGG is known to have emitted this".
  `--coverage` emits the registry as its own Quirks tab plus one
  extra tab per tab-level quirk (short colours, empty and homoglyph
  names), separate from the systematic sweep. Registry items may
  deliberately carry keys `item.h` does not model (e.g.
  `talismanTier`) — those probe the tolerant reader and the raw-byte
  cache rather than the typed parse. The registry was seeded from a
  research sweep over GGG's developer-docs changelog, archived doc
  snapshots (the 2016-10-15 snapshot embeds a real 4,658-item
  public-stash feed, re-verified locally), Acquisition's own issue
  history, and third-party parser workarounds (PoB, exile-diary,
  Procurement, exilence, poe-custom-elements). New quirks from
  research or from other players' old accounts land here as one JSON
  entry each.
- **Detect what is not.** `quirks.py` scans any real userstore DB for
  keys the `src/poe/types/` headers do not declare, mixed-type keys,
  frame pairings, and colour-length histograms. The surveyed account
  is fully modeled — every key in its 43k items is declared in
  `item.h` — so genuinely unknown shapes must come from older
  accounts; when one surfaces, the scanner turns it into a concrete
  catalog delta instead of a silent parse gap. Pre-3.10 shapes are now
  partly evidenced by the archived 2016–2019 public-stash captures;
  what remains unseen (e.g. the real `prophecyText` wire shape, which
  the docs call `?string` but `item.h` models as an array) is recorded
  in the registry as unresolved rather than guessed at.

## Rejected alternatives

- **Sample-and-mutate from a real account** (built first, retired):
  realistic but structurally capped at the source account's coverage,
  and every emitted item risks residual linkage to real, publicly
  archived items. Survives only as the profile extractor's ancestry.
- **Committing generated datasets**: even anonymized, a dataset is an
  audit burden and a licensing question (the underlying data shapes
  are GGG's); a generator plus a seed reproduces any dataset for free.
- **Trade-API / stash-river harvesting for coverage**: sanctioned
  access is frozen (`service:psapi` applications) and scraping is out
  under ToS; RePoE covers the mod space without it.

## Known limits / future work

- Uniques are rare-mod items with unique `frameType` (RePoE carries no
  unique roll ranges); wrong for unique-specific realism, fine for
  scale and parse coverage. PoB's uniques data could close this later.
- Numeric rolls are per-stat independent; cross-stat correlation
  (hybrid mods) follows the template but not GGG's exact pairing.
- GUI validation is one manual measured run, not a checked-in
  guarantee. On Aug 8 2026 the full GUI was exercised by hand against
  a 100k-item / 8,610-tab synthetic store (measured): login with a
  real OAuth token, `ParseCachedItems` over the synthetic database,
  ~45 minutes of tab browsing, tooltip rendering, mod/name/rarity/tab
  searches, item and tab buyout round trips, short-colour and
  homoglyph tab handling, the currency window, and an RSS observation
  of 639→865 MB across the session. No authenticated *refresh* was
  performed — the network never saw the synthetic data — and none of
  this is asserted by the checked-in suite, which establishes
  repo-layer parsing, per-probe presence, and schema equivalence
  only. Anything beyond that (model, search, tooltip, worker
  behavior over synthetic stores) is future work if the redesign
  needs it asserted repeatably.
- Delta/refresh sequences are out of scope here: `tests/spikedataset.h`
  already owns churn (`ChurnTab`); feeding generated items into it is
  the intended route.
