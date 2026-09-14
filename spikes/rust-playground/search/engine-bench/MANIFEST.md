# engine-bench — `raw/` manifest

`raw/` is gitignored (`search/*/raw/`) and holds the owner's account data. Nothing here is
committed; it dies with the machine. Every file is regenerable from the two item-facts stores.

| File | Date | Source | Captured by | Access method | sha256 | Bytes |
| --- | --- | --- | --- | --- | --- | ---: |
| `spike.db` | 2026-09-13 | `../item-facts/raw/spike-GERWARIC_7694-2026-09-13.db` (facts v7) | agent, owner's approval 2026-09-13 | `sqlite3 'file:…?mode=ro' ".backup"` — never `cp`, the originals are under WAL | `fbae8786d79c67e3d9f277bae3e8ce60382f55eeabc2dc5bdb5f89965bccfda7` | 54,378,496 |
| `cpp.db` | 2026-09-13 | `../item-facts/raw/cpp-userstore-GERWARIC_7694-2026-09-13.db` | same | same | `65f1622c3fa0dedc1229c8b5b778076f1beae2077ce89c7997f3d2b98b7ac0d1` | 57,196,544 |
| `corpus.jsonl` | 2026-09-13 | the two files above | `scripts/build-corpus.py` | local | `70755f6794fce10d8e43601e28633f9e62c7c901c334f66f57a3299d5b4b6086` | 53,569,958 |
| `bench-template-x1.db` | 2026-09-13 | `corpus.jsonl` | `bench/` (`cargo run --release`) | local | — (rebuilt on every run) | 24,911,872 |
| `bench-stat-x1.db` | 2026-09-13 | `corpus.jsonl` | same | local | — (rebuilt on every run) | 15,863,808 |
| `seat-projection.db` | 2026-09-14 | `corpus.jsonl` | `bench/` (`cargo run --release --offline -- --seat`) | local | `beb2ba47f42a238dfa31993254a2cc94e2a4b447b9c497b5ce38c7ab88278b88` | 30,818,304 |

The x10 and x30 databases the run built (254 MB / 163 MB and 765 MB / 490 MB) were deleted
after their numbers were recorded; a rerun rebuilds them. The `.backup` copies left the
originals' bytes untouched — only their `-shm` files, which SQLite touches for a WAL reader.
