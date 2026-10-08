#!/usr/bin/env python3
"""The category table and the beast colour table (C106, C68; item search
step 9d-B): the trade site's category ids with what makes each hold, and
the colour the site lists each captured beast under — regenerated from
committed, pinned sources into the two reviewed files the search crate
ships.

    python3 tools/category-table.py            # writes crates/acquisition-search/reference/categories-v1.toml
                                               #    and crates/acquisition-search/reference/beast-colours-v1.toml
    python3 tools/category-table.py --check    # exits 1 when a file on disk differs from what the sources give

Inputs (read-only, all committed; nothing is fetched and no clone is read):
  search/trade-query/data/filters-2026-09-12.json   the site's category filter (`type_filters.category`):
                                                    the ids as spelled, their texts, their order
  crates/acquisition-search/reference/classes-v1.toml   the class table: the class names a leaf is keyed by,
                                                    and the realms both tables cover
  search/category/data/grouping-members.csv         the reviewed members of the seven groupings, a cell a
                                                    row, each saying what answered it (J4, J12)
  search/category/data/beast-colours.csv            the colour table as the sittings left it: a name, its
                                                    colour, the searches that evidence it (J1; rounds k and l)

Rules, each a reading of a source or a ruling and never a judgment (C106).
The rulings are `search/LEDGER.md`, "Holes ruled", the 9d rows; what each id
is produced by was measured in `search/category/README.md`, F1 and F2, whose
table is `search/category/data/categories.csv`.
  1. An id enters as the filter spells it, with the site's text, in the
     site's order. Nothing is added, renamed or dropped.
  2. A leaf is a class's (CLASSES, below): the id an item of that class has.
     The class is named as the class table names it, and every class of the
     class table either gives an id or is listed as giving none (NO_ID), so
     a class the next pull adds stops this script until it is placed.
  3. A leaf may be a class's with a rule on the base name (NAME_RULES): an
     item of the class whose base the rule holds has that id, in place of
     the class's own. A name rule is one clause on a name within one
     class; a case one clause cannot say is an exception row citing its
     capture, never a second clause (the owner, 2026-10-07). J3: the
     Maven's Invitation leaf holds a base named for the Maven, and every
     other invitation takes `map`. J23: a Stackable Currency base whose
     name holds the word Tattoo is `currency.tattoo`. Every base of a
     rule's class that holds the leaf's word and misses the rule stops
     the script unless it is named with its reason (NEAR_MISSES).
  4. A grouping holds its reviewed members alone (J4, J12): the cells of
     grouping-members.csv marked `in`, each with what said so.
  5. A captured beast is `monster.beast`, known by its bestiary line, and a
     colour leaf holds the beasts the colour table gives that colour (J1);
     a colour leaf is under `monster.beast`, which a count needs to say
     which id is the more specific (J11, J15).
  6. A parent is an id with another under it by the dot (J2): nothing is
     written for it, the dot being in the ids themselves.
An id none of these explains stops the script — but `graft`, whose class
the export prints no name for and the class table therefore lacks (J13):
it is listed as held by no class of this table.

The colour table is the track's file row for row, restated as TOML so the
crate reads one format: 250 yellow, 79 red, 32 unknown at 2026-10-04. Its
check against the count of bestiary mods on the owner's copy is the
track's (`search/category/README.md`, "Round l") and is not repeated here:
it reads the copy, which is local.
"""

import collections
import csv
import json
import os
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
FILTERS = os.path.join(ROOT, "search", "trade-query", "data", "filters-2026-09-12.json")
FILTER_DATE = "2026-09-12"
CLASS_TABLE = os.path.join(ROOT, "crates", "acquisition-search", "reference", "classes-v1.toml")
GROUPINGS = os.path.join(ROOT, "search", "category", "data", "grouping-members.csv")
COLOURS = os.path.join(ROOT, "search", "category", "data", "beast-colours.csv")
OUT_CATEGORIES = os.path.join(ROOT, "crates", "acquisition-search", "reference", "categories-v1.toml")
OUT_COLOURS = os.path.join(ROOT, "crates", "acquisition-search", "reference", "beast-colours-v1.toml")
VERSION = 1
COLOURS_VERSION = 1

# Rule 2: the id an item of a class has, the class as the class table names
# it (search/category/data/categories.csv, the columns `classes` and
# `produced_by`; README F1: 58 leaves a class's). `map` and `currency` are
# parents and a class's own id at once (F2; J2).
CLASSES = {
    "weapon.bow": ["Bows"],
    "weapon.claw": ["Claws"],
    "weapon.basedagger": ["Daggers"],
    "weapon.runedagger": ["Rune Daggers"],
    "weapon.oneaxe": ["One Hand Axes"],
    "weapon.basesword": ["One Hand Swords"],
    "weapon.rapier": ["Thrusting One Hand Swords"],
    "weapon.basemace": ["One Hand Maces"],
    "weapon.sceptre": ["Sceptres"],
    "weapon.basestaff": ["Staves"],
    "weapon.warstaff": ["Warstaves"],
    "weapon.twoaxe": ["Two Hand Axes"],
    "weapon.twomace": ["Two Hand Maces"],
    "weapon.twosword": ["Two Hand Swords"],
    "weapon.wand": ["Wands"],
    "weapon.rod": ["Fishing Rods"],
    "armour.chest": ["Body Armours"],
    "armour.boots": ["Boots"],
    "armour.gloves": ["Gloves"],
    "armour.helmet": ["Helmets"],
    "armour.shield": ["Shields"],
    "armour.quiver": ["Quivers"],
    "accessory.amulet": ["Amulets"],
    "accessory.belt": ["Belts"],
    "accessory.ring": ["Rings"],
    "accessory.trinket": ["Trinkets"],
    "gem.activegem": ["Skill Gems"],
    "gem.supportgem": ["Support Gems"],
    "jewel.base": ["Jewels"],
    "jewel.abyss": ["Abyss Jewels"],
    "flask": ["Hybrid Flasks", "Life Flasks", "Mana Flasks", "Utility Flasks"],
    "map": ["Instance Local Items", "Maps", "Misc Map Items", "Vault Keys"],
    "map.fragment": ["Labyrinth Map Items", "Map Fragments"],
    "map.breachstone": ["Breachstones"],
    "leaguestone": ["Leaguestones"],
    "memoryline": ["Memories"],
    "card": ["Divination Cards"],
    "heistequipment.heistweapon": ["Heist Gear"],
    "heistequipment.heisttool": ["Heist Tools"],
    "heistequipment.heistutility": ["Heist Cloaks"],
    "heistequipment.heistreward": ["Heist Brooches"],
    "heistmission.contract": ["Contracts"],
    "heistmission.blueprint": ["Blueprints"],
    "logbook": ["Expedition Logbooks"],
    "sanctum.research": ["Sanctum Research"],
    "sanctum.relic": ["Relics", "Sanctified Relics"],
    "tincture": ["Tinctures"],
    "corpse": ["Corpses"],
    "idol": ["Idols"],
    "wombgift": ["Wombgifts"],
    "enshrouded": ["Enshrouded Items"],
    "chart": ["Chart"],
    "currency": ["Currency", "Gold", "Stackable Currency"],
    "currency.piece": ["Pieces"],
    "currency.resonator": ["Delve Socketable Currency", "Delve Stackable Socketable Currency"],
    "currency.incubator": ["Incubators"],
    "currency.heistobjective": ["Heist Targets"],
}

# Rule 3: (the leaf, the class, how the base is tested, the text, what says
# so). Five are the track's name rules (search/repoe/scripts/
# base-taxonomy.py, NAME_RULES; README F1: 7 leaves a class's with a name
# rule); the invitation's is J3's and the tattoo's J23's, each of which
# replaced that file's.
NAME_RULES = [
    ("gem.supportgemplus", "Support Gems", "starts", "Awakened ", "the leaf's text, Awakened Support Gem"),
    ("jewel.cluster", "Jewels", "ends", "Cluster Jewel", "the leaf's text, Cluster Jewel"),
    ("map.scarab", "Map Fragments", "contains", "Scarab", "the leaf's text, Scarab"),
    ("map.invitation", "Misc Map Items", "starts", "Maven's Invitation",
     "J3: the leaf holds what its text says, a base named for the Maven; every other invitation takes `map` (round j, j08)"),
    ("currency.fossil", "Stackable Currency", "ends", " Fossil", "the leaf's text, Fossil"),
    ("currency.omen", "Stackable Currency", "starts", "Omen of ", "the leaf's text, Omen"),
    ("currency.tattoo", "Stackable Currency", "contains", "Tattoo",
     "J23: a base whose name holds the word, not only one beginning `Tattoo of ` — the site lists Honoured and Journey tattoos under the leaf (round m, m01, m03)"),
]

# Rule 3's check: a base of a rule's class that holds the leaf's word and
# that the rule misses, each named with why the miss is right. The word is
# the first word of the site's text for the leaf, a possessive dropped
# (Awakened, Cluster, Scarab, Maven, Fossil, Omen, Tattoo), in any case. A
# near miss not named here stops the script: it is a rule too narrow (the
# tattoo rule was, until J23) or an exception to write down.
NEAR_MISSES = {
    ("currency.fossil", "Fossilised Delirium Orb"):
        "a Delirium Orb, named for what it does to a map: no Fossil, and `currency` by its class",
}

# Rule 2's other half: the classes of the class table that give no id
# (README F6; categories.csv names no id for them). An item of one lacks a
# category: known absence (C105; J19 for a quest-frame invitation).
NO_ID = [
    "Archnemesis Mods", "Atlas Upgrade Items", "Charms", "Cursed Ducat", "Embers of the Allflame",
    "Hidden Items", "Incursion Items", "Labyrinth Items", "Labyrinth Trinkets", "Metamorph Samples",
    "Microtransactions", "Pantheon Souls", "Quest Items", "Sentinels",
]

# Rule 5.
BEAST = "monster.beast"
COLOUR_LEAVES = {"monster.yellowbeast": "yellow", "monster.redbeast": "red"}
COLOUR_WORDS = ["yellow", "red", "unknown"]

# An id no class of the class table gives, and why (the module docstring).
NO_CLASS = {
    "graft": "J13: the export prints no name for its class (RemovedItem), so the class table lacks its bases and each is undecided",
}


def toml_str(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def rows_of(path):
    with open(path, newline="") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


def site_ids():
    filters = json.load(open(FILTERS))
    for group in filters["result"]:
        if group["id"] != "type_filters":
            continue
        for one in group["filters"]:
            if one["id"] == "category":
                return [(o["id"], o["text"]) for o in one["option"]["options"] if o["id"] is not None]
    sys.exit(f"{FILTERS} holds no type_filters.category")


def categories():
    ids = site_ids()
    known = [i for i, _ in ids]
    if len(set(known)) != len(known):
        sys.exit("the filter spells an id twice")
    table = tomllib.load(open(CLASS_TABLE, "rb"))
    class_names = [c["name"] for c in table["class"]]

    # rule 2: every class placed once, every class named a class of the table
    placed = collections.Counter(c for classes in CLASSES.values() for c in classes)
    placed.update(NO_ID)
    for name, n in placed.items():
        if name not in class_names:
            sys.exit(f"`{name}` is no class of the class table (classes v{table['version']})")
        if n > 1:
            sys.exit(f"the class `{name}` is placed {n} times: one class, one id — two classes of one displayed name under two ids is a question for the owner")
    unplaced = [c for c in class_names if c not in placed]
    if unplaced:
        sys.exit(f"classes of the class table this script places nowhere: {', '.join(unplaced)}")
    for leaf, cls, how, text, _ in NAME_RULES:
        if leaf not in known or cls not in class_names or how not in ("starts", "ends", "contains") or not text:
            sys.exit(f"a name rule that names no id, no class or no test: {leaf}")
        if cls not in placed or cls in NO_ID:
            sys.exit(f"the name rule of `{leaf}` is on a class with no id of its own: {cls}")
    # rule 3's check: the near misses, each named or the script stops
    bases_of = {c["name"]: c["bases"] for c in table["class"]}
    texts = dict(ids)
    tests = {"starts": str.startswith, "ends": str.endswith, "contains": str.__contains__}
    near = {}
    for leaf, cls, how, text, _ in NAME_RULES:
        word = texts[leaf].split()[0].removesuffix("'s").lower()
        for base in bases_of[cls]:
            if word in base.lower() and not tests[how](base, text):
                near[(leaf, base)] = word
    unnamed = sorted(k for k in near if k not in NEAR_MISSES)
    if unnamed:
        sys.exit("bases that hold a leaf's word and miss its name rule, none named as an exception: "
                 + "; ".join(f"`{base}` ({leaf}, the word `{near[(leaf, base)]}`)" for leaf, base in unnamed))
    stale = sorted(k for k in NEAR_MISSES if k not in near)
    if stale:
        sys.exit(f"named near misses that are none: {stale}")
    for leaf in list(CLASSES) + list(NO_CLASS) + [BEAST] + list(COLOUR_LEAVES):
        if leaf not in known:
            sys.exit(f"`{leaf}` is no id of the filter")

    # rule 4: the groupings' cells
    members = collections.OrderedDict()
    cells = rows_of(GROUPINGS)
    for cell in cells:
        if cell["grouping"] not in known or cell["leaf"] not in known:
            sys.exit(f"a grouping cell names no id: {cell['grouping']} {cell['leaf']}")
        if cell["member"] not in ("in", "out"):
            sys.exit(f"a grouping cell is neither in nor out: {cell}")
        members.setdefault(cell["grouping"], [])
        if cell["member"] == "in":
            if cell["leaf"] not in CLASSES:
                sys.exit(f"a grouping's member is no class's leaf: {cell['leaf']}")
            members[cell["grouping"]].append((cell["leaf"], cell["said_by"]))

    rules = {leaf: (cls, how, text, says) for leaf, cls, how, text, says in NAME_RULES}
    out_rows = []
    kinds = collections.Counter()
    for i, text in ids:
        under = [j for j in known if j.startswith(i + ".")]
        row = [("id", toml_str(i)), ("text", toml_str(text))]
        cites = []
        if i in members:
            if i in CLASSES or i in rules or under:
                sys.exit(f"`{i}` is a grouping and something else")
            kind = "grouping"
            row.append(("members", members[i]))
            cites.append("J4, J12: its reviewed members alone, search/category/data/grouping-members.csv")
        elif i == BEAST:
            kind = "beast"
            cites.append("J1: a captured beast, known by its bestiary line; J10: never by its class")
        elif i in COLOUR_LEAVES:
            kind = "colour"
            row.append(("colour", toml_str(COLOUR_LEAVES[i])))
            row.append(("under", toml_str(BEAST)))
            cites.append("J1: the beasts the colour table gives this colour; J15: under monster.beast")
        elif under:
            kind = "parent"
            cites.append("J2: its own classes and every id under it by the dot")
        elif i in CLASSES or i in rules or i in NO_CLASS:
            kind = "leaf"
        else:
            sys.exit(f"`{i}` is an id nothing here explains")
        if i in CLASSES:
            row.append(("classes", "[" + ", ".join(toml_str(c) for c in CLASSES[i]) + "]"))
            cites.append("C106: a class's; search/category/data/categories.csv")
        if i in rules:
            cls, how, test, says = rules[i]
            row.append(("rule", "{ class = " + toml_str(cls) + ", " + how + " = " + toml_str(test) + " }"))
            cites.append("C106: a class's with a rule on the base name; " + says)
        if i in NO_CLASS:
            cites.append(NO_CLASS[i])
        row.insert(2, ("kind", toml_str(kind)))
        row.append(("cites", toml_str("; ".join(cites))))
        kinds[kind] += 1
        out_rows.append(row)

    n_classes = sum(len(c) for c in CLASSES.values())
    n_in = sum(len(m) for m in members.values())
    out = [
        f"# The category table — v{VERSION} (C106, C68; decisions/search.md), item search step 9d-B.",
        "#",
        "# Reference data: reviewed, committed, shipped inside the binary",
        "# (`include_str!` in `src/category.rs`), read-only, never in a store file,",
        "# enumerable by `acq search --describe category`, cited by version in every",
        "# basis (C98). Generated by tools/category-table.py from the trade site's",
        f"# category filter as the owner captured it {FILTER_DATE}",
        "# (search/trade-query/data/filters-2026-09-12.json; SURFACES.md), the class",
        "# table and the category track's reviewed grouping members, and reviewed as",
        "# a diff at each change (the admission test, search/DESIGN.md, C106): a",
        "# convention — the word is the trade site's — never a judgment. Its rules",
        "# are the script's docstring.",
        "#",
        "# What a row means: the id is the site's, as spelled, with the site's text",
        "# for it. `classes`: an item of one of these classes has the id. `rule`: an",
        "# item of that class whose base the test holds has this id in place of the",
        "# class's own. `members`: a grouping holds an item that has one of these",
        "# leaves, and no other. `under`: an item that has this id has that one too.",
        "# An id holds an item that has an id under it by the dot. What an id",
        "# answers of an item from there is `src/category.rs`, said once.",
        "#",
        f"# Measured at generation: {len(ids)} ids — {', '.join(f'{kinds[k]} {k}' for k in ('parent', 'grouping', 'leaf', 'beast', 'colour'))};",
        f"# {n_classes} classes of the class table's {len(class_names)} give an id, {len(NO_ID)} give none;",
        f"# {len(NAME_RULES)} name rules, {len(near)} near {'miss' if len(near) == 1 else 'misses'} named ({', '.join(base for _, base in sorted(near))});",
        f"# {n_in} of {len(cells)} grouping cells in.",
        "",
        f"version = {VERSION}",
        f"realms = [{', '.join(toml_str(r) for r in table['realms'])}]",
        f"source = {toml_str(f'the trade site’s category filter, captured by the owner {FILTER_DATE} (search/trade-query/data/filters-2026-09-12.json, type_filters.category; SURFACES.md); the class table, classes v' + str(table['version']) + '; the reviewed grouping members, search/category/data/grouping-members.csv')}",
        f"filter = {toml_str(FILTER_DATE)}",
        f"classes = {table['version']}",
        'generated_by = "tools/category-table.py"',
        "",
    ]
    for row in out_rows:
        out.append("[[id]]")
        for key, value in row:
            if key == "members":
                out.append("members = [")
                for leaf, said in value:
                    out.append(f"  {{ leaf = {toml_str(leaf)}, said_by = {toml_str(said)} }},")
                out.append("]")
            else:
                out.append(f"{key} = {value}")
        out.append("")
    return "\n".join(out), table


def colours(table):
    rows = rows_of(COLOURS)
    names = [r["name"] for r in rows]
    if len(set(names)) != len(names):
        sys.exit("the colour table names a beast twice")
    count = collections.Counter()
    for r in rows:
        if r["colour"] not in COLOUR_WORDS:
            sys.exit(f"`{r['name']}` has a colour the site has no leaf for: {r['colour']}")
        if not r["name"] or not r["evidence"]:
            sys.exit(f"a colour row with no name or no search that evidences it: {r}")
        count[r["colour"]] += 1
    out = [
        f"# The beast colour table — v{COLOURS_VERSION} (C106, C68; decisions/search.md), item search step 9d-B.",
        "#",
        "# Reference data: reviewed, committed, shipped inside the binary",
        "# (`include_str!` in `src/category.rs`), read-only, never in a store file,",
        "# named by `acq search --describe category`, cited by version in every",
        "# basis (C98). Generated by tools/category-table.py from",
        "# search/category/data/beast-colours.csv, the category track's reading of",
        "# the owner's sittings of 2026-10-04 (rounds k and l; SURFACES.md), and",
        "# reviewed as a diff at each change. It is admitted under clause (a) of",
        "# the admission test (search/DESIGN.md, C106): the trade site publishes the",
        "# words Yellow Beast and Red Beast and no definition of them, so the",
        "# definition is evidenced by dated observations of what the site answers,",
        "# each row citing its searches. A convention, never a judgment.",
        "#",
        "# What a row means: the site lists a captured beast of this name under the",
        "# leaf of this colour — `yellow` found under Yellow Beast (round k), `red`",
        "# found under Red Beast (round l), `unknown` found under neither. A beast",
        "# whose name is `unknown` here, or is not here, has no colour the search",
        "# can state: its colour leaves are undecided (J1; the owner, 2026-10-05:",
        "# \"Let's leave the undecided beasts undecided.\"). `evidence` names the",
        "# searches by their rows in search/pseudo-stats/data/search-sheet.csv.",
        "#",
        f"# Measured at generation: {len(rows)} names — {', '.join(f'{count[w]} {w}' for w in COLOUR_WORDS)}.",
        "",
        f"version = {COLOURS_VERSION}",
        f"realms = [{', '.join(toml_str(r) for r in table['realms'])}]",
        f"source = {toml_str('the trade site’s answers to the owner’s sittings of 2026-10-04, rounds k and l: each of the site’s 361 monster names asked under Yellow Beast and, where not found, under Red Beast (search/category/data/beast-colours.csv; SURFACES.md)')}",
        'generated_by = "tools/category-table.py"',
        "",
    ]
    for r in rows:
        out.append("[[beast]]")
        out.append(f"name = {toml_str(r['name'])}")
        out.append(f"colour = {toml_str(r['colour'])}")
        out.append(f"evidence = {toml_str(r['evidence'])}")
        out.append("")
    return "\n".join(out)


def main():
    check = "--check" in sys.argv[1:]
    categories_text, table = categories()
    texts = [(OUT_CATEGORIES, categories_text), (OUT_COLOURS, colours(table))]
    if check:
        for path, text in texts:
            if not os.path.exists(path) or open(path, newline="").read() != text:
                sys.exit(f"{path} differs from what the sources give: regenerate and review the diff")
            print(f"ok      {os.path.relpath(path, ROOT)} is what the sources give")
        return
    for path, text in texts:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", newline="") as f:
            f.write(text)
        print(f"wrote   {os.path.relpath(path, ROOT)}")


if __name__ == "__main__":
    main()
