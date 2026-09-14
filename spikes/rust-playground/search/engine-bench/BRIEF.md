# engine-bench — brief for the seat projection (2026-09-14)

A second, bounded run on this track, on the owner's call (2026-09-14):
extend the bench's projection with what existing rulings already
require, so that agent-seat's phase two measures SQL as an interface
rather than the bench's known omissions. **Nothing speculative goes
in** — the seat's job is to find what is still missing. Committed
before the run, deleted at its close, cited by hash.

## Rules that bind this run

- Read `search/README.md`, this track's `README.md`, `bench/src/main.rs`
  and `scripts/build-corpus.py`. Nothing else is needed.
- Work only inside `search/engine-bench/`. Never modify `search/README.md`,
  another track, the workspace `Cargo.toml`, `CONTEXT.md`, `decisions/`
  or `SURFACES.md`. Never commit, push, fetch, or spawn agents.
- **Do not touch the timed schema or rerun the timings.** The README's
  numbers stay reproducible from the crate as it is: add a separate
  mode that writes a separate file, and leave `build_db` and the twelve
  queries alone.
- The corpus is the owner's account data (`raw/`, gitignored); the new
  file lives beside it and is never committed. Absolute paths; never
  `cd` in a compound command.
- Finish with a report of at most 200 words.

## What to build

A `--seat` mode of the bench binary (`cargo run --release --offline -- --seat`)
that reads `raw/corpus.jsonl` and writes `raw/seat-projection.db`, real
scale only, no clones, with this schema and no more:

- `items` — the bench's columns as they are (keep `pretty`, `base`,
  `frame`, the requirements, the defences, `quality`, `identified`,
  `corrupted`, `ilvl`), plus, verbatim as GGG sends them: `name`,
  `type_line`, `base_type`, `rarity`, `frame_type_id`; and the location
  coordinate as the corpus carries it: `realm`, `league`,
  `location_kind`, `location_id`, `tab_name`, `container`,
  `socketed_in`, `last_seen` (the corpus's `fetched`; there is no
  first-seen in it — say so in the DDL comment). `gid` stays the GGG id.
- `lines` — the bench's `item`, `arr`, `line` (the template) and
  `value` (the mean, kept so the timed queries' meaning is unchanged),
  plus `kind` — the C++ app's bucket derived from the array and the
  line's flags: `implicit`, `explicit`, `crafted`, `fractured`,
  `mutated`, `enchant`, or the array name for the arrays it never read
  — `flags` (the line's GGG flags as a comma list, empty when none),
  `n` (how many numbers the line carries), `n0` and `n1` (the first
  two, NULL when absent — the owner's ruling that min and max stay
  separate), and `numbers` (every number, as a JSON array text).
- Indexes: the same shapes the timed schema has, no more.
- Every `CREATE TABLE` carries `--` comments naming what each column is
  and where it comes from; SQLite keeps them in `sqlite_master`, so an
  agent reading the DDL reads them too. Keep them to one short clause
  per column.

Then: `scripts/seat-projection-check.py` (read-only, `?mode=ro`) that
prints the DDL, the row counts, the `kind` histogram, and the count of
lines with `n >= 2`, into `data/seat-projection.md` (first line names
the script and the input). A `MANIFEST.md` row for the new file (bytes,
sha256, how it is regenerated). One short README addition — `**F7 —
The seat's projection**`: what was added, that it holds ruled facts
only, its size — folding elsewhere so the README stays about 12.5 KB.

## Acceptance

`sqlite3 -readonly raw/seat-projection.db .schema` shows the two
tables with their comments; the twelve timed queries still run
unchanged against the timed files; the owner's first question — a base
with a specific fractured modifier — is expressible as one SQL
statement against the new file.
