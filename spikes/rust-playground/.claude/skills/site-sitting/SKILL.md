---
name: site-sitting
description: Ask the trade site what one of its computed values counts — write the searches from the rows under test, have the owner run them in a browser and record the sitting, split and read the captures, and bring the table's changes as rows. Use before composing any trade search, and before reading any capture of one. Tooling sends nothing; a person opens every link.
---

# Site sitting

The trade site publishes a pseudo's name and never its definition: the
definition is what the site answers when asked (C106's admission test,
clause (a); V6). A sitting asks it. Ran as one pilot, one round of
checks and three recorded rounds on 2026-09-26 — 212 searches — and met
every trap below, so it is a skill (P6). What it found is
`search/pseudo-stats/README.md`; the rounds' story is the commits'.

The boundary is `SURFACES.md`, the trade site's rows (C79): the access
method is `browser`. A script composes a link and lays links out on a
page; the owner opens each by a click. No tool opens a link, on a timer
or otherwise, sends a request, or holds a session.

## The question a search asks

A pseudo's rows are a claim about every item, so a search asks the site
of every listed item at once:

- **complete** — the pseudo required, every id of every row in a `not`
  group: is there an item showing the pseudo while carrying none of its
  rows? What is found names what is missing.
- **sound** — a row required, the pseudo in a `not` group: is there an
  item carrying a row while showing no pseudo? What is found names a
  row, a twin or an id the site does not count.

A search that finds nothing proves nothing until the same search, with
one row changed, finds something: a new form of search has its mutant
(`c2`, `c4`, `d02`). A search of one line at a time reads ten items and
is not this: part 1 of 9c counted 264 of them and none was run.

## Before the sitting

1. The rows under test are `tools/trade_rows.py`: each pseudo's first
   version is the table as shipped then
   (`search/pseudo-stats/data/totals-v1.toml`) or its own text, each
   later version one change with the capture that asked for it.
   **Append a version; never edit one.** A search already captured was
   composed from the version it names.
2. A round is a function in `tools/trade-sheet.py` that **pins each
   pseudo's version by number, written out**. A pin computed from how
   many versions a pseudo has moves the day it gains one.
3. `tools/trade-sheet.py` writes `search/pseudo-stats/data/search-sheet.csv`
   and prints each link beside its name. Then, each run bare:
   `--verify` (every link decodes to its query, every search names a
   control), `--self-test` (the composer reproduces the site's own ids),
   and `cmp` of the sheet's earlier rows against what they were.
4. `tools/trade-pages.py <round>` writes the round's pages, 25 links
   each, under `search/pseudo-stats/raw/sitting/`: the rows with no
   capture yet, so the same command after a sitting writes what is
   still missing. It checks each page against the sheet as it writes;
   `--check` does so again.
5. Tell the owner how many searches, which are the core, the pace, and
   the first page's path.

## The sitting (the owner's)

Signed in. Open the round's first page, `raw/sitting/<round>-1.html`.
Its first link opens the sitting tab, blank: drag that tab to a window
of its own, open its network panel there, keeping its log across pages.
Then one search at a time, a plain click each, at the pace the page
counts. When a page's links are opened, export the panel as a HAR file
**with its content** into `search/pseudo-stats/raw/searches/`, clear the
log, and follow the page's link to the next; the sitting tab stays as it
is. Order does not matter; a link opened twice does no harm.

Without the pages, the sheet's links go into the tab's address bar one
at a time, and the export is every twenty-five links.

## After the sitting

6. `tools/trade-split.py <file.har>` says what the recording holds and
   writes nothing; `--write` writes each capture under the name of the
   row its query answers. The recording stays under `raw/`, never
   committed; its row goes in the track's `MANIFEST.md`.
7. `tools/trade-captures.py --check`, then without the flag:
   `data/captures.json`, scrubbed, its guard refusing seller data.
8. `tools/trade-evidence.py`: every capture against each pseudo's
   latest rows. **An item that disagrees is the finding**: read its
   lines, append the version the capture asks for, read again.
9. When it agrees, break it: a mutant for each rule the rows gained,
   each seen to disagree. Agreement after the rows were fitted to the
   items is a fit; the next round's searches are the test.
10. `tools/trade-changes.py`: `data/table-changes.csv`, the totals
    table's changes as rows, none applied. They go to the owner.
    `tools/totals-table.py` applies what he rules.

## Traps

- **A sanitized export holds no response body.** Export with content.
  The splitter reads bodies and the site's rate-limit headers and
  nothing else, and says whether the file holds a cookie.
- **The browser keeps only so much.** Of 107 searches in one recording
  the first fifteen had lost their bodies. What is left is the
  recording's shape (`--shape`): a small answer no fetch followed found
  nothing, by the page's behaviour and not the site's word.
- **A recording's size is the page's, never the search's**: 4 to 5 MB
  a search, most of it the site's four data files, stored again at
  every page; the search and its fetch are under a hundredth. A page of
  25 is 105 to 113 MB however its links are opened.
- **A page reaches only a tab it opened.** A tab opened by hand cannot
  be a link's target, so the page opens the tab and the panel is set up
  after. A link that opens a new tab has lost the first: a click with a
  key held, or a page on the way that cut the tab from its opener — the
  search page sends no such header. Then the link goes into the sitting
  tab's address bar.
- **The owner creates empty files before filling them.** Mid-sitting,
  `--check` only; a capture saved under another row's name is named and
  left out.
- **A weighted group is refused past some cost**: 56 and 4 an id, taken
  at 148, refused at 220, status 400, "Query is too complex". Ask it in
  parts of twenty ids. A count or a `not` of 71 ids was taken.
- **A part finds what the whole would not**: items whose lines cancel
  across parts. A `count` of rows finds every item whose lines cancel
  at all (c3: 3,338). A total whose lines carry a sign is asked sound
  by the site's own sum.
- **A `not` on an eldritch form's id lets some items through.** A
  complete check that finds only items carrying an id it excluded has
  found nothing new; the pseudo is explained, never closed.
- **One id can display two texts, and one text two ids.** The text
  decides what a row is; an id is left out of the rows only on items
  that display the row's own text under it.
- **The pace.** The site allows 15 searches in 60 s and 60 in 300 s an
  address; a sitting reached 13 and 44. One link every ten seconds is
  half of it. No response has asked a sitting to slow down; the
  splitter says so loudly if one does.
- **The site rewrites an id on the way back**, dropping every
  `"disabled":false`: compare queries with those removed, never ids.
- **A number in a document is measured by a script after the last
  edit**, never read off the screen or recalled: counts, times, ranges.
- **A command that commits stops at the first error**: `set -e` on its
  own line, the check's output read before the commit.
