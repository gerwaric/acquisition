#!/usr/bin/env python3
"""Replicate LegacyItem::hash() (the v4 legacy buyout hash) and measure
how many buyouts in an old acquisition db can be matched to GGG item ids."""

import hashlib
import json
import re
import sqlite3
import sys

SET_RE = re.compile(r"^(<<.*?>>)*")


def effective_type_line(item):
    hybrid = item.get("hybrid")
    if hybrid and not hybrid.get("isVaalGem", False):
        result = hybrid.get("baseTypeName", item.get("typeLine", ""))
    else:
        result = item.get("typeLine", "")
    return SET_RE.sub("", result)


def legacy_hash(item):
    parts = [item.get("name", ""), "~", effective_type_line(item), "~"]
    for mod in item.get("explicitMods") or []:
        parts += [mod, "~"]
    for mod in item.get("implicitMods") or []:
        parts += [mod, "~"]
    for prop in item.get("properties") or []:
        parts += [prop.get("name", ""), "~"]
        for val in prop.get("values") or []:
            parts += [str(val[0]), "~"]
    parts.append("~")
    for prop in item.get("additionalProperties") or []:
        parts += [prop.get("name", ""), "~"]
        for val in prop.get("values") or []:
            parts += [str(val[0]), "~"]
    parts.append("~")
    for socket in item.get("sockets") or []:
        attr = socket.get("attr")
        if attr:
            parts += [str(socket.get("group", 0)), "~", attr, "~"]
    if item.get("_character"):
        parts += ["~character:", item["_character"]]
    else:
        parts += ["~stash:", item.get("_tab_label", "")]
    return hashlib.md5("".join(parts).encode("utf-8")).hexdigest()


def main(path):
    db = sqlite3.connect(f"file:{path}?mode=ro", uri=True)
    buyouts = json.loads(db.execute(
        "SELECT value FROM data WHERE key='buyouts'").fetchone()[0])
    tab_buyouts = json.loads(db.execute(
        "SELECT value FROM data WHERE key='tab_buyouts'").fetchone()[0])
    stashes = json.loads(db.execute(
        "SELECT value FROM tabs WHERE type=0").fetchone()[0])

    hash_to_ids = {}      # legacy hash -> list of (ggg id, tab loc)
    n_items = 0
    for loc, blob in db.execute("SELECT loc, value FROM items"):
        try:
            items = json.loads(blob)
        except json.JSONDecodeError:
            print(f"  bad JSON in items row {loc}", file=sys.stderr)
            continue
        for item in items:
            n_items += 1
            h = legacy_hash(item)
            hash_to_ids.setdefault(h, []).append((item.get("id"), loc))

    matched = {h: v for h, v in buyouts.items() if h in hash_to_ids}
    ambiguous = {h: hash_to_ids[h] for h in matched if len(hash_to_ids[h]) > 1}
    orphaned = [h for h in buyouts if h not in hash_to_ids]

    print(f"items scanned:        {n_items}")
    print(f"distinct hashes:      {len(hash_to_ids)}")
    print(f"item buyouts:         {len(buyouts)}")
    print(f"  matched:            {len(matched)}")
    print(f"  ambiguous (multi):  {len(ambiguous)}")
    print(f"  orphaned:           {len(orphaned)}")

    # tab buyouts: label -> tab ids
    label_to_ids = {}
    for s in stashes:
        label_to_ids.setdefault("stash:" + s.get("n", s.get("name", "")), []).append(s.get("id"))
    tb_matched = sum(1 for k in tab_buyouts if k in label_to_ids)
    tb_ambig = sum(1 for k in tab_buyouts if len(label_to_ids.get(k, [])) > 1)
    print(f"tab buyouts:          {len(tab_buyouts)}")
    print(f"  matched:            {tb_matched}")
    print(f"  ambiguous:          {tb_ambig}")

    if ambiguous:
        print("\nsample ambiguous:")
        for h, v in list(ambiguous.items())[:5]:
            print(f"  {h}: {len(v)} items in tabs {sorted({t for _, t in v})}")
    if orphaned:
        print("\nsample orphaned:", orphaned[:5])


if __name__ == "__main__":
    main(sys.argv[1])
