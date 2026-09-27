# 34 — The record's 9c rows, drafted, and the room they need

Written 2026-09-26 at the end of the session that ran step 9c, for the
session that writes the record. The owner, on why it is a session of
its own: "This session is getting dangerously full, so rather than work
on a new task (the record budget), what if you write everything you'd
like to add, then we can start a new session laser focused on
addressing the record?"

This note is a draft and a count, never an authority. It is deleted in
the commit that writes the record, cited as "note 34 at `<commit>`".

## The task

`SEARCH-SLICE.md` owes step 9c its entries — a ledger row, a line of
holes ruled, a verdict line for the measurement, the observations still
open — and has no room: it stands at **34,972 of 35,000 bytes**
(`tools/docs-check.sh`, `budget SEARCH-SLICE.md 35000`). What leaves it
to make room is the owner's to rule, from the counts below. Never trim
words to reach a number: a budget reached is a prompt to route a fact
to its home (`CONTEXT.md`, "Decisions — the registry").

Step 9c is otherwise built, routed and green. Nothing here asks for
code.

## What to read, in order

1. `AGENTS.md`, `README.md`, `CONTEXT.md`.
2. `SEARCH-SLICE.md` whole: it is the thing being edited.
3. `search/BUILD-PLAN.md`, the rows 9c, 9c2, 9d and the foot (T3).
4. `.claude/skills/session-close/SKILL.md`.
5. `search/pseudo-stats/README.md`, only to check a claim below.

`git log 40383f19..HEAD` is the step's story, sixteen commits from
`6362255a` to `6b033c60` and then this note's.

## What 9c adds to the record — 1,826 bytes as drafted

Measured by script over these four drafts as they stood before the
owner's two last rulings were added to the holes line; measure again
after any edit.

**The step ledger, one row:**

> | 9c · the trade site's computed values | `6362255a`–`6b033c60` | A second pass of `search/pseudo-stats/`: 212 searches the owner ran on composed links, recorded and split by the query each response carries; the rows under test by version (`tools/trade_rows.py`), every capture read against them, 912 readings; `data/table-changes.csv`, 227 changes a row. Built: totals v2, 36 totals and 155 rows, and a total of nothing lacked with `has:` on a total (`tests/pseudo.rs`: `v6_…`, the two `c94_…`, seen to fail first). 33 of 57 pseudos closed, 24 explained; the ranged family's 21 totals wait for 9d's category. |

**Holes ruled, one line** (the owner's words verbatim, each from this
session):

> | 9c | a total of nothing is lacked and `has:` asks its presence, T2's "every item has a total" taken back ("A"); a row's eldritch forms are the row ("include the eldritch mods"); what the site leaves out and the search counts: the twin on That Which Was Taken ("I believe this is a bug. Let's count the mod"), the skill gems' own text ("yes, include the socketed skill gems"); a ranged total asked with no slot word is its `avg` ("yes, bare means average"), built at 9d; composed links ("yes"); the site's other totals at launch | `totals.rs`, `pseudo.rs`; the reference, *A sum's status*; `totals-v2.toml`; `SURFACES.md`; the plan, 9c2 and 9d | `37faf903`, `e81cc8ca`, `6362255a` |

**What the measurements taught, one line:**

> - M3 at totals v2 (`e81cc8ca`): every ask 445–488 but the totals' two, 579 and 578, 50 and 55 more than at 9b: a total's cost is its rows, 155 where there were 104.

**Observations still open, a block:**

> **Step 9c.**
>
> - A `not` on an eldritch form's id lets some items through: 24 pseudos are explained, never closed (`search/pseudo-stats/README.md`).
> - What the search counts and the site leaves out is said in the table's changes and nowhere a user reads; the translation (C99) is where it bites.
> - The owner reported the twin id to GGG (2026-09-26); it is left out of total Strength and Intelligence as of total life, and a fix on the site moves three `not mimicked` rows to agreement.

## What is revised in place — no growth to speak of

- **Holes ruled, step 7, T2.** It reads "T2 `has:` applies to a derived
  field, never to a total — `-has:pseudo.dps` routes the lacked count,
  `has:pseudo.total_res` an error with readings", and names
  `tree::has_on_computed`, which `37faf903` removed. T1 stands. T2
  becomes: `has:` asks a derived field's presence, and a total's since
  9c.
- **The first seat's observation** "After 9b every `class:` term
  carries 158 undecided…" stands: 9d is not built.
- **The findings' checklist** may gain one occurrence under "A claim
  the code did not make": two commit messages of 9c claimed what a
  failed step had not shown (`f304cf1a`, `e1937bff` amended to
  `dbbdd35f`), and counts were written from the screen. Their story is
  the commits'; whether the checklist names them is the writer's call.

## What could leave — measured 2026-09-26 at `784852f7`

| What | Bytes | Where it would live | Cost |
| --- | ---: | --- | --- |
| The step 7 observation on `total_life` ("ships in no table until one trade search per candidate line closes it") | 441 | nowhere: 9c closed it, and the table and `v6_…` hold it | none |
| "How the design was reached" | 1,545 | the record at its commit, two lines left pointing there | notes 22, 24–28 and 31 may lose their only citation by path and be reported uncited: check what else cites each before moving it |
| The ledger's rows for steps 1 to 8, ten rows | 5,917 | the record at its commit, a line each or a range | the largest; makes room for 9d, 10 and 11 as well |
| The first seat's evidence on category and the variant field | 544 | the plan's 9d row, which is at 27,729 of 28,000 itself | moves the problem |
| Raising the budget | — | `tools/docs-check.sh` | the owner's: the slice has gained three steps since the number was set |

The first alone is not enough. The first two free 1,986, which holds 9c
and nothing after it. The first and the third free 6,358.

The session that ran 9c recommended the first and the third. It is a
recommendation, and no ruling.

## What comes after the record

In the owner's order (the plan):

1. **9c2, the site's other totals** — the sum-like pseudos 9c left
   `unresolved`: "They should be present at launch." By the site-sitting
   skill; it needs the owner's sittings.
2. **9d** — category, the percentile (the site's own rule, pinned on 30
   captured items; eight cases no capture reaches are the sheet's
   C1–C8 under `tools/trade-sheet.py --method if`), and the ranged
   family's 21 totals, whose 52 twin rows need "is a weapon" from
   category. A bare ranged total means its `avg`.
3. **Step 10**, `--explain` first.

Standing beside them: nine candidate claims for
`docs/design/trade-ground-truth.md`, authored master-side
(`search/pseudo-stats/README.md`, "Candidate claims"); the totals'
asks at 579 ms against a budget of 500, the totals batch parked until
step 11 (V9), and more rows coming.

## Checks the record's commit runs

Each bare, its exit read: `tools/docs-check.sh`, `git diff --check`,
`wc -c SEARCH-SLICE.md` after the last edit. No crate is touched, so
the cargo gate is not owed; if any file under `crates/` moves, it is.
