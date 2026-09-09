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
    tools/site-join.py --self-test        # the reason precedence over synthetic rows

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
- `note_trailing_text` — the site read a price out of a note ("Price
  with Note:") that our parser did not read the same way. Parser v1
  refused any suffix and let the tab apply; v2 (2026-09-08) reads
  through a suffix as the site does, so under v2 such a row agrees
  when the amount and word match (`note_price_agrees`) and this reason
  is left for a suffix the two still read apart.
- `ratio_tab_name` — the item's public tab is named with a ratio, which
  lists nothing on the item search (T11): our state still calls the
  tab public with no statement, so such an item is expected and absent
  (state-only); one with its own valid note is listed and compared.
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

An exchange row (the bulk-exchange page, `label` `exchange`) is read
against the same game side: the offer is `lot` of the stack for
`amount` of the currency, which is T2's `wanted/lot` read off a ratio
name, or a plain note's amount for a lot of one. `exchange_agrees` and
`exchange_differs` are its two outcomes; a stack in a non-public tab on
the exchange came through the forum (`forum_channel`).

Per tab, the join also prints what the store holds against what the
site shows, since `public` is a tab-level fact.
"""

import argparse
import collections
import json
import re
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


def self_test():
    """The reason precedence, pinned over synthetic rows: a table and a
    report small enough to read, run through this script, each id's
    reason asserted. Exit 1 on the first miss."""
    import os
    import subprocess
    import tempfile

    def tab(tid, name, public):
        return {
            "subject": {"target": {"scope": "tab", "realm": "pc", "id": tid}, "name": name, "tab_type": "PremiumStash", "index": 0},
            "chain": [],
            "game": {"reading": {"kind": "none"}, "tab": tid, "tab_name": {"text": name, "reading": {"kind": "none"}}, "public": public},
            "relation": "none", "why": "", "effective": {"kind": "none", "why": ""}, "basis": {},
        }

    def item(iid, tid, reading, public, note=None):
        g = {"reading": reading, "tab": tid, "public": public}
        if note:
            g["note"] = {"text": note, "reading": reading}
            g["source"] = "note"
        return {
            "subject": {"target": {"scope": "item", "id": iid}, "name": iid, "type_line": "Thing", "location": {"scope": "tab", "realm": "pc", "id": tid}, "x": 0, "y": 0},
            "chain": [], "game": g, "relation": "none", "why": "", "effective": {"kind": "none", "why": ""}, "basis": {},
        }

    exact = lambda a, c: {"kind": "exact", "amount": a, "currency": c}
    report = {
        "league": "Standard", "taken_at": 0,
        "counts": {"items": 6, "by_game_statement": {}, "priced_tabs": 0, "priced_tabs_public": 0},
        "rows": {}, "listings": [
            tab("t-priv", "Private", False),
            tab("t-pub", "Public", True),
            tab("t-ratio", "~price 5/2 chaos", True),
            item("i-priv-note", "t-priv", exact("7", "chaos"), False, "~price 7 chaos testing"),
            item("i-pub-note", "t-pub", exact("7", "chaos"), True, "~price 7 chaos testing"),
            item("i-ratio", "t-ratio", {"kind": "none"}, True),
            item("i-priv", "t-priv", {"kind": "none"}, False),
            item("i-skip", "t-pub", {"kind": "skip"}, True, "~skip"),
            item("i-bad", "t-pub", {"kind": "invalid", "why": "x"}, True, "~price chaos"),
        ],
    }
    row = lambda iid, label, amount=None, currency=None, note=None: {
        "id": iid, "name": None, "base": "Thing", "note": note, "label": label, "amount": amount,
        "currency": currency, "currency_name": None, "listed": "listed 1 day ago", "verified": True,
        "channel": "stash", "thread": None, "lot": None, "stock": None, "offer": None, "account": "A#1", "source": "t",
    }
    site = {"unique": 6, "by_label": {}, "parts": [], "rows": [
        row("i-priv-note", "note_price", "7", "chaos", "~price 7 chaos testing"),
        row("i-pub-note", "note_price", "7", "chaos", "~price 7 chaos testing"),
        row("i-priv", "no_price"),
        row("i-skip", "no_price"),
        row("i-bad", "note_price", "9", "chaos", "~price chaos"),
        row("i-nowhere", "no_price"),
    ]}
    want = {
        "i-priv-note": "public_flag",       # the public finding beats the parser's
        "i-pub-note": None,                 # agrees under parser v2
        "i-ratio": "ratio_tab_name",  # state-only: expected, absent
        "i-priv": "public_flag",
        "i-skip": "skip_listed",
        "i-bad": "note_trailing_text",
        "i-nowhere": "not_in_store",
    }
    with tempfile.TemporaryDirectory() as d:
        sp, rp = os.path.join(d, "site.json"), os.path.join(d, "report.json")
        json.dump(site, open(sp, "w"))
        json.dump(report, open(rp, "w"))
        out = subprocess.run([sys.executable, __file__, "--site", sp, "--report", rp, "--rows"], capture_output=True, text=True, check=True)
    got = {r["id"]: r["reason"] for r in json.loads(out.stdout)["rows"]}
    reasons = json.loads(out.stdout)["reasons"]
    for iid, reason in want.items():
        if got.get(iid) != reason:
            sys.exit(f"self-test: {iid}: wanted {reason!r}, got {got.get(iid)!r}")
    if reasons.get("note_price_agrees") != 1:
        sys.exit(f"self-test: the public note-priced row should agree; reasons {reasons}")
    print("self-test: the reason precedence holds over 6 site rows and 7 store items")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--site")
    ap.add_argument("--report")
    ap.add_argument("--rows", action="store_true", help="print every disagreeing row")
    ap.add_argument("--self-test", action="store_true", help="the reason precedence over synthetic rows")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return
    if not (a.site and a.report):
        ap.error("--site and --report are required")
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
    site_rows = {r["id"]: r for r in site["rows"] if r["label"] != "exchange"}
    exchange_rows = [r for r in site["rows"] if r["label"] == "exchange"]

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
            # The store says: not listed — because the tab is not public
            # (Q11's condition, the one this audit exists to test, read
            # before anything about the note) or because the game says
            # `skip`. Whatever label the site's row carries, the tab-level
            # fact is the finding.
            if g.get("public") is not True:
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
            # Parser v2 reads through the suffix as the site does: the
            # label alone is no difference — the amount and word decide.
            if exp["kind"] == "exact" and (exp["amount"], exp["currency"]) == (r["amount"], r["currency"]):
                classes["agree"] += 1
                reasons["note_price_agrees"] += 1
            else:
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

    # Exchange rows: the offer against the text the game side read.
    on_exchange = set()
    for r in exchange_rows:
        rid = r["id"]
        l = items.get(rid)
        theirs = f"exchange {r['lot']} for {r['amount']} {r['currency']} (stock {r['stock']})"
        if l is None:
            record("site_only", "not_in_store", rid, None, theirs, None, {"channel": "exchange"})
            continue
        on_exchange.add(rid)
        tab = tab_of(l)
        where = {
            "tab": tab["subject"]["name"] if tab else None,
            "tab_id": tab["subject"]["target"]["id"] if tab else None,
            "public": l["game"].get("public"),
            "location": l["subject"].get("location"),
            "name": l["subject"].get("name") or l["subject"].get("type_line"),
        }
        g = l["game"]
        if g.get("public") is not True:
            record("site_only", "forum_channel", rid, "not public; no row" if not l.get("manual") else "not public; a row", theirs, where, {"channel": "exchange"})
            continue
        text = (g.get("note") or {}).get("text") or (g.get("tab_name") or {}).get("text") or ""
        m = re.match(r"~(?:price|b/o)\s+(\S+)\s+(\S+)", text)
        wanted, lot, word = (None, None, None)
        if m:
            wanted, word = m.group(1), m.group(2)
            if "/" in wanted:
                wanted, lot = wanted.split("/", 1)
            else:
                lot = "1"
        ours = f"{text!r} read as {reading_text(g['reading'])}"
        # A stack priced by note and by tab is offered twice; the tab's
        # offer is read against the tab name when the note's is not it.
        alt = (g.get("tab_name") or {}).get("text") or ""
        m2 = re.match(r"~(?:price|b/o)\s+(\S+)\s+(\S+)", alt)
        alt_triple = None
        if m2:
            w, word2 = m2.group(1), m2.group(2)
            alt_triple = (w.split("/", 1)[0], w.split("/", 1)[1] if "/" in w else "1", word2)
        # The site's compact layout shows the offer per unit (5000 for 2
        # becomes 2500 for 1): the comparison is the rate, as a fraction.
        from fractions import Fraction

        def rate(amount, lot):
            try:
                return Fraction(str(amount)) / Fraction(str(lot))
            except (ValueError, ZeroDivisionError, TypeError):
                return None

        site_rate = rate(r["amount"], r["lot"])
        ours_rates = [(rate(w, l), c) for (w, l, c) in ((wanted, lot, word), alt_triple or (None, None, None))]
        # The exchange shows one row per item type per account, every
        # offer of every public stack of that type under it: an offer the
        # row's own stack does not carry may be another stack's.
        type_line = l["subject"].get("type_line")
        for other in items.values():
            if other is l or other["subject"].get("type_line") != type_line or other["game"].get("public") is not True:
                continue
            for t in ((other["game"].get("note") or {}).get("text"), (other["game"].get("tab_name") or {}).get("text")):
                m3 = re.match(r"~(?:price|b/o)\s+(\S+)\s+(\S+)", t or "")
                if m3:
                    w = m3.group(1)
                    ours_rates.append((rate(w.split("/", 1)[0], w.split("/", 1)[1] if "/" in w else "1"), m3.group(2)))
        if any(rt is not None and rt == site_rate and c == r["currency"] for rt, c in ours_rates):
            classes["agree"] += 1
            reasons["exchange_agrees"] += 1
        else:
            record("differ", "exchange_differs", rid, ours, theirs, where)

    # Store items the site should show and does not.
    for rid, l in items.items():
        tab = tab_of(l)
        if tab:
            pt = per_tab[tab["subject"]["target"]["id"]]
            pt["store_items"] += 1
        exp = expectation(l)
        if exp is not None and tab:
            pt["expected"] += 1
        if rid in site_rows or rid in on_exchange or exp is None:
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
