#!/usr/bin/env python3
"""What SQLite does with each read the store crate exposes: the plan, and
whether an index is used or the table is scanned.

Input: crates/acquisition-store/src/schema.sql (the real schema, applied
to an empty in-memory database — no data, so this measures the *plan*,
never a time; timing is engine-bench's track).

Each query below is copied verbatim from the source, with its file and
symbol named. If a query is edited in the crate and not here, the
citation is stale — re-copy, do not guess.

Output: data/query-plans.txt.
"""

import re
import sqlite3
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TRACK = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "crates" / "acquisition-store" / "src" / "schema.sql"
LIB = ROOT / "crates" / "acquisition-store" / "src" / "lib.rs"
OUT = TRACK / "data" / "query-plans.txt"

# The listing order both Store::tabs and refresh_snapshot use, read from
# the crate so this file cannot drift from it.
TAB_ORDER_SQL = re.search(
    r'pub\(crate\) const TAB_ORDER_SQL: &str = "(.*?)";', LIB.read_text(), re.S
).group(1)

# (title, source citation, sql, bindings). Bindings matter: SQLite's LIKE
# optimization is decided at prepare time from the bound pattern, so a
# plan taken with no value would not be the plan the CLI gets.
QUERIES = [
    (
        "Store::search — acq items search / MCP search_items",
        "lib.rs Store::search",
        """SELECT id, league, location_kind, location_id, socketed_in, COALESCE(name, ''), COALESCE(type_line, ''),
                    COALESCE(base_type, ''), rarity, stack_size, first_seen, last_seen, removed_at, json, realm, container
               FROM items
              WHERE (name LIKE ?1 ESCAPE '\\' OR type_line LIKE ?1 ESCAPE '\\' OR base_type LIKE ?1 ESCAPE '\\')
                AND (?5 IS NULL OR realm = ?5)
                AND (?2 IS NULL OR league = ?2)
                AND (?3 OR removed_at IS NULL)
              ORDER BY location_kind, location_id, y, x
              LIMIT ?4""",
        ['%kaom%', None, 0, 50, None],
    ),
    (
        "the same predicate anchored (no leading wildcard) — what an index could serve",
        "not in the crate: the counterfactual",
        """SELECT id FROM items WHERE name LIKE 'Kaom%'""",
        [],
    ),
    (
        "Store::item — acq items show / MCP get_item",
        "lib.rs Store::item",
        """SELECT id, league, location_kind, location_id, socketed_in, COALESCE(name, ''), COALESCE(type_line, ''),
                    COALESCE(base_type, ''), rarity, stack_size, first_seen, last_seen, removed_at, json, realm, container FROM items WHERE id = ?1""",
        ['abc'],
    ),
    (
        "Store::tabs — acq tabs / MCP tabs (the per-tab live item count is a correlated subquery)",
        "lib.rs Store::tabs",
        f"""SELECT t.realm, t.league, t.id, t.parent, COALESCE(t.name, ''), COALESCE(t.type, ''), t.idx, t.listed_at, t.fetched_at, t.removed_at,
                    (SELECT count(*) FROM items i WHERE i.realm = t.realm AND i.league = t.league AND i.location_kind = 'stash' AND i.location_id = t.id AND i.removed_at IS NULL),
                    CASE WHEN t.fetched_at IS NULL THEN NULL
                         ELSE COALESCE(json_extract(t.json, '$.stash._split.items'), json_extract(t.json, '$._split.items')) END
               FROM tabs t WHERE t.realm = ?1 AND t.league = ?2 AND t.removed_at IS NULL {TAB_ORDER_SQL}""",
        ['pc', 'Standard'],
    ),
    (
        "Store::characters — acq store characters / MCP characters",
        "lib.rs Store::characters",
        """SELECT c.id, c.name, c.realm, c.league, c.class, c.level, c.listed_at, c.fetched_at,
                    (SELECT count(*) FROM items i WHERE i.realm = c.realm AND i.location_kind = 'character' AND i.location_id = c.id AND i.removed_at IS NULL),
                    CASE WHEN c.fetched_at IS NULL OR json_type(c.json, '$._split') IS NULL THEN NULL
                         ELSE (SELECT COALESCE(SUM(value), 0) FROM json_each(json_extract(c.json, '$._split'))) END
               FROM characters c
              WHERE c.removed_at IS NULL AND (?1 IS NULL OR c.realm = ?1) AND (?2 IS NULL OR c.league = ?2)
              ORDER BY c.realm, c.league, c.level DESC, c.name""",
        ['pc', 'Standard'],
    ),
    (
        "Store::events_since — acq store events / MCP item_events",
        "lib.rs Store::events_since",
        """SELECT e.at, e.item_id, e.kind, e.from_location, e.to_location, i.name, i.type_line
               FROM item_events e LEFT JOIN items i ON i.id = e.item_id
              WHERE e.at >= ?1 ORDER BY e.at, e.id LIMIT ?2""",
        [0, 200],
    ),
    (
        "read_items — the item body of PricingSnapshot (acq price, acq shop)",
        "snapshot.rs read_items",
        """SELECT i.id, i.location_kind, i.location_id, i.container, i.socketed_in,
                COALESCE(i.name, ''), COALESCE(i.type_line, ''), i.stack_size, i.x, i.y,
                json_extract(i.json, '$.note'), json_extract(i.json, '$.inventoryId'),
                i.seen_response, i.last_seen
           FROM items i
          WHERE i.realm = ?1 AND i.removed_at IS NULL
            AND ((i.location_kind = 'stash' AND i.league = ?2
                  AND EXISTS (SELECT 1 FROM tabs t WHERE t.realm = i.realm AND t.league = i.league
                                                     AND t.id = i.location_id AND t.removed_at IS NULL))
              OR (i.location_kind = 'character'
                  AND EXISTS (SELECT 1 FROM characters c WHERE c.realm = i.realm AND c.id = i.location_id
                                                           AND c.removed_at IS NULL AND (c.league = ?2 OR c.league IS NULL))))
          ORDER BY i.location_kind, i.location_id, i.y IS NULL, i.y, i.x, i.id""",
        ['pc', 'Standard'],
    ),
    (
        "the location filter alone — what items_location serves",
        "not in the crate: the counterfactual",
        """SELECT id FROM items WHERE location_kind = 'stash' AND location_id = ?1 AND removed_at IS NULL""",
        ['tab1'],
    ),
    (
        "stash_basis — the listing a plan cites",
        "snapshot.rs stash_basis",
        """SELECT id, fetched_at FROM responses
              WHERE endpoint = 'stashes' AND status BETWEEN 200 AND 299
                AND COALESCE(json_extract(params, '$.realm'), 'pc') = ?1
                AND COALESCE(json_extract(params, '$.league'), 'Standard') = ?2
              ORDER BY id DESC LIMIT 1""",
        ['pc', 'Standard'],
    ),
    (
        "exact name — the only shape items_names can serve (no crate read does this)",
        "not in the crate: the counterfactual",
        """SELECT id FROM items WHERE name = ?1""",
        ['Kaom\'s Heart'],
    ),
    (
        "anchored LIKE under case_sensitive_like=ON — why the default scans",
        "not in the crate: the counterfactual (pragma set for this query only)",
        """SELECT id FROM items WHERE name LIKE 'Kaom%'""",
        [],
    ),
    (
        "a whole league's live items — no index spans (realm, league, removed_at)",
        "not in the crate: the counterfactual",
        """SELECT id FROM items WHERE realm = ?1 AND league = ?2 AND removed_at IS NULL""",
        ['pc', 'Standard'],
    ),
]


def main():
    conn = sqlite3.connect(":memory:")
    conn.executescript(SCHEMA.read_text())
    lines = [
        "# generated by scripts/query-plans.py from "
        "crates/acquisition-store/src/schema.sql (schema applied to an empty",
        "# in-memory database) and the queries copied verbatim from lib.rs / snapshot.rs.",
        f"# sqlite {sqlite3.sqlite_version}; EXPLAIN QUERY PLAN, no data, no timings.",
        "",
    ]
    scans = 0
    for title, cite, sql, binds in QUERIES:
        pragma = "case_sensitive_like=ON" in title
        if pragma:
            conn.execute("PRAGMA case_sensitive_like=ON")
        try:
            plan = conn.execute("EXPLAIN QUERY PLAN " + sql, binds).fetchall()
        except sqlite3.Error as e:  # a stale copy would land here
            sys.exit(f"{title}: {e}")
        lines.append(f"## {title}")
        lines.append(f"   source: {cite}")
        for row in plan:
            detail = row[3]
            if detail.startswith("SCAN"):
                scans += 1
            lines.append(f"   {detail}")
        if pragma:
            conn.execute("PRAGMA case_sensitive_like=OFF")
        lines.append("")
    OUT.write_text("\n".join(lines))
    print(f"wrote {OUT.relative_to(TRACK)}: {len(QUERIES)} queries, {scans} SCAN steps")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
