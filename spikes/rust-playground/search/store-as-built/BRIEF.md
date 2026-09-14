# store-as-built — brief for the first-pass run (2026-09-13)

## Rules that bind this run

- Read, in order: `search/README.md` (rules in force, traps), this track's `README.md` and `MANIFEST.md` if present, then only the inputs named below. Do not read the other tracks' READMEs except where an input points there.
- Work only inside this track's directory. Never modify `search/README.md`, any other track, `CONTEXT.md`, `decisions/`, or `SURFACES.md`. Never commit, push, fetch from the network, or spawn agents.
- Absolute paths under `search/`; never `cd` in a compound command. Extracts are strict JSON or CSV with `lineterminator="\n"`; every `data/` file's first line names the script and the inputs; `raw/` is never committed.
- Every finding traces to a `data/` file or a cited path at a commit. Nothing from memory of the app, the game, the site or the tool: a fact you cannot point at is marked open.
- Write the README last, from the script outputs, and its headline block last of all: a status line (`Status: first pass complete — 2026-09-13`), at most five headline bullets, findings as tables or short paragraphs headed `**F<n> — …**`, numbers in one table, every open question naming the one read or experiment that closes it, Provenance, an empty Review table. Budget about 12 KB; per-item detail goes to `data/`.
- Finish with a report of at most 300 words: what landed, what is open, what surprised you, what you could not do. This brief is deleted at the track's close.

## This track

Inputs: `crates/acquisition-store/src/schema.sql` (the schema), `lib.rs` (the module doc, "As built"), `snapshot.rs`, `index.rs`, `annotations.rs`, `world.rs`; the tests that pin the read surface (`crates/acquisition-store/tests/`, and the frontends' reads: grep `acquisition_store::` under `crates/acquisition-cli`, `acquisition-mcp`, `acquisition-plan`); `decisions/store.md` and CONTEXT's C12, C34, C48; `search/item-facts/data/field-census.csv`.

Outputs:
- `data/read-surface.md` — one table: every public read type and function a frontend can call, what it returns, what it is keyed on (id, realm, league, container), its cost class (one row, a table scan, a JSON parse per item), and which frontend calls it today.
- `data/columns-vs-json.csv`, by `scripts/columns-vs-json.py` reading `schema.sql` and `field-census.csv`: every column of `items` (and the tables around it) beside the JSON path it is lifted from; then every JSON path with its item share and no column.
- Findings: what a search consumer can do through the surface as it stands (filter, group, locate, freshness, events); the first thing it cannot without a store change, and where that change would fall (a column, a derived table under C34, a new read); what the module doc already rules so the README points at it rather than restating it.

Acceptance: a reader who has never opened the store crate can say, from this README and its data, what a search over the store gets for free and what it must build.
