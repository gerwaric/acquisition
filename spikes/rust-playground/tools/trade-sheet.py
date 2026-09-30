#!/usr/bin/env python3
"""Write the sheet of trade searches the owner runs: search/pseudo-stats/data/search-sheet.csv.

A search is a query object; the site's search id is that object, serialized
compactly, gzipped and base64url-encoded without padding (trade-query F9), so
a link is composed here and opened by a human in a browser. Nothing in this
script touches the network (C79; SURFACES.md, the trade site's rows: access
method `browser`).

    tools/trade-sheet.py                # the pilot, the checks and the rounds: what the owner is asked to run
    tools/trade-sheet.py --method if    # the pilot and the batch, one search per candidate line
    tools/trade-sheet.py --method and   # the pilot and the batch, one search per pair
    tools/trade-sheet.py --self-test    # compose each captured request and compare ids
    tools/trade-sheet.py --verify       # decode every link in the sheet, compare its query

The checks ask the site of every listed item at once whether a pseudo's rows
are complete and sound (`checks` and `round_two`, below, the second written
from tools/trade_rows.py); the pilot showed the method, and the committed sheet is
the pilot and the checks. A generated search pins the version of the rows it
was composed from and says so in what it decides. The batch is the form before it,
kept one command away and not run: 264 searches, one line at a time.

The batch is generated from search/pseudo-stats/data/candidates.csv (search/pseudo-stats/scripts/candidates.py),
in two forms, since the pilot's answer was not in: `if` — one search per
candidate line, every pseudo it might feed in one `if` group (p2 asks whether
an `if` group shows a value at all); `and` — one search per pair, the pseudo
required as in p1. Rows are ordered by what a search is worth: the pilot, then
lines whose best pair is `sources-differ`, `text-only`, `one-source`,
`sources-agree`, then the percentile's own cases (PERCENTILE_CASES), then
the first pass's open question 5 (R1).

The self-test reads `search/trade-query/raw/searches/` (local only, never
committed): for every `<q>-request.json` with a `<q>-search.json` beside it,
the id composed from the request must equal the id the site returned. It
skips, saying so, where the captures are absent.

Each search names what it decides. A search whose answer would read the same
under two hypotheses is not a search: every one carries a control — a pseudo
the returned items are known to have — so that an absent value is the site's
answer and never the method's failure.
"""

import base64
import csv
import json
import struct
import sys
import zlib
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1] / "search" / "pseudo-stats"
OUT = TRACK / "data" / "search-sheet.csv"
CAPTURED = TRACK.parent / "trade-query" / "raw" / "searches"

SITE = "https://www.pathofexile.com/trade/search"
LEAGUE = "Standard"

# The ten bytes every captured id opens with: gzip, deflate, no flags, no
# time, no extra flags, OS 3.
GZIP_HEADER = bytes.fromhex("1f8b0800000000000003")


def compose(query):
    """The site's id for a query object."""
    body = json.dumps(query, separators=(",", ":"), ensure_ascii=False).encode()
    deflate = zlib.compressobj(6, zlib.DEFLATED, -15)
    packed = deflate.compress(body) + deflate.flush()
    trailer = struct.pack("<II", zlib.crc32(body), len(body))
    whole = GZIP_HEADER + packed + trailer
    return base64.urlsafe_b64encode(whole).decode().rstrip("=")


def link(query, league=LEAGUE):
    return f"{SITE}/{league}/{compose(query)}"


def query(stats, rarity="rare"):
    """A query in the shape the owner's captures pin (trade-query F1)."""
    return {
        "status": {"option": "any"},
        "stats": stats,
        "filters": {
            "type_filters": {
                "disabled": False,
                "filters": {"rarity": {"option": rarity}},
            },
            "trade_filters": {
                "disabled": True,
                "filters": {"sale_type": {"option": "any"}},
            },
        },
    }


def group(kind, ids, value=None):
    out = {"type": kind, "filters": [{"id": i, "disabled": False} for i in ids]}
    if value is not None:
        out["value"] = value
    return out


# The pilot: two searches that settle the method before the batch is written.
# Both are new to the site, so either opening shows a composed link is taken.
PILOT = [
    {
        "search": "p1",
        "decides": (
            "the method: a composed link opens. The evidence: whether "
            "`+# to Strength and Intelligence` counts toward "
            "`+# total maximum Life`, and at what weight — the total less "
            "the item's life lines and half its Strength lines is the answer"
        ),
        "control": "pseudo.pseudo_total_life is required, so every item shows it",
        "query": query(
            [
                group("and", ["pseudo.pseudo_total_life"]),
                # Two ids display this one text (trade-query F4), so the line
                # goes out as a count of at least one (the contract detail, C99).
                group(
                    "count",
                    ["explicit.stat_1535626285", "explicit.stat_2543977012"],
                    {"min": 1},
                ),
            ]
        ),
    },
    {
        "search": "p2",
        "decides": (
            "the method: an `if` group shows a pseudo's value without "
            "filtering by it. The evidence: whether "
            "`#% increased Attack and Cast Speed` counts toward "
            "`+#% total Attack Speed` and `+#% total Cast Speed`"
        ),
        "control": (
            "every item carries `+#% to Fire Resistance`, so "
            "pseudo.pseudo_total_fire_resistance shown and a speed total "
            "absent is the site's answer; none of the three shown means an "
            "`if` group shows nothing"
        ),
        "query": query(
            [
                group(
                    "and",
                    ["explicit.stat_2672805335", "explicit.stat_3372524247"],
                ),
                group(
                    "if",
                    [
                        "pseudo.pseudo_total_fire_resistance",
                        "pseudo.pseudo_total_attack_speed",
                        "pseudo.pseudo_total_cast_speed",
                    ],
                ),
            ]
        ),
    },
]

CANDIDATES = TRACK / "data" / "candidates.csv"
FETCH_CENSUS = "search/trade-query/data/fetch-census.json"

# The controls: a line the site was seen counting under a pseudo, so that an
# item carrying the line shows the pseudo. Each names what it also feeds, so
# a candidate the control feeds is read by value, never by presence.
CONTROLS = {
    "strength": {
        "line": "explicit.stat_4080418644",
        "text": "+# to Strength",
        "pseudo": "pseudo.pseudo_total_life",
        "evidence": f"{FETCH_CENSUS} queries.q4.fetched[7]: +9 to Strength counted at 0.5",
        "feeds": {"pseudo.pseudo_total_life", "pseudo.pseudo_total_strength"},
    },
    "fire": {
        "line": "explicit.stat_3372524247",
        "text": "+#% to Fire Resistance",
        "pseudo": "pseudo.pseudo_total_fire_resistance",
        "evidence": f"{FETCH_CENSUS} queries.q5: counted on all ten items",
        "feeds": {"pseudo.pseudo_total_fire_resistance", "pseudo.pseudo_total_elemental_resistance",
                  "pseudo.pseudo_total_resistance"},
    },
}
PERCENTILE = "pseudo.pseudo_base_defence_percentile"
PERCENTILE_CONTROL = (
    "extended.base_defence_percentile: every armour item fetched carries it "
    f"(30 of 30, {FETCH_CENSUS}); an armour item returned without it is the site's answer"
)
CATEGORY_ORDER = ["explicit", "implicit", "enchant", "crafted", "fractured", "scourge", "crucible"]
STATUS_ORDER = ["sources-differ", "text-only", "one-source", "sources-agree"]


def batch_query(stats, filters=None, item_type=None):
    """A query in the pilot's shape without a rarity: a line can be on any."""
    out = {"status": {"option": "any"}}
    if item_type:
        out["type"] = item_type
    out["stats"] = stats
    out["filters"] = dict(filters or {})
    out["filters"]["trade_filters"] = {"disabled": True, "filters": {"sale_type": {"option": "any"}}}
    return out


def category(option):
    return {"type_filters": {"disabled": False, "filters": {"category": {"option": option}}}}


def percentile_filter(minimum=1):
    return {"armour_filters": {"disabled": False, "filters": {"base_defence_percentile": {"min": minimum}}}}


def line_filter(stat_ids):
    """The line as a filter: one id in an `and` group; a text two ids display
    goes out as a `count` of at least one (C99; p1). The ids are those of one
    category: explicit when the line has one, else the first the line has."""
    ids = [i for i in stat_ids.split(";") if i]
    for cat in CATEGORY_ORDER + sorted({i.split(".")[0] for i in ids}):
        chosen = sorted(i for i in ids if i.split(".")[0] == cat)
        if chosen:
            break
    kind, value = ("and", None) if len(chosen) == 1 else ("count", {"min": 1})
    out = group(kind, chosen, value)
    # an id the capture lists as `<id>|<option>` is one option of a stat:
    # it goes out as the stat with that option (APT pathofexile-trade.ts L101)
    for f in out["filters"]:
        base, sep, option = f["id"].partition("|")
        if sep:
            f["id"] = base
            f["value"] = {"option": int(option) if option.isdigit() else option}
    return out, chosen


def read_candidates():
    with CANDIDATES.open(newline="") as fh:
        rows = list(csv.DictReader(fh))
    lines = {}
    for r in rows:
        if not r["search"] or r["search"].startswith("none") or r["note"].startswith("family:"):
            continue
        line_id, _, k = r["search"].partition(".")
        e = lines.setdefault(line_id, {"stat_ids": r["stat_ids"], "pairs": {}, "templates": set(),
                                       "status": [], "items": 0, "families": 0})
        e["templates"].add(r["template"])
        e["status"].append(STATUS_ORDER.index(r["status"]))
        e["items"] += int(r["corpus_items"])
        if r["note"].startswith("represents"):
            e["families"] += 1
        pair = e["pairs"].setdefault(k or "1", {"pseudo": r["pseudo"], "text": r["pseudo_text"],
                                                "status": r["status"]})
        if pair["pseudo"] != r["pseudo"]:
            raise SystemExit(f"{r['search']} names two pseudos")
    return lines


def pick_control(pseudos):
    """The control that feeds the fewest of the candidates."""
    return min(CONTROLS, key=lambda c: (len(CONTROLS[c]["feeds"] & set(pseudos)), c != "strength"))


def control_text(name, read_by_value):
    c = CONTROLS[name]
    text = (f"{c['pseudo']} shown: `{c['text']}` is required, and the site counted it "
            f"({c['evidence']}); a candidate absent beside it is the site's answer")
    if read_by_value:
        text += "; the control also feeds " + ", ".join(sorted(read_by_value)) + ", read by value"
    text += ". No result decides nothing: rerun with the other control"
    return text


def line_names(e):
    return " / ".join(f"`{t}`" for t in sorted(e["templates"]))


def if_searches(lines):
    out = []
    for line_id, e in lines.items():
        if line_id.startswith("P"):
            continue
        pseudos = [e["pairs"][k]["pseudo"] for k in sorted(e["pairs"], key=int)]
        name = pick_control(pseudos)
        c = CONTROLS[name]
        lf, _ = line_filter(e["stat_ids"])
        shown = [c["pseudo"]] + [p for p in pseudos if p != c["pseudo"]]
        stats = [lf, group("and", [c["line"]]), group("if", shown)]
        fam = f" (represents {e['families']} families)" if e["families"] else ""
        out.append({
            "search": line_id,
            "decides": (f"whether {line_names(e)}{fam} counts toward each of: "
                        + "; ".join(f"`{e['pairs'][k]['text']}` ({line_id}.{k}, "
                                    f"{e['pairs'][k]['status']})" for k in sorted(e["pairs"], key=int))
                        + ". A pseudo shown, less the item's other lines it is known to count, is the "
                          "line's weight; a pseudo absent while the control shows is not fed"),
            "control": control_text(name, c["feeds"] & set(pseudos)),
            "query": batch_query(stats),
            "rank": (min(e["status"]), -e["items"], line_id),
        })
    return out


def and_searches(lines):
    out = []
    for line_id, e in lines.items():
        if line_id.startswith("P"):
            continue
        lf, _ = line_filter(e["stat_ids"])
        for k in sorted(e["pairs"], key=int):
            pair = e["pairs"][k]
            pid = pair["pseudo"]
            if pid == CONTROLS["strength"]["pseudo"]:
                stats = [group("and", [pid]), lf]
                control = f"{pid} is required, so every item shows it (p1)"
            else:
                name = pick_control([pid])
                c = CONTROLS[name]
                stats = [lf, group("and", [c["line"], c["pseudo"], pid])]
                control = control_text(name, c["feeds"] & {pid})
            out.append({
                "search": f"{line_id}.{k}",
                "decides": (f"whether {line_names(e)} counts toward `{pair['text']}` ({pair['status']}): "
                            "the pseudo's value less the item's other lines it is known to count is the "
                            "line's weight; no result, with the line common on the site, means not fed"),
                "control": control,
                "query": batch_query(stats),
                "rank": (STATUS_ORDER.index(pair["status"]), -e["items"], line_id, int(k)),
            })
    return out


def percentile_line_searches(lines):
    out = []
    for line_id, e in lines.items():
        if not line_id.startswith("P"):
            continue
        lf, _ = line_filter(e["stat_ids"])
        pair = e["pairs"]["1"]
        out.append({
            "search": line_id,
            "decides": (f"whether the site reads {line_names(e)} into `#% Base Defence Percentile` "
                        f"({pair['status']}): recover each roll as search/pseudo-stats/scripts/percentile.py does, with the "
                        "line read and without; the reading that gives the site's value is the answer. "
                        "No result: the line is not seen on armour with a percentile above 0"),
            "control": PERCENTILE_CONTROL,
            "query": batch_query([lf], {**category("armour"), **percentile_filter()}),
            "rank": (STATUS_ORDER.index(pair["status"]), -e["items"], line_id),
        })
    return out


# What no capture reaches (README, the percentile): each case is fetched and
# scored by search/pseudo-stats/scripts/percentile.py's reading.
PERCENTILE_CASES = [
    ("C1", "three rolls averaged: a base with armour, evasion and energy shield",
     batch_query([], percentile_filter(), "Sacrificial Garb")),
    ("C2", "ward read like the other defences: a ward base",
     batch_query([], percentile_filter(), "Runic Crown")),
    ("C3", "a shield's defences (the base also carries block, no roll)",
     batch_query([], {**category("armour.shield"), **percentile_filter()})),
    ("C4", "a helmet (no helmet captured)",
     batch_query([], {**category("armour.helmet"), **percentile_filter()})),
    ("C5", "gloves (none captured)",
     batch_query([], {**category("armour.gloves"), **percentile_filter()})),
    ("C6", "quality above 20: the display's multiplier past the site's 20% figure",
     batch_query([], {**category("armour"), **percentile_filter(),
                      "misc_filters": {"disabled": False, "filters": {"quality": {"min": 21}}}})),
    ("C7", "`Quality does not increase Defences` (the one enchant that zeroes the quality term)",
     batch_query([group("and", ["enchant.stat_2677401098"])], {**category("armour"), **percentile_filter()})),
    ("C8", "a unique's base (APT reads a unique's own base, Parser.ts L1267)",
     batch_query([], {"type_filters": {"disabled": False, "filters": {
         "category": {"option": "armour.chest"}, "rarity": {"option": "unique"}}},
         **percentile_filter()})),
]


# First-pass open question 5: does `# total Resistances` count lines or
# resistance types? One all-elemental line decides it: 1 or 3.
OPEN_QUESTION_SEARCHES = [
    {
        "search": "R1",
        "decides": (
            "first-pass open question 5: whether `# total Resistances` and `# total Elemental "
            "Resistances` count lines or resistances — on an item whose one resistance line is "
            "`+#% to all Elemental Resistances`, 1 is lines and 3 is resistances"
        ),
        "control": (
            "pseudo.pseudo_total_fire_resistance shown: the all-elemental line is required, and the "
            f"site counted it toward fire ({FETCH_CENSUS} queries.q5.fetched[0]); a count absent "
            "beside it is the site's answer. Read only items with no other resistance line"
        ),
        "query": batch_query([
            group("and", ["explicit.stat_2901986750"]),
            group("if", ["pseudo.pseudo_total_fire_resistance", "pseudo.pseudo_count_resistances",
                         "pseudo.pseudo_count_elemental_resistances"]),
        ]),
    },
]


def percentile_case_searches():
    return [{"search": s, "decides": f"the percentile's formula on {what}", "control": PERCENTILE_CONTROL,
             "query": q} for s, what, q in PERCENTILE_CASES]


def batch(method):
    lines = read_candidates()
    main_rows = if_searches(lines) if method == "if" else and_searches(lines)
    ranked = sorted(main_rows + percentile_line_searches(lines), key=lambda r: r["rank"])
    return ranked + percentile_case_searches() + OPEN_QUESTION_SEARCHES


STATS = TRACK.parent / "trade-query" / "data" / "stats-2026-09-12.json"


def ids_of(*texts):
    """Every id, in every category, that displays one of the texts."""
    wanted = set(texts)
    found = {t: [] for t in texts}
    for g in json.loads(STATS.read_text())["result"]:
        if g["id"] == "pseudo":
            continue
        for e in g["entries"]:
            if e["text"] in wanted:
                found[e["text"]].append(e["id"])
    for t, ids in found.items():
        if not ids:
            raise SystemExit(f"no stat displays {t!r}")
    return [i for t in texts for i in found[t]]


# The site as the checker. A pseudo's rows are complete when no listed item
# shows the pseudo while carrying none of them, and sound when no listed item
# carries one while showing no pseudo: each is one search over everything
# listed, where the batch above asks one line at a time of ten items. A
# search that finds nothing proves nothing until the same search, with one
# row taken out, is seen to find something: every check has its mutant.
LIFE = "pseudo.pseudo_total_life"
LIFE_ROWS = ["+# to maximum Life", "+# to Strength", "+# to Strength and Dexterity",
             "+# to Strength and Intelligence", "+# to all Attributes"]
SPEED_ROWS = ["#% increased Attack Speed", "#% increased Attack Speed (Local)"]


def checks():
    return [
        {
            "search": "c1",
            "decides": (
                "complete: whether any listed item shows `+# total maximum Life` while "
                "carrying none of the five lines p1 and q4 evidenced. None found: the rows "
                "are all the site counts. Any found: its lines name what is missing"
            ),
            "control": "c2, this search with one row taken out, must find items",
            "query": batch_query([group("and", [LIFE]), group("not", ids_of(*LIFE_ROWS))]),
        },
        {
            "search": "c2",
            "decides": (
                "c1's mutant: the same search with `+# to all Attributes` left out of the "
                "`not` group. It must find items, each carrying that line and no other row, "
                "each showing half of it"
            ),
            "control": "the mutant is the control: nothing found means a `not` group of this size decides nothing",
            "query": batch_query([group("and", [LIFE]),
                                  group("not", ids_of(*[t for t in LIFE_ROWS if t != "+# to all Attributes"]))]),
        },
        {
            "search": "c3",
            "decides": (
                "sound: whether any listed item carries one of the five lines while showing "
                "no `+# total maximum Life` — the pseudo inside a `not` group. None found: "
                "every row is counted wherever it appears"
            ),
            "control": "c4, this search with an uncounted line among the rows, must find items",
            "query": batch_query([group("count", ids_of(*LIFE_ROWS), {"min": 1}),
                                  group("not", [LIFE])]),
        },
        {
            "search": "c4",
            "decides": (
                "c3's mutant: `#% increased maximum Life`, which p1 showed uncounted, added "
                "to the rows. It must find items carrying that line and no counted one"
            ),
            "control": "the mutant is the control: nothing found means a pseudo inside a `not` group decides nothing",
            "query": batch_query([group("count", ids_of(*LIFE_ROWS, "#% increased maximum Life"), {"min": 1}),
                                  group("not", [LIFE])]),
        },
        {
            "search": "c5",
            "decides": (
                "the method: every member of an `if` group displays, not the first alone "
                "(p2's control was its group's first). Every item carries a cold and a fire "
                "resistance line, so both totals must show, the second as well as the first"
            ),
            "control": "`+#% to Cold Resistance` and `+#% to Fire Resistance` are both required; the site counted fire (q5, p2)",
            "query": batch_query([
                group("and", ["explicit.stat_4220027924", "explicit.stat_3372524247"]),
                group("if", ["pseudo.pseudo_total_cold_resistance",
                             "pseudo.pseudo_total_fire_resistance"]),
            ]),
        },
        {
            "search": "c6",
            "decides": (
                "complete, for a total the build ships: whether any listed item shows "
                "`+#% total Attack Speed` while carrying no `#% increased Attack Speed` "
                "line, local or not — the shipped total's one row. What is found names "
                "what else the site counts, the combined speed line of p2 among the candidates"
            ),
            "control": "c1 and c2 show what a `not` group over a line's every id finds",
            "query": batch_query([group("and", ["pseudo.pseudo_total_attack_speed"]),
                                  group("not", ids_of(*SPEED_ROWS))]),
        },
    ]


sys.path.insert(0, str(Path(__file__).resolve().parent))
import trade_rows as table  # noqa: E402


def weighted(version, ids, extra=(), at_least=0.5):
    """A version's rows as a `weight2` group: each id at its row's weight, the
    sum at least the smallest a row can give. Where a `count` of the rows finds
    every item whose lines cancel (c3: 3,338 found, most of them a sum of
    nothing), the site's own sum leaves those out."""
    filters = []
    for (t, which), w in version["rows"].items():
        for i in ids[t]:
            if (i not in version["never"] and table.means(which, i)) or i in extra:
                filters.append({"id": i, "value": {"weight": float(w)}, "disabled": False})
    return {"type": "weight2", "filters": filters, "value": {"min": at_least}}


def complete(search, name, number, version, ids, texts):
    text = texts[version["pseudo"]]
    counted = table.ids_of(version, ids)
    stats = [group("and", [version["pseudo"]])]
    if counted:
        stats.append(group("not", counted))
        decides = (f"{name} v{number}, complete: whether any listed item shows `{text}` while carrying none of "
                   f"its {len(version['rows'])} rows ({len(counted)} ids). None found: the rows "
                   "are all the site counts. Any found: its lines name what is missing")
    else:
        decides = (f"{name} v{number}, what carries `{text}`: no line displays the pseudo's text, so the items "
                   "found name its first rows")
    return {
        "search": search,
        "decides": decides,
        "control": "c1 and c2: a `not` group over every id of a row finds what it should",
        "query": batch_query(stats),
    }


def round_two():
    """d01–d24. Each pins the version of the rows it was composed from, so a
    later version changes no search here: a capture answers the row it names."""
    ids, texts = table.stats()
    versions = table.versions()
    life = versions["total_life"][1]
    speed = versions["total_attack_speed"][1]
    out = [
        {
            "search": "d01",
            "decides": (
                "total_life v2, sound, by the site's own sum: whether any listed item's rows sum to a half "
                "or more while it shows no `+# total maximum Life` — the rows weighted in a "
                "`weight2` group, the twin id c3 found left out. None found: c3's two kinds "
                "were all there is"
            ),
            "control": "d02, this search with the twin id among the rows, must find items",
            "query": batch_query([weighted(life, ids), group("not", [life["pseudo"]])]),
        },
        {
            "search": "d02",
            "decides": (
                "d01's mutant: the twin id `explicit.stat_2543977012` weighted with the rows. "
                "It must find That Which Was Taken, which c3 showed carrying the line and no total"
            ),
            "control": "the mutant is the control: nothing found means a `weight2` group beside a `not` decides nothing",
            "query": batch_query([weighted(life, ids, extra=life["never"]),
                                  group("not", [life["pseudo"]])]),
        },
        complete("d03", "total_attack_speed", 2, speed, ids, texts),
    ]
    order = [f"adds_{k}{scope}" for k in ("fire", "elemental", "damage", "cold", "lightning",
                                          "physical", "chaos")
             for scope in table.SCOPES]
    for number, name in enumerate(order, start=4):
        out.append(complete(f"d{number:02d}", name, 1, versions[name][0], ids, texts))
    return out


def sound(search, name, number, version, ids, texts, at_least=0.5):
    """Whether any listed item carries a row while showing no pseudo. A total
    whose lines carry a sign is asked by the site's own sum, which leaves out
    most items whose lines cancel (d01 found one where c3's count found 3,338);
    a ranged pseudo, whose lines have none, by a count of its rows."""
    text = texts[version["pseudo"]]
    if version["reads"] == "avg":
        rows = group("count", table.ids_of(version, ids), {"min": 1})
        how = "a count of its rows"
    else:
        rows = weighted(version, ids, at_least=at_least)
        how = "the site's own sum of its rows, " + ("a half" if at_least == 0.5 else str(at_least)) + " or more"
    return {
        "search": search,
        "decides": (f"{name} v{number}, sound: whether any listed item carries a row — {how} "
                    f"— while showing no `{text}`. None found: every row is counted wherever "
                    "it appears. Any found: a row, a twin or an id the site does not count"),
        "control": "c4 and d02: a pseudo inside a `not` group, beside a count or a sum, finds what it should",
        "query": batch_query([rows, group("not", [version["pseudo"]])]),
    }


def round_three():
    """e001 on: every pseudo in scope at its latest rows when this round was
    written, complete and sound, less what a capture has already closed —
    total life (c1, d01) and the five plain ranged pseudos' completeness
    (d04, d13, d16, d19, d22). Each pins its version."""
    ids, texts = table.stats()
    versions = table.versions()
    pinned = {name: 2 for name in versions if name.startswith("adds_")}
    pinned.update({f"adds_{kind}": 1 for kind in table.TYPES})
    pinned["total_attack_speed"] = 3
    order = [f"adds_{k}{scope}" for k in ("fire", "cold", "lightning", "physical", "chaos",
                                          "elemental", "damage")
             for scope in table.SCOPES]
    order += ["total_attack_speed"]
    order += [n for n in versions if n not in order and n != "total_life" and n not in table.OTHER]
    out = []
    for name in order:
        number = pinned.get(name, 1)
        version = versions[name][number - 1]
        closed = name in ("adds_fire", "adds_cold", "adds_lightning", "adds_physical", "adds_chaos")
        if not closed:
            out.append(complete(f"e{len(out) + 1:03d}", name, number, version, ids, texts))
        out.append(sound(f"e{len(out) + 1:03d}", name, number, version, ids, texts))
    return out


# The most ids a `weight2` group has been seen taken with: the site answers a
# query's cost as `complexity`, 56 and 4 an id for a weighted group beside a
# `not` (96 at 10 ids, 128 at 18, 148 at 23), and refused 41 and 45 ids as too
# complex (e047, e051). A count or a `not` of 71 ids was taken.
WEIGHTED_AT_MOST = 20

# The pseudos whose rows moved after round three was written, each at the
# version round four asks of: written out, so that a later version changes no
# search here.
ROUND_FOUR = {
    "total_cold_res": 3,
    "total_fire_res": 3,
    "total_lightning_res": 3,
    "total_ele_res": 3,
    "total_chaos_res": 3,
    "total_res": 3,
    "total_str": 2,
    "total_int": 2,
    "total_cast_speed": 2,
    "total_increased_phys": 3,
    "total_spell_crit": 2,
    "total_skill_gem_levels": 2,
    "adds_physical": 2,
    "adds_lightning": 2,
    "adds_cold": 2,
    "adds_fire": 2,
    "adds_chaos": 2,
    "adds_elemental": 3,
    "adds_damage": 3,
    "adds_physical_to_attacks": 3,
    "adds_lightning_to_attacks": 3,
    "adds_cold_to_attacks": 3,
    "adds_fire_to_attacks": 3,
    "adds_chaos_to_attacks": 3,
    "adds_elemental_to_attacks": 3,
    "adds_damage_to_attacks": 3,
    "adds_physical_to_spells": 3,
    "adds_lightning_to_spells": 3,
    "adds_cold_to_spells": 3,
    "adds_fire_to_spells": 3,
    "adds_chaos_to_spells": 3,
    "adds_elemental_to_spells": 3,
    "adds_damage_to_spells": 3,
}


def round_four():
    """f001 on: the pseudos of ROUND_FOUR complete and sound at their pinned
    versions; a total whose weighted rows are more than the site takes in one
    search is asked in parts, each part sound alone."""
    ids, texts = table.stats()
    versions = table.versions()
    out = []
    for name, number in ROUND_FOUR.items():
        version = versions[name][number - 1]
        out.append(complete(f"f{len(out) + 1:03d}", name, number, version, ids, texts))
        whole = sound("", name, number, version, ids, texts)
        group_ = whole["query"]["stats"][0]
        if group_["type"] != "weight2" or len(group_["filters"]) <= WEIGHTED_AT_MOST:
            whole["search"] = f"f{len(out) + 1:03d}"
            out.append(whole)
            continue
        parts = [group_["filters"][i:i + WEIGHTED_AT_MOST]
                 for i in range(0, len(group_["filters"]), WEIGHTED_AT_MOST)]
        for index, part in enumerate(parts, start=1):
            out.append({
                "search": f"f{len(out) + 1:03d}",
                "decides": whole["decides"].replace(
                    ", sound:", f", sound, part {index} of {len(parts)} of its ids:"),
                "control": whole["control"],
                "query": batch_query([dict(group_, filters=part), group("not", [version["pseudo"]])]),
            })
    return out


# The plan's step 9c2, the site's other totals: each at the version round
# five asks of, written out. A pseudo with a row whose eldritch forms the site
# lists is asked at the version that holds them, the rule being ruled.
ROUND_FIVE = {
    "count_res": 1,
    "count_ele_res": 1,
    "total_all_ele_res": 1,
    "total_all_attributes": 1,
    "total_mana": 1,
    "total_energy_shield": 1,
    "total_increased_energy_shield": 1,
    "increased_movement_speed": 2,
    "global_crit_chance": 1,
    "global_crit_multi": 1,
    "increased_ele_damage": 1,
    "increased_lightning_damage": 2,
    "increased_cold_damage": 2,
    "increased_fire_damage": 2,
    "increased_spell_damage": 2,
    "increased_lightning_spell_damage": 1,
    "increased_cold_spell_damage": 1,
    "increased_fire_spell_damage": 1,
    "increased_lightning_attack_damage": 1,
    "increased_cold_attack_damage": 1,
    "increased_fire_attack_damage": 1,
    "increased_ele_attack_damage": 1,
    "increased_rarity": 1,
    "increased_burning_damage": 1,
    "life_regen": 1,
    "life_regen_pct": 1,
    "phys_attack_life_leech": 1,
    "phys_attack_mana_leech": 1,
    "increased_mana_regen": 2,
}
# The smallest a row can give where it is under a half: a leech line is
# displayed to a hundredth of a percent.
SMALLEST = {"phys_attack_life_leech": 0.01, "phys_attack_mana_leech": 0.01}


def round_five():
    """g001 on: the pseudos of ROUND_FIVE at their pinned versions — complete
    and sound where a version has rows, and where it has none the one search
    of what carries the pseudo, whose items name its first rows."""
    ids, texts = table.stats()
    versions = table.versions()
    if set(ROUND_FIVE) != set(table.OTHER):
        raise SystemExit("round five pins what tools/trade_rows.py does not name, or leaves one out")
    out = []
    for name, number in ROUND_FIVE.items():
        version = versions[name][number - 1]
        out.append(complete(f"g{len(out) + 1:03d}", name, number, version, ids, texts))
        if not version["rows"]:
            continue
        check = sound(f"g{len(out) + 1:03d}", name, number, version, ids, texts,
                      at_least=SMALLEST.get(name, 0.5))
        if len(check["query"]["stats"][0]["filters"]) > WEIGHTED_AT_MOST:
            raise SystemExit(f"{name}: more weighted ids than the site takes; ask it in parts")
        out.append(check)
    return out


# Round six: the pseudos whose rows round five's captures moved, each at the
# version asked, written out. Rows read off ten fetched items are a fit; these
# searches are its test. `increased_rarity` gained a row under an id it already
# asked, so its searches would be g033 and g034 again and are not written.
ROUND_SIX = {
    "total_all_attributes": 2,
    "total_mana": 2,
    "total_energy_shield": 2,
    "total_increased_energy_shield": 3,
    "global_crit_chance": 2,
    "global_crit_multi": 2,
    "increased_lightning_damage": 3,
    "increased_cold_damage": 3,
    "increased_fire_damage": 3,
    "increased_lightning_spell_damage": 3,
    "increased_cold_spell_damage": 3,
    "increased_fire_spell_damage": 3,
    "increased_lightning_attack_damage": 3,
    "increased_cold_attack_damage": 3,
    "increased_fire_attack_damage": 3,
    "increased_ele_attack_damage": 2,
    "increased_burning_damage": 3,
    "life_regen": 2,
    "life_regen_pct": 2,
}
SMALLEST.update({"life_regen": 0.1, "life_regen_pct": 0.01})
# The most ids a weighted group was taken with beside a `not`: d01, at a
# complexity of 148.
WEIGHTED_TAKEN = 23
# A row's `reduced` spelling is under the row's id, a number below nothing
# (g007, g034): asked of three totals the build ships and three of round
# five's, each at the version named.
BELOW_NOTHING = {
    "total_attack_speed": 3,
    "total_cast_speed": 2,
    "total_increased_phys": 4,
    "increased_movement_speed": 2,
    "increased_mana_regen": 2,
    "global_crit_chance": 2,
}


def round_six():
    """h001 on: round five's fits tested, complete and sound; the three
    readings that are no sum of lines asked where round five left them on one
    item or none; the mutant round five's new bound owes; and a row's
    `reduced` spelling, asked below nothing."""
    ids, texts = table.stats()
    versions = table.versions()
    out = []

    def name_next():
        return f"h{len(out) + 1:03d}"

    for name, number in ROUND_SIX.items():
        version = versions[name][number - 1]
        out.append(complete(name_next(), name, number, version, ids, texts))
        check = sound(name_next(), name, number, version, ids, texts,
                      at_least=SMALLEST.get(name, 0.5))
        if len(check["query"]["stats"][0]["filters"]) > WEIGHTED_TAKEN:
            raise SystemExit(f"{name}: more weighted ids than the site has taken; ask it in parts")
        out.append(check)

    all_ele = "pseudo.pseudo_total_all_elemental_resistances"
    line = ids_of("+#% to all Elemental Resistances") + [
        i for prefix in table.PRESENCE for i in ids.get(prefix + "#% to all Elemental Resistances", [])]
    out.append({
        "search": name_next(),
        "decides": (f"total_all_ele_res, the least of the three elemental totals: items showing "
                    f"`{texts[all_ele]}` while carrying no `+#% to all Elemental Resistances` — g003 "
                    "fetched one. Each shows its least elemental total, or the reading is wrong"),
        "control": "g003: the pseudo alone finds items; c1 and c2: a `not` group finds what it should",
        "query": batch_query([group("and", [all_ele]), group("not", line)]),
    })
    count = "pseudo.pseudo_count_resistances"
    out.append({
        "search": name_next(),
        "decides": (f"count_res, whether `{texts[count]}` counts chaos: items carrying "
                    "`+#% to Chaos Resistance`, the count required and the elemental count and the "
                    "chaos total shown beside it. No item round five fetched carries the line"),
        "control": ("the chaos total is shown in the `if` group: an item showing it and a count "
                    "equal to its elemental count is the site leaving chaos out"),
        "query": batch_query([
            group("and", [count]),
            group("count", ids_of("+#% to Chaos Resistance"), {"min": 1}),
            group("if", ["pseudo.pseudo_count_elemental_resistances",
                         "pseudo.pseudo_total_chaos_resistance"]),
        ]),
    })

    life = versions["phys_attack_life_leech"][0]
    mana = versions["phys_attack_mana_leech"][0]
    both = weighted(life, ids, at_least=SMALLEST["phys_attack_life_leech"])
    both["filters"] += [{"id": i, "value": {"weight": 1.0}, "disabled": False}
                        for i in table.ids_of(mana, ids)]
    out.append({
        "search": name_next(),
        "decides": ("g040's mutant: the mana leech's ids weighted with the life leech's rows, at "
                    "a hundredth or more. It must find items carrying "
                    f"`{texts[mana['pseudo']]}` and no life leech, each showing no "
                    f"`{texts[life['pseudo']]}`"),
        "control": ("the mutant is the control: nothing found means a sum of a hundredth or more "
                    "beside a `not` decides nothing, and g040 and g042 with it"),
        "query": batch_query([both, group("not", [life["pseudo"]])]),
    })

    for name, number in BELOW_NOTHING.items():
        version = versions[name][number - 1]
        text = texts[version["pseudo"]]
        below = dict(weighted(version, ids), value={"max": -0.5})
        shows = name_next()
        out.append({
            "search": shows,
            "decides": (f"{name} v{number}, a `reduced` line: items whose rows sum to a half below "
                        f"nothing or less, by the site's own sum, `{text}` required. Each shows a "
                        "total below nothing, or the site does not count the line so"),
            "control": "g007: Carnage Heart shows -25 over `25% reduced maximum Energy Shield`",
            "query": batch_query([group("and", [version["pseudo"]]), below]),
        })
        out.append({
            "search": name_next(),
            "decides": (f"{name} v{number}, sound below nothing: whether any listed item's rows sum "
                        f"to a half below nothing or less while it shows no `{text}`. None found: "
                        "a `reduced` line is counted wherever it appears"),
            "control": f"{shows}, the same sum with the pseudo required, must find items",
            "query": batch_query([below, group("not", [version["pseudo"]])]),
        })
    return out


# Round i (before step 9d): the percentile's shapes no capture reaches, and
# the ranged family's one weapon question. PERCENTILE_CASES was read against
# the sittings' captures, 543 distinct armour items scored by
# search/pseudo-stats/scripts/percentile.py: helmets, gloves, shields, uniques,
# quality above 20 and a defence the base lacks are answered there and not
# asked again (search/pseudo-stats/data/percentile-shapes.csv, which also names
# the cases C9 on). What each search here decides is read by percentile.py.
# The plain ranged pseudos at the version asked, written out.
ROUND_I_PLAIN = {
    "adds_physical": 2,
    "adds_lightning": 2,
    "adds_cold": 2,
    "adds_fire": 2,
    "adds_chaos": 2,
}
# The texts a local defence line shares with a global stat: the site lists
# each under the global stat without `(Local)` (C14 of percentile-shapes.py).
DEFENCE_TWINS = ["+# to Armour", "+# to Evasion Rating", "+# to maximum Energy Shield",
                 "#% increased Armour", "#% increased Evasion Rating"]


def round_i():
    """i01–i08: C1, C7, C10 and its mutant, C14, the ranged family on weapons
    (sound, its mutant and its control), then C2."""
    ids, texts = table.stats()
    versions = table.versions()
    quality = {"misc_filters": {"disabled": False, "filters": {"quality": {"min": 1}}}}
    armour = category("armour")
    plain = [versions[name][number - 1] for name, number in ROUND_I_PLAIN.items()]
    pseudos = [v["pseudo"] for v in plain]
    global_ids = [i for v in plain for i in table.ids_of(v, ids)]
    local_ids = [i for name in ROUND_I_PLAIN for i in ids[f"Adds # to # {name[5:].capitalize()} Damage"]
                 if table.means("local", i)]
    pinned = ", ".join(f"{name} v{number}" for name, number in ROUND_I_PLAIN.items())
    rows = [
        ("i01",
         "C1, the percentile over three defences: Sacrificial Garb, a base with armour, evasion and energy "
         "shield (the copy's three-defence items are this base). The site's value against the average of the "
         "three recovered rolls, and against min, max and the first type, by percentile.py; none captured yet",
         PERCENTILE_CONTROL,
         batch_query([], percentile_filter(), "Sacrificial Garb")),
        ("i02",
         "C7, `Quality does not increase Defences`, on armour with quality: whether the site recovers the roll "
         "with the quality term dropped, for its percentile and for its 20%-quality figures (`extended.ar|ev|es`); "
         "percentile.py reads the enchant as the quality term's zero. None captured yet; the copy holds two",
         PERCENTILE_CONTROL,
         batch_query([group("and", ["enchant.stat_2677401098"])], {**armour, **percentile_filter(), **quality})),
        ("i03",
         "C10, a roll over the base's maximum: whether the site ever shows a percentile over 100 (the contract "
         "detail under C101 reads such an item over 100). Any found: the site does not clamp, and percentile.py's "
         "`beyond_range` names the roll. None found: the site clamps, or lists no such item — i04 says the filter "
         "finds what it should",
         "i04, this search at 100, must find items: the captures hold one at 100 (h015)",
         batch_query([], {**armour, **percentile_filter(101)})),
        ("i04",
         "i03's mutant: armour at a percentile of 100 or more. It must find items; each fetched is scored, and "
         "an item whose display no roll in range gives is a roll over the maximum shown clamped",
         "the mutant is the control: nothing found means the percentile filter decides nothing",
         batch_query([], {**armour, **percentile_filter(100)})),
        ("i05",
         "C14, a defence line on armour under its global twin — a line a private item shows exactly as the local "
         f"one: {', '.join('`' + t + '`' for t in DEFENCE_TWINS)}, each under every id displaying the text without "
         "`(Local)`. Whether the site's value leaves the line out (percentile.py by its ids reproduces it) or reads "
         "it (only the text reading does). The copy holds one such trace; none captured",
         PERCENTILE_CONTROL,
         batch_query([group("count", ids_of(*DEFENCE_TWINS), {"min": 1})], {**armour, **percentile_filter()})),
        ("i06",
         f"the ranged family on weapons, sound, at {pinned}: whether any weapon carries the unsuffixed "
         "`Adds # to # <type> Damage` under the stat without `(Local)` while showing none of the five plain "
         "pseudos. None found: the site counts that stat toward the plain pseudo on a weapon as anywhere else. "
         "Any found: a weapon whose global line the site does not count. Asked because 36 of the copy's weapon "
         "lines on uniques are ones the export cannot say local or global (data/ranged-uniques.csv)",
         "i07, the mutant, must find items; i08 says whether any weapon carries the stat at all",
         batch_query([group("count", global_ids, {"min": 1}), group("not", pseudos)], category("weapon"))),
        ("i07",
         "i06's mutant: the `(Local)` twins in place of the global ids. It must find weapons, each showing no "
         "plain pseudo, as round three found (e016, e021, e027, e033)",
         "the mutant is the control: nothing found means a `not` of the plain pseudos beside a count decides nothing",
         batch_query([group("count", local_ids, {"min": 1}), group("not", pseudos)], category("weapon"))),
        ("i08",
         "i06's control: weapons carrying the unsuffixed line under the stat without `(Local)`, the five plain "
         "pseudos shown. Any found: the site lists a weapon's line as global, and the build, which reads a weapon's "
         "line as its own (the plan, 9d), would not count what the site counts; each item names its unique. None "
         "found: no listed weapon carries the global stat, and the class decides as the site does",
         "i07: a count beside the weapon category finds weapons",
         batch_query([group("count", global_ids, {"min": 1}), group("if", pseudos)], category("weapon"))),
        ("i09",
         "C2, ward read as the other defences are, on any base with ward (two captured, both reproduced; the copy "
         "holds thirteen): more rolls against the ward range of base-defences.csv",
         PERCENTILE_CONTROL,
         batch_query([], {**armour, "armour_filters": {"disabled": False, "filters": {
             "ward": {"min": 1}, "base_defence_percentile": {"min": 1}}}})),
    ]
    return [{"search": s, "decides": d, "control": c, "query": q} for s, d, c, q in rows]


def report():
    """b1, b2: the two searches a report of c3's twin id rests on."""
    twin = "explicit.stat_2543977012"
    return [
        {
            "search": "b1",
            "decides": (
                "the report: `+# to Strength and Intelligence` under the id That Which Was "
                "Taken carries, with `+# total maximum Life` required. c3 says none will be found"
            ),
            "control": "b2, the line alone, finds the items b1 should have",
            "query": batch_query([group("and", [twin, "pseudo.pseudo_total_life"])]),
        },
        {
            "search": "b2",
            "decides": "the report's control: the same line with no total asked, which finds the jewels",
            "control": "c3 fetched five of them",
            "query": batch_query([group("and", [twin])]),
        },
    ]


def decode(link_text):
    token = link_text.rsplit("/", 1)[1]
    raw = base64.urlsafe_b64decode(token + "=" * (-len(token) % 4))
    return json.loads(zlib.decompress(raw, 16 + zlib.MAX_WBITS))


def verify():
    bad = 0
    with OUT.open(newline="") as fh:
        rows = list(csv.DictReader(fh))
    for row in rows:
        if decode(row["link"]) != json.loads(row["query"]):
            bad += 1
            print(f"DIFFER  {row['search']}")
        if not row["control"].strip():
            bad += 1
            print(f"NO CONTROL  {row['search']}")
    print(f"{len(rows)} links decoded, {bad} differ from their query or lack a control")
    return 1 if bad else 0


def self_test():
    requests = sorted(CAPTURED.glob("*-request.json"))
    if not requests:
        print(f"skipped: no captures under {CAPTURED} (raw/ is local only)")
        return 0
    failed = 0
    checked = 0
    for request in requests:
        answer = request.with_name(request.name.replace("-request", "-search"))
        if not answer.exists() or answer.stat().st_size == 0:
            continue
        want = json.loads(answer.read_text())["id"]
        got = compose(json.loads(request.read_text())["query"])
        checked += 1
        if got != want:
            failed += 1
            print(f"DIFFER  {request.name}")
    print(f"{checked} captured ids composed, {failed} differ")
    return 1 if failed else 0


def main():
    args = sys.argv[1:]
    if args == ["--self-test"]:
        return self_test()
    if args == ["--verify"]:
        return verify()
    if args == []:
        method = "site"
    elif args == ["--method", "if"]:
        method = "if"
    elif args == ["--method", "and"]:
        method = "and"
    else:
        print(__doc__)
        return 2
    searches = PILOT + (checks() + round_two() + report() + round_three() + round_four()
                        + round_five() + OPEN_QUESTION_SEARCHES + round_six() + round_i()
                        if method == "site" else batch(method))
    seen = set()
    with OUT.open("w", newline="") as out:
        sheet = csv.writer(out, lineterminator="\n")
        sheet.writerow(["search", "decides", "control", "league", "link", "query"])
        for row in searches:
            if row["search"] in seen:
                raise SystemExit(f"two searches named {row['search']}")
            seen.add(row["search"])
            body = json.dumps(row["query"], separators=(",", ":"), ensure_ascii=False)
            sheet.writerow(
                [
                    row["search"],
                    row["decides"],
                    row["control"],
                    LEAGUE,
                    link(row["query"]),
                    body,
                ]
            )
            if method == "site":
                print(f"{row['search']}  {link(row['query'])}")
    kinds = {"pilot": 0, "check": 0, "line": 0, "pair": 0, "percentile line": 0,
             "percentile case": 0, "open question": 0}
    for row in searches:
        s = row["search"]
        kind = ("pilot" if s.startswith("p")
                else "check" if s.startswith(("c", "d", "b", "e", "f", "g", "h"))
                else "percentile case" if s.startswith("C")
                else "open question" if s.startswith("R")
                else "percentile line" if s.startswith("P") else "pair" if "." in s else "line")
        kinds[kind] += 1
    print(f"--method {method}: " + ", ".join(f"{n} {k}" for k, n in kinds.items() if n))
    print(f"{len(searches)} searches written to {OUT.relative_to(TRACK.parent.parent)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
