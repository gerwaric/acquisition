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
`source` (the file the row was first seen in). The header reports each file's "Showing N
results" line, its row count, the union's size, and how many ids the
parts share, so the union's coverage of the site's matched count (a
number the owner reads off the search form; the saved DOM does not
carry it) is arithmetic.

A row that does not parse is reported on stderr with its id and kept
with nulls, never dropped: the join classifies it.
"""

import argparse
import html
import json
import pathlib
import re
import sys

ROW = re.compile(r'<div class="row"\s+data-id="([0-9a-f]{64})"')
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
        yield m.group(1), text[m.end() : end]


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
        "listed": squash(listed.group(1)) if listed else None,
        "verified": "verifiedStatus\">Verified" in chunk,
        "channel": None if acct is None else ("forum" if acct.group(1).startswith("forum") else "stash"),
        "thread": acct.group(2) if acct and acct.group(1).startswith("forum") else None,
        "account": squash(acct.group(3)) if acct else None,
        "source": source,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="+", help="saved pages, or a directory of them")
    a = ap.parse_args()
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
        for rid, chunk in rows_of(text):
            n += 1
            row = parse_row(rid, chunk, f.name, problems)
            if rid in table:
                shared += 1
                prior = table[rid]
                same = {k: v for k, v in row.items() if k != "source"} == {
                    k: v for k, v in prior.items() if k != "source"
                }
                if not same:
                    problems.append(f"{rid}: differs between {prior['source']} and {f.name}")
                continue
            table[rid] = row
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
        "by_label": dict(sorted(labels.items())),
        "rows": sorted(table.values(), key=lambda r: r["id"]),
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
