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
3. A search with no results has no fetch: save the search response
   alone. A link the site refuses is a finding: note what the page
   showed.
4. One link at a time, at the pace of reading each result.

A fetch response carries seller accounts and whisper tokens: `raw/`
only, never `data/`. What is committed is an extract, scrubbed, its
guard refusing seller data as `trade-query/scripts/fetch-census.py`'s
does.

## The captures

| File | Bytes | Captured | sha256 |
| --- | --- | --- | --- |
