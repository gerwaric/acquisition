#!/usr/bin/env python3
"""Scan a real userstore DB for wire shapes the app's types do not model.

Path of Exile item JSON has accumulated idiosyncrasies over a decade —
non-spec colour strings, fields that predate the developer docs, league
mechanics that survive only on Standard. `src/poe/types/` models what has
been seen so far; this tool reports what a given database contains that
those headers do not declare, so old accounts can extend the catalog
instead of silently exercising unknown parse paths.

Output: unmodeled keys with counts and sample values, keys whose JSON type
varies across items, the frameType/frameTypeId pairing table, and colour
string length histograms.

Usage: quirks.py <userstore.db> [--types-dir src/poe/types]
"""

import argparse
import collections
import json
import pathlib
import re
import sqlite3

def decl_name(buf):
    """Member name of one declaration statement, or None.

    Matching by type allowlist (std::/poe::/Qt/primitives) misses members
    declared with unqualified project types (`FrameType frameType;`) and
    default initializers, so instead: reject anything that is not a plain
    data member, strip any initializer, and take the last identifier —
    which requires a preceding type token, so lone words never match.
    """
    decl = re.sub(r"//[^\n]*", "", buf).strip()
    if "(" in decl or decl.startswith(("using ", "typedef ", "friend ",
                                       "static ", "struct", "enum")):
        return None
    decl = re.sub(r"=[^;]*$", "", decl).strip()
    m = re.search(r"[&*\s>]([A-Za-z_]\w*)$", decl)
    return m.group(1) if m else None


def modeled_fields(types_dir, struct="Item", header="item.h"):
    """Direct member names of one struct (nested struct members excluded).

    A flattened union of every member in every header would hide an
    unmodeled top-level key whenever some nested or unrelated struct
    happens to declare the same name (e.g. a new top-level "tier").
    Tracks brace depth so only declarations at the struct's own member
    depth count; nested struct *definitions* contribute their name (the
    member declaration referencing them is what matters), not their
    members.
    """
    text = pathlib.Path(types_dir, header).read_text()
    m = re.search(rf"\bstruct\s+{re.escape(struct)}\b", text)
    if not m:
        raise SystemExit(f"struct {struct} not found in {header}")
    start = m.start()
    depth = 0
    member_depth = None
    fields = set()
    buf = ""
    for ch in text[start:]:
        if ch == "{":
            depth += 1
            if member_depth is None:
                member_depth = depth
            buf = ""
        elif ch == "}":
            depth -= 1
            if depth == 0:
                break
            buf = ""
        elif ch == ";":
            if depth == member_depth:
                name = decl_name(buf)
                if name:
                    fields.add(name)
            buf = ""
        else:
            buf += ch
    return fields


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("db")
    ap.add_argument("--types-dir",
                    default=str(pathlib.Path(__file__).resolve()
                                .parents[2] / "src" / "poe" / "types"))
    args = ap.parse_args()

    known = modeled_fields(args.types_dir)
    unmodeled = collections.Counter()
    samples = {}
    key_types = collections.defaultdict(set)
    frames = collections.Counter()
    colour_lens = collections.Counter()
    n_items = 0

    def scan_item(it):
        nonlocal n_items
        n_items += 1
        for k, v in it.items():
            key_types[k].add(type(v).__name__)
            if k not in known:
                unmodeled[k] += 1
                samples.setdefault(k, json.dumps(v)[:80])
        frames[(it.get("frameType"), it.get("frameTypeId"))] += 1
        for sub in it.get("socketedItems") or []:
            scan_item(sub)

    db = sqlite3.connect(args.db)
    for (blob,) in db.execute(
            "SELECT json_data FROM stashes WHERE json_data IS NOT NULL"):
        d = json.loads(blob)
        colour = (d.get("metadata") or {}).get("colour")
        if colour is not None:
            colour_lens[len(colour)] += 1
        for it in d.get("items") or []:
            scan_item(it)
    for (blob,) in db.execute(
            "SELECT json_data FROM characters WHERE json_data IS NOT NULL"):
        d = json.loads(blob)
        for key in ("equipment", "inventory", "jewels", "rucksack",
                    "guardian", "skills"):
            for it in d.get(key) or []:
                scan_item(it)

    print(f"items scanned: {n_items}")
    print(f"\nunmodeled keys (not declared in {args.types_dir}):")
    for k, n in unmodeled.most_common() or [("(none)", 0)]:
        print(f"  {k}: {n}  sample={samples.get(k, '')}")
    mixed = {k: ts for k, ts in key_types.items() if len(ts) > 1}
    print("\nkeys with mixed JSON types:")
    for k, ts in sorted(mixed.items()) or [("(none)", set())]:
        print(f"  {k}: {sorted(ts)}")
    print("\nframeType / frameTypeId pairs:")
    for (ft, fid), n in sorted(frames.items(),
                               key=lambda x: (x[0][0] is None, str(x[0]))):
        print(f"  {ft} / {fid}: {n}")
    print(f"\nstash colour string lengths: {dict(sorted(colour_lens.items()))}")


if __name__ == "__main__":
    main()
