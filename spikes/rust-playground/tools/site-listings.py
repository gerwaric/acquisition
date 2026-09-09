#!/usr/bin/env python3
"""site-listings.py — one table from the trade site's saved result pages.

The site as the oracle (pricing slice, plan step 7, item 5;
`brainstorming-notes/15`): the owner runs the seller-account search for
one account and league in a browser and saves the rendered DOM
(`copy(document.documentElement.outerHTML)`) in parts, since one search
shows at most 100 rows. This tool reads those saved files — never the
site (C79, `SURFACES.md`: access method `browser`; a tool only reads
what a human recorded) — and prints one JSON table: one row per item
id, deduped across the parts, with what a result row shows.

    tools/site-listings.py <dir-or-files>... > site-listings-<date>.json
    tools/site-listings.py --exchange-items <saved exchange page> > exchange-items-<date>.json

Per row: `id` (the 64-hex item id the stash API also gives — the join
key), `name` and `base` (the popup's header lines), `note` (the price
note the popup shows verbatim, or null), `label` (`exact` for "Exact
Price:", `negotiable` for "Asking Price:", `no_price` for "No Price
Set", `note_price` for "Price with Note:" — a price the site read out
of a note that carries more than the grammar — and `none` when the row
carries no price block at all), `amount` (the text the site shows, or
null), `currency` (the image's `alt`, the site's short word) and
`currency_name` (the display name), `listed` (the "listed N ago" text),
`verified`, `channel` (`stash` when the seller link is the account's
profile, `forum` when it is a shop thread — T1's two channels, told
apart by the link) with the `thread` number for a forum row, `account`,
`source` (the file the row was first seen in). A saved page of the
bulk exchange (the site's other search, where currency-class stacks
are offered, T2) yields rows with `label` `exchange` and `channel`
`exchange`: `base` is the item offered, `lot` how many of it per
`amount` of `currency`, `stock` the site's count; the exchange links
the profile for both channels, so a forum offer is not told apart
here. The header reports each file's "Showing N
results" line, its row count, the union's size, and how many ids the
parts share, so the union's coverage of the site's matched count (a
number the owner reads off the search form; the saved DOM does not
carry it) is arithmetic.

A row that does not parse is reported on stderr with its id and kept
with nulls, never dropped: the join classifies it.

`--exchange-items` reads the other thing a saved exchange page holds:
the item groups of its "Items I Want" / "Items I Have" panels — the
bulk-eligible list T2's About text calls "listed below" (Q10). Each
group the owner expanded before saving yields its ids and display
names; a group left collapsed is reported as unexpanded, not empty.
This is a proposal for a reference table under C68 (`decisions/
pricing.md`, Parked): a human reads and commits it; no code reads it
until its trigger fires.
"""

import argparse
import html
import json
import pathlib
import re
import sys

ROW = re.compile(r'<div class="row( exchange)?"\s+data-id="([0-9a-f]{64})"')
EXCHANGE_GET = re.compile(
    r'<small>what you get</small><div class="price-block"[^>]*><span class="currency-text">([^<]*)</span>.*?<span class="amount">([^<]*)</span>',
    re.S,
)
EXCHANGE_PAY = re.compile(
    r'<small>what you pay</small><div class="price-block[^"]*"[^>]*><span class="amount">([^<]*)</span>.*?alt="([^"]*)"',
    re.S,
)
STOCK = re.compile(r'data-field="stock">Stock:&nbsp;<span>([^<]*)</span>')
# The compact layout: one `per-have` block per offer — `lot × item ⇐ amount × currency`.
PER_HAVE = re.compile(
    r'<div class="per-have">\s*<span>\s*<span\s+class="amount">([^<]*)</span>.*?alt="([^"]*)".*?⇐.*?<span\s+class="amount">([^<]*)</span>.*?alt="([^"]*)"',
    re.S,
)
SHOWING = re.compile(r"Showing\s+([\d,]+)\s+results?")
HEADER_LINE = re.compile(r'<div class="item-popup__header-line">\s*(.*?)\s*</div>', re.S)
LABEL = re.compile(r'class="price-label(?: ([a-z-]+))?">\s*([^<]*?)\s*<', re.S)
AMOUNT = re.compile(r"</span>\s*(?:&nbsp;)?<br>\s*<span>([^<]*)</span>\s*<span>×</span>")
CURRENCY = re.compile(r'alt="([^"]*)"\s+title="[^"]*">\s*<span>([^<]*)</span>')
NOTE = re.compile(r'<div style="color: var\(--colour-currency\);">\s*(.*?)\s*</div>', re.S)
LISTED = re.compile(r"<small>\s*(listed\s+[^<]*?)\s*</small>", re.S)
ACCOUNT = re.compile(r'href="/(account/view-profile|forum/view-thread)/([^"]*)"\s+target="_blank">([^<]*)</a>')

LABELS = {
    "fixed-price": "exact",
    "buyout-price": "negotiable",
    "no-price": "no_price",
    None: "note_price",
}


def squash(s):
    return " ".join(html.unescape(s).split())


def rows_of(text):
    """(id, chunk) for every result row; a chunk ends where the next row starts."""
    hits = list(ROW.finditer(text))
    for i, m in enumerate(hits):
        end = hits[i + 1].start() if i + 1 < len(hits) else len(text)
        yield m.group(2), m.group(1) is not None, text[m.end() : end]


def parse_exchange_row(rid, chunk, source, problems):
    """A bulk-exchange row: the site's offers of `lot` of the item for
    `amount` of the currency, with the stock it counts (T2, T3). The
    default layout names both sides ("what you get" / "what you pay");
    the compact layout shows one `per-have` block per offer, and a stack
    priced two ways (its note and its tab) carries two. One row per
    offer; `offer` numbers them."""
    stock = STOCK.search(chunk)
    acct = ACCOUNT.search(chunk)
    offers = []
    get = EXCHANGE_GET.search(chunk)
    pay = EXCHANGE_PAY.search(chunk)
    if get and pay:
        offers.append((squash(get.group(1)), squash(get.group(2)), squash(pay.group(1)), html.unescape(pay.group(2))))
    else:
        for lot, item, amount, currency in PER_HAVE.findall(chunk):
            offers.append((html.unescape(item), squash(lot), squash(amount), html.unescape(currency)))
    if not offers:
        problems.append(f"{rid}: exchange row without an offer in {source}")
        offers.append((None, None, None, None))
    return [
        {
            "id": rid,
            "name": None,
            "base": item,
            "note": None,
            "label": "exchange",
            "amount": amount,
            "currency": currency,
            "currency_name": None,
            "lot": lot,
            "stock": squash(stock.group(1)) if stock else None,
            "offer": n,
            "listed": None,
            "verified": None,
            "channel": "exchange",
            "thread": None,
            "account": squash(acct.group(3)) if acct else None,
            "source": source,
        }
        for n, (item, lot, amount, currency) in enumerate(offers)
    ]


def parse_row(rid, chunk, source, problems):
    heads = [squash(h) for h in HEADER_LINE.findall(chunk)]
    lab = LABEL.search(chunk)
    label = None
    if lab is None:
        label = "none"
    else:
        label = LABELS.get(lab.group(1))
        if label is None or (lab.group(1) is None and squash(lab.group(2)) != "Price with Note:"):
            problems.append(
                f"{rid}: unknown price label {lab.group(1)!r} {lab.group(2)!r} in {source}"
            )
            label = lab.group(1) or lab.group(2)
    amount = AMOUNT.search(chunk)
    cur = CURRENCY.search(chunk)
    if label in ("exact", "negotiable", "note_price") and (amount is None or cur is None):
        problems.append(f"{rid}: {label} without an amount or currency in {source}")
    note = NOTE.search(chunk)
    listed = LISTED.search(chunk)
    acct = ACCOUNT.search(chunk)
    if acct is None:
        problems.append(f"{rid}: no account link in {source}")
    if not heads:
        problems.append(f"{rid}: no header line in {source}")
    return {
        "id": rid,
        "name": heads[0] if len(heads) > 1 else None,
        "base": heads[-1] if heads else None,
        "note": squash(note.group(1)) if note else None,
        "label": label,
        "amount": squash(amount.group(1)) if amount else None,
        "currency": html.unescape(cur.group(1)) if cur else None,
        "currency_name": squash(cur.group(2)) if cur else None,
        "lot": None,
        "stock": None,
        "offer": None,
        "listed": squash(listed.group(1)) if listed else None,
        "verified": "verifiedStatus\">Verified" in chunk,
        "channel": None if acct is None else ("forum" if acct.group(1).startswith("forum") else "stash"),
        "thread": acct.group(2) if acct and acct.group(1).startswith("forum") else None,
        "account": squash(acct.group(3)) if acct else None,
        "source": source,
    }


GROUP_OR_ENTRY = re.compile(
    r'<div class="filter-title filter-title-clickable"><span>\s*([^<]*?)\s*<'
    r'|<div data-id="([^"]+)" data-title="([^"]*)" class="exchange-filter-item'
)


def exchange_items(path):
    """The item groups of a saved exchange page, from whichever of its two
    panels holds more expanded groups (they list the same items)."""
    text = pathlib.Path(path).read_text(encoding="utf-8")
    best = None
    markers = [text.rfind(m) for m in (">Items I Want<", ">Items I Have<")]
    for marker, start in zip((">Items I Want<", ">Items I Have<"), markers):
        if start < 0:
            continue
        # A panel ends where the other rendered panel begins.
        end = min([m for m in markers if m > start], default=len(text))
        groups = {}
        order = []
        cur = None
        for m in GROUP_OR_ENTRY.finditer(text, start, end):
            if m.group(1):
                cur = squash(m.group(1))
                if cur not in groups:
                    groups[cur] = []
                    order.append(cur)
            elif cur is not None:
                groups[cur].append({"id": m.group(2), "name": squash(m.group(3))})
        expanded = sum(1 for g in order if groups[g])
        if best is None or expanded > best[0]:
            best = (expanded, marker.strip("<>"), order, groups)
    if best is None:
        sys.exit("no exchange panel in the page")
    expanded, panel, order, groups = best
    ids = {e["id"] for g in order for e in groups[g]}
    return {
        "about": "The bulk exchange's item groups as the trade site renders them in its "
        "'Items I Want' / 'Items I Have' panels, read from a page the owner saved "
        "(tools/site-listings.py --exchange-items). The bulk-eligible list T2 calls "
        "'listed below' (Q10, docs/design/trade-ground-truth.md). A group the owner "
        "did not expand before saving is listed under `unexpanded`, not as empty. "
        "A proposal for a reference table under C68; no code reads it until its "
        "trigger fires (decisions/pricing.md, Parked).",
        "source": pathlib.Path(path).name,
        "panel": panel,
        "groups_expanded": expanded,
        "unexpanded": [g for g in order if not groups[g]],
        "entries": len(ids),
        "groups": [{"group": g, "items": groups[g]} for g in order if groups[g]],
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="+", help="saved pages, or a directory of them")
    ap.add_argument("--exchange-items", action="store_true", help="the exchange's item groups from one saved page")
    a = ap.parse_args()
    if a.exchange_items:
        if len(a.paths) != 1:
            sys.exit("--exchange-items takes one saved exchange page")
        out = exchange_items(a.paths[0])
        json.dump(out, sys.stdout, indent=1, ensure_ascii=False)
        print()
        print(
            f"{out['groups_expanded']} groups expanded, {len(out['unexpanded'])} not "
            f"({', '.join(out['unexpanded'])}); {out['entries']} items",
            file=sys.stderr,
        )
        return
    files = []
    for p in a.paths:
        p = pathlib.Path(p)
        files.extend(sorted(p.glob("*.html")) if p.is_dir() else [p])
    if not files:
        sys.exit("no saved pages found")

    table = {}
    parts = []
    shared = 0
    problems = []
    for f in files:
        text = f.read_text(encoding="utf-8")
        showing = SHOWING.search(text)
        n = 0
        for rid, exchange, chunk in rows_of(text):
            n += 1
            rows = parse_exchange_row(rid, chunk, f.name, problems) if exchange else [parse_row(rid, chunk, f.name, problems)]
            for row in rows:
                # An item search row and an exchange offer for one id are
                # separate claims (a forum-listed stack shows on both; a
                # stack priced two ways is offered twice): the key tells them apart.
                key = (rid, "exchange", row["offer"]) if exchange else (rid, "item")
                if key in table:
                    shared += 1
                    prior = table[key]
                    same = {k: v for k, v in row.items() if k != "source"} == {
                        k: v for k, v in prior.items() if k != "source"
                    }
                    if not same:
                        problems.append(f"{rid}: differs between {prior['source']} and {f.name}")
                    continue
                table[key] = row
        parts.append(
            {
                "file": f.name,
                "showing": int(showing.group(1).replace(",", "")) if showing else None,
                "rows": n,
            }
        )
        if showing and int(showing.group(1).replace(",", "")) != n:
            problems.append(f"{f.name}: says {showing.group(1)} results, holds {n} rows")

    labels = {}
    for r in table.values():
        labels[r["label"]] = labels.get(r["label"], 0) + 1
    out = {
        "about": "The trade site's seller-account search, one account and league, saved as "
        "rendered DOM in parts (a search shows at most 100 rows) and normalized by "
        "tools/site-listings.py: one row per item id, deduped across the parts. The "
        "oracle for the listing state (plan step 7, item 5).",
        "parts": parts,
        "rows_in_parts": sum(p["rows"] for p in parts),
        "shared_between_parts": shared,
        "unique": len(table),
        "unique_ids": len({k[0] for k in table}),
        "by_label": dict(sorted(labels.items())),
        "rows": sorted(table.values(), key=lambda r: (r["id"], r["label"] == "exchange", r["offer"] or 0)),
    }
    json.dump(out, sys.stdout, indent=1, ensure_ascii=False)
    print()
    for p in problems:
        print(p, file=sys.stderr)
    print(
        f"{len(files)} files, {out['rows_in_parts']} rows, {shared} shared, {len(table)} unique; "
        f"labels {out['by_label']}; {len(problems)} problems",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
