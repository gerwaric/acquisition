#!/usr/bin/env python3
"""The candidate table: one row per pair (pseudo, line template) that might
contribute, who says so, and which search would decide it. Writes
../data/candidates.csv and prints the counts.

    candidates.py          # write ../data/candidates.csv, print the counts
    candidates.py --check  # open every locator in the file at its commit

Part 1 of the second pass (BRIEF.md): it decides no contributor. A pair is a
candidate when a source names the line under the pseudo, or when the
template's own words put it in reach of the pseudo's text (the WORDS table
below, one entry per pseudo, printed in the README). A pseudo is a row of
PSEUDOS and WORDS, never a rule in the code: the site's other sum-like
pseudos join by adding rows.

Status, and nothing else (the brief's table): `site` (the fetch capture
shows the site counting the line, or returning the pseudo without it),
`sources-agree`, `sources-differ`, `one-source`, `text-only`. The sources
are separate readings: the C++ app's tables (pseudomods.toml; the shipped
totals are generated from it, so the two are one source), Awakened PoE
Trade's pseudo rules, Path of Building's stat-to-pseudo map. Agreement among
them is not the site's answer, and nothing here raises a status on
likelihood.

Scope (the brief): the 21 `pseudo_adds_*`, `pseudo_total_life`,
`pseudo_base_defence_percentile`, and the 35 shipped totals for the lines no
total counts (search/MEASUREMENTS.md, M6).

Families: a template of a conditional shape (the SHAPES table: a passive's
grant, a minion's, a monster's, during a flask's effect, while, if/recently,
per, against, a weapon's or hand's, a socketed gem's, a description) keeps
its row; the family (pseudo, shape) names one representative for the sheet
and the others say `family:` in their note. A shape whose words are in the
pseudo's own text is not a shape for that pseudo.

Search ids: `L<n>` is one search per candidate line (the sheet's `--method
if`); `L<n>.<k>` is the pair's own search (`--method and`); the column
carries the pair id, whose prefix is the line's. `P<n>` is a percentile
line, the same in both forms. A family member carries its representative's
id.
"""

import csv
import itertools
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
SEARCH = TRACK.parent
REPO = SEARCH.parent  # spikes/rust-playground
CLONES = SEARCH.parents[3]
OUT = TRACK / "data" / "candidates.csv"

STATS_CAPTURE = SEARCH / "trade-query/data/stats-2026-09-12.json"
FETCH = SEARCH / "trade-query/data/fetch-census.json"
TEMPLATES = SEARCH / "item-facts/data/mod-templates.csv"
TVT = SEARCH / "repoe/data/template-vs-translation.csv"
TRADE_MAP = SEARCH / "repoe/data/trade-stat-map.csv"
TOTALS = REPO / "crates/acquisition-search/reference/totals-v1.toml"
PSEUDOMODS = SEARCH / "cpp-search/data/pseudomods.toml"
PERCENTILE_CHECK = TRACK / "data/percentile-check.csv"  # scripts/percentile.py, run first

CLONE_COMMITS = {
    "poe1": "e2bd511a",
    "PathOfBuilding": "16de4b82",
    "awakened-poe-trade": "ce551eb7",
}
EXPORT_STATS = "poe1@e2bd511a:data/stats.json"
APT_INDEX = "awakened-poe-trade@ce551eb7:renderer/src/web/price-check/filters/pseudo/index.ts"
APT_Q20 = "awakened-poe-trade@ce551eb7:renderer/src/parser/calc-q20.ts"
POB_TQG = "PathOfBuilding@16de4b82:src/Classes/TradeQueryGenerator.lua"
POB_ITEM = "PathOfBuilding@16de4b82:src/Classes/Item.lua"
P_TEMPLATES = "search/item-facts/data/mod-templates.csv"
P_FETCH = "search/trade-query/data/fetch-census.json"
P_PSEUDOMODS = "search/cpp-search/data/pseudomods.toml"

STATUS_ORDER = ["sources-differ", "text-only", "one-source", "sources-agree", "site"]

# ---------------------------------------------------------------------------
# The reach table: one entry per pseudo in scope. A template is in reach when
# every group holds at least one of its words, matched whole and with case.
# UNSCOPED is met by a template naming neither Attack(s) nor Spell(s).
# `scope` is `all` (every template in reach) or `uncounted` (the M6 scope:
# only templates no shipped total counts).
UNSCOPED = "(neither Attack nor Spell)"
RES = ["Resistance", "Resistances"]
SOCK = ["Level of Socketed"]
WORDS = [
    # the ranged family
    ("pseudo.pseudo_adds_physical_damage", "all", [["Adds # to #"], ["Physical"], ["Damage"]]),
    ("pseudo.pseudo_adds_lightning_damage", "all", [["Adds # to #"], ["Lightning"], ["Damage"]]),
    ("pseudo.pseudo_adds_cold_damage", "all", [["Adds # to #"], ["Cold"], ["Damage"]]),
    ("pseudo.pseudo_adds_fire_damage", "all", [["Adds # to #"], ["Fire"], ["Damage"]]),
    ("pseudo.pseudo_adds_elemental_damage", "all",
     [["Adds # to #"], ["Fire", "Cold", "Lightning", "Elemental"], ["Damage"]]),
    ("pseudo.pseudo_adds_chaos_damage", "all", [["Adds # to #"], ["Chaos"], ["Damage"]]),
    ("pseudo.pseudo_adds_damage", "all", [["Adds # to #"], ["Damage"]]),
    ("pseudo.pseudo_adds_physical_damage_to_attacks", "all",
     [["Adds # to #"], ["Physical"], ["Damage"], ["Attack", "Attacks", UNSCOPED]]),
    ("pseudo.pseudo_adds_lightning_damage_to_attacks", "all",
     [["Adds # to #"], ["Lightning"], ["Damage"], ["Attack", "Attacks", UNSCOPED]]),
    ("pseudo.pseudo_adds_cold_damage_to_attacks", "all",
     [["Adds # to #"], ["Cold"], ["Damage"], ["Attack", "Attacks", UNSCOPED]]),
    ("pseudo.pseudo_adds_fire_damage_to_attacks", "all",
     [["Adds # to #"], ["Fire"], ["Damage"], ["Attack", "Attacks", UNSCOPED]]),
    ("pseudo.pseudo_adds_elemental_damage_to_attacks", "all",
     [["Adds # to #"], ["Fire", "Cold", "Lightning", "Elemental"], ["Damage"],
      ["Attack", "Attacks", UNSCOPED]]),
    ("pseudo.pseudo_adds_chaos_damage_to_attacks", "all",
     [["Adds # to #"], ["Chaos"], ["Damage"], ["Attack", "Attacks", UNSCOPED]]),
    ("pseudo.pseudo_adds_damage_to_attacks", "all",
     [["Adds # to #"], ["Damage"], ["Attack", "Attacks", UNSCOPED]]),
    ("pseudo.pseudo_adds_physical_damage_to_spells", "all",
     [["Adds # to #"], ["Physical"], ["Damage"], ["Spell", "Spells", UNSCOPED]]),
    ("pseudo.pseudo_adds_lightning_damage_to_spells", "all",
     [["Adds # to #"], ["Lightning"], ["Damage"], ["Spell", "Spells", UNSCOPED]]),
    ("pseudo.pseudo_adds_cold_damage_to_spells", "all",
     [["Adds # to #"], ["Cold"], ["Damage"], ["Spell", "Spells", UNSCOPED]]),
    ("pseudo.pseudo_adds_fire_damage_to_spells", "all",
     [["Adds # to #"], ["Fire"], ["Damage"], ["Spell", "Spells", UNSCOPED]]),
    ("pseudo.pseudo_adds_elemental_damage_to_spells", "all",
     [["Adds # to #"], ["Fire", "Cold", "Lightning", "Elemental"], ["Damage"],
      ["Spell", "Spells", UNSCOPED]]),
    ("pseudo.pseudo_adds_chaos_damage_to_spells", "all",
     [["Adds # to #"], ["Chaos"], ["Damage"], ["Spell", "Spells", UNSCOPED]]),
    ("pseudo.pseudo_adds_damage_to_spells", "all",
     [["Adds # to #"], ["Damage"], ["Spell", "Spells", UNSCOPED]]),
    # life
    ("pseudo.pseudo_total_life", "all", [["maximum Life"]]),
    # the 35 shipped totals, for the lines no total counts
    ("pseudo.pseudo_total_cold_resistance", "uncounted", [["Cold", "Elemental", "all"], RES]),
    ("pseudo.pseudo_total_fire_resistance", "uncounted", [["Fire", "Elemental", "all"], RES]),
    ("pseudo.pseudo_total_lightning_resistance", "uncounted", [["Lightning", "Elemental", "all"], RES]),
    ("pseudo.pseudo_total_elemental_resistance", "uncounted",
     [["Fire", "Cold", "Lightning", "Elemental", "all"], RES]),
    ("pseudo.pseudo_total_chaos_resistance", "uncounted", [["Chaos", "all"], RES]),
    ("pseudo.pseudo_total_resistance", "uncounted", [RES]),
    ("pseudo.pseudo_total_strength", "uncounted", [["Strength", "Attributes"]]),
    ("pseudo.pseudo_total_dexterity", "uncounted", [["Dexterity", "Attributes"]]),
    ("pseudo.pseudo_total_intelligence", "uncounted", [["Intelligence", "Attributes"]]),
    ("pseudo.pseudo_total_attack_speed", "uncounted", [["Attack", "Attacks"], ["Speed"]]),
    ("pseudo.pseudo_total_cast_speed", "uncounted", [["Cast"], ["Speed"]]),
    ("pseudo.pseudo_increased_physical_damage", "uncounted", [["increased", "reduced"], ["Physical Damage"]]),
    ("pseudo.pseudo_critical_strike_chance_for_spells", "uncounted",
     [["Critical Strike Chance"], ["Spell", "Spells"]]),
    ("pseudo.pseudo_total_additional_gem_levels", "uncounted", [SOCK, ["Gem", "Gems"]]),
    ("pseudo.pseudo_total_additional_elemental_gem_levels", "uncounted", [SOCK, ["Elemental"]]),
    ("pseudo.pseudo_total_additional_fire_gem_levels", "uncounted", [SOCK, ["Fire"]]),
    ("pseudo.pseudo_total_additional_cold_gem_levels", "uncounted", [SOCK, ["Cold"]]),
    ("pseudo.pseudo_total_additional_lightning_gem_levels", "uncounted", [SOCK, ["Lightning"]]),
    ("pseudo.pseudo_total_additional_chaos_gem_levels", "uncounted", [SOCK, ["Chaos"]]),
    ("pseudo.pseudo_total_additional_spell_gem_levels", "uncounted", [SOCK, ["Spell"]]),
    ("pseudo.pseudo_total_additional_projectile_gem_levels", "uncounted", [SOCK, ["Projectile"]]),
    ("pseudo.pseudo_total_additional_bow_gem_levels", "uncounted", [SOCK, ["Bow"]]),
    ("pseudo.pseudo_total_additional_melee_gem_levels", "uncounted", [SOCK, ["Melee"]]),
    ("pseudo.pseudo_total_additional_minion_gem_levels", "uncounted", [SOCK, ["Minion"]]),
    ("pseudo.pseudo_total_additional_strength_gem_levels", "uncounted", [SOCK, ["Strength"]]),
    ("pseudo.pseudo_total_additional_dexterity_gem_levels", "uncounted", [SOCK, ["Dexterity"]]),
    ("pseudo.pseudo_total_additional_intelligence_gem_levels", "uncounted", [SOCK, ["Intelligence"]]),
    ("pseudo.pseudo_total_additional_aura_gem_levels", "uncounted", [SOCK, ["Aura"]]),
    ("pseudo.pseudo_total_additional_movement_gem_levels", "uncounted", [SOCK, ["Movement"]]),
    ("pseudo.pseudo_total_additional_curse_gem_levels", "uncounted", [SOCK, ["Curse"]]),
    ("pseudo.pseudo_total_additional_vaal_gem_levels", "uncounted", [SOCK, ["Vaal"]]),
    ("pseudo.pseudo_total_additional_support_gem_levels", "uncounted", [SOCK, ["Support"]]),
    ("pseudo.pseudo_total_additional_skill_gem_levels", "uncounted", [SOCK, ["Skill"]]),
    ("pseudo.pseudo_total_additional_warcry_gem_levels", "uncounted", [SOCK, ["Warcry"]]),
    ("pseudo.pseudo_total_additional_golem_gem_levels", "uncounted", [SOCK, ["Golem"]]),
    # the computed one
    ("pseudo.pseudo_base_defence_percentile", "all",
     [["Armour", "Evasion Rating", "Evasion", "Energy Shield", "Ward", "Defences"],
      ["# to", "#% increased", "#% reduced"]]),
]
PERCENTILE = "pseudo.pseudo_base_defence_percentile"

# Conditional shapes, first match wins. A shape is skipped for a pseudo whose
# own text it matches.
SHAPES = [
    ("a passive's grant", r"Passive Skills? (in Radius )?(also )?grants?|^Allocates |Notable Passive|"
                          r"\bPassives?\b.*in Radius|Passive Skills in Radius"),
    ("a minion's or ally's", r"\bMinions?\b|Raised Zombies|Spectres|Skeletons|Golems? |Totems?\b|"
                             r"Reinforcements|\b[Aa]llies\b|Companion|Animated|Sentinel"),
    ("a monster's or an enemy's", r"Monsters?\b|\b[Ee]nemies\b.*\bhave\b|Nearby Enemies|against you|"
                                  r"\bPlayers?\b|Area |Unique Boss|Enemies Withered|Exposure|"
                                  r"Penetrat|\bof Enemies\b"),
    ("during a flask's effect", r"during (any )?Flask Effect|during Effect|\bFlasks?\b"),
    ("while", r"\bwhile\b|\bWhile\b"),
    ("if or recently", r"\bif\b|\bIf\b|Recently"),
    ("per", r"\bper\b|for each|for every"),
    ("against", r"\bagainst\b"),
    ("a weapon's or hand's", r"in Main Hand|in Off Hand|with this Weapon|while Unarmed|Unarmed|"
                             r"Retaliation|\bwith (Axes|Bows|Claws|Daggers|Maces|Staves|Swords|Wands|"
                             r"Sceptres|One Handed|Two Handed|Melee)"),
    ("a socketed gem's or skill's", r"Socketed|Supported Skills|Supported by|^Grants |Skills? (deal|have)|"
                                    r"\bAura\b|\bGem\b|^Buff grants"),
    ("a description", r"\(#-#\)|^[A-Z][A-Za-z' ]*: |^\[|Alert Level"),
]

# ---------------------------------------------------------------------------
# Sources, as each states itself. A source "speaks of" a pseudo when it
# lists that pseudo's lines; it names a template at a weight. Locators are a
# path and a line range at the pinned commit; `--check` opens each and looks
# for the probe string in it.
#
# Awakened PoE Trade: PSEUDO_RULES and the two info tables in index.ts.
APT_SPEAKS = {
    "pseudo.pseudo_total_elemental_resistance": f"{APT_INDEX}#L47-L53",
    "pseudo.pseudo_total_fire_resistance": f"{APT_INDEX}#L55-L60",
    "pseudo.pseudo_total_cold_resistance": f"{APT_INDEX}#L62-L67",
    "pseudo.pseudo_total_lightning_resistance": f"{APT_INDEX}#L69-L74",
    "pseudo.pseudo_total_chaos_resistance": f"{APT_INDEX}#L76-L79",
    "pseudo.pseudo_total_strength": f"{APT_INDEX}#L99-L104",
    "pseudo.pseudo_total_dexterity": f"{APT_INDEX}#L106-L111",
    "pseudo.pseudo_total_intelligence": f"{APT_INDEX}#L113-L118",
    "pseudo.pseudo_total_life": f"{APT_INDEX}#L120-L127",
    "pseudo.pseudo_total_attack_speed": f"{APT_INDEX}#L149-L154",
    "pseudo.pseudo_total_cast_speed": f"{APT_INDEX}#L156-L161",
    "pseudo.pseudo_increased_physical_damage": f"{APT_INDEX}#L169-L173",
    "pseudo.pseudo_critical_strike_chance_for_spells": f"{APT_INDEX}#L182-L188",
    PERCENTILE: f"{APT_Q20}#L5-L38",
}
# (pseudo, template, weight, locator, probe). Only what bears on a row in
# scope: the named lines the shipped totals already count are not rows.
APT_NAMES = [
    ("pseudo.pseudo_total_life", "# to maximum Life", "1", f"{APT_INDEX}#L119-L127", "+# to maximum Life"),
    ("pseudo.pseudo_total_life", "# to Strength", "0.5", f"{APT_INDEX}#L24,L119-L127", "+# to Strength"),
    ("pseudo.pseudo_total_life", "# to Strength and Intelligence", "0.5", f"{APT_INDEX}#L27,L119-L127",
     "+# to Strength and Intelligence"),
    ("pseudo.pseudo_total_life", "# to Strength and Dexterity", "0.5", f"{APT_INDEX}#L28,L119-L127",
     "+# to Strength and Dexterity"),
    ("pseudo.pseudo_total_life", "# to all Attributes", "0.5", f"{APT_INDEX}#L23,L119-L127",
     "+# to all Attributes"),
    ("pseudo.pseudo_total_elemental_resistance", "#% to All Resistances", "3", f"{APT_INDEX}#L8,L46-L53",
     "+#% to All Resistances"),
    ("pseudo.pseudo_total_fire_resistance", "#% to All Resistances", "1", f"{APT_INDEX}#L8,L54-L60",
     "+#% to All Resistances"),
    ("pseudo.pseudo_total_cold_resistance", "#% to All Resistances", "1", f"{APT_INDEX}#L8,L61-L67",
     "+#% to All Resistances"),
    ("pseudo.pseudo_total_lightning_resistance", "#% to All Resistances", "1", f"{APT_INDEX}#L8,L68-L74",
     "+#% to All Resistances"),
    ("pseudo.pseudo_total_chaos_resistance", "#% to All Resistances", "1", f"{APT_INDEX}#L8,L75-L79",
     "+#% to All Resistances"),
    # the percentile: the lines it reads, by role (flat before the increase,
    # or the increase)
    (PERCENTILE, "# to Armour", "flat", f"{APT_Q20}#L6-L7", "+# to Armour"),
    (PERCENTILE, "#% increased Armour", "increased", f"{APT_Q20}#L8-L9", "#% increased Armour"),
    (PERCENTILE, "#% increased Armour and Energy Shield", "increased", f"{APT_Q20}#L8-L13",
     "#% increased Armour and Energy Shield"),
    (PERCENTILE, "#% increased Armour and Evasion", "increased", f"{APT_Q20}#L8-L13",
     "#% increased Armour and Evasion"),
    (PERCENTILE, "#% increased Armour, Evasion and Energy Shield", "increased", f"{APT_Q20}#L8-L13",
     "#% increased Armour, Evasion and Energy Shield"),
    (PERCENTILE, "# to Evasion Rating", "flat", f"{APT_Q20}#L15-L16", "+# to Evasion Rating"),
    (PERCENTILE, "#% increased Evasion Rating", "increased", f"{APT_Q20}#L17-L18", "#% increased Evasion Rating"),
    (PERCENTILE, "#% increased Evasion and Energy Shield", "increased", f"{APT_Q20}#L17-L22",
     "#% increased Evasion and Energy Shield"),
    (PERCENTILE, "# to maximum Energy Shield", "flat", f"{APT_Q20}#L24-L25", "+# to maximum Energy Shield"),
    (PERCENTILE, "#% increased Energy Shield", "increased", f"{APT_Q20}#L26-L27", "#% increased Energy Shield"),
    (PERCENTILE, "# to Ward", "flat", f"{APT_Q20}#L33-L34", "+# to Ward"),
    (PERCENTILE, "#% increased Ward", "increased", f"{APT_Q20}#L35-L36", "#% increased Ward"),
]
# Path of Building: TradeQueryGenerator's pseudoMap and ignoredStats (a
# single stat mapped to its pseudo; the hybrids and the all-lines skipped
# "since we get the weight for them from individual stats"); Item.lua's local
# defence modifiers by kind (matched to a template by the kind's words: the
# text itself is PoB's ModParser, not an input here).
POB_SPEAKS = {
    "pseudo.pseudo_total_fire_resistance": f"{POB_TQG}#L999-L1031",
    "pseudo.pseudo_total_cold_resistance": f"{POB_TQG}#L999-L1031",
    "pseudo.pseudo_total_lightning_resistance": f"{POB_TQG}#L999-L1031",
    "pseudo.pseudo_total_chaos_resistance": f"{POB_TQG}#L999-L1031",
    "pseudo.pseudo_total_strength": f"{POB_TQG}#L999-L1031",
    "pseudo.pseudo_total_dexterity": f"{POB_TQG}#L999-L1031",
    "pseudo.pseudo_total_intelligence": f"{POB_TQG}#L999-L1031",
}
# Item.lua names the percentile's lines by modifier kind (`calcLocal(modList,
# "Armour", "BASE", 0)`), never by text: the text is PoB's ModParser, not an
# input. A kind matched to a template by its words is therefore a note on the
# row, never a source that names it or leaves it out; `Defences` INC (L2365)
# matches no template by its words.
POB_UNMATCHED = f"{POB_ITEM}#L2365"
POB_KINDS = [
    (PERCENTILE, "# to Armour", "flat", f"{POB_ITEM}#L2347", '"Armour", "BASE"'),
    (PERCENTILE, "#% increased Armour", "increased", f"{POB_ITEM}#L2358", '"Armour", "INC"'),
    (PERCENTILE, "#% increased Armour and Evasion", "increased", f"{POB_ITEM}#L2359", '"ArmourAndEvasion", "INC"'),
    (PERCENTILE, "# to Evasion Rating", "flat", f"{POB_ITEM}#L2350", '"Evasion", "BASE"'),
    (PERCENTILE, "#% increased Evasion Rating", "increased", f"{POB_ITEM}#L2360", '"Evasion", "INC"'),
    (PERCENTILE, "#% increased Evasion and Energy Shield", "increased", f"{POB_ITEM}#L2361",
     '"EvasionAndEnergyShield", "INC"'),
    (PERCENTILE, "# to maximum Energy Shield", "flat", f"{POB_ITEM}#L2353", '"EnergyShield", "BASE"'),
    (PERCENTILE, "#% increased Energy Shield", "increased", f"{POB_ITEM}#L2362", '"EnergyShield", "INC"'),
    (PERCENTILE, "# to Ward", "flat", f"{POB_ITEM}#L2356", '"Ward", "BASE"'),
    (PERCENTILE, "# to Armour and Evasion Rating", "flat", f"{POB_ITEM}#L2349", '"ArmourAndEvasion", "BASE"'),
    (PERCENTILE, "# to Evasion Rating and Energy Shield", "flat", f"{POB_ITEM}#L2352",
     '"EvasionAndEnergyShield", "BASE"'),
    (PERCENTILE, "# to Armour and Energy Shield", "flat", f"{POB_ITEM}#L2355", '"ArmourAndEnergyShield", "BASE"'),
    (PERCENTILE, "#% increased Ward", "increased", f"{POB_ITEM}#L2363", '"Ward", "INC"'),
    (PERCENTILE, "#% increased Armour and Energy Shield", "increased", f"{POB_ITEM}#L2364",
     '"ArmourAndEnergyShield", "INC"'),
]
# The site's own answers (fetch-census.json): for each pseudo it put on a
# fetched item, the weights that reproduce its value on every such item.
# Checked in site_rows(): a weight is admitted only if the sum reproduces
# every item's value.
SITE_WEIGHTS = {
    "pseudo.pseudo_total_life": {"# to maximum Life": 1.0, "# to Strength": 0.5},
    # pseudo_total_fire_resistance: the shipped rows (read from totals-v1.toml)
}

# ---------------------------------------------------------------------------


def read_commented_csv(path):
    """(file line, row) for a CSV with a `# generated by` line before the
    header, the line being the row's first (a quoted template can span
    lines). `newline="\\n"`: mod-templates.csv carries a bare CR inside 260
    templates (search/README.md's trap)."""
    out = []
    with path.open(newline="\n") as fh:
        next(fh)
        reader = csv.DictReader(fh)
        before = 1  # the header's line, in the file after the comment
        for row in reader:
            out.append((before + 2, row))  # the row's first line, counting the comment
            before = reader.line_num
    return out


def has(template, word):
    if word == UNSCOPED:
        return not re.search(r"\b(Attacks?|Spells?)\b", template)
    return re.search(r"(?<![\w])" + re.escape(word) + r"(?![\w])", template) is not None


def reach(template, groups):
    """The words that put the template in reach, or None."""
    hit = []
    for group in groups:
        found = [w for w in group if has(template, w)]
        if not found:
            return None
        hit.append(found[0])
    return hit


def shape_of(template, pseudo_text):
    for name, pattern in SHAPES:
        if re.search(pattern, template) and not re.search(pattern, pseudo_text):
            return name
    return None


MARKUP = re.compile(r"\[(?:[^\]|]*\|)?([^\]]*)\]")


def normal(text):
    """A capture, fetched or census text in the census's template form, with
    the census's `[Key|Shown]` markup read as the shown word."""
    t = MARKUP.sub(r"\1", text.replace("\r", ""))
    t = re.sub(r"\d+(?:\.\d+)?", "#", t)
    t = t.replace(" (Local)", "")
    return t[1:] if t.startswith("+") else t


def load_pseudo_texts():
    doc = json.loads(STATS_CAPTURE.read_text())
    group = next(g for g in doc["result"] if g["id"] == "pseudo")
    return {e["id"]: e["text"] for e in group["entries"]}


def load_capture_ids():
    """template form -> trade ids (never pseudo) whose capture text shows it."""
    doc = json.loads(STATS_CAPTURE.read_text())
    out = defaultdict(set)
    for g in doc["result"]:
        if g["id"] == "pseudo":
            continue
        for e in g["entries"]:
            out[normal(e["text"])].add(e["id"])
    return out


def load_templates():
    """template -> {items, row, arrays} over every array of the census."""
    out = {}
    for lineno, row in read_commented_csv(TEMPLATES):
        t = row["template"]
        e = out.setdefault(t, {"items": 0, "row": lineno, "best": -1, "arrays": Counter()})
        items = int(row["items"])
        e["items"] += items
        e["arrays"][row["array"]] += items
        if items > e["best"]:
            e["best"], e["row"] = items, lineno
    return out


def load_entries():
    """template -> export stat entries (each a `|`-joined stat list)."""
    out = defaultdict(set)
    for _, row in read_commented_csv(TVT):
        for entry in filter(None, row["entry_ids"].split(";")):
            out[row["template"]].add(entry)
    return out


def load_trade_by_entry():
    out = defaultdict(set)
    for _, row in read_commented_csv(TRADE_MAP):
        if not row["trade_id"].startswith("pseudo."):
            out[row["repoe_ids"]].add(row["trade_id"])
    return out


def load_totals():
    text = TOTALS.read_text()
    counted, rows = set(), {}
    for block in text.split("[[total]]")[1:]:
        site = "pseudo." + re.search(r'^site = "(.*)"$', block, re.M).group(1)
        found = re.findall(r'\{ template = "((?:[^"\\]|\\.)*)", slot = "\w+", weight = (\d+) \}', block)
        rows[site] = {t: float(w) for t, w in found}
        counted.update(rows[site])
    return counted, rows


def load_cpp_speaks():
    """pseudo text -> the line range of its [[pseudomod]] in pseudomods.toml."""
    lines = PSEUDOMODS.read_text().split("\n")
    out, start, name = {}, None, None
    for i, line in enumerate(lines, 1):
        m = re.match(r'^name = "(.*)"$', line)
        if m:
            start, name = i, m.group(1)
        if line.strip() == "]" and name:
            out[name] = f"{P_PSEUDOMODS}#L{start}-L{i}"
            name = None
    return out


def json_key_range(path, key):
    """Line range of a top-level key in a two-space-indented JSON object."""
    start = None
    with path.open() as fh:
        for i, line in enumerate(fh, 1):
            if start is None and line.startswith(f'  "{key}": {{'):
                start = i
            elif start is not None and line.startswith("  }"):
                return start, i
    return None


def site_rows(texts, counted_rows, reach_of, in_scope):
    """(pseudo, template) -> (weight, locator) from the site's own values."""
    fetch = json.loads(FETCH.read_text())
    weights = dict(SITE_WEIGHTS)
    weights["pseudo.pseudo_total_fire_resistance"] = counted_rows["pseudo.pseudo_total_fire_resistance"]
    out, checked = {}, Counter()
    for qid, query in fetch["queries"].items():
        for i, item in enumerate(query["fetched"]):
            pseudo_lines = item["lines"].get("pseudoMods") or []
            lines = [(a, j, l) for a, arr in item["lines"].items() if a != "pseudoMods"
                     for j, l in enumerate(arr)]
            for k, pl in enumerate(pseudo_lines):
                pid = pl["hash"].removeprefix("stat.")
                if pid not in weights:
                    continue
                value = float(re.search(r"-?\d+(?:\.\d+)?", pl["description"]).group())
                want = 0.0
                for a, j, l in lines:
                    t = normal(l["description"])
                    nums = [float(x) for x in re.findall(r"\d+(?:\.\d+)?", l["description"])]
                    if t in weights[pid] and nums:
                        want += weights[pid][t] * nums[0]
                if abs(want - value) > 1e-9:
                    raise SystemExit(f"site weights miss queries.{qid}.fetched[{i}]: {want} != {value}")
                checked[pid] += 1
                loc = f"{P_FETCH} queries.{qid}.fetched[{i}].lines.pseudoMods[{k}]"
                for a, j, l in lines:
                    t = normal(l["description"])
                    if t in weights[pid]:
                        if in_scope(pid, t):
                            out.setdefault((pid, t), (weights[pid][t], loc))
                    elif reach_of(pid, t):
                        out.setdefault((pid, t), (0.0, loc))
    return out, checked


def fmt_weight(w):
    return ("%g" % w) if isinstance(w, float) else w


def build():
    texts = load_pseudo_texts()
    capture_ids = load_capture_ids()
    templates = load_templates()
    entries = load_entries()
    trade_by_entry = load_trade_by_entry()
    counted, counted_rows = load_totals()
    cpp_speaks = load_cpp_speaks()
    export = json.loads((CLONES / "poe1/data/stats.json").read_text())

    order = [p for p, _, _ in WORDS]
    words = {p: (scope, groups) for p, scope, groups in WORDS}
    missing = [p for p in order if p not in texts]
    if missing:
        raise SystemExit(f"not in the stats capture: {missing}")

    def reach_of(pid, template):
        if pid not in words:
            return None
        scope, groups = words[pid]
        if scope == "uncounted" and template in counted:
            return None
        return reach(template, groups)

    names = defaultdict(list)  # (pseudo, template) -> [(source, weight, locator)]
    for pid, t, w, loc, _ in APT_NAMES:
        names[(pid, t)].append(("apt", w, loc))
    kinds = {(pid, t): (w, loc) for pid, t, w, loc, _ in POB_KINDS}
    read_on = Counter()  # template -> captured items whose percentile read it, all reproduced
    reproduced = True
    with PERCENTILE_CHECK.open(newline="") as fh:
        for item in csv.DictReader(fh):
            reproduced &= item["site_within"] == "yes"
            for line in filter(None, item["local_lines"].split("; ")):
                read_on[normal(line)] += 1
    def in_scope(pid, template):
        return pid in words and (words[pid][0] == "all" or template not in counted)

    site, site_checked = site_rows(texts, counted_rows, reach_of, in_scope)

    def speakers(pid):
        out = {}
        cpp = cpp_speaks.get(texts[pid])
        if cpp:
            out["cpp"] = cpp
        if pid in APT_SPEAKS:
            out["apt"] = APT_SPEAKS[pid]
        if pid in POB_SPEAKS:
            out["pob"] = POB_SPEAKS[pid]
        return out

    pairs = {}
    for pid in order:
        for t in templates:
            hit = reach_of(pid, t)
            if hit:
                pairs[(pid, t)] = hit
    for key in list(names) + list(site):
        pairs.setdefault(key, None)

    rows = []
    for (pid, t), hit in pairs.items():
        entry_ids = sorted(entries.get(t, ()) or entries.get(normal(t), ()))
        ids = set()
        for e in entry_ids:
            ids |= trade_by_entry.get(e, set())
        ids |= capture_ids.get(normal(t), set())
        local = []
        for e in entry_ids:
            for stat in e.split("|"):
                if stat in export:
                    local.append(f"{stat}={'local' if export[stat]['is_local'] else 'not local'}")
        named = names.get((pid, t), [])
        spk = speakers(pid)
        tinfo = templates.get(t)
        note = []
        if (pid, t) in site:
            w, loc = site[(pid, t)]
            status = "site"
            sources = [loc] + [n[2] for n in named]
            weights = [fmt_weight(w)] + [n[1] for n in named]
            note.append("the site's value reproduced on every fetched item showing this pseudo"
                        if w else "the site returned this pseudo on an item carrying the line, without it")
        elif named:
            srcs = {n[0] for n in named}
            leave_out = [(s, loc) for s, loc in spk.items() if s not in srcs]
            wset = {n[1] for n in named}
            sources = [n[2] for n in named] + [loc for _, loc in leave_out]
            weights = [n[1] for n in named] + ["absent"] * len(leave_out)
            if leave_out or len(wset) > 1:
                status = "sources-differ"
                if leave_out:
                    note.append("named by " + ", ".join(sorted(srcs)) + "; left out by "
                                + ", ".join(s for s, _ in leave_out) + ", which list this pseudo's lines")
            elif len(named) >= 2:
                status = "sources-agree"
            else:
                status = "one-source"
        else:
            status = "text-only"
            sources = [f"{P_TEMPLATES}#L{tinfo['row']}"]
            weights = ["-"]
            note.append("reach: " + " + ".join(hit))
            if spk:
                note.append("listed without it by " + ", ".join(sorted(spk)))
        if (pid, t) in kinds:
            w, loc = kinds[(pid, t)]
            note.append(f"PoB's Item.lua reads a local kind of these words as {w} ({loc}); "
                        "a kind, not the text, so not counted as a source")
        if pid == PERCENTILE and read_on.get(t) and reproduced:
            note.append(f"read as local on {read_on[t]} of the captured items, and the site's stated rule "
                        "reproduces every one (data/percentile-check.csv): consistent, not the site naming it")
        if (pid, t) in kinds:
            pass
        elif pid == PERCENTILE and status != "text-only":
            note.append(f"PoB's `Defences` INC kind ({POB_UNMATCHED}) may be this text; its words do not say")
        rows.append({
            "pseudo": pid, "pseudo_text": texts[pid], "template": t,
            "stat_ids": ";".join(sorted(ids)), "status": status,
            "weights": ";".join(weights), "sources": ";".join(sources),
            "corpus_items": tinfo["items"] if tinfo else 0,
            "local": ";".join(local), "search": "", "note": note,
            "shape": shape_of(t, texts[pid]) if status != "site" else None,
        })

    # families and representatives
    fam = defaultdict(list)
    for r in rows:
        if r["shape"]:
            fam[(r["pseudo"], r["shape"])].append(r)
    reps = {}
    for key, members in fam.items():
        members.sort(key=lambda r: (not r["stat_ids"], STATUS_ORDER.index(r["status"]),
                                    -r["corpus_items"], r["template"]))
        reps[key] = members[0]
        for m in members[1:]:
            m["note"].insert(0, f"family: {key[1]}, represented by `{members[0]['template']}`")
        members[0]["note"].insert(0, f"represents the family `{key[1]}` ({len(members)} templates)")

    # searches: one per line for the if form, one per pair for the and form,
    # ordered by what a search is worth
    searchable = [r for r in rows if r["status"] != "site"
                  and (not r["shape"] or reps[(r["pseudo"], r["shape"])] is r)]
    by_line = defaultdict(list)
    for r in searchable:
        if not r["stat_ids"]:
            r["search"] = "none: no stat id displays this text"
            continue
        kind = "P" if r["pseudo"] == PERCENTILE else "L"
        # one line is one set of stat ids: a template in markup and the
        # same template plain are one search
        by_line[(kind, r["stat_ids"])].append(r)

    def worth(item):
        (kind, ids), members = item
        return (kind == "P", min(STATUS_ORDER.index(m["status"]) for m in members),
                -sum(m["corpus_items"] for m in members), min(m["template"] for m in members))

    n = {"L": 0, "P": 0}
    for (kind, ids), members in sorted(by_line.items(), key=worth):
        n[kind] += 1
        line_id = f"{kind}{n[kind]:03d}"
        pseudos = sorted({m["pseudo"] for m in members},
                         key=lambda p: (min(STATUS_ORDER.index(m["status"]) for m in members
                                            if m["pseudo"] == p), order.index(p)))
        for r in members:
            k = pseudos.index(r["pseudo"]) + 1
            r["search"] = line_id if kind == "P" else f"{line_id}.{k}"
    for key, members in fam.items():
        for m in members[1:]:
            m["search"] = reps[key]["search"]
    rows.sort(key=lambda r: (order.index(r["pseudo"]), STATUS_ORDER.index(r["status"]),
                             -r["corpus_items"], r["template"]))
    return rows, site_checked, templates, entries, export


COLUMNS = ["pseudo", "pseudo_text", "template", "stat_ids", "status", "weights", "sources",
           "corpus_items", "local", "search", "note"]


def write(rows):
    with OUT.open("w", newline="") as fh:
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(COLUMNS)
        for r in rows:
            w.writerow([r[c] if c != "note" else "; ".join(r["note"]) for c in COLUMNS])


def report(rows, site_checked, templates, entries, export):
    order = [p for p, _, _ in WORDS]
    group_of = {}
    for p in order:
        group_of[p] = ("ranged" if "pseudo_adds_" in p else "life" if p == "pseudo.pseudo_total_life"
                       else "percentile" if p == PERCENTILE else "totals")
    print(f"{len(rows)} candidate pairs over {len({r['pseudo'] for r in rows})} pseudos")
    print(f"site answers checked: " + ", ".join(f"{k} on {v} items" for k, v in sorted(site_checked.items())))
    print()
    print(f"{'status':15} {'ranged':>7} {'life':>5} {'totals':>7} {'pctile':>7} {'all':>6}")
    for s in STATUS_ORDER:
        c = Counter(group_of[r["pseudo"]] for r in rows if r["status"] == s)
        print(f"{s:15} {c['ranged']:>7} {c['life']:>5} {c['totals']:>7} {c['percentile']:>7} {sum(c.values()):>6}")
    c = Counter(group_of[r["pseudo"]] for r in rows)
    print(f"{'total':15} {c['ranged']:>7} {c['life']:>5} {c['totals']:>7} {c['percentile']:>7} {len(rows):>6}")
    print()
    fam = Counter(r["note"][0].split(",")[0].replace("family: ", "") for r in rows
                  if r["note"] and r["note"][0].startswith("family:"))
    reps = sum(1 for r in rows if r["note"] and r["note"][0].startswith("represents"))
    print(f"families: {reps} (pseudo, shape) families; {sum(fam.values())} rows say `family:`")
    for shape, k in sorted(Counter(r["note"][0].split("`")[1] for r in rows
                                   if r["note"] and r["note"][0].startswith("represents")).items()):
        members = sum(1 for r in rows if r["note"] and (
            (r["note"][0].startswith("family: " + shape + ",")) or
            (r["note"][0].startswith("represents the family `" + shape + "`"))))
        print(f"  {shape:30} {k:>4} families, {members:>5} rows")
    lines = {r["search"].split(".")[0] for r in rows if r["search"].startswith("L")}
    pairs = {r["search"] for r in rows if r["search"].startswith("L")}
    plines = {r["search"] for r in rows if r["search"].startswith("P")}
    none = sum(1 for r in rows if r["search"].startswith("none"))
    print()
    print(f"searches: {len(lines)} lines (--method if), {len(pairs)} pairs (--method and), "
          f"{len(plines)} percentile lines; {none} rows no stat id can search")
    print()
    print("the ranged family's own templates, by what the export says is local:")
    for p in order:
        if group_of[p] != "ranged":
            continue
        t = next((r for r in rows if r["pseudo"] == p and r["template"] == r["pseudo_text"]), None)
        if t is None:
            print(f"  {p}: its own text is no template in the corpus")
            continue
        arrays = templates[t["template"]]["arrays"]
        print(f"  {t['template']!r}: {t['corpus_items']} items "
              f"({', '.join(f'{a} {n}' for a, n in arrays.most_common())}); {t['local'] or 'export silent'}")


def spawn_report(rows):
    """For the ranged family's own templates: the export's mods carrying each
    stat behind them (poe1 data/mods.json), by domain and generation type, and
    the item tags they spawn on with a weight above zero."""
    mods = json.loads((CLONES / "poe1/data/mods.json").read_text())
    print()
    print("where the stats behind the unsuffixed ranged templates can appear (poe1@e2bd511a data/mods.json):")
    stats = sorted({e.split("=")[0] for r in rows if "pseudo_adds_" in r["pseudo"]
                    and r["template"] == r["pseudo_text"] for e in r["local"].split(";")
                    if e.startswith(("local_minimum", "global_minimum"))})
    for stat in stats:
        kinds, tags = Counter(), set()
        for m in mods.values():
            if any(x["id"] == stat for x in m["stats"]):
                kinds[f"{m['domain']}/{m['generation_type']}"] += 1
                tags |= {w["tag"] for w in m["spawn_weights"] if w["weight"] > 0}
        weaponish = sorted(t for t in tags if re.search(r"weapon|sword|axe|mace|bow|claw|dagger|staff|wand|"
                                                        r"sceptre|rapier|adjudicator|basilisk|eyrie|crusader|"
                                                        r"elder|shaper|redeemer", t))
        other = sorted(tags - set(weaponish))
        print(f"  {stat}: {sum(kinds.values())} mods ({', '.join(f'{k} {n}' for k, n in kinds.most_common())}); "
              f"spawn tags: {len(weaponish)} weapon, other: {', '.join(other) or 'none'}")


def check():
    """Open every locator in the written file at its commit."""
    heads = {}
    for name, commit in CLONE_COMMITS.items():
        head = subprocess.run(["git", "-C", str(CLONES / name), "rev-parse", "HEAD"],
                              capture_output=True, text=True).stdout.strip()
        heads[name] = head.startswith(commit)
    probes = {}
    for pid, t, _, loc, probe in APT_NAMES:
        probes[(pid, t, loc)] = probe
    bad, seen = 0, 0
    fetch = json.loads(FETCH.read_text())
    with OUT.open(newline="") as fh:
        for row in csv.DictReader(fh):
            for loc in row["sources"].split(";"):
                seen += 1
                ok = open_locator(loc, row, heads, probes, fetch)
                if not ok:
                    bad += 1
                    print(f"DOES NOT OPEN  {row['pseudo']} / {row['template']}: {loc}")
    print(f"{seen} locators opened, {bad} do not")
    return 1 if bad else 0


def open_locator(loc, row, heads, probes, fetch):
    if loc.startswith(P_FETCH + " "):
        path = loc.split(" ", 1)[1]
        m = re.match(r"queries\.(\w+)\.fetched\[(\d+)\]\.lines\.pseudoMods\[(\d+)\]$", path)
        if not m:
            return False
        line = fetch["queries"][m.group(1)]["fetched"][int(m.group(2))]["lines"]["pseudoMods"][int(m.group(3))]
        return line["hash"] == "stat." + row["pseudo"]
    m = re.match(r"^(?:(\S+?)@([0-9a-f]+):)?(\S+?)#L(.+)$", loc)
    if not m:
        return False
    clone, commit, path, ranges = m.groups()
    if clone:
        if not heads.get(clone) or not CLONE_COMMITS[clone].startswith(commit):
            return False
        file = CLONES / clone / path
    else:
        file = REPO / path
    if not file.exists():
        return False
    text = file.read_text(newline="\n" if path.endswith(".csv") else None).split("\n")
    span = []
    for part in ranges.split(","):
        a, _, b = part.lstrip("L").partition("-L")
        a, b = int(a), int(b or a)
        if not (1 <= a <= b <= len(text)):
            return False
        span.extend(text[a - 1:b])
    body = "\n".join(span)
    if (row["pseudo"], row["template"], loc) in probes:
        return probes[(row["pseudo"], row["template"], loc)] in body
    if path.endswith("mod-templates.csv"):
        a = int(ranges.lstrip("L").split("-")[0])
        with file.open(newline="\n") as fh:
            parsed = next(csv.reader(itertools.islice(fh, a - 1, None)))
        return len(parsed) > 1 and parsed[1] == row["template"]
    if path.endswith("pseudomods.toml") or path.endswith("index.ts"):
        return row["pseudo_text"].lstrip("+") in body
    if path.endswith("calc-q20.ts"):
        return "QUALITY_STATS" in body
    if path.endswith("TradeQueryGenerator.lua"):
        return row["pseudo"] in body
    return False


def main():
    if sys.argv[1:] == ["--check"]:
        return check()
    if sys.argv[1:]:
        print(__doc__)
        return 2
    rows, site_checked, templates, entries, export = build()
    write(rows)
    report(rows, site_checked, templates, entries, export)
    spawn_report(rows)
    print(f"\nwrote {OUT.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
