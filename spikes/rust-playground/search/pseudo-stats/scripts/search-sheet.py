#!/usr/bin/env python3
"""Write the sheet of trade searches the owner runs: ../data/search-sheet.csv.

A search is a query object; the site's search id is that object, serialized
compactly, gzipped and base64url-encoded without padding (trade-query F9), so
a link is composed here and opened by a human in a browser. Nothing in this
script touches the network (C79; SURFACES.md, the trade site's rows: access
method `browser`).

    search-sheet.py                # the pilot and the batch, if form (the default)
    search-sheet.py --method if    # the same
    search-sheet.py --method and   # the pilot and the batch, one search per pair
    search-sheet.py --self-test    # compose each captured request and compare ids
    search-sheet.py --verify       # decode every link in the sheet, compare its query

The batch is generated from ../data/candidates.csv (scripts/candidates.py),
in two forms, since the pilot's answer is not in: `if` — one search per
candidate line, every pseudo it might feed in one `if` group (p2 asks whether
an `if` group shows a value at all); `and` — one search per pair, the pseudo
required as in p1. The committed sheet is the `if` form; the other is one
command away. Rows are ordered by what a search is worth: the pilot, then
lines whose best pair is `sources-differ`, `text-only`, `one-source`,
`sources-agree`, then the percentile's own cases (PERCENTILE_CASES), then
the first pass's open question 5 (R1).

The self-test reads `../../trade-query/raw/searches/` (local only, never
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

TRACK = Path(__file__).resolve().parents[1]
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
                        f"({pair['status']}): recover each roll as scripts/percentile.py does, with the "
                        "line read and without; the reading that gives the site's value is the answer. "
                        "No result: the line is not seen on armour with a percentile above 0"),
            "control": PERCENTILE_CONTROL,
            "query": batch_query([lf], {**category("armour"), **percentile_filter()}),
            "rank": (STATUS_ORDER.index(pair["status"]), -e["items"], line_id),
        })
    return out


# What no capture reaches (README, the percentile): each case is fetched and
# scored by scripts/percentile.py's reading.
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
    if args in ([], ["--method", "if"]):
        method = "if"
    elif args == ["--method", "and"]:
        method = "and"
    else:
        print(__doc__)
        return 2
    searches = PILOT + batch(method)
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
    kinds = {"pilot": 0, "line": 0, "pair": 0, "percentile line": 0, "percentile case": 0,
             "open question": 0}
    for row in searches:
        s = row["search"]
        kind = ("pilot" if s.startswith("p") else "percentile case" if s.startswith("C")
                else "open question" if s.startswith("R")
                else "percentile line" if s.startswith("P") else "pair" if "." in s else "line")
        kinds[kind] += 1
    print(f"--method {method}: " + ", ".join(f"{n} {k}" for k, n in kinds.items() if n))
    print(f"{len(searches)} searches written to {OUT.relative_to(TRACK.parent.parent)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
