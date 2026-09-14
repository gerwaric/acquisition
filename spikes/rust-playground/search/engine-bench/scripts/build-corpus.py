#!/usr/bin/env python3
"""engine-bench corpus: every item of both stores, deduped by id, as one JSONL file.

Inputs (read-only copies made with sqlite's `.backup`, never `cp`, because the originals are under WAL):
  raw/spike.db  <- ../item-facts/raw/spike-GERWARIC_7694-2026-09-13.db     (facts v7)
  raw/cpp.db    <- ../item-facts/raw/cpp-userstore-GERWARIC_7694-2026-09-13.db

Output (raw/, never committed: the bodies are the owner's account data):
  raw/corpus.jsonl   one line per item: {"c": {coordinates}, "i": {the item body, verbatim}}

Dedupe follows ../item-facts/scripts/census.py: keyed by GGG id, the newer fetch wins
(the spike's `last_seen`, the C++ store's `json_fetched_at`). Socketed gems are lifted to
their own records with the host's id in `socketed_in`; a PoE2 granted skill (no id) is skipped.
A body that will not parse is skipped and counted, never a crash.
"""

import datetime as dt
import json
import os
import sqlite3
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TRACK = os.path.dirname(HERE)
RAW = os.path.join(TRACK, "raw")
SPIKE = os.path.join(RAW, "spike.db")
CPP = os.path.join(RAW, "cpp.db")
OUT = os.path.join(RAW, "corpus.jsonl")

skipped = {"body_parse": 0, "no_id": 0, "granted_skill": 0}


def is_item(v):
    return isinstance(v, dict) and "typeLine" in v


def item_arrays(body):
    return [k for k, v in body.items() if isinstance(v, list) and v and all(is_item(e) for e in v)]


def iso_to_epoch(s):
    return int(dt.datetime.fromisoformat(s).timestamp())


def lift(body, coords):
    """Every item-shaped array in a body; socketed gems as their own records."""
    for container in item_arrays(body):
        for it in body[container]:
            gems = it.pop("socketedItems", None)
            if not it.get("id"):
                skipped["no_id"] += 1
            else:
                yield dict(coords, container=container, socketed_in=None), it
            for g in gems or []:
                if "id" not in g:  # PoE2 granted skill: a property of its host, never a row
                    skipped["granted_skill"] += 1
                    continue
                yield dict(coords, container=container, socketed_in=it.get("id")), g


def spike_records():
    db = sqlite3.connect(f"file:{SPIKE}?mode=ro", uri=True)
    q = """SELECT i.id, i.realm, i.league, i.location_kind, i.location_id, i.container,
                  i.socketed_in, i.last_seen, i.json, t.name
           FROM items i LEFT JOIN tabs t
             ON t.realm = i.realm AND t.league = i.league AND t.id = i.location_id
            AND i.location_kind = 'stash'
           WHERE i.removed_at IS NULL"""
    for iid, realm, league, kind, loc, container, socketed_in, seen, body, tab in db.execute(q):
        try:
            it = json.loads(body)
        except (ValueError, TypeError):
            skipped["body_parse"] += 1
            continue
        it.pop("socketedItems", None)
        yield {
            "id": iid, "source": "spike", "realm": realm, "league": league, "kind": kind,
            "loc": loc, "tab": tab if kind == "stash" else loc, "container": container,
            "socketed_in": socketed_in, "fetched": seen,
        }, it
    db.close()


def cpp_records():
    db = sqlite3.connect(f"file:{CPP}?mode=ro", uri=True)
    for sid, realm, league, name, fetched, body in db.execute(
            "SELECT id, realm, league, name, json_fetched_at, json_data FROM stashes WHERE json_data IS NOT NULL"):
        try:
            parsed = json.loads(body)
        except (ValueError, TypeError):
            skipped["body_parse"] += 1
            continue
        coords = {"source": "cpp", "realm": realm, "league": league, "kind": "stash",
                  "loc": sid, "tab": name, "fetched": iso_to_epoch(fetched)}
        for c, it in lift(parsed, coords):
            yield dict(c, id=it.get("id")), it
    for cid, realm, league, name, fetched, body in db.execute(
            "SELECT id, realm, league, name, json_fetched_at, json_data FROM characters WHERE json_data IS NOT NULL"):
        try:
            parsed = json.loads(body)
        except (ValueError, TypeError):
            skipped["body_parse"] += 1
            continue
        coords = {"source": "cpp", "realm": realm, "league": league, "kind": "character",
                  "loc": cid, "tab": name, "fetched": iso_to_epoch(fetched)}
        for c, it in lift(parsed, coords):
            yield dict(c, id=it.get("id")), it
    db.close()


def main():
    by_id = {}
    for coords, body in list(spike_records()) + list(cpp_records()):
        iid = coords.get("id")
        if not iid:
            skipped["no_id"] += 1
            continue
        old = by_id.get(iid)
        if old is None or coords["fetched"] > old[0]["fetched"]:
            by_id[iid] = (coords, body)
    with open(OUT, "w") as f:
        for coords, body in by_id.values():
            f.write(json.dumps({"c": coords, "i": body}, separators=(",", ":"), ensure_ascii=False))
            f.write("\n")
    print(f"items {len(by_id)}  bytes {os.path.getsize(OUT)}  skipped {skipped}", file=sys.stderr)


if __name__ == "__main__":
    main()
