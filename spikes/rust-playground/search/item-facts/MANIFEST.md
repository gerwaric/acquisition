# Evidence manifest — item-facts

Every file this track read, with what it is and how it was obtained (index rule 4). `raw/` files are local only, gitignored, and described here; `data/` files are committed and regenerable from them. Both stores hold the owner's own account (`GERWARIC#7694`); they are account data and are never committed. Access method: a local file on the owner's machine, copied with `sqlite3 .backup` on 2026-09-13.

| File | Bin | Bytes | Source | sha256 |
| --- | --- | --- | --- | --- |
| `raw/spike-GERWARIC_7694-2026-09-13.db` | raw | 54378496 | `~/Library/Application Support/gerwaric.acquisition-playground/store/ggg/GERWARIC_7694.db` — the spike's facts store (facts v7), items seen 2026-09-02..11 | `fbae8786d79c67e3d9f277bae3e8ce60382f55eeabc2dc5bdb5f89965bccfda7` |
| `raw/cpp-userstore-GERWARIC_7694-2026-09-13.db` | raw | 57196544 | `~/Library/Application Support/Acquisition/data/userstore-GERWARIC#7694.db` — the C++ app's store (master `946a4f51`, `src/datastore/stashrepo.cpp` binds the response bytes verbatim); Standard fetched 2026-08-13, Mirage 2026-07-20, Allflame / Hardcore / Solo Self-Found refreshed by the owner 2026-09-13 with maps and uniques | `65f1622c3fa0dedc1229c8b5b778076f1beae2077ce89c7997f3d2b98b7ac0d1` |
| `../trade-query/data/items-2026-09-12.json` | data (trade-query) | 344615 | the trade site's base-type taxonomy; its row is `../trade-query/MANIFEST.md` | `89b722ada424665e5d714cc431c504b00a936d2d19df5f21b6ba24109ac74ead` |
| the C++ app's backup stores, `~/Library/Application Support/Acquisition/data-backup-*/userstore-GERWARIC#7694.db` (30 files, 2026-06..09) | read in place, not copied | — | `scripts/line-format-dates.sh` reads them read-only to date the mod-line format switch; nothing else reads them | — |

Declined: `~/Library/Application Support/gerwaric.acquisition-playground/store/ggg/_vagabond_6960.db` (a second account, pre-realm schema, one character response — nothing the census needs).
