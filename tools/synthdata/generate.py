#!/usr/bin/env python3
"""Generate a synthetic Acquisition userstore from RePoE data and a profile.

Items are built from the same RePoE data the application consumes (plus
mods.min.json): released base items, mod pools filtered by spawn-weight tags,
values rolled inside real tier ranges, display text instantiated from the
mod's own template. Account shape (tab mix, items per tab, leagues, buyout
density) comes from a statistics-only profile extracted by profile.py.

Deterministic for a given (seed, scale, profile, RePoE version).

Usage:
  generate.py --profile veteran-standard.json --items 100000 --seed 42 \
      --sqlite out/userstore-SYNTH#0000.db [--json out/stashes.jsonl] \
      [--coverage] [--now 2026-08-01T12:00:00]

The synthetic account identity lives only in the output filename, which the
--sqlite path controls (the application derives userstore-<account>.db from
the logged-in account; use a fictional one like SYNTH#0000).
"""

import argparse
import json
import os
import random
import re
import sqlite3
import sys
from datetime import datetime, timedelta

import repoe_data

# ---------------------------------------------------------------------------
# RePoE indexing

EQUIP_CLASSES = {
    "Body Armour", "Boots", "Gloves", "Helmet", "Shield", "Belt", "Amulet",
    "Ring", "Quiver", "Claw", "Dagger", "Wand", "One Hand Sword",
    "Thrusting One Hand Sword", "One Hand Axe", "One Hand Mace", "Sceptre",
    "Bow", "Staff", "Two Hand Sword", "Two Hand Axe", "Two Hand Mace",
    "Warstaff", "Rune Dagger",
}
STACKABLE_CLASSES = {"StackableCurrency", "DivinationCard"}
MAP_CLASS = "Map"
FLASK_CLASSES = {"LifeFlask", "ManaFlask", "HybridFlask", "UtilityFlask"}
JEWEL_CLASSES = {"Jewel", "AbyssJewel"}

RARE_NAME_A = ["Storm", "Dire", "Ghoul", "Sol", "Dusk", "Ember", "Grim",
               "Rune", "Vex", "Hate", "Doom", "Gale", "Onyx", "Rapture",
               "Beast", "Corpse", "Blood", "Bramble", "Fate", "Skull"]
RARE_NAME_B = ["Grip", "Bane", "Song", "Veil", "Ward", "Roar", "Spiral",
               "Whisper", "Bite", "Coil", "Edge", "Fist", "Knell", "March",
               "Pledge", "Shroud", "Snare", "Spur", "Turn", "Vise"]

CHAR_CLASSES = ["Marauder", "Ranger", "Witch", "Duelist", "Templar",
                "Shadow", "Scion", "Juggernaut", "Deadeye", "Necromancer",
                "Champion", "Inquisitor", "Trickster", "Ascendant"]

INFLUENCES = ["shaper", "elder", "crusader", "redeemer", "hunter", "warlord"]

GRID = {"QuadStash": 24, "PremiumStash": 12, "NormalStash": 12}

# Live frameTypeId vocabulary (capitalized enum names). A small share of
# real cached items predate the field entirely and omit it — a legacy
# idiosyncrasy the generator reproduces (gerwaric, Aug 8: 3.10-era data).
FRAME_IDS = {0: "Normal", 1: "Magic", 2: "Rare", 3: "Unique", 4: "Gem",
             5: "Currency", 6: "DivinationCard", 7: "Quest", 9: "Foil",
             10: "SupporterFoil"}
RARITY_NAMES = {0: "Normal", 1: "Magic", 2: "Rare", 3: "Unique"}
# Socket sColour -> socketed-gem colour letter (S=str, D=dex, I=int, G=white).
GEM_COLOUR = {"R": "S", "G": "D", "B": "I", "W": "G"}


def icon_url(base):
    dds = (base.get("visual_identity") or {}).get("dds_file")
    if not dds:
        return None
    return "https://web.poecdn.com/image/" + dds.rsplit(".", 1)[0] + ".png"


class RepoeIndex:
    def __init__(self, data):
        self.version = data["version"]
        self.dir = data["dir"]
        released = {
            k: v for k, v in data["base_items"].items()
            if v.get("release_state") == "released" and v.get("name")
        }
        self.by_class = {}
        for v in released.values():
            self.by_class.setdefault(v["item_class"], []).append(v)
        self.equip = [v for v in released.values()
                      if v.get("domain") == "item" and v["item_class"] in EQUIP_CLASSES]
        self.maps = self.by_class.get(MAP_CLASS, [])
        self.stackables = [v for v in released.values()
                           if v["item_class"] in STACKABLE_CLASSES]
        self.mods = data["mods"]
        # Prefix/suffix pools indexed by mod domain, so flask and jewel bases
        # draw from their own domains rather than item-domain mods that merely
        # share the "default" spawn tag.
        # NOTE: repoe-fork's mods.min.json carries a "text" display template on
        # ~99.9% of item prefixes/suffixes (upstream RePoE does not); mod text
        # is instantiated from it rather than from stat_translations.
        self.mods_by_domain = {}
        for m in self.mods.values():
            if m.get("generation_type") in ("prefix", "suffix") \
                    and not m.get("is_essence_only") and m.get("spawn_weights"):
                self.mods_by_domain.setdefault(m.get("domain"), []).append(m)

    def eligible_mods(self, base, ilvl):
        tags = set(base.get("tags") or [])
        out = []
        for m in self.mods_by_domain.get(base.get("domain"), []):
            if m.get("required_level", 1) > ilvl:
                continue
            # RePoE spawn-weight semantics: first listed tag present on the
            # item decides; weight 0 means cannot spawn.
            for sw in m["spawn_weights"]:
                if sw["tag"] in tags:
                    if sw["weight"] > 0:
                        out.append(m)
                    break
        return out

    def pool(self, *classes, name_contains=None):
        out = []
        for c in classes:
            out.extend(self.by_class.get(c, []))
        if name_contains:
            out = [b for b in out
                   if any(s in b["name"] for s in name_contains)]
        return out


# ---------------------------------------------------------------------------
# Item factory

class ItemFactory:
    def __init__(self, index, rng):
        self.ix = index
        self.rng = rng
        self.counter = 0

    def _id(self):
        self.counter += 1
        return self.rng.getrandbits(256).to_bytes(32, "big").hex()

    # Both bounds may be negative: (3-9), (-9--5), (-20--10), (0.5-1.5).
    RANGE = re.compile(r"\((-?\d+(?:\.\d+)?)-(-?\d+(?:\.\d+)?)\)")

    def _roll_text(self, template):
        """Instantiate a mod template: each '(lo-hi)' becomes a rolled value."""
        def roll(m):
            lo, hi = float(m.group(1)), float(m.group(2))
            v = self.rng.uniform(lo, hi)
            if lo == int(lo) and hi == int(hi):
                return str(int(round(v)))
            return f"{v:.2f}".rstrip("0").rstrip(".")
        return self.RANGE.sub(roll, template)

    def _mods_for(self, base, ilvl, n_prefix, n_suffix):
        pool = self.ix.eligible_mods(base, ilvl)
        self.rng.shuffle(pool)
        chosen, groups = [], set()
        counts = {"prefix": n_prefix, "suffix": n_suffix}
        for m in pool:
            g = m["generation_type"]
            if counts[g] <= 0:
                continue
            mg = tuple(m.get("groups") or [m.get("type")])
            if any(x in groups for x in mg):
                continue
            groups.update(mg)
            counts[g] -= 1
            chosen.append(m)
            if counts["prefix"] <= 0 and counts["suffix"] <= 0:
                break
        return chosen

    def _sockets(self, base, ilvl):
        cls = base["item_class"]
        if cls in ("Ring", "Amulet", "Belt", "Quiver") or cls in JEWEL_CLASSES \
                or cls in FLASK_CLASSES:
            return None
        # Live socket caps are class-based: 6 for body armour and two-handers,
        # 4 for helmets/gloves/boots, 3 for one-hand weapons and shields.
        if cls in ("Body Armour", "Bow", "Staff", "Two Hand Sword",
                   "Two Hand Axe", "Two Hand Mace", "Warstaff"):
            class_cap = 6
        elif cls in ("Helmet", "Gloves", "Boots"):
            class_cap = 4
        else:
            class_cap = 3
        # Item-level socket caps: 2 at ilvl 1, 3 from 2, 4 from 25, 5 from
        # 35, 6 from 50.
        max_sockets = min(
            class_cap,
            {1: 2, 2: 3, 25: 4, 35: 5, 50: 6}.get(
                max(k for k in (1, 2, 25, 35, 50) if k <= max(1, ilvl)), 2))
        n = self.rng.randint(0, max_sockets)
        if n == 0:
            return None
        req = base.get("requirements") or {}
        weights = [max(1, req.get("strength", 0)),
                   max(1, req.get("dexterity", 0)),
                   max(1, req.get("intelligence", 0))]
        sockets, group = [], 0
        for i in range(n):
            colour = self.rng.choices(["R", "G", "B"], weights=weights)[0]
            if i and self.rng.random() > 0.6:
                group += 1
            sockets.append({"group": group, "attr": {"R": "S", "G": "D", "B": "I"}[colour],
                            "sColour": colour})
        return sockets

    def _properties(self, base, quality):
        props = []
        for pname, p in (base.get("properties") or {}).items():
            if not isinstance(p, dict) or "min" not in p:
                continue
            v = self.rng.randint(p["min"], p["max"]) if p["max"] >= p["min"] else p["min"]
            if quality and pname in ("armour", "evasion", "energy_shield"):
                v = int(v * (1 + quality / 100))
            label = pname.replace("_", " ").title().replace("Energy Shield", "Energy Shield")
            props.append({"name": label, "values": [[str(v), 0]], "displayMode": 0})
        if quality:
            props.append({"name": "Quality", "values": [[f"+{quality}%", 1]],
                          "displayMode": 0, "type": 6})
        return props

    def _requirements(self, base):
        req = base.get("requirements") or {}
        out = []
        for key, label in (("level", "Level"), ("strength", "Str"),
                           ("dexterity", "Dex"), ("intelligence", "Int")):
            v = req.get(key) or 0
            if v > 1:
                out.append({"name": label, "values": [[str(v), 0]], "displayMode": 1})
        return out

    def equipment(self, league, ilvl=None, rarity=None, axes=None, cls=None,
                  max_req_level=None):
        pool = self.ix.by_class.get(cls) if cls else self.ix.equip
        if max_req_level is not None:
            capped = [b for b in pool
                      if (b.get("requirements") or {}).get("level", 0) <= max_req_level]
            # Never empty: fall back to the lowest-requirement base.
            pool = capped or [min(pool, key=lambda b:
                                  (b.get("requirements") or {}).get("level", 0))]
        base = self.rng.choice(pool)
        ilvl = ilvl or self.rng.randint(max(1, base.get("drop_level", 1)), 86)
        rarity = rarity if rarity is not None else self.rng.choices(
            [0, 1, 2], weights=[15, 35, 50])[0]
        n_pre, n_suf = [(0, 0), (self.rng.randint(0, 1), self.rng.randint(0, 1)),
                        (self.rng.randint(1, 3), self.rng.randint(1, 3)),
                        (self.rng.randint(1, 3), self.rng.randint(1, 3))][rarity]
        if rarity == 1 and n_pre + n_suf == 0:
            n_suf = 1
        mods = self._mods_for(base, ilvl, n_pre, n_suf)
        quality = self.rng.choice([0, 0, 0, 10, 20])
        item = {
            "verified": True,
            "w": base.get("inventory_width", 1),
            "h": base.get("inventory_height", 1),
            "icon": icon_url(base),
            "league": league,
            "id": self._id(),
            "name": (self.rng.choice(RARE_NAME_A) + " " + self.rng.choice(RARE_NAME_B))
                    if rarity >= 2 else "",
            "typeLine": base["name"],
            "baseType": base["name"],
            "identified": True,
            "ilvl": ilvl,
            "frameType": rarity,
        }
        # Legacy cache entries omit frameTypeId; most modern items carry
        # rarity as a string alongside frameType.
        if self.rng.random() > 0.002:
            item["frameTypeId"] = FRAME_IDS[rarity]
        if self.rng.random() > 0.2:
            item["rarity"] = RARITY_NAMES[rarity]
        # 3.29 wire format: implicit/explicit mods are ItemMod objects with the
        # full (possibly multi-line, \r\n) text in "description"; crafted and
        # fractured are flags on the mod, not separate arrays.
        implicits = [{"description":
                      self._roll_text(self.ix.mods[m]["text"]).replace("\n", "\r\n")}
                     for m in (base.get("implicits") or [])
                     if m in self.ix.mods and self.ix.mods[m].get("text")]
        if implicits:
            item["implicitMods"] = implicits
        if mods:
            explicit = []
            for m in mods:
                if not m.get("text"):
                    continue
                entry = {"description": self._roll_text(m["text"]).replace("\n", "\r\n")}
                if self.rng.random() < 0.05:
                    entry["flags"] = {"crafted": True}
                explicit.append(entry)
            item["explicitMods"] = explicit
        props = self._properties(base, quality)
        if props:
            item["properties"] = props
        reqs = self._requirements(base)
        if reqs:
            item["requirements"] = reqs
        sockets = self._sockets(base, ilvl)
        if sockets:
            item["sockets"] = sockets
            socketed = []
            for idx, sock in enumerate(sockets):
                if self.rng.random() < 0.15:
                    g = self.gem(league)
                    g["socket"] = idx
                    g["colour"] = GEM_COLOUR.get(sock["sColour"], "G")
                    socketed.append(g)
            if socketed:
                item["socketedItems"] = socketed
        for axis, value in (axes or {}).items():
            item[axis] = value
        return item

    def stackable(self, league, pool=None, fill=None):
        base = self.rng.choice(pool or self.ix.stackables)
        max_stack = base.get("stack_size") or (10 if base["item_class"] == "DivinationCard" else 5000)
        stack = self.rng.randint(1, max_stack) if fill is None \
            else min(max_stack, max(1, round(fill * max_stack)))
        item = {
            "verified": True, "w": 1, "h": 1, "icon": icon_url(base),
            "league": league, "id": self._id(), "name": "",
            "typeLine": base["name"], "baseType": base["name"],
            "identified": True, "ilvl": 0,
            "stackSize": stack,
            "maxStackSize": max_stack,
            # Real wire data carries the display property too; the app's
            # Item::m_count comes from this "n/m" string, not stackSize.
            "properties": [{"name": "Stack Size",
                            "values": [[f"{stack}/{max_stack}", 0]],
                            "displayMode": 0, "type": 32}],
            "frameType": 6 if base["item_class"] == "DivinationCard" else 5,
        }
        if self.rng.random() > 0.01:
            item["frameTypeId"] = FRAME_IDS[item["frameType"]]
        return item

    def gem(self, league):
        base = self.rng.choice(self.ix.pool("Active Skill Gem", "Support Skill Gem"))
        return {
            "verified": True, "w": 1, "h": 1, "icon": icon_url(base),
            "support": base["item_class"] == "Support Skill Gem",
            "league": league, "id": self._id(), "name": "",
            "typeLine": base["name"], "baseType": base["name"],
            "identified": True, "ilvl": 0,
            "properties": [{"name": "Level", "values":
                            [[str(self.rng.randint(1, 21)), 0]], "displayMode": 0}],
            "frameType": 4, "frameTypeId": "Gem",
        }

    def map_item(self, league):
        # Built directly rather than via equipment(): equipment mods cannot
        # spawn on maps, and map (area-domain) mods are not needed for the
        # coverage this tool targets, so maps carry no explicit mods at all.
        base = self.rng.choice(self.ix.maps)
        tier = self.rng.randint(1, 16)
        item = {
            "verified": True, "w": 1, "h": 1, "icon": icon_url(base),
            "league": league, "id": self._id(), "name": "",
            "typeLine": base["name"], "baseType": base["name"],
            "identified": True, "ilvl": 67 + tier,
            "properties": [{"name": "Map Tier", "values": [[str(tier), 0]],
                            "displayMode": 0}],
            "frameType": 0, "frameTypeId": "Normal",
        }
        if self.rng.random() < 0.1:
            item["corrupted"] = True
        return item


# ---------------------------------------------------------------------------
# Stash/character assembly

# Must match profile.py's quantiles() default points.
QUANTILE_POINTS = (0, 0.25, 0.5, 0.75, 0.9, 1.0)


def sample_quantiles(rng, qs):
    """Draw from the piecewise-linear inverse CDF through profile quantiles.

    Profiles record values at uneven percentiles (QUANTILE_POINTS); picking
    a landmark uniformly would give the 90-100 tail the same weight as each
    quartile, inflating tab density. Returns a float; callers round.
    """
    if len(qs) != len(QUANTILE_POINTS):
        return rng.choice(qs)
    p = rng.random()
    for p0, v0, p1, v1 in zip(QUANTILE_POINTS, qs, QUANTILE_POINTS[1:], qs[1:]):
        if p <= p1:
            return v0 + (p - p0) / (p1 - p0) * (v1 - v0)
    return qs[-1]


class Assembler:
    def __init__(self, factory, profile, leagues, rng, now):
        self.f = factory
        self.p = profile
        self.leagues = leagues
        self.rng = rng
        self.now = now
        self.stash_rows = []
        self.item_buyouts = []
        self.location_buyouts = []
        self.char_rows = []
        self._used_stash_ids = set()

    def _stash_id(self):
        # 40 bits collide with high probability at million-tab scale
        # (birthday bound), and stashes.id is a primary key.
        while True:
            sid = self.rng.getrandbits(40).to_bytes(5, "big").hex()
            if sid not in self._used_stash_ids:
                self._used_stash_ids.add(sid)
                return sid

    def _sample_count(self, ttype, cap):
        qs = (self.p.get("items_per_tab_quantiles") or {}).get(ttype)
        if not qs:
            return self.rng.randint(cap // 4, cap)
        # Zero is a legitimate sample: real accounts hold many empty tabs.
        return min(cap, round(sample_quantiles(self.rng, qs)))

    def _pack(self, items, grid, height=None):
        """Shelf-pack items into a grid; drops what no longer fits."""
        x = y = shelf_h = 0
        placed = []
        max_y = height if height is not None else grid
        for it in items:
            w, h = it.get("w", 1), it.get("h", 1)
            if x + w > grid:
                x, y = 0, y + shelf_h
                shelf_h = 0
            if y + h > max_y:
                break
            it["x"], it["y"] = x, y
            x += w
            shelf_h = max(shelf_h, h)
            placed.append(it)
        return placed

    # Buyout timestamps are ISO-8601 text on purpose: the schema declares the
    # column INTEGER, but BuyoutRepo binds a QDateTime (stored as text) and
    # reads it back with QVariant::toDateTime, so production databases hold
    # ISO strings there. Match the app, not the column affinity.
    def _maybe_buyout(self, item, stash_id):
        share = self.p.get("item_buyout_share", 0.01)
        if self.rng.random() < share:
            cur = self.rng.choice(["chaos", "divine", "exalted"])
            val = float(self.rng.choice([1, 2, 5, 10, 20, 50, 100]))
            note = f"~price {int(val)} {cur}"
            item["note"] = note
            self.item_buyouts.append(
                (item["id"], stash_id, "stash", cur, 0,
                 self.now.isoformat(timespec="milliseconds"), "manual", "b/o", val))

    def _maybe_location_buyout(self, stash_id):
        if self.rng.random() < self.p.get("location_buyout_share", 0.0):
            cur = self.rng.choice(["chaos", "divine", "exalted"])
            self.location_buyouts.append(
                (stash_id, "stash", cur, 0,
                 self.now.isoformat(timespec="milliseconds"), "manual", "b/o",
                 float(self.rng.choice([1, 5, 10, 20]))))

    def _row(self, sid, league, name, ttype, index, payload, parent=None,
             folder=None, colour=None):
        ts = (self.now - timedelta(seconds=self.rng.randint(0, 86400))) \
            .isoformat(timespec="milliseconds")
        return (sid, "pc", league, parent, folder, name, ttype, index,
                1 if self.rng.random() < self.p.get("public_share", 0.1) else 0,
                1 if ttype == "Folder" else 0, colour, ts, ts,
                json.dumps(payload, separators=(",", ":")), 2)

    def folder(self, league, name, index):
        """A Folder row, mirroring the live shape: no items, folder metadata."""
        fid = self._stash_id()
        payload = {"id": fid, "name": name, "type": "Folder", "index": index,
                   "metadata": {"folder": True, "colour": "7c5436"}}
        self.stash_rows.append(
            self._row(fid, league, name, "Folder", index, payload,
                      colour="7c5436"))
        return fid

    def _maker_for(self, ttype, league):
        """Type-appropriate contents: dedicated stashes never hold equipment."""
        ix = self.f.ix
        stack_pools = {
            # Currency tabs hold currency only; the mixed stackables pool
            # would let divination cards in.
            "CurrencyStash": lambda: ix.pool("StackableCurrency"),
            "DivinationCardStash": lambda: ix.pool("DivinationCard"),
            "EssenceStash": lambda: ix.pool(
                "StackableCurrency", name_contains=["Essence", "Remnant of"]),
            "FragmentStash": lambda: ix.pool("MapFragment", "Breachstone", "MapKey"),
            "DelveStash": lambda: ix.pool(
                "DelveSocketableCurrency", "DelveStackableSocketableCurrency"),
            "BlightStash": lambda: ix.pool("StackableCurrency", name_contains=["Oil"]),
            "DeliriumStash": lambda: ix.pool(
                "StackableCurrency", name_contains=["Delirium Orb", "Simulacrum"]),
        }
        if ttype in stack_pools:
            pool = stack_pools[ttype]() or None
            fills = self.p.get("stack_fill_quantiles") or None
            return lambda: self.f.stackable(
                league, pool=pool,
                fill=sample_quantiles(self.rng, fills) if fills else None)
        if ttype == "GemStash":
            return lambda: self.f.gem(league)
        if ttype == "FlaskStash":
            # Flasks can only be normal or magic (uniques aside).
            return lambda: self.f.equipment(
                league, cls=self.rng.choice(sorted(FLASK_CLASSES)),
                rarity=self.rng.choices([0, 1], weights=[30, 70])[0])
        if ttype in GRID:
            return lambda: self.f.equipment(league)
        # A dedicated stash type this table does not model (MetamorphStash,
        # UltimatumStash, ...) cannot hold equipment; currency-like stackables
        # are the least-wrong filler until the type is modeled explicitly.
        pool = ix.pool("StackableCurrency")
        return lambda: self.f.stackable(league, pool=pool)

    def normal_tab(self, league, ttype, name, index, folder=None):
        sid = self._stash_id()
        make = self._maker_for(ttype, league)
        if ttype in GRID:
            grid = GRID[ttype]
            n = self._sample_count(ttype, grid * grid)
            items = self._pack([make() for _ in range(n)], grid)
        else:
            # Dedicated tabs are slot-based, not grid-packed, and hold far
            # more than a 12x12 grid (a currency tab has thousands of
            # slots); capping them at 144 would silently truncate profiled
            # populations.
            n = self._sample_count(ttype, 5000)
            items = [make() for _ in range(n)]
            for slot, it in enumerate(items):
                it["x"], it["y"] = slot, 0
        for it in items:
            self._maybe_buyout(it, sid)
        self._maybe_location_buyout(sid)
        colour = "%06x" % self.rng.getrandbits(24)
        payload = {"id": sid, "name": name, "type": ttype, "index": index,
                   "metadata": {"colour": colour},
                   "items": items}
        self.stash_rows.append(
            self._row(sid, league, name, ttype, index, payload, folder=folder,
                      colour=colour))
        return len(items)

    def special_parent(self, league, ttype, name, index, n_children,
                       folder=None):
        parent_id = self._stash_id()
        children_meta, emitted = [], 0
        for ci in range(n_children):
            cid = self._stash_id()
            cname = f"{ci + 1} (Remove-only)" if self.rng.random() < 0.5 else str(ci + 1)
            n = self._sample_count(ttype, 144)
            make = (lambda: self.f.map_item(league)) if ttype == "MapStash" \
                else (lambda: self.f.equipment(league, rarity=3))
            items = self._pack([make() for _ in range(n)], 12)
            for it in items:
                self._maybe_buyout(it, cid)
            self._maybe_location_buyout(cid)
            child_payload = {"id": cid, "parent": parent_id, "name": cname,
                             "type": ttype, "metadata": {"items": len(items)},
                             "items": items}
            self.stash_rows.append(
                self._row(cid, league, cname, ttype, None, child_payload,
                          parent=parent_id))
            children_meta.append({"id": cid, "parent": parent_id, "name": cname,
                                  "type": ttype, "metadata": {"items": len(items)}})
            emitted += len(items)
        payload = {"id": parent_id, "name": name, "type": ttype, "index": index,
                   "metadata": {"colour": "7c5436", "layout": {}},
                   "items": [], "children": children_meta}
        self.stash_rows.append(self._row(parent_id, league, name, ttype, index,
                                         payload, folder=folder,
                                         colour="7c5436"))
        return emitted

    def character(self, league, name):
        level = round(sample_quantiles(
            self.rng, self.p.get("character_level_quantiles") or [70]))
        cid = self.f._id()
        slots = [("Helm", "Helmet"), ("BodyArmour", "Body Armour"),
                 ("Gloves", "Gloves"), ("Boots", "Boots"), ("Belt", "Belt"),
                 ("Amulet", "Amulet"), ("Ring", "Ring"), ("Ring2", "Ring")]
        equipment = []
        for inv_id, cls in slots:
            if not self.f.ix.by_class.get(cls):
                continue
            # Roll from the slot's own base so mods, properties, requirements,
            # and sockets all belong to the item actually worn there, and cap
            # the base's level requirement at the character's level.
            it = self.f.equipment(league, ilvl=min(86, level), rarity=2, cls=cls,
                                  max_req_level=level)
            it.update({"inventoryId": inv_id, "x": 0, "y": 0})
            equipment.append(it)
        # The live character inventory is 12 wide by 5 tall.
        inventory = self._pack(
            [self.f.equipment(league) for _ in range(self.rng.randint(0, 20))],
            12, height=5)
        payload = {"id": cid, "name": name, "realm": "pc", "class":
                   self.rng.choice(CHAR_CLASSES), "league": league,
                   "level": level, "experience": level * 10_000_000,
                   "current": False, "equipment": equipment,
                   "inventory": inventory, "jewels": [],
                   "metadata": {"version": "3.29.0"}}
        ts = self.now.isoformat(timespec="milliseconds")
        self.char_rows.append((cid, name, "pc", league, ts, ts,
                               json.dumps(payload, separators=(",", ":")), 2))
        return len(equipment) + len(inventory)


# ---------------------------------------------------------------------------
# Coverage tabs: a systematic sweep of src/poe/types/item.h — one deliberate
# item per optional field, with type-correct plausible values. The header is
# the app's accumulated catalog of everything GGG has ever sent (developer
# docs plus years of real payloads back to 3.10 and earlier), so sweeping it
# is the closest available approximation of historical item diversity.
# Shapes nobody has modeled yet are quirks.py's job to detect, not this
# list's job to guess.

PROP = [{"name": "Coverage", "values": [["1", 0]], "displayMode": 0}]


def coverage_axes():
    axes = []
    # Influences: each singly and paired.
    for inf in INFLUENCES:
        axes.append({"influences": {inf: True}})
    for a, b in [("shaper", "elder"), ("crusader", "hunter"),
                 ("redeemer", "warlord")]:
        axes.append({"influences": {a: True, b: True}})
    # Always-true-if-present booleans, one item each (PoE1-era history:
    # legacy elder/shaper flags, race rewards, veiled, delve, ...).
    for flag in ["elder", "shaper", "searing", "tangled", "memoryItem",
                 "mutated", "vestigial", "abyssJewel", "delve", "fractured",
                 "synthesised", "lockedToCharacter", "lockedToAccount",
                 "duplicated", "split", "corrupted", "unmodifiable",
                 "unmodifiableExceptChaos", "cisRaceReward", "seaRaceReward",
                 "thRaceReward", "veiled", "isRelic", "replica",
                 "foreseeing", "ruthless"]:
        axes.append({flag: True})
    axes += [
        {"identified": False},
        {"foilVariation": 1, "frameType": 9, "frameTypeId": "Foil"},
        {"frameType": 7, "frameTypeId": "Quest"},
        {"frameType": 10, "frameTypeId": "SupporterFoil"},
        # Scalar/string fields.
        {"itemLevel": 100}, {"monsterLevel": 84},
        {"iconTierText": "IV"}, {"iconStackLevel": "1"},
        {"stackSizeText": "2/10"},
        {"builtInSupport": "Supported by Level 1 Multistrike"},
        {"artFilename": "CoverageCard"},
        {"secDescrText": "Coverage secondary description"},
        {"note": "~b/o 5 chaos", "forum_note": "~price 5 chaos"},
        {"flavourText": ["\"Coverage flavour,\r", "line two.\""]},
        {"flavourTextNote": ["coverage note"]},
        {"prophecyText": ["You will encounter coverage."]},
        # Mod arrays and mod-flag encodings, including GGG's empty-array
        # flags quirk (their serializer encodes no-flags as [] — see the
        # custom reader in itemmod.h).
        {"enchantMods": ["Trigger Commandment of Fury on Hit"]},
        {"utilityMods": ["40% increased Movement Speed"]},
        {"cosmeticMods": ["Has Coverage Skin"]},
        {"veiledMods": ["SModCoverage1"], "veiled": True},
        {"scourgeMods": ["Coverage scourge mod"], "scourged": {"tier": 1}},
        {"crucibleMods": ["Coverage crucible mod"]},
        {"explicitMods": [{"description": "+13 to maximum Life",
                           "flags": {"fractured": True}}], "fractured": True},
        {"explicitMods": [{"description": "+18% to Quality",
                           "flags": {"crafted": True}}]},
        {"explicitMods": [{"description": "+1 coverage mod", "flags": []}]},
        # Structured objects, minimal correct shapes per the item.h structs.
        {"rewards": [{"label": "Coverage", "rewards": {"currency": "3"}}]},
        {"logbookMods": [{"name": "Coverage Area",
                          "faction": {"id": "Faction1", "name": "Order of the Chalice"},
                          "mods": ["Coverage logbook mod"]}]},
        {"ultimatumMods": [{"type": "CoverageChallenge", "tier": 3}]},
        {"mercenarySkills": [{"hash": 1, "icon": "cov.png", "name": "Coverage Strike",
                              "supports": [{"hash": 2, "name": "Coverage Support",
                                            "tier": 1}]}]},
        {"incubatedItem": {"name": "Coverage Item", "level": 68,
                           "progress": 10, "total": 100}},
        {"enshrouded": {"progress": 1, "total": 9}},
        {"scourged": {"tier": 2, "level": 68, "progress": 5, "total": 30}},
        {"crucible": {"layout": "https://web.poecdn.com/crucible/coverage.png",
                      "nodes": {"0": {"skill": 100, "tier": 1, "allocated": True,
                                      "stats": ["Coverage crucible stat"],
                                      "orbit": 0, "orbitIndex": 0,
                                      "out": [], "in": []}}}},
        {"hybrid": {"isVaalGem": True, "baseTypeName": "Coverage Gem",
                    "explicitMods": ["Coverage hybrid mod"],
                    "secDescrText": "Coverage hybrid text"}},
        {"extended": {"prefixes": 2, "suffixes": 1}},
        {"notableProperties": PROP}, {"additionalProperties": PROP},
        {"nextLevelRequirements": PROP},
        # PoE2-only parse paths (the app models these; realm flows stay pc).
        {"realm": "poe2", "gemSockets": ["W", "W"]},
        {"realm": "poe2", "runeMods": ["+25 to maximum Life"],
         "sockets": [{"group": 0, "type": "rune", "item": "rune"}]},
        {"realm": "poe2", "sanctified": True},
        {"realm": "poe2", "doubleCorrupted": True, "corrupted": True},
        {"realm": "poe2", "unidentifiedTier": 2, "identified": False},
        {"realm": "poe2", "desecrated": True},
        {"realm": "poe2", "bondedMods": ["Coverage bonded mod"]},
        {"realm": "poe2", "weaponRequirements": PROP},
        {"realm": "poe2", "supportGemRequirements": PROP},
        {"realm": "poe2", "tamedBeastProperties": PROP},
        {"realm": "poe2", "grantedSkills": PROP},
        {"realm": "poe2", "gemTabs": [{"name": "Coverage", "pages": []}],
         "gemBackground": "coverage", "gemSkill": "CoverageSkill"},
        {"realm": "poe2", "socketedIcon": "https://web.poecdn.com/cov.png"},
    ]
    return axes


# ---------------------------------------------------------------------------
# Curated quirk registry (quirks-registry.json): hand-picked historical wire
# idiosyncrasies with provenance, injected as their own tab. An entry's
# "item" object merges into a generated rare (a null value DELETES the key,
# expressing predates-the-field quirks); "tab" objects merge into the Quirks
# tab metadata and row.

def load_quirk_registry():
    import pathlib
    path = pathlib.Path(__file__).resolve().parent / "quirks-registry.json"
    return json.loads(path.read_text())["quirks"]


def emit_quirks_tab(factory, asm, index, probes=None):
    """Emit the Quirks item tab plus one empty tab per tab-level quirk.

    Returns (items_emitted, tabs_emitted). Item quirks merge into a
    generated rare (null deletes a key); tab quirks each get their own
    tab so metadata idiosyncrasies (short colours, empty or duplicated
    names) do not overwrite one another.
    """
    quirks = load_quirk_registry()
    items = []
    tab_quirks = []
    for q in quirks:
        for t in q.get("tabs", [q["tab"]] if "tab" in q else []):
            tab_quirks.append((q["id"], t))
        if "item" not in q:
            continue
        it = factory.equipment("Standard", rarity=2)
        it["w"] = it["h"] = 1
        it["x"], it["y"] = len(items) % 24, len(items) // 24
        for k, v in q["item"].items():
            if v is None:
                it.pop(k, None)
            elif v == "__null__":
                it[k] = None
            else:
                it[k] = v
        items.append(it)
    assert len(items) <= 24 * 24, "quirk registry exceeds one quad tab"
    sid = asm._stash_id()
    if probes is not None:
        for q, it in zip([q for q in quirks if "item" in q], items):
            probes.append({"id": q["id"], "kind": "quirk-item",
                           "item_id": it["id"], "stash_id": sid,
                           "item_json": json.dumps(it, separators=(",", ":"))})
    payload = {"id": sid, "name": "Quirks", "type": "QuadStash",
               "index": index, "metadata": {"colour": "00cc00"},
               "items": items}
    asm.stash_rows.append(asm._row(sid, "Standard", "Quirks", "QuadStash",
                                   index, payload, colour="00cc00"))
    tabs = 1
    for qid, t in tab_quirks:
        index += 1
        tabs += 1
        sid = asm._stash_id()
        name = t.get("name", qid)
        meta = {"colour": t["colour"]} if "colour" in t else {}
        payload = {"id": sid, "name": name, "type": "NormalStash",
                   "index": index, "metadata": meta, "items": []}
        asm.stash_rows.append(asm._row(sid, "Standard", name, "NormalStash",
                                       index, payload,
                                       colour=meta.get("colour")))
        if probes is not None:
            probes.append({"id": qid, "kind": "quirk-tab", "stash_id": sid,
                           "name": name, "colour": meta.get("colour")})
    return len(items), tabs


# ---------------------------------------------------------------------------
# Emitters

SCHEMA = """
CREATE TABLE characters (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    realm           TEXT NOT NULL,
    league          TEXT,
    listed_at       TEXT NOT NULL,
    json_fetched_at TEXT,
    json_data       TEXT,
    json_version    INTEGER);
CREATE TABLE stashes (
    id              TEXT PRIMARY KEY,
    realm           TEXT NOT NULL,
    league          TEXT NOT NULL,
    parent          TEXT,
    folder          TEXT,
    name            TEXT NOT NULL,
    type            TEXT NOT NULL,
    stash_index     INTEGER,
    meta_public     INTEGER NOT NULL CHECK (meta_public IN (0,1)),
    meta_folder     INTEGER NOT NULL CHECK (meta_folder IN (0,1)),
    meta_colour     TEXT,
    listed_at       TEXT,
    json_fetched_at TEXT,
    json_data       TEXT,
    json_version    INTEGER);
CREATE INDEX idx_stashes_realm_league_parent ON stashes(realm, league, parent);
CREATE INDEX idx_stashes_realm_league_folder ON stashes(realm, league, folder);
CREATE TABLE item_buyouts (
    item_id         TEXT PRIMARY KEY,
    location_id     TEXT NOT NULL,
    location_type   TEXT NOT NULL CHECK (location_type IN ('character', 'stash')),
    currency        TEXT NOT NULL,
    inherited       INTEGER NOT NULL CHECK (inherited IN (0,1)),
    last_update     INTEGER NOT NULL,
    source          TEXT NOT NULL,
    type            TEXT NOT NULL,
    value           REAL NOT NULL);
CREATE TABLE location_buyouts (
    location_id     TEXT PRIMARY KEY,
    location_type   TEXT NOT NULL CHECK (location_type IN ('character', 'stash')),
    currency        TEXT NOT NULL,
    inherited       INTEGER NOT NULL CHECK (inherited IN (0,1)),
    last_update     INTEGER NOT NULL,
    source          TEXT NOT NULL,
    type            TEXT NOT NULL,
    value           REAL NOT NULL);
"""


def emit_sqlite(path, asm):
    import os
    if os.path.dirname(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
    if os.path.exists(path):
        os.remove(path)
    db = sqlite3.connect(path)
    db.executescript(SCHEMA)
    db.execute("PRAGMA user_version = 3")
    db.executemany(
        "INSERT INTO stashes (id, realm, league, parent, folder, name, type,"
        " stash_index, meta_public, meta_folder, meta_colour, listed_at,"
        " json_fetched_at, json_data, json_version)"
        " VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)", asm.stash_rows)
    db.executemany(
        "INSERT INTO characters (id, name, realm, league, listed_at,"
        " json_fetched_at, json_data, json_version) VALUES (?,?,?,?,?,?,?,?)",
        asm.char_rows)
    db.executemany(
        "INSERT INTO item_buyouts (item_id, location_id, location_type,"
        " currency, inherited, last_update, source, type, value)"
        " VALUES (?,?,?,?,?,?,?,?,?)", asm.item_buyouts)
    db.executemany(
        "INSERT INTO location_buyouts (location_id, location_type,"
        " currency, inherited, last_update, source, type, value)"
        " VALUES (?,?,?,?,?,?,?,?)", asm.location_buyouts)
    db.commit()
    db.close()


def emit_jsonl(path, asm):
    import os
    if os.path.dirname(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        for row in asm.stash_rows:
            f.write(row[13] + "\n")


# ---------------------------------------------------------------------------

def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--profile", required=True)
    ap.add_argument("--items", type=int, default=100_000,
                    help="approximate minimum stash-item count: generation"
                    " stops after the tab that reaches the target, then"
                    " characters (and with --coverage, probe items) are"
                    " added on top; not an exact output cardinality")
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument("--sqlite")
    ap.add_argument("--json")
    ap.add_argument("--coverage", action="store_true")
    ap.add_argument("--now", default="2026-08-01T12:00:00")
    ap.add_argument("--leagues", default="Standard,Hardcore,Allflame,Mirage")
    ap.add_argument("--repoe-dir", help="load RePoE files from this directory"
                    " (offline; e.g. the checked-in fixtures/) instead of the"
                    " live cache")
    args = ap.parse_args(argv)

    profile = json.load(open(args.profile))
    leagues = args.leagues.split(",")
    rng = random.Random(args.seed)
    now = datetime.fromisoformat(args.now)

    ix = RepoeIndex(repoe_data.load(args.repoe_dir))
    factory = ItemFactory(ix, rng)
    asm = Assembler(factory, profile, leagues, rng, now)

    league_shares = profile.get("league_tab_shares") or [1.0]
    type_shares = profile.get("tab_type_shares") or {"PremiumStash": 1.0}
    # Children are emitted by their parents, not sampled directly.
    direct = {t: s for t, s in type_shares.items() if t != "Folder"}
    types, weights = zip(*direct.items())

    children_qs = profile.get("children_per_parent_quantiles") or {}
    foldered_share = profile.get("foldered_member_share", 0.0)
    folder_ids = {}  # league -> (folder id, members left)

    emitted, index, stalled = 0, 0, 0
    while emitted < args.items:
        league = rng.choices(
            leagues[:len(league_shares)], weights=league_shares[:len(leagues)])[0]
        ttype = rng.choices(types, weights=weights)[0]
        index += 1
        before = emitted
        # Folder membership applies to every non-folder tab type,
        # Map/Unique parents included: the profile's foldered_member_share
        # is measured over all top-level non-folder tabs.
        folder = None
        if ttype != "Folder" and rng.random() < foldered_share:
            fid, left = folder_ids.get(league, (None, 0))
            if left <= 0:
                # The folder takes this index; its first member gets the
                # next one, so top-level indices stay distinct.
                fid, left = asm.folder(league, f"Folder {index}", index), \
                    rng.randint(4, 10)
                index += 1
            folder_ids[league] = (fid, left - 1)
            folder = fid
        if ttype in ("MapStash", "UniqueStash"):
            qs = children_qs.get(ttype)
            n_children = (round(sample_quantiles(rng, qs))
                          if qs else rng.randint(3, 30))
            emitted += asm.special_parent(
                league, ttype, f"{ttype[:-5]}s {index}", index,
                n_children=n_children, folder=folder)
        else:
            emitted += asm.normal_tab(league, ttype, f"Tab {index}", index,
                                      folder=folder)
        # A profile whose tabs are all empty would otherwise loop forever.
        stalled = stalled + 1 if emitted == before else 0
        if stalled >= 1000:
            raise SystemExit(
                "no items emitted in 1000 consecutive tabs: the profile's"
                " items-per-tab quantiles are all zero; nothing to generate"
                f" at --items {args.items}")

    stash_items = emitted
    for i in range(profile.get("character_count", 5)):
        emitted += asm.character(
            rng.choice(leagues[:max(1, len(league_shares))]),
            f"Synth{rng.choice(RARE_NAME_A)}{rng.choice(RARE_NAME_B)}{i}")
    char_items = emitted - stash_items

    probes = []
    if args.coverage:
        # Tab 1: the systematic item.h sweep. Coverage items are axis
        # probes, not layout tests: force 1x1 and place directly so no axis
        # can be dropped by packing overflow.
        index += 1
        sid = asm._stash_id()
        items = []
        for slot, axes in enumerate(coverage_axes()):
            it = factory.equipment("Standard", rarity=2, axes=axes)
            it["w"] = it["h"] = 1
            it["x"], it["y"] = slot % 24, slot // 24
            items.append(it)
            probes.append({"id": f"axis-{slot:03d}-" + "+".join(axes),
                           "kind": "coverage-axis",
                           "item_id": it["id"], "stash_id": sid,
                           "item_json": json.dumps(it, separators=(",", ":"))})
        assert len(items) <= 24 * 24, "coverage axes exceed one quad tab"
        payload = {"id": sid, "name": "Coverage", "type": "QuadStash",
                   "index": index, "metadata": {"colour": "cc0000"},
                   "items": items}
        asm.stash_rows.append(asm._row(sid, "Standard", "Coverage",
                                       "QuadStash", index, payload,
                                       colour="cc0000"))
        emitted += len(items)

        # Tab 2+: the curated quirk registry (quirks-registry.json) — the
        # hand-picked historical idiosyncrasies the sweep cannot express,
        # including keys item.h deliberately does not model, plus one
        # extra tab per tab-level quirk.
        index += 1
        n_items, n_tabs = emit_quirks_tab(factory, asm, index, probes)
        emitted += n_items
        index += n_tabs - 1

    if args.sqlite:
        emit_sqlite(args.sqlite, asm)
        # Sidecar validation manifest: one probe per coverage axis and per
        # registry quirk. item_json is the item's exact serialized bytes as
        # embedded in the payload (same dict, same dump options), so a
        # validator can assert raw-byte presence — covering emitted nulls
        # and deleted keys — without re-deriving generation logic. The repro
        # block hashes every input, so a dataset stays identifiable after
        # the repository and upstream RePoE have moved on.
        def sha256(path):
            import hashlib
            return hashlib.sha256(open(path, "rb").read()).hexdigest()
        here = os.path.dirname(os.path.abspath(__file__))
        manifest = {
            "repro": {"seed": args.seed, "items": args.items,
                      "profile": os.path.basename(args.profile),
                      "profile_sha256": sha256(args.profile),
                      "leagues": args.leagues, "coverage": args.coverage,
                      "now": args.now,
                      "repoe_version": ix.version,
                      "repoe_sha256": {name: sha256(os.path.join(ix.dir, name))
                                       for name in repoe_data.FILES},
                      "quirks_registry_sha256": sha256(
                          os.path.join(here, "quirks-registry.json"))},
            "totals": {"items": emitted, "stash_rows": len(asm.stash_rows),
                       "characters": len(asm.char_rows)},
            "probes": probes,
        }
        with open(args.sqlite + ".manifest.json", "w") as f:
            json.dump(manifest, f, indent=1)
            f.write("\n")
    if args.json:
        emit_jsonl(args.json, asm)
    # Counts are top-level items only (socketed sub-items excluded):
    # stash-tab items, character equipment+inventory, coverage/quirk probes.
    print(f"generated {emitted} items ({stash_items} stash,"
          f" {char_items} character,"
          f" {emitted - stash_items - char_items} probe),"
          f" {len(asm.stash_rows)} stash rows,"
          f" {len(asm.char_rows)} characters, {len(asm.item_buyouts)} buyouts"
          f" (seed={args.seed}, repoe={ix.version})")


if __name__ == "__main__":
    sys.exit(main())
