# 15 — The site as the oracle (plan step 7, item 5): the session's brief

Deliberation for the session that runs item 5 of the pricing test pass
(`PRICING-SLICE.md`, "Plan", step 7). Disposable: what it settles goes
to the record's ledger and observations, to claims master-side, or to a
ruling; nothing here is a second authority. Written 2026-09-08 from a
first look at one saved page and the join against the owner's facts.

## What item 5 is for

The listing state claims, per item, "the site lists this at price P" or
"the site does not list this" (C69, C81). The render acts on the claim:
it omits what the site lists and must never post a price the site
contradicts (C74). So the oracle question is narrow: **over the items
both sides can see, where do the two claims differ, and does any
difference change what a page would do?** The line between completeness
and over-complexity sits exactly there — a difference earns a rule only
if it flips a render verdict for some item; otherwise it is an
observation with a trigger.

## The inputs

- The owner's captures: the trade site's seller-account search for the
  account in Standard, saved as rendered DOM (console
  `copy(document.documentElement.outerHTML)`) in several parts under
  `runs/site/standard-trade-listings-2026-09-08/` (gitignored). The
  site caps one search at 100 rows and said "560 matched" on
  2026-09-07; the parts are the owner's way past the cap.
- One result row carries: `data-id` (the item's 64-hex id — the same id
  the stash API gives, so the join is exact), name, base, mods, the raw
  note, the price label (`Exact Price:` / `Asking Price:` / `No Price
  Set` / none), amount and the currency's display name, the account,
  a verified flag, "listed N ago". No tab name.
- The facts: the owner's store, the ggg provider directory. The
  full-account Standard refresh ran on 2026-09-08 (run ledger,
  `LIVE-TESTING.md`): every top-level and folder-child tab fetched
  (384), 41 characters, 19,828 live items in tabs. **The domain is the
  top-level tabs.** The 2,945 substash stubs under the 40 map and unique
  tabs were planned and declined (~10 h of limiter holds; the site
  cannot link a substash item, Q3); they stay unfetched until something
  needs them, and a site row from one of them is "outside the domain".

## The method

1. **Normalize first.** One read-only tool under `tools/` in the mold
   of `notes-check.py`: a directory of saved pages in, one JSON table
   out (id, name, base, note, label, amount, currency word, listed-age,
   verified, source file), deduped by id across files, with each file's
   own "N results (M matched)" line reported so the union's coverage of
   the matched count is a number. Commit the table beside
   `website-tabs-2026-09-07.json`; the raw pages stay in `runs/`.
2. **Our side as the same kind of table.** A sibling of
   `examples/redact-pricing.rs`: an example binary that opens a store
   directory the way every frontend read does and prints the listing
   report JSON, unredacted, to a local file — no daemon, no network,
   `ACQ_GGG` untouched (the standing rule keeps it for live runs only).
3. **Join by item id; classify under a closed list.** Three sets —
   agree, site-only, state-only — plus "outside the domain" for site
   rows in tabs the store has not fetched. Each disagreement takes one
   reason from a list fixed before the join: tab not fetched; tab public
   on the site but not in facts; note read differently; amount
   displayed differently; currency word; listed long ago and never
   re-verified; unlisted since. A row fitting no reason is the
   interesting kind; there should be few.
4. **One verdict per reason, written before any code.** Either it
   changes a render verdict — then a ruling amendment or a claim with a
   test — or it does not — then an open observation with a trigger.
5. **Stop when the classification is complete**, not when every row is
   explained. Four years of listings hold oddities no rule should be
   built for.

## What the first hundred rows already showed (2026-09-07)

- 60 Exact, 10 Asking, 29 No Price Set, 1 with no price block at all
  (the `~price 777 chaos testing` note: listed, no price — C69's "no
  effect", displayed unlike the site's own No Price Set).
- Every currency word the site read is a word our table carries as the
  game's emit word (`scour`, `gcp`, `annul`, `grand-ember`, …): no loose
  match was needed for any of them.
- 28 of the 100 were in the store, all in the test tab; 72 were in tabs
  never fetched. Hence the refresh first.
- **The `~price 8888 chaos` tab** is not public in the facts (listed
  2026-09-04, no `public` in its metadata, never fetched), yet the site
  lists its items at 8888 chaos, "listed 4 days ago". Either the flag
  was set after the listing, or `metadata.public` is not the whole
  condition T1 and C81 rest on. The owner knows which; it is the one
  finding that could change a verdict.
- **The site drops the fraction on chaos**: `999.1234`, `999.123`,
  `999.12`, `999.1` all read "Exact Price: 999"; `1.5 divine` keeps its
  fraction. A display fact, currency-dependent, that changes no verdict
  — a claim once the rule is understood.
- Tab-name prices from unfetched tabs: six at 4321 blessed, one at 2222
  jeweller's, one at 55555 chromatic, all Asking with no note — the
  census's 4321 rows, alive on the site. Coverage, not rules.
- 29 No Price Set rows, mostly "listed 4 years ago": a public tab's
  unpriced items sit on the site indefinitely. The render already keeps
  such items off the page (nothing applies); the state only counts what
  is fetched.

## Opening line for the session

    Pricing slice, plan step 7, item 5: the site as the oracle. Start
    from brainstorming-notes/15-site-as-the-oracle.md, then
    PRICING-SLICE.md's State bullet and the 7.2–7.4 ledger rows.
