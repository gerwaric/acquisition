# Synthetic Data

Status: living topic doc for ADR 0003 exploration. Tooling lives in
`tools/synthdata/`; this document records the design, its rationale,
and rejected alternatives. Provenance: distilled from the synthetic
data brainstorm (Auro, Aug 8, 2026) following the maintainer's Aug 8
Discord note (synthetic data, UX instrumentation, rust CLI).

## Principle

**Commit generators, not data.** The repo carries tooling and
statistics; every dataset is produced locally, parameterized by scale
and seed. Nothing derived from a real account is committed, so the
anonymity question collapses to reviewing a small histogram file
instead of auditing tens of thousands of items.

## Inputs

Two, both account-free:

- **RePoE** (`repoe_data.py`) — the same `https://repoe-fork.github.io`
  source the application fetches (`src/repoe/repoe.cpp`), plus
  `mods.min.json`, which the app does not consume but the generator
  needs for legal mod pools, tier value ranges, and display templates.
  Cached under `tools/synthdata/.cache/<version>/` (gitignored), keyed
  by upstream `version.txt` so a game update invalidates naturally.
- **A profile** (`profile.py`) — statistics extracted from a real
  userstore DB: tab-type mix, items-per-tab quantiles, league shares
  (recorded by rank, never by name), character levels, buyout and
  public-tab density. Counts and histograms only — no ids, names,
  items, or mod values — so a profile is committable where its source
  database never is. `fresh-account.json` (committed) is a small
  new-player account; a veteran-account profile can be added later if
  the maintainer wants his account's shape represented.

## Generation (`generate.py`)

- **Mods are RePoE-legal**: prefix/suffix pools filtered by
  spawn-weight tags against the base item's tags, group exclusivity
  respected, values rolled inside real tier ranges, display text
  instantiated from the mod's own template. Emitted in the 3.29 wire
  shape: `ItemMod{description, flags}` objects for implicit/explicit
  (multi-line text stays one description with `\r\n`), plain strings
  for enchant/utility/rune arrays, per `src/poe/types/item.h`.
- **Structure matches the live API**: Map/Unique stashes are parent
  rows plus child rows, with `children` metadata in the parent payload
  (F49); items shelf-pack into real grid bounds; stackables use base
  `stack_size`; icons use the `web.poecdn.com/image/<dds path>.png`
  form of RePoE `visual_identity`, which resolves publicly.
- **`--coverage`** appends a tab sweeping schema axes from `item.h`:
  each influence singly and paired, corruption states,
  fracture/synthesis (as mod flags), foil, relic, scourge, crucible,
  and the PoE2-only parse fields (`runeMods`, `gemSockets`,
  `sanctified`, `doubleCorrupted`, `unidentifiedTier`, `desecrated`).
  PoE2 fidelity deliberately matches the app: parse-level only, realm
  flows unchanged (F63's deferral stands).
- **Deterministic** per (seed, profile, RePoE version). 100k items ≈
  43 s / 65 MB; ~2M extrapolates to ≈14 min / 1.3 GB. Scale is a
  parameter, so the webview-scale spike's target is a sweep, not a
  decision.

## Validation

`tests/tst_synthdata.cpp` opens a generated database through the
application's own `UserStore`/repo layer and requires every stash and
character payload to parse into the typed `poe::` structs — the same
path `ItemsManagerWorker::ParseCachedItems` uses. It skips unless
`ACQ_SYNTH_DATA_DIR`/`ACQ_SYNTH_ACCOUNT` point at a database, so the
regular suite is unaffected. This test caught two wire-format facts on
its first run (mods-as-objects; crafted/fractured as flags rather than
arrays), which is the argument for keeping it: the generator tracks
the app's parse layer, not the other way around.

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
  stash colour on the coverage tab. The `--coverage` sweep now
  enumerates essentially every optional field in `item.h` — the app's
  accumulated catalog of everything GGG has ever sent — rather than a
  hand-picked subset, including Standard-only survivals (crucible,
  scourge, logbooks, ultimatum, race rewards) and PoE2 parse fields.
- **Curate what is known.** `quirks-registry.json` is the injection
  point for hand-picked idiosyncrasies: each entry carries provenance
  and a status (verified-in-data / documented / reported / community),
  and `--coverage` emits the registry as its own Quirks tab plus one
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
- Loading is proven at the datastore/parse layer; a logged-in GUI
  session over a synthetic store has not been exercised.
- Delta/refresh sequences are out of scope here: `tests/spikedataset.h`
  already owns churn (`ChurnTab`); feeding generated items into it is
  the intended route.
