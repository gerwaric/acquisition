# Evidence manifest — pseudo-stats

Every capture this track reads, with what it is and how it was obtained
(index rule 4). `raw/` is local only and described here; `data/` is
committed. The first pass read no capture of its own: its inputs are
other tracks', named in the README's provenance. Access method:
`browser` (`SURFACES.md`, the trade site's rows; C79).

## How a search is captured

The searches are `data/search-sheet.csv`, written by
`scripts/search-sheet.py`: each row is a query, what it decides, its
control, and a link composed from the query. A person opens the link;
no tool does.

1. Signed in, with the browser's network panel open before the link is
   loaded.
2. Open one link. Save the response of the call to `/api/trade/search/`
   as `raw/searches/<search>-search.json`, and the response of the
   first call to `/api/trade/fetch/` as
   `raw/searches/<search>-fetch.json`. If the page sent a request body,
   save it as `raw/searches/<search>-request.json`; the link already
   holds the query, so its absence loses nothing.
3. The first ten items are enough unless the sheet's row says otherwise.
   Where more are wanted, scroll once or twice and save each further
   fetch as `raw/searches/<search>-fetch-2.json`, `-3`.
4. A search with no results has no fetch: save the search response
   alone. A link the site refuses is a finding: note what the page
   showed.
5. One link at a time, at the pace of reading each result.

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
