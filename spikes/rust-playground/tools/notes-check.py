#!/usr/bin/env python3
"""notes-check.py — read the price evidence a live refresh just landed.

Read-only, like tools/census.py (same `immutable=1` open, same refusal
of an uncheckpointed WAL: stop the daemon first). No network.

Answers the two questions the forum-shop reading left open
(brainstorming-notes/12, "What the evidence does not settle"):

1. What GGG's own client writes into `note` for each in-game currency —
   every item note in the tabs whose name matches `--tab` (default:
   `ACQUISITION-PRICE-TEST`), with the tab's own name and `metadata`.
2. Whether `/character` or `/stash` carries `forum_note` for an item
   listed through a forum post — every stored item that has one, and
   every item of the characters named by `--character` with its
   `note`/`forum_note`/`inventoryId`/x/y.

Item ids are printed here (unlike the census): this is a working check
on the owner's own account, not a record.

    tools/notes-check.py [--facts PATH] [--tab NAME] [--character NAME ...]
"""

import argparse
import json
import os
import pathlib
import sqlite3
import urllib.parse

DEFAULT_FACTS = os.path.expanduser(
    "~/Library/Application Support/gerwaric.acquisition-playground/store/ggg/GERWARIC_7694.db"
)


def ro(path):
    # The spike's own facts file, not the owner's C++ file: when the WAL
    # holds pages (a daemon exited without a final checkpoint), a plain
    # `mode=ro` open reads them through the existing `-shm`; it never
    # writes the database. `immutable=1` is used when the WAL is empty.
    wal = pathlib.Path(path + "-wal")
    q = urllib.parse.quote(path)
    if wal.exists() and wal.stat().st_size > 0:
        print(f"(note: {wal.name} holds {wal.stat().st_size} bytes; reading through it, mode=ro)")
        return sqlite3.connect(f"file:{q}?mode=ro", uri=True)
    return sqlite3.connect(f"file:{q}?immutable=1", uri=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--facts", default=DEFAULT_FACTS)
    ap.add_argument("--tab", default="ACQUISITION-PRICE-TEST")
    ap.add_argument("--character", action="append", default=[])
    a = ap.parse_args()
    c = ro(a.facts)

    print(f"== tabs named like {a.tab!r}")
    tabs = c.execute(
        "select realm, league, id, name, type, idx, fetched_at, json from tabs "
        "where upper(name) like ? order by league, idx",
        (f"%{a.tab.upper()}%",),
    ).fetchall()
    if not tabs:
        print("none listed — the stash listing that names it has not landed")
    for realm, league, tid, name, typ, idx, fetched, js in tabs:
        meta = (json.loads(js) if js else {}).get("metadata")
        print(f"{realm}/{league} {tid} idx={idx} {typ} fetched={'yes' if fetched else 'NO'} "
              f"name={name!r} metadata={meta}")
        items = c.execute(
            "select id, name, type_line, stack_size, x, y, json from items "
            "where location_kind='stash' and location_id=? and removed_at is null "
            "order by y, x",
            (tid,),
        ).fetchall()
        print(f"  {len(items)} live items")
        for iid, nm, tl, stack, x, y, ijs in items:
            j = json.loads(ijs)
            print(f"  ({x},{y}) {iid[:12]}… {nm or ''} {tl} x{stack or 1}: "
                  f"note={j.get('note')!r} forum_note={j.get('forum_note')!r}")

    print()
    print("== every stored item carrying `forum_note`")
    hits = c.execute(
        "select location_kind, location_id, league, id, name, type_line, json from items "
        "where json like '%forum_note%'"
    ).fetchall()
    if not hits:
        print("none")
    for kind, loc, league, iid, nm, tl, ijs in hits:
        j = json.loads(ijs)
        if "forum_note" not in j:
            continue
        print(f"{kind} {loc[:12]}… {league} {iid[:12]}… {nm or ''} {tl}: "
              f"note={j.get('note')!r} forum_note={j.get('forum_note')!r}")

    for cname in a.character:
        print()
        print(f"== character {cname!r}")
        chars = c.execute(
            "select id, realm, league, class, level, fetched_at from characters "
            "where upper(name)=? and removed_at is null",
            (cname.upper(),),
        ).fetchall()
        if not chars:
            print("not in the facts")
        for cid, realm, league, cls, lvl, fetched in chars:
            print(f"{realm}/{league} {cls} L{lvl} id={cid[:12]}… fetched={'yes' if fetched else 'NO'}")
            for iid, nm, tl, cont, x, y, ijs in c.execute(
                "select id, name, type_line, container, x, y, json from items "
                "where location_kind='character' and location_id=? and removed_at is null "
                "order by container, y, x",
                (cid,),
            ):
                j = json.loads(ijs)
                print(f"  {cont}/{j.get('inventoryId')} ({x},{y}) {iid[:12]}… {nm or ''} {tl}: "
                      f"note={j.get('note')!r} forum_note={j.get('forum_note')!r}")


if __name__ == "__main__":
    main()
