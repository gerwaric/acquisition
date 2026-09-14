#!/usr/bin/env python3
"""Every fact column beside the JSON path it is lifted from, and every
JSON path the item-facts census saw that no column carries.

Inputs (read, never written):
  crates/acquisition-store/src/schema.sql        the tables and columns
  crates/acquisition-store/src/lib.rs            Columns::of (the lift map),
                                                 ItemRow (what a read returns)
  crates/acquisition-store/src/snapshot.rs       ItemSnapshot, read_items
  search/item-facts/data/field-census.csv        path -> items, share

Output: data/columns-vs-json.csv (CSV, lineterminator "\n").

Everything mechanical is parsed. The one hand-supplied table is
SOURCE_NOTES: how a column that is *not* lifted from the row's own json
gets its value, each entry citing the ruling that says so. Nothing there
is a guess; each is quoted from schema.sql or the module doc.
"""

import csv
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TRACK = Path(__file__).resolve().parents[1]
STORE = ROOT / "crates" / "acquisition-store" / "src"
SCHEMA = STORE / "schema.sql"
LIB = STORE / "lib.rs"
SNAP = STORE / "snapshot.rs"
CENSUS = TRACK.parent / "item-facts" / "data" / "field-census.csv"
OUT = TRACK / "data" / "columns-vs-json.csv"

# How a column that is not lifted from the row's own json gets its value.
# key: "table.column" -> (source, json_path_or_empty, note)
#   ingest-fact    the daemon's request or the ingest pass knows it; it is
#                  not in the body, so `rebuild` cannot recompute it
#   verbatim       the body (or the body minus its item arrays)
#   bookkeeping    the store's own clock / row ids
#   identity       the GGG id the row is keyed on
SOURCE_NOTES = {
    "items.id": ("identity", "id", "GGG item id, stable across moves; an id-less item is refused (C29/C57)"),
    "items.realm": ("ingest-fact", "", "the REQUEST's realm, never the body's `realm` field (C60)"),
    "items.league": ("ingest-fact", "", "the request's league; NULL for a character item"),
    "items.location_kind": ("ingest-fact", "", "stash | character"),
    "items.location_id": ("ingest-fact", "", "tab/substash id, or character id (C55)"),
    "items.container": ("ingest-fact", "", "the array the item came from; not in the json, so rebuild cannot recompute it (C56)"),
    "items.socketed_in": ("ingest-fact", "", "parent item id for a lifted socketed gem"),
    "items.json": ("verbatim", "(whole item)", "the item verbatim minus socketedItems; a granted-skill subtree stays inside (C57)"),
    "items.first_seen": ("bookkeeping", "", "store clock"),
    "items.last_seen": ("bookkeeping", "", "store clock"),
    "items.seen_response": ("bookkeeping", "", "responses.id of the fetch that last saw it here; membership is per response (C54)"),
    "items.removed_at": ("bookkeeping", "", "set when a fetch of its location no longer had it, or the location was retired"),
    "tabs.realm": ("ingest-fact", "", "the request's realm (C60)"),
    "tabs.league": ("ingest-fact", "", ""),
    "tabs.id": ("identity", "id", ""),
    "tabs.parent": ("ingest-fact", "", "folder id from the list, or parent tab id for a substash"),
    "tabs.name": ("verbatim", "name", ""),
    "tabs.type": ("verbatim", "type", ""),
    "tabs.idx": ("verbatim", "index", ""),
    "tabs.json": ("verbatim", "(whole tab)", "the fetched tab minus items and children"),
    "tabs.listed_json": ("verbatim", "(listing entry)", "the latest list entry / substash stub; a fetch never touches it (C54)"),
    "tabs.listed_at": ("bookkeeping", "", ""),
    "tabs.listed_response": ("bookkeeping", "", "responses.id of the listing that last listed this tab"),
    "tabs.fetched_at": ("bookkeeping", "", "cleared when a listing revives a retired row (C54)"),
    "tabs.removed_at": ("bookkeeping", "", ""),
    "characters.id": ("identity", "id", "identity, stable across renames (C55)"),
    "characters.realm": ("ingest-fact", "", "the request's realm (C60)"),
    "characters.name": ("verbatim", "name", "the ADDRESS a fetch takes; listing-owned (C55)"),
    "characters.league": ("verbatim", "league", "listing-owned: a fetch never overwrites it (C54)"),
    "characters.class": ("verbatim", "class", "listing-owned"),
    "characters.level": ("verbatim", "level", "listing-owned"),
    "characters.json": ("verbatim", "(whole character)", "the fetched character minus its item arrays"),
    "characters.listed_json": ("verbatim", "(listing entry)", "deleted/expired/experience live here (C61, C62)"),
    "characters.listed_at": ("bookkeeping", "", ""),
    "characters.listed_response": ("bookkeeping", "", ""),
    "characters.fetched_at": ("bookkeeping", "", ""),
    "characters.removed_at": ("bookkeeping", "", ""),
    "item_events.id": ("bookkeeping", "", ""),
    "item_events.response_id": ("bookkeeping", "", ""),
    "item_events.at": ("bookkeeping", "", ""),
    "item_events.item_id": ("identity", "id", ""),
    "item_events.kind": ("ingest-fact", "", "added | moved | changed | removed; veiledMods ignored (N36)"),
    "item_events.from_location": ("ingest-fact", "", "the full coordinate as a string (C29/C54)"),
    "item_events.to_location": ("ingest-fact", "", "the full coordinate as a string"),
}

# Read-time json_extract: no column, but a named read lifts it anyway.
# "table.path" -> (read, note)
READ_TIME_EXTRACTS = {
    "items.$.note": ("PricingSnapshot.items[].note", "snapshot.rs read_items"),
    "items.$.inventoryId": ("PricingSnapshot.items[].inventory_id", "snapshot.rs read_items"),
}

TABLES_OF_INTEREST = ["items", "item_events", "tabs", "characters"]


def parse_schema(text):
    """table -> [(column, sql type)] in declaration order."""
    out = {}
    for m in re.finditer(
        r"CREATE TABLE IF NOT EXISTS (\w+) \((.*?)\n\);", text, re.S
    ):
        table, body = m.group(1), m.group(2)
        cols = []
        for line in body.splitlines():
            line = line.split("--")[0].strip().rstrip(",")
            if not line or line.startswith("PRIMARY KEY"):
                continue
            parts = line.split()
            if len(parts) < 2:
                continue
            cols.append((parts[0], " ".join(parts[1:])))
        out[table] = cols
    return out


def parse_columns_of(text):
    """items column -> the json key Columns::of reads (the lift map)."""
    body = re.search(r"fn of\(v: &'a Value\) -> Columns<'a> \{(.*?)\n    \}", text, re.S)
    if not body:
        sys.exit("Columns::of not found in lib.rs — the lift map moved")
    out = {}
    for m in re.finditer(r"(\w+): [si]\(\"(\w+)\"\)", body.group(1)):
        out[m.group(1)] = m.group(2)
    return out


def parse_struct_fields(text, name):
    body = re.search(r"pub struct " + name + r" \{(.*?)\n\}", text, re.S)
    if not body:
        sys.exit(f"{name} not found — the read type moved")
    return {m.group(1) for m in re.finditer(r"^\s*pub (?:r#)?(\w+):", body.group(1), re.M)}


def main():
    schema = parse_schema(SCHEMA.read_text())
    lib = LIB.read_text()
    lift = parse_columns_of(lib)
    item_row = parse_struct_fields(lib, "ItemRow")
    item_snap = parse_struct_fields(SNAP.read_text(), "ItemSnapshot")

    census = {}
    with CENSUS.open() as fh:
        first = fh.readline()
        if not first.startswith("#"):
            sys.exit("field-census.csv lost its provenance line")
        for row in csv.DictReader(fh):
            census[row["path"]] = (row["items"], row["share"])

    rows = []
    seen_paths = set()
    for table in TABLES_OF_INTEREST:
        for col, sqltype in schema[table]:
            key = f"{table}.{col}"
            if table == "items" and col in lift:
                source, path, note = "derived-from-json", lift[col], "re-extracted by Store::rebuild, never a refetch (C29)"
            elif key in SOURCE_NOTES:
                source, path, note = SOURCE_NOTES[key]
            else:
                sys.exit(f"{key} has no source on record — add it to SOURCE_NOTES")
            # The census counts item bodies, so only an items column's
            # path can be joined to it; a tab's or character's `name` is
            # a different `name`.
            items, share = census.get(path, ("", "")) if table == "items" else ("", "")
            if table == "items" and path in census:
                seen_paths.add(path)
            exposed = []
            if table == "items" and col in item_row:
                exposed.append("ItemRow")
            if table == "items" and col in item_snap:
                exposed.append("ItemSnapshot")
            rows.append(
                {
                    "kind": "column",
                    "table": table,
                    "name": col,
                    "sql_type": sqltype,
                    "source": source,
                    "json_path": path,
                    "census_items": items,
                    "census_share": share,
                    "depth": "",
                    "exposed_in": "+".join(exposed),
                    "note": note,
                }
            )

    for path, (read, where) in sorted(READ_TIME_EXTRACTS.items()):
        table, jpath = path.split(".", 1)
        key = jpath[2:]
        items, share = census.get(key, ("", ""))
        seen_paths.add(key)
        rows.append(
            {
                "kind": "read-time-extract",
                "table": table,
                "name": jpath,
                "sql_type": "",
                "source": "json_extract at read",
                "json_path": key,
                "census_items": items,
                "census_share": share,
                "depth": "0",
                "exposed_in": read,
                "note": where + " — parsed per row, no column, no index",
            }
        )

    for path, (items, share) in census.items():
        if path in seen_paths:
            continue
        depth = path.count(".") + path.count("[]")
        rows.append(
            {
                "kind": "json-only",
                "table": "items",
                "name": path,
                "sql_type": "",
                "source": "items.json",
                "json_path": path,
                "census_items": items,
                "census_share": share,
                "depth": str(depth),
                "exposed_in": "ItemRow.json+get_item",
                "note": "reachable only by parsing the whole body",
            }
        )

    rows.sort(
        key=lambda r: (
            {"column": 0, "read-time-extract": 1, "json-only": 2}[r["kind"]],
            r["table"] if r["kind"] == "column" else "",
            -float(r["census_share"] or 0) if r["kind"] == "json-only" else 0,
            r["name"],
        )
    )

    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", newline="") as fh:
        fh.write(
            "# generated by scripts/columns-vs-json.py from "
            "crates/acquisition-store/src/{schema.sql,lib.rs,snapshot.rs} "
            "and ../item-facts/data/field-census.csv\n"
        )
        w = csv.DictWriter(
            fh,
            fieldnames=[
                "kind",
                "table",
                "name",
                "sql_type",
                "source",
                "json_path",
                "census_items",
                "census_share",
                "depth",
                "exposed_in",
                "note",
            ],
            lineterminator="\n",
        )
        w.writeheader()
        w.writerows(rows)

    counts = {}
    for r in rows:
        counts[r["kind"]] = counts.get(r["kind"], 0) + 1
    print(f"wrote {OUT.relative_to(TRACK)}: " + ", ".join(f"{k}={v}" for k, v in sorted(counts.items())))
    top = [r for r in rows if r["kind"] == "json-only" and r["depth"] == "0"]
    print(f"json-only top-level paths: {len(top)}")
    for r in top[:15]:
        print(f"  {r['name']:<20} share={r['census_share']}")


if __name__ == "__main__":
    main()
