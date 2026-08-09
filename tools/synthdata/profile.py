#!/usr/bin/env python3
"""Extract a statistics-only realism profile from a userstore database.

The profile captures the *shape* of an account — tab-type mix, items-per-tab
distributions, league mix, character levels, buyout density — as counts and
histograms only. It contains no ids, no names, no items, and no mod values,
so a profile extracted from a real account is committable.

Usage: profile.py <userstore.db> [output.json]
"""

import collections
import json
import sqlite3
import sys


def quantiles(values, points=(0, 0.25, 0.5, 0.75, 0.9, 1.0)):
    if not values:
        return []
    vs = sorted(values)
    return [vs[min(len(vs) - 1, int(p * (len(vs) - 1)))] for p in points]


def extract(db_path):
    db = sqlite3.connect(db_path)
    profile = {"source_schema_version": db.execute("PRAGMA user_version").fetchone()[0]}

    # League mix (public league names are not identifying; a private league
    # in a profile would be, so leagues are bucketed by rank, not name).
    # Top-level tabs only: generation picks a league once per top-level tab,
    # so counting Map/Unique child rows would inflate their league's weight.
    # Only content-bearing rows count anywhere in the profile: a row whose
    # json_data was never fetched would contribute a type/league share with
    # no items-per-tab quantiles, and _sample_count would then invent a
    # fill for it instead of reproducing the account's.
    league_tabs = collections.Counter(
        dict(db.execute("SELECT league, COUNT(*) FROM stashes"
                        " WHERE parent IS NULL AND json_data IS NOT NULL"
                        " GROUP BY league")))
    profile["league_tab_shares"] = [
        round(n / max(1, sum(league_tabs.values())), 4)
        for _, n in league_tabs.most_common()]

    # Tab-type mix over TOP-LEVEL tabs only: Map/Unique children are emitted
    # by their parents at generation time, so counting the (numerous) child
    # rows here would double-count them into the sampling weights.
    type_counts = collections.Counter(
        dict(db.execute("SELECT type, COUNT(*) FROM stashes"
                        " WHERE parent IS NULL AND json_data IS NOT NULL"
                        " GROUP BY type")))
    # Item-count distributions still cover every content-bearing row,
    # children included: a child tab's fill is a per-tab property.
    items_per_tab = collections.defaultdict(list)
    stack_fill = []
    for ttype, parent, blob in db.execute(
            "SELECT type, parent, json_data FROM stashes"
            " WHERE json_data IS NOT NULL"):
        # Top-level Map/Unique rows are structural parents whose payloads
        # carry "items": [] by design; counting those zeros would bias the
        # child fill distribution toward empty.
        if parent is None and ttype in ("MapStash", "UniqueStash"):
            continue
        d = json.loads(blob)
        items = d.get("items")
        if items is None:
            continue
        items_per_tab[ttype].append(len(items))
        for it in items:
            if isinstance(it.get("maxStackSize"), int) and it["maxStackSize"] > 1:
                stack_fill.append(round(it.get("stackSize", 1) / it["maxStackSize"], 3))
    total_tabs = sum(type_counts.values())
    profile["tab_type_shares"] = {
        t: round(n / max(1, total_tabs), 4) for t, n in type_counts.most_common()}
    profile["items_per_tab_quantiles"] = {
        t: quantiles(v) for t, v in items_per_tab.items()}
    # Counted per top-level parent so childless parents contribute zeros
    # rather than vanishing from the distribution.
    profile["children_per_parent_quantiles"] = {
        t: quantiles([n for (n,) in db.execute(
            "SELECT (SELECT COUNT(*) FROM stashes c WHERE c.parent = p.id)"
            " FROM stashes p WHERE p.parent IS NULL AND p.type = ?", (t,))])
        for t in ("MapStash", "UniqueStash")}
    profile["stack_fill_quantiles"] = quantiles(stack_fill)
    # Share of top-level, non-folder tabs that live inside a folder.
    profile["foldered_member_share"] = round(db.execute(
        "SELECT AVG(folder IS NOT NULL) FROM stashes"
        " WHERE parent IS NULL AND type != 'Folder'").fetchone()[0] or 0, 4)
    profile["public_share"] = round(
        db.execute("SELECT AVG(meta_public) FROM stashes").fetchone()[0] or 0, 4)

    # Characters.
    levels = [r[0] for r in db.execute(
        "SELECT json_extract(json_data, '$.level') FROM characters"
        " WHERE json_data IS NOT NULL") if r[0] is not None]
    profile["character_count"] = db.execute(
        "SELECT COUNT(*) FROM characters").fetchone()[0]
    profile["character_level_quantiles"] = quantiles(levels)

    # Buyout density. The location-buyout ratio is measured over exactly the
    # population generation applies it to: normal top-level tabs plus
    # Map/Unique children — never folders or structural parents — and only
    # stash-type location buyouts.
    n_items = sum(sum(v) for v in items_per_tab.values())
    n_buyouts = db.execute(
        "SELECT COUNT(*) FROM item_buyouts"
        " WHERE location_type = 'stash'").fetchone()[0]
    n_loc_buyouts = db.execute(
        "SELECT COUNT(*) FROM location_buyouts"
        " WHERE location_type = 'stash'").fetchone()[0]
    eligible_tabs = db.execute(
        "SELECT COUNT(*) FROM stashes WHERE parent IS NOT NULL"
        " OR type NOT IN ('Folder', 'MapStash', 'UniqueStash')").fetchone()[0]
    profile["item_buyout_share"] = round(n_buyouts / max(1, n_items), 5)
    profile["location_buyout_share"] = round(
        n_loc_buyouts / max(1, eligible_tabs), 5)
    profile["total_tabs"] = total_tabs
    profile["total_items"] = n_items
    return profile


if __name__ == "__main__":
    prof = extract(sys.argv[1])
    out = json.dumps(prof, indent=2)
    if len(sys.argv) > 2:
        open(sys.argv[2], "w").write(out + "\n")
        print(f"wrote {sys.argv[2]}")
    else:
        print(out)
