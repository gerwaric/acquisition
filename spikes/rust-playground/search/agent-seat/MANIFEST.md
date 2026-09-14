# agent-seat — manifest of `raw/` (gitignored, local only)

Every file under `raw/` is the owner's account data or a derivative of it; nothing here is committed. Copies were made by the owner from a terminal on 2026-09-14 with sqlite's `.backup` (never `cp`: the source is under WAL), per `BRIEF.md`; sha256 is of the copy as read by this run. The source's own sha256 differs from the copy's (`.backup` writes a checkpointed, header-normalized image), so the source digest is recorded beside it.

| Date | File under `raw/` | Source | Captured by | Access method | sha256 (copy) | sha256 (source, same day) |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-09-14 | `world/mock/GERWARIC_7694.db` (54,378,496 B) | `~/Library/Application Support/gerwaric.acquisition-playground/store/ggg/GERWARIC_7694.db` | the owner | `sqlite3 <src> ".backup <copy>"` | `fbae8786d79c67e3d9f277bae3e8ce60382f55eeabc2dc5bdb5f89965bccfda7` | `a2b6d840dc73d7dc83e904a9ffc8b85b7c2a9b2075b3baafdd803ef39c877b1a` |
| 2026-09-14 | `world/mock/cac319d8-e65f-4afa-92be-1de85d620033.annotations.db` (20,480 B) | same directory, same name | the owner | `sqlite3 .backup` | `b6aa3e012673cfa7199796478c52b76ae1cb8d616f44590c8f75b4e899053ccf` | `8407950791c611006ca5c89d95bd20c0ac442bf3d4548f039473dfa60198bb1d` |
| 2026-09-14 | `world/mock/accounts.json` (180 B) | same directory's `accounts.json`, filtered to the `GERWARIC` account | the owner | the python one-liner in `BRIEF.md` | `64b73d5aa4aa0b9f7843c78ec8793e74d84758c97d918fd548c49316055c72c1` | — |
| 2026-09-14 | `logs/` | empty at the start of the run; `ACQ_LOG_DIR` for the run (no daemon ever starts: `ACQ_NO_SPAWN=1`) | this run | — | — | — |
| 2026-09-14 | `calls/NNN.out` | stdout of every logged call (`scripts/call.py`), one file per row of `data/calls.csv` | this run | — | — | — |

Read in phase two but not copied here: `../engine-bench/raw/seat-projection.db` (30,818,304 B, sha256 `beb2ba47f42a238dfa31993254a2cc94e2a4b447b9c497b5ce38c7ab88278b88` at the start of this run; written by engine-bench's crate `--seat` mode, commit 93c3a63b). Opened read-only only (`sqlite3 -readonly`). The annotations file is never opened by SQL.
