#!/usr/bin/env python3
"""site-join.py — the site's rows against the listing state, by item id.

The site as the oracle (pricing slice, plan step 7, item 5;
`brainstorming-notes/15`). Two tables in: the trade site's seller-account
search normalized by `tools/site-listings.py`, and our listing state
written by the `listing-report` example (`acq price list --json`'s
`ListingReport`, unredacted, under `runs/`). One classification out,
under a list of reasons fixed before the join ran — a row fitting none
is `unexplained`, the interesting kind. Read-only; no store, no network.

    tools/site-join.py --site <site-listings.json> --report <listing-report.json> [--rows]

**What is compared.** The site shows the *game* side of a stash item —
its note, else its tab's name — for the items in public tabs (T1), plus
whatever a forum thread links (T1, T12). Our game side is the listing's
`game.reading` where `game.public` is true (C81: a statement only where
the index can see it); a stash row on the site is read against that,
never against our manual side, which reaches the site only through a
forum post. A forum row is read against the manual side instead.

**The four sets.** `agree` — both sides make the same claim (the same
kind, amount and word; "No Price Set" against a public tab's item with
no statement; absent on both when the store says not public or `skip`).
`site_only` — the site lists an id the store says is not listed.
`state_only` — the store says the site lists it and the site has no
row. `differ` — both list it and the claims differ.

**The reasons** (closed; one per disagreement, the first that fits):

- `not_in_store` — the site's id is not in the facts at all: an item of
  an unfetched substash (the 2,945 stubs, declined 2026-09-08), or one
  the store has never seen. Outside the domain.
- `forum_channel` — the site's row is a forum listing (the seller link
  is a thread): intent we hold, never a fact we observe (C69).
- `note_trailing_text` — the note carries text after the grammar; the
  site read a price from it ("Price with Note:"), our parser reads it
  `invalid` and lets the tab apply.
- `ratio_tab_name` — the item's public tab is named with a ratio, which
  T11 says lists nothing; whatever the site shows for it is this.
- `public_flag` — the store's `metadata.public` and the site's listing
  disagree at the tab level: a stash row from a tab the facts hold as
  not public, or a public tab's item the site does not list.
- `amount_display` — same kind and word, the site's amount text differs
  from ours (the chaos fraction dropped).
- `currency_word` — same kind and amount, the site's word is not our
  tag (its loose matching, T16).
- `skip_listed` — the store reads `~skip` (the game's "Do not index")
  and the site lists the item anyway.
- `kind_differs` — both price it, the prefix differs (exact against
  asking, priced against "No Price Set").
- `unlisted_since` — the store expects a stash row and the site has
  none, with the tab public and other items of the same tab present:
  the index lags, or the item moved.
- `unexplained` — none of the above.

Two reasons the first run found under `unlisted_since` (62 rows) and
the list did not hold, added 2026-09-08 with the run recorded that way
(`PRICING-SLICE.md`, the item 5 row): `socketed` — the item sits in a
socket of another (T13: no position), and the site does not list it on
its own; `stackable` — a currency-class stack (`stack_size` present:
scrolls, omens, boxes), which the site trades on the bulk exchange, not
the item search (T2, T3). Both are read before `unlisted_since`.

Per tab, the join also prints what the store holds against what the
site shows, since `public` is a tab-level fact.
"""

import argparse
import collections
import json
import sys


def load(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def reading_text(reading):
    k = reading.get("kind")
    if k in ("exact", "negotiable"):
        return f"{k} {reading['amount']} {reading['currency']}"
    if k == "invalid":
        return f"invalid ({reading.get('why')})"
    return k


def site_text(row):
    if row["label"] in ("exact", "negotiable", "note_price"):
        return f"{row['label']} {row['amount']} {row['currency']}"
    return row["label"]


def is_ratio(reading_or_text):
    return "/" in str(reading_or_text)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--site", required=True)
    ap.add_argument("--report", required=True)
    ap.add_argument("--rows", action="store_true", help="print every disagreeing row")
    a = ap.parse_args()
    site = load(a.site)
    report = load(a.report)

    tabs = {}
    items = {}
    for l in report["listings"]:
        t = l["subject"]["target"]
        if t["scope"] == "tab":
            tabs[t["id"]] = l
        elif t["scope"] == "item":
            items[t["id"]] = l
    site_rows = {r["id"]: r for r in site["rows"]}

    def tab_of(l):
        loc = l["subject"].get("location") or {}
        if loc.get("scope") == "tab":
            return tabs.get(loc["id"])
        return None

    def tab_name_reading(l):
        return (l["game"].get("tab_name") or {}).get("reading", {})

    # What the store expects the site to show for a stash item.
    def expectation(l):
        g = l["game"]
        public = g.get("public") is True
        kind = g["reading"]["kind"]
        if not public:
            return None
        if kind == "skip":
            return None
        if kind in ("exact", "negotiable"):
            return g["reading"]
        return {"kind": "no_price"}

    classes = collections.Counter()
    reasons = collections.Counter()
    rows = []
    per_tab = collections.defaultdict(
        lambda: {"store_items": 0, "expected": 0, "site_rows": 0, "site_labels": collections.Counter()}
    )

    def record(cls, reason, rid, ours, theirs, where, extra=None):
        classes[cls] += 1
        if reason:
            reasons[reason] += 1
        rows.append(
            {
                "class": cls,
                "reason": reason,
                "id": rid,
                "ours": ours,
                "theirs": theirs,
                "where": where,
                **(extra or {}),
            }
        )

    # Site rows first: every id on the site is in the store or not.
    for rid, r in site_rows.items():
        l = items.get(rid)
        theirs = site_text(r)
        if l is None:
            record("site_only", "not_in_store", rid, None, theirs, None, {"listed": r["listed"], "channel": r["channel"]})
            continue
        tab = tab_of(l)
        where = {
            "tab": tab["subject"]["name"] if tab else None,
            "tab_id": tab["subject"]["target"]["id"] if tab else None,
            "public": l["game"].get("public"),
            "location": l["subject"].get("location"),
            "name": l["subject"].get("name") or l["subject"].get("type_line"),
        }
        if tab:
            pt = per_tab[tab["subject"]["target"]["id"]]
            pt["site_rows"] += 1
            pt["site_labels"][r["label"]] += 1
        g = l["game"]
        ours_reading = g["reading"]
        ours = reading_text(ours_reading) + ("" if g.get("public") is True else " (not public)")
        if r["channel"] == "forum":
            m = l.get("manual")
            ours_m = (
                f"manual {m['value'].get('type')} {m['value'].get('amount', '')} {m['value'].get('currency', '')}".strip()
                if m
                else "no row"
            )
            record("site_only" if expectation(l) is None else "differ", "forum_channel", rid, f"{ours}; {ours_m}", theirs, where, {"thread": r["thread"], "listed": r["listed"]})
            continue
        exp = expectation(l)
        if exp is None:
            # The store says: not listed.
            if r["label"] == "note_price" or (ours_reading["kind"] == "invalid" and r["note"] and r["note"] != (g.get("note") or {}).get("text")):
                reason = "note_trailing_text"
            elif ours_reading["kind"] == "invalid" and r["label"] == "note_price":
                reason = "note_trailing_text"
            elif tab and is_ratio(tab_name_reading(l).get("why", "")) or (tab and is_ratio(tab["subject"]["name"]) and g.get("public") is True):
                reason = "ratio_tab_name"
            elif g.get("public") is not True:
                reason = "public_flag"
            elif ours_reading["kind"] == "skip":
                reason = "skip_listed"
            else:
                reason = "unexplained"
            record("site_only", reason, rid, ours, theirs, where, {"note": r["note"], "listed": r["listed"]})
            continue
        # Both list it: compare the claims. A "Price with Note:" row is
        # the site reading a price out of a note our parser refused,
        # whatever the tab says.
        if r["label"] == "note_price":
            record("differ", "note_trailing_text", rid, ours, theirs, where, {"note": r["note"]})
            continue
        if exp["kind"] == "no_price":
            if r["label"] == "no_price":
                classes["agree"] += 1
            elif r["label"] == "note_price" or ours_reading["kind"] == "invalid":
                record("differ", "note_trailing_text", rid, ours, theirs, where, {"note": r["note"]})
            elif tab and is_ratio(tab["subject"]["name"]):
                record("differ", "ratio_tab_name", rid, ours, theirs, where)
            else:
                record("differ", "kind_differs", rid, ours, theirs, where, {"note": r["note"]})
            continue
        # A priced expectation.
        if r["label"] == "no_price":
            record("differ", "kind_differs", rid, ours, theirs, where, {"note": r["note"]})
            continue
        site_kind = "exact" if r["label"] in ("exact", "note_price") else r["label"]
        if site_kind != exp["kind"]:
            record("differ", "kind_differs", rid, ours, theirs, where, {"note": r["note"]})
            continue
        same_amount = r["amount"] == exp["amount"]
        same_word = r["currency"] == exp["currency"]
        if same_amount and same_word:
            classes["agree"] += 1
        elif same_word:
            record("differ", "amount_display", rid, ours, theirs, where, {"note": r["note"]})
        elif same_amount:
            record("differ", "currency_word", rid, ours, theirs, where, {"note": r["note"]})
        else:
            record("differ", "unexplained", rid, ours, theirs, where, {"note": r["note"]})

    # Store items the site should show and does not.
    for rid, l in items.items():
        tab = tab_of(l)
        if tab:
            pt = per_tab[tab["subject"]["target"]["id"]]
            pt["store_items"] += 1
        exp = expectation(l)
        if exp is not None and tab:
            pt["expected"] += 1
        if rid in site_rows or exp is None:
            if exp is None and rid not in site_rows:
                classes["agree_absent"] += 1
            continue
        where = {
            "tab": tab["subject"]["name"] if tab else None,
            "tab_id": tab["subject"]["target"]["id"] if tab else None,
            "public": l["game"].get("public"),
            "location": l["subject"].get("location"),
            "name": l["subject"].get("name") or l["subject"].get("type_line"),
        }
        ours = reading_text(l["game"]["reading"])
        if l["subject"].get("socketed_in"):
            reason = "socketed"
        elif l["subject"].get("stack_size") is not None:
            reason = "stackable"
        elif tab and is_ratio(tab["subject"]["name"]):
            reason = "ratio_tab_name"
        elif tab and per_tab[tab["subject"]["target"]["id"]]["site_rows"] == 0:
            reason = "public_flag"
        else:
            reason = "unlisted_since"
        record("state_only", reason, rid, ours, None, where, {"note": (l["game"].get("note") or {}).get("text")})

    tab_table = []
    for tid, pt in per_tab.items():
        t = tabs[tid]
        if pt["site_rows"] == 0 and pt["expected"] == 0:
            continue
        tab_table.append(
            {
                "tab": t["subject"]["name"],
                "type": t["subject"].get("tab_type"),
                "index": t["subject"].get("index"),
                "public": t["game"].get("public"),
                "name_reading": reading_text(tab_name_reading(t) or t["game"]["reading"]),
                "store_items": pt["store_items"],
                "expected_on_site": pt["expected"],
                "site_rows": pt["site_rows"],
                "site_labels": dict(pt["site_labels"]),
            }
        )
    tab_table.sort(key=lambda t: (t["index"] is None, t["index"]))

    out = {
        "site": {"unique": site["unique"], "by_label": site["by_label"], "parts": site["parts"]},
        "store": {
            "league": report["league"],
            "taken_at": report["taken_at"],
            "items": report["counts"]["items"],
            "by_game_statement": report["counts"]["by_game_statement"],
            "priced_tabs": report["counts"]["priced_tabs"],
            "priced_tabs_public": report["counts"]["priced_tabs_public"],
            "expected_on_site": sum(1 for l in items.values() if expectation(l) is not None),
        },
        "classes": dict(classes),
        "reasons": dict(reasons),
        "by_class_and_reason": dict(collections.Counter(f"{r['class']}/{r['reason']}" for r in rows)),
        "tabs": tab_table,
        "rows": rows if a.rows else [],
    }
    json.dump(out, sys.stdout, indent=1, ensure_ascii=False)
    print()
    print(f"classes {dict(classes)}", file=sys.stderr)
    print(f"reasons {dict(reasons)}", file=sys.stderr)


if __name__ == "__main__":
    main()
