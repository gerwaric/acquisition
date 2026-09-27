# Evidence manifest — pseudo-stats

Every capture this track reads, with what it is and how it was obtained
(index rule 4). `raw/` is local only and described here; `data/` is
committed. The first pass read no capture of its own: its inputs are
other tracks', named in the README's provenance. Access method:
`browser` (`SURFACES.md`, the trade site's rows; C79).

## How a search is captured

The procedure is the site-sitting skill (`.claude/skills/site-sitting/SKILL.md`):
the searches are `data/search-sheet.csv`, a person opens each link, a sitting
is recorded and split by `tools/trade-split.py`.

A fetch response carries seller accounts and whisper tokens: `raw/`
only, never `data/`. What is committed is an extract, scrubbed, its
guard refusing seller data as `trade-query/scripts/fetch-census.py`'s
does.

## The captures

The pilot, 2026-09-26, 18:28–18:31 US Central, by the owner, from the links
of `data/search-sheet.csv` at `d18fa6dd`, Standard league, PC realm. No request
body was saved: the link holds the query, and the search response returns
it.

| File | Bytes | Captured | sha256 |
| --- | --- | --- | --- |
| `raw/searches/p1-fetch.json` | 64883 | 2026-09-26 18:29 | `1c4ce0d5140cf51fa7170113802a74ca1bfc7c1f8b5b34cb4e022357da5c5c65` |
| `raw/searches/p1-search.json` | 7969 | 2026-09-26 18:28 | `5d1ed7c167f96d68364184961c21d21db7a8a8880453e667861377f7ba4a6fba` |
| `raw/searches/p2-fetch.json` | 69252 | 2026-09-26 18:31 | `9922f9e3df50c565a671bd6041c7c822aca47bac5735f98704f3c0e34e4afe77` |
| `raw/searches/p2-search.json` | 7961 | 2026-09-26 18:31 | `2e3622508623b928ba00e825b97cbe676b100f8f511cd976377b3974fcc16e7a` |

The checks, 2026-09-26, 19:08–19:11 US Central, by the owner, from the links
of `data/search-sheet.csv` at `9166e88b`. c1 found nothing, so it has no fetch:
the file is empty.

| File | Bytes | Captured | sha256 |
| --- | --- | --- | --- |
| `raw/searches/c1-fetch.json` | 0 | 2026-09-26 19:08 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `raw/searches/c1-search.json` | 454 | 2026-09-26 19:08 | `49eddcf0d41ea007320846d97e0a3f25fa3d219e799e89b1ca2825cc0236c3d2` |
| `raw/searches/c2-fetch.json` | 84461 | 2026-09-26 19:09 | `eab9818dcc358a7036722be9b45b780ff70a2ade9f12a79be4e0508939068acd` |
| `raw/searches/c2-search.json` | 8059 | 2026-09-26 19:09 | `5611d7eaf63d727ac5dcaed2f6f84ae361b7906016e218479a45e2cdb93aa692` |
| `raw/searches/c3-fetch.json` | 80584 | 2026-09-26 19:10 | `25261cc007e32c8e1845b3865164f33c92a59c5d06a813024fec13a0f29e8f10` |
| `raw/searches/c3-search.json` | 8078 | 2026-09-26 19:09 | `1963bf1ec60fcc698bcabeea59ed3dcce812a11621eba447628ee5bb10de4a50` |
| `raw/searches/c4-fetch.json` | 94694 | 2026-09-26 19:10 | `750c088cf8b9756204f76aa960e1b9e2f916eebc9a0a3265335733b8de925216` |
| `raw/searches/c4-search.json` | 8140 | 2026-09-26 19:10 | `ef387078c06942545f0914f7f1726ea7b85dd0ff47888c6ea59bab05d2c31dff` |
| `raw/searches/c5-fetch.json` | 103443 | 2026-09-26 19:11 | `2876878489f4f4752cfca6a78aeeb72757b6ed3130cca4913eae3e46878551cf` |
| `raw/searches/c5-search.json` | 7948 | 2026-09-26 19:11 | `36f14d1221c20a644b543e66dc53d43fdcee3bb40021bae4163c042c0bfa7d6a` |
| `raw/searches/c6-fetch.json` | 124471 | 2026-09-26 19:11 | `4e6f2c92bd1ec7948648523f6664f6687c6b108fb2ba346316c553a6be63a3e4` |
| `raw/searches/c6-search.json` | 7980 | 2026-09-26 19:11 | `78d27ea9fa2e8fc27e13a3444e873ff489b1024972a233acdc12e46b5faf4a0a` |

Round two, 2026-09-26, 19:41–19:44 US Central (2026-09-27 00:41–00:44 UTC), by
the owner, from the links of `data/search-sheet.csv` at `dbbdd35f`: one
recording of 26 searches, exported sanitized. It holds no response body in any
of its 2,260 entries and no cookie; its shape is `data/recording-shape.csv`.

| File | Bytes | Captured | sha256 |
| --- | --- | --- | --- |
| `raw/searches/d-searches.har` | 15439357 | 2026-09-26 19:45 | `f4ab149a29aed66e85095546b1043520a136fb55aa1f878687a9c9aeb8a9baa4` |
| `raw/searches/www.pathofexile.com.har` | 114023184 | 2026-09-26 19:57 | `5555bf4a73215d32103d002fe674016b686fdeb21be24af66ed4a018c52112b6` |

The second is the same sitting exported again with its content: the same
2,260 entries and the same 26 searches, 00:41:17–00:44:32 UTC, every body held,
no cookie. `tools/trade-split.py --write` wrote its 46 captures,
`raw/searches/d01-search.json` to `b2-fetch.json`, each named by the row its
query answers; they are the recording's bytes and are not listed apart.

Round three, 2026-09-26, 20:32–20:58 US Central (2026-09-27 01:32–01:58 UTC),
by the owner, from the links of `data/search-sheet.csv` at `ecd682e2`: one
recording of 107 searches, exported with its content, no cookie. The bodies of
its first fifteen searches (e001–e015) were gone from the browser by the
export; two searches were refused, status 400, "Query is too complex" (e047,
e051). `tools/trade-split.py --write` wrote the 90 others' captures.

| File | Bytes | Captured | sha256 |
| --- | --- | --- | --- |
| `raw/searches/www.pathofexile.com-2.har` | 397205793 | 2026-09-26 20:58 | `96965cf10fef27380b7142852133fbe7def44e4159ef49e8b80bb28b099e660b` |

Round three's first fifteen again, and round four, 2026-09-26, by the owner,
from the links of `data/search-sheet.csv` at `41ed6467`: four recordings, each
exported with its content, every body held, no cookie. The third holds
e001–e015, the fourth f001–f025, the fifth f026–f050 with f050 asked twice
(other results the second time; the first is kept), the sixth f051–f073.

| File | Bytes | Captured | sha256 |
| --- | --- | --- | --- |
| `raw/searches/www.pathofexile.com-3.har` | 66052352 | 2026-09-26 21:21 | `5bdb57e760557d17282b944556d6e1db4642a1835ac2d712cf3e77bffeb5774e` |
| `raw/searches/www.pathofexile.com-4.har` | 113918168 | 2026-09-26 21:32 | `51dbf7af58c9ced57292fe1d28bd510935719e02ee804d6e02921467e70a1305` |
| `raw/searches/www.pathofexile.com-5.har` | 116803803 | 2026-09-26 21:35 | `7a5cb7bd0150f2385389bd5a95090d0962fd2e809cdd5e7c0df74e696fb74789` |
| `raw/searches/www.pathofexile.com-6.har` | 104354088 | 2026-09-26 21:42 | `69969e36e4ed9aa7d0fda5243c3d5808fcef668e25046adff05b71d6242b6e40` |

Round five (the plan's step 9c2), 2026-09-27, 09:48–09:55 US Central
(14:48–14:55 UTC), by the owner, from the links of `data/search-sheet.csv` at
`01eb4b7a`: two recordings, each exported with its content, every body held,
no cookie. The seventh holds g001–g025, the eighth g026–g044 and R1.
`tools/trade-split.py --write` wrote their 72 captures.

| File | Bytes | Captured | sha256 |
| --- | --- | --- | --- |
| `raw/searches/www.pathofexile.com-7.har` | 112588130 | 2026-09-27 09:51 | `595cf19c879fcf6a131ce13038c44f599ac4d27d9d72240ae3993073f432030e` |
| `raw/searches/www.pathofexile.com-8.har` | 87933411 | 2026-09-27 09:55 | `cf39b195ede7a2a8fc40e210457d75cc15b515c018631ba951556162530cd551` |
