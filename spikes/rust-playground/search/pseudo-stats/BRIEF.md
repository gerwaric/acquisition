# Brief — step 9c2, the site's other totals

Run by a session, the sittings by the owner; no subagent, since every
round waits on what the site answered in the one before. The procedure
is the site-sitting skill and is not restated here. This brief holds
what the skill does not: the scope, what a first version is, what comes
to the owner, and what the step closes on. The owner read it before
any search was composed; his answers are at its foot. It is deleted at
the step's close and cited by hash.

## The question

For each pseudo in scope: which lines does the site count under it, at
what weight? The site publishes the name and never the definition (C106,
the admission test, clause (a)); the definition is what it answers when
asked, complete and sound. Owner, 2026-09-26: "They should be present
at launch."

## Scope — fixed by script

`scripts/other-totals.py` writes `data/other-totals.csv`, one row for
each of the site's 298 pseudo stats, by three reads and no list of
names:

| Stands | Pseudos | By |
| --- | ---: | --- |
| under test | 57 | `tools/trade_rows.py` holds its rows: 9c's, the ranged family's 21 among them (9d) |
| classed | 138 | the first pass gave it a class other than `unresolved` |
| another read | 74 | `unresolved`, and the first pass names a capture of a logbook or a tablet, or the export's base ranges |
| **in scope** | **29** | `unresolved`, not under test, and the first pass names a search read against the item's lines |

The 29, as the site writes them. *Ids* is how many stats display the
pseudo's own text, *local* how many of those the site marks `(Local)`,
*eldritch* how many display it behind one of the two presence prefixes,
*items* how many of the owner's items carry the text.

| Pseudo | Ids | Local | Eldritch | Items |
| --- | ---: | ---: | ---: | ---: |
| `# total Resistances` | 0 | — | — | — |
| `# total Elemental Resistances` | 0 | — | — | — |
| `+#% total to all Elemental Resistances` | 0 | — | — | — |
| `+# total to all Attributes` | 0 | — | — | — |
| `+# total maximum Mana` | 0 | — | — | — |
| `+# total maximum Energy Shield` | 0 | — | — | — |
| `#% total increased maximum Energy Shield` | 0 | — | — | — |
| `#% increased Movement Speed` | 6 | 0 | 2 | 1252 |
| `+#% Global Critical Strike Chance` | 0 | — | — | — |
| `+#% Global Critical Strike Multiplier` | 0 | — | — | — |
| `#% increased Elemental Damage` | 4 | 0 | 0 | 206 |
| `#% increased Lightning Damage` | 6 | 0 | 2 | 237 |
| `#% increased Cold Damage` | 6 | 0 | 2 | 200 |
| `#% increased Fire Damage` | 6 | 0 | 2 | 226 |
| `#% increased Spell Damage` | 6 | 0 | 2 | 718 |
| `#% increased Lightning Spell Damage` | 0 | — | — | — |
| `#% increased Cold Spell Damage` | 0 | — | — | — |
| `#% increased Fire Spell Damage` | 0 | — | — | — |
| `#% increased Lightning Damage with Attack Skills` | 2 | 0 | 0 | 0 |
| `#% increased Cold Damage with Attack Skills` | 2 | 0 | 0 | 13 |
| `#% increased Fire Damage with Attack Skills` | 1 | 0 | 0 | 0 |
| `#% increased Elemental Damage with Attack Skills` | 5 | 0 | 0 | 252 |
| `#% increased Rarity of Items found` | 5 | 0 | 0 | 714 |
| `#% increased Burning Damage` | 3 | 0 | 0 | 54 |
| `# Life Regenerated per Second` | 0 | — | — | — |
| `#% of Life Regenerated per Second` | 0 | — | — | — |
| `#% of Physical Attack Damage Leeched as Life` | 10 | 5 | 0 | 246 |
| `#% of Physical Attack Damage Leeched as Mana` | 10 | 5 | 0 | 154 |
| `#% increased Mana Regeneration Rate` | 5 | 0 | 2 | 531 |

The file is the scope; this table is its print. A pseudo struck here
becomes a row of the script's `LEFT_OUT`, in the owner's words, and the
file is written again. The rule is not edited to fit a cut.

## What a first version is

- **15 pseudos whose own text a line displays:** the first rows are
  that text, under every id that displays it, at weight 1. Nothing
  else: no line a tool names, none that looks likely.
- **14 pseudos whose text no line displays:** no rows. The first search
  asks what carries the pseudo, and the items fetched name the first
  rows. Rows read off ten items are a fit, so these need a second
  round before anything is said of them.
- **The eldritch forms** are ruled ("include the eldritch mods") and
  are a version of their own on every row whose form the site lists,
  as at 9c.

Versions are appended to `tools/trade_rows.py`, never edited; the round
is a function of `tools/trade-sheet.py` that pins each by number.

## What the sittings cost

The first round is 44 searches: a complete and a sound check for each
of the 15, one search for each of the 14. `R1`, written at 9c and never
run, asks whether the two counts count lines or resistances, and makes
45. The skill exports a recording every forty
links or so, so the round is two. How many rounds follow is not known
before the first is read; 9c took four rounds and 212 searches for 57
pseudos.

Standard league, PC realm, as 9c. No tool opens a link (C79).

## What comes to the owner

Each as a row of `data/table-changes.csv`, none applied before he rules:

- every new total and its rows, with the captures that ask for them and
  its status: `closed`, `explained` or `open`;
- what the site leaves out that the rows would count, or counts that
  they would not: whether the search mimics it is his;
- a pseudo whose rows mean one of two stats displaying one text. The
  table cannot say which until 9d, so such a total waits there as the
  ranged family does;
- a pseudo that turns out to be no sum of lines. It is no total row
  (C101) and is brought as a question, not built.

## Closes on

The plan's row: a change a row in `data/table-changes.csv`; the table
regenerated. With it: the crate's tests for the new totals, the
references regenerated, M3 again on the totals' asks, the gate; open
questions 1 and 5 of the track's README closed; the record's
observation "9c2's sittings" removed or ruled; one ledger line; the
plan's row gone; this file deleted.

Not this step's: the ranged family and the percentile (9d), the 74
pseudos of another read, the trade translation (C99).

## Settled by the owner, 2026-09-27

1. **The two counts**, `# total Resistances` and `# total Elemental
   Resistances`, asked in or out: "In". They count something and sum
   nothing. If the site counts lines, a row with a weight and no slot
   says it (C94); if it counts resistances, no row can, and they come
   back as a question.
2. **The two leech pseudos**, five of the ten ids behind each a
   `(Local)` twin, asked in or out to 9d: "find out at the sitting".
3. **Whether `explained` is enough at launch**, as at 9c, where 24 of
   57 pseudos were explained and never closed: "Yes".
4. **The order:** "Order doesn't matter".
