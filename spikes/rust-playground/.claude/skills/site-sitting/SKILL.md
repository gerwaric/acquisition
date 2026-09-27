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
method is `browser`. A script composes a link; the owner opens it. No
tool opens a link, sends a request, or holds a session.

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
4. Tell the owner how many searches, which are the core, and the pace.

## The sitting (the owner's)

Signed in, the browser's network panel open before the first link and
keeping its log across pages. One link at a time, at the pace of
reading a result. Export the panel as a HAR file **with its content**,
into `search/pseudo-stats/raw/searches/`, **every forty links or so**.
Order does not matter; a link opened twice does no harm.

## After the sitting

5. `tools/trade-split.py <file.har>` says what the recording holds and
   writes nothing; `--write` writes each capture under the name of the
   row its query answers. The recording stays under `raw/`, never
   committed; its row goes in the track's `MANIFEST.md`.
6. `tools/trade-captures.py --check`, then without the flag:
   `data/captures.json`, scrubbed, its guard refusing seller data.
7. `tools/trade-evidence.py`: every capture against each pseudo's
   latest rows. **An item that disagrees is the finding**: read its
   lines, append the version the capture asks for, read again.
8. When it agrees, break it: a mutant for each rule the rows gained,
   each seen to disagree. Agreement after the rows were fitted to the
   items is a fit; the next round's searches are the test.
9. `tools/trade-changes.py`: `data/table-changes.csv`, the totals
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
