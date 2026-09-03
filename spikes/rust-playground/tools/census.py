#!/usr/bin/env python3
"""census.py — a read-only census of pricing evidence (pricing slice, step 2c).

Reads, never writes: the owner's 0.18 userstore (`userstore-<name>.db`,
the C++ app's per-account file), the spike's facts file for the same
account, and its annotations file. Every connection is opened
`immutable=1` through a percent-encoded URI, and two traps are handled:

- the C++ filename carries a `#` (`GERWARIC#7694`), which a raw `file:`
  URI truncates at the fragment, so an un-encoded open silently reads
  the wrong (empty) file;
- a plain `mode=ro` open of a WAL-mode file still creates `-shm` and
  `-wal` beside it (SQLite needs the wal-index even to read) — a census
  must leave the owner's directory exactly as found, so the open is
  `immutable=1` (no locks, no side files), which is only honest while
  nothing is writing: the tool refuses a file whose `-wal` holds
  uncheckpointed pages (close the app first; it checkpoints on exit).

No network.

What it counts is what the slice's rulings need as local evidence (the
packet, brainstorming-notes/10 §2 step 2c): rows by type, source,
inherited flag, currency and location type; amount shapes; `~c/o`;
character and remove-only rows; and in the facts — `metadata.public`,
tab index, positions, containers, socketing, and every tilde-prefixed
item note and tab name, classified against the C++ parser's regex
(`(~\\S+)\\s+(\\d+\\.?\\d*)\\s+(\\w+)`, `BuyoutManager::StringToBuyout`,
master) so "parsed or not" means what the 0.18 app meant by it.

    tools/census.py --userstore PATH [--facts PATH] [--annotations PATH]

Item ids are never printed: the record wants shapes and counts.
"""

import argparse
import collections
import json
import pathlib
import re
import sqlite3
import urllib.parse

CPP_PRICE = re.compile(r"(~\S+)\s+(\d+\.?\d*)\s+(\w+)")


def ro(path):
    wal = pathlib.Path(path + "-wal")
    if wal.exists() and wal.stat().st_size > 0:
        raise SystemExit(
            f"{wal} holds uncheckpointed pages; an immutable read would miss them. "
            "Close the app that owns the file (it checkpoints on exit) and rerun."
        )
    uri = "file:" + urllib.parse.quote(path) + "?immutable=1"
    return sqlite3.connect(uri, uri=True)


def rows(conn, sql, *args):
    return conn.execute(sql, args).fetchall()


def section(title):
    print()
    print(f"== {title}")


def userstore(path):
    c = ro(path)
    section(f"userstore {path}")
    (v,) = rows(c, "pragma user_version")[0]
    (j,) = rows(c, "pragma journal_mode")[0]
    # Under immutable=1 SQLite reports `delete` whatever the header says.
    print(f"user_version={v} journal_mode(immutable view)={j}")
    for t in ("item_buyouts", "location_buyouts", "stashes", "characters"):
        (n,) = rows(c, f'select count(*) from "{t}"')[0]
        print(f"{t}: {n} rows")

    for table in ("item_buyouts", "location_buyouts"):
        section(f"{table} by type / source / inherited / location_type")
        for r in rows(
            c,
            f"select type, source, inherited, location_type, count(*) from {table}"
            " group by 1,2,3,4 order by 5 desc",
        ):
            print("  ", r)
        print("  currencies:", rows(c, f"select currency, count(*) from {table} group by 1 order by 2 desc"))
        print(
            "  amounts: integral/fractional/min/max/distinct =",
            rows(
                c,
                f"select sum(value = cast(value as integer)), sum(value != cast(value as integer)),"
                f" min(value), max(value), count(distinct value) from {table}",
            )[0],
        )
        print("  fractional amounts:", rows(c, f"select distinct value from {table} where value != cast(value as integer)"))
        print("  non-positive amounts:", rows(c, f"select type, source, count(*) from {table} where value <= 0 group by 1,2"))
        print("  last_update: type/min/max =", rows(c, f"select typeof(last_update), min(last_update), max(last_update) from {table}")[0])
        print("  location_id length by type:", rows(c, f"select location_type, length(location_id), count(*) from {table} group by 1,2"))
        print("  ~c/o rows:", rows(c, f"select count(*) from {table} where type = 'c/o'")[0][0])
        print(
            "  location ids absent from stashes:",
            rows(
                c,
                f"select count(*) from {table} b where b.location_type = 'stash'"
                " and not exists (select 1 from stashes s where s.id = b.location_id)",
            )[0][0],
        )
    print("  item_id length:", rows(c, "select length(item_id), count(*) from item_buyouts group by 1"))
    print(
        "  character rows:",
        rows(
            c,
            "select c.league, b.type, b.source, count(*) from item_buyouts b"
            " left join characters c on c.id = b.location_id where b.location_type = 'character' group by 1,2,3",
        ),
    )

    section("stashes by realm / league: total, remove-only, public, fetched")
    for r in rows(
        c,
        "select realm, league, count(*), sum(name like '%Remove-only%'), sum(meta_public),"
        " sum(json_data is not null) from stashes group by 1,2",
    ):
        print("  ", r)

    section("tilde tab names (league, name, parsed by the C++ regex, prefix)")
    for name, league in rows(c, "select name, league from stashes where name like '%~%' order by league, name"):
        m = CPP_PRICE.search(name)
        print("  ", (league, name, bool(m), m.group(1) if m else None, m.group(3) if m else None))

    section("item notes inside stash json_data")
    n_items = n_note = 0
    notes = collections.Counter()
    classes = collections.Counter()
    amounts = collections.Counter()
    unparsed = []
    by_tab = collections.Counter()
    for league, name, blob in rows(c, "select league, name, json_data from stashes where json_data is not null"):
        try:
            d = json.loads(blob)
        except ValueError as e:
            print("   unreadable json_data:", league, name, e)
            continue
        for it in d.get("items", []):
            n_items += 1
            note = it.get("note")
            if note is None:
                continue
            n_note += 1
            notes[note] += 1
            if not note.startswith("~"):
                continue
            by_tab[(league, name)] += 1
            m = CPP_PRICE.search(note)
            if m:
                classes[(m.group(1), m.group(3))] += 1
                amounts[m.group(2)] += 1
            else:
                classes[("UNPARSED", note)] += 1
                unparsed.append(note)
    print(f"  items={n_items} with_note={n_note} distinct_notes={len(notes)}")
    print("  tilde classes (prefix, currency word):", sorted(classes.items(), key=lambda x: -x[1]))
    print("  unparsed tilde notes:", unparsed)
    print("  non-tilde notes:", [n for n in notes if not n.startswith("~")])
    print("  amount texts:", sorted(amounts.items(), key=lambda x: float(x[0])))
    print("  tabs holding tilde notes:", sorted(by_tab.items(), key=lambda x: -x[1]))
    c.close()


def facts(path):
    f = ro(path)
    section(f"facts {path}")
    print("user_version:", rows(f, "pragma user_version")[0][0])
    print("account:", rows(f, "select uuid, name from account"))
    print("tabs by realm/league: total, removed, fetched:", rows(f, "select realm, league, count(*), sum(removed_at is not null), sum(fetched_at is not null) from tabs group by 1,2"))
    print("tab types (count, substashes):", rows(f, "select type, count(*), sum(parent is not null) from tabs group by 1"))
    print("idx null / present:", rows(f, "select sum(idx is null), sum(idx is not null) from tabs")[0])
    keys = collections.Counter()
    public = 0
    for (j,) in rows(f, "select json from tabs"):
        meta = json.loads(j).get("metadata") or {}
        keys.update(meta.keys())
        public += 1 if meta.get("public") else 0
    print("metadata keys:", dict(keys), "public=true:", public)
    section("tilde tab names in facts (name, type, removed, parsed)")
    for name, t, removed in rows(f, "select name, type, removed_at is not null from tabs where name like '%~%' order by name"):
        m = CPP_PRICE.search(name)
        print("  ", (name, t, bool(removed), bool(m)))
    section("items")
    print("by kind/container (count, removed):", rows(f, "select location_kind, container, count(*), sum(removed_at is not null) from items group by 1,2"))
    print("x/y null by kind:", rows(f, "select location_kind, sum(x is null), sum(y is null), count(*) from items group by 1"))
    print("socketed (socketed_in set) by kind:", rows(f, "select location_kind, sum(socketed_in is not null), count(*) from items group by 1"))
    print("notes present by kind:", rows(f, "select location_kind, sum(json_extract(json,'$.note') is not null), count(*) from items group by 1"))
    print("distinct notes:", rows(f, "select json_extract(json,'$.note'), count(*) from items where json_extract(json,'$.note') is not null group by 1 order by 2 desc"))
    print("characters by realm/league (count, fetched):", rows(f, "select realm, league, count(*), sum(fetched_at is not null) from characters group by 1,2"))
    f.close()


def annotations(path):
    a = ro(path)
    section(f"annotations {path}")
    print("user_version:", rows(a, "pragma user_version")[0][0], "meta:", rows(a, "select key, value from meta"))
    print("rows (scope, kind, live, tombstoned):", rows(a, "select scope, kind, sum(deleted_at is null), sum(deleted_at is not null) from annotations group by 1,2"))
    a.close()


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--userstore", help="the 0.18 userstore-<name>.db")
    p.add_argument("--facts", help="the spike's facts file for the same account")
    p.add_argument("--annotations", help="the spike's <uuid>.annotations.db")
    args = p.parse_args()
    if not (args.userstore or args.facts or args.annotations):
        p.error("nothing to read")
    if args.userstore:
        userstore(args.userstore)
    if args.facts:
        facts(args.facts)
    if args.annotations:
        annotations(args.annotations)


if __name__ == "__main__":
    main()
