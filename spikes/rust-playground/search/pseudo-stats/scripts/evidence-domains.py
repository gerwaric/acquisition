#!/usr/bin/env python3
"""What the captures already say of a line's domain (BRIEF.md, steps 1 and 2):
writes ../data/evidence-domains.csv and prints every count the README states of it.

    evidence-domains.py

A line's **domain** is the word after `stat.` in its `hash`: the group of the
site's stats the line was filed under. A line the capture holds with no hash
has none, and is counted apart as `(no hash)`.

Step 1. The readings are tools/trade-evidence.py's, made with its own
functions (`asked`, `shown`, `given`, `read_of`, `lines_counted`): an item under
a pseudo its search named, read against the pseudo's latest rows
(tools/trade_rows.py). Its `main` is also run here, its output file kept in
memory and never written, and the two counts are printed side by side. Under
each reading, the lines **named** are those whose template a row of the
pseudo's latest version names; for a pseudo that is no sum of lines (a count,
a least), the rows are those of the totals it reads. A named line is
**counted** where the version gives it a weight under its id (a twin the row
does not mean, a `never` id: named and not counted).

The test: the same reading with the entries of one domain taken off the item,
everything else as it was — the item handed to the same functions. A domain is

    shown counted    where a reading that agrees as the rows stand disagrees
                     with the domain's lines left out;
    shown left out   where a reading that disagrees as the rows stand agrees
                     with them left out;
    not shown        otherwise: `no line captured` (no named line of the
                     domain under any reading) or `same either way` (named
                     lines captured, and no reading changed by leaving them out).

It is asked of every domain, of every pseudo and domain, and of every
(pseudo, template, domain, array, flags), there leaving out those lines alone.
A verdict is what the captured readings show and no more: `not shown` is not
evidence that a domain is left out.

The mapping from a domain to what a private item carries is read off the
captures: every line's hash word beside its `domain` field, its array and its
flags, over every captured line and over the lines a row names.

Step 2. The copy: for every template a row of the shipped table
(crates/acquisition-search/reference/totals-v5.toml) or of the 21 ranged
totals (data/table-changes.csv, `add row` and `twin`) names, the copy's live
`pc` lines by array and flags (the store opened `mode=ro`, as
scripts/ranged-census.py opens it; a mod of several rows read row by row, the
template scripts/ranged-expected.py's). Each group stands beside the domain
the captures file that array and those flags under — over the lines a row
names; where several, each — and that domain's verdict, and beside what the
captures show of that very template in that array under those flags.

Beside step 1, since the second thing the expected answers rest on is J7: the
captured lines of the five unsuffixed `Adds # to # <Type> Damage`, each once
an item, by domain, array, flags, the twin the site filed the line under
(`local`, a `(Local)` id; `other`; none where the line has no hash) and the
item's side as scripts/ranged-expected.py's class reading gives it from the
base (`ranged-line` rows).

One file, the parts marked apart by its first column: `mapping`, `domain`,
`pseudo-domain`, `line`, `ranged-line`, `copy`. A column a part does not use
is empty; `site_twin` is the `line` and `ranged-line` rows', `side` the
`ranged-line` rows'.
"""

import contextlib
import csv
import importlib.util
import io
import json
import sqlite3
import subprocess
import sys
import tomllib

sys.dont_write_bytecode = True  # an import here leaves no file behind

from collections import Counter, defaultdict  # noqa: E402
from pathlib import Path  # noqa: E402

TRACK = Path(__file__).resolve().parents[1]
SEARCH = TRACK.parent
ROOT = SEARCH.parent
OUT = TRACK / "data" / "evidence-domains.csv"
CAPTURES = TRACK / "data" / "captures.json"
CHANGES = TRACK / "data" / "table-changes.csv"
# A read made before round n, at `0a6f7d93`: the table it read is v5, which the
# build replaced at 9d-C. Run again it reads today's captures and the table of
# that commit, by `git show`; its committed file is that commit's.
TOTALS_AT = "0a6f7d93:spikes/rust-playground/crates/acquisition-search/reference/totals-v5.toml"
TRADE_STATS = SEARCH / "trade-query/data/stats-2026-09-12.json"
STORE = SEARCH / "item-facts/raw/m3/store/mock/GERWARIC_7694.db"
INPUTS = ("data/captures.json, tools/trade-evidence.py and tools/trade_rows.py (each pseudo's latest rows), "
          "data/candidates.csv (trade-evidence.py's own run), ../trade-query/data/stats-2026-09-12.json, "
          "crates/acquisition-search/reference/totals-v5.toml, data/table-changes.csv, "
          "../item-facts/raw/m3/store/mock/GERWARIC_7694.db (read-only)")
NO_HASH = "(no hash)"
VERDICTS = ["shown counted", "shown left out", "not shown: same either way", "not shown: no line captured"]
HEADER = ["part", "pseudo", "pseudo_text", "template", "domain", "domain_field", "array", "flags", "site_twin",
          "side", "lines",
          "counted_lines", "items", "readings", "agree", "breaks_left_out", "agrees_only_left_out",
          "same_either_way", "verdict", "note"]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


ev = load("trade_evidence", ROOT / "tools" / "trade-evidence.py")
table = ev.table
expected = load("ranged_expected", TRACK / "scripts" / "ranged-expected.py")


class Kept:
    """trade-evidence.py's output file, kept in memory: its `main` runs and writes nothing."""

    def __init__(self):
        self.buffer = io.StringIO()

    def open(self, *args, **kwargs):
        held = self.buffer

        class Handle(io.StringIO):
            def close(self):
                held.write(self.getvalue())
                super().close()
        return Handle(newline=kwargs.get("newline"))

    def relative_to(self, _):
        return "memory"


def evidence_run():
    """(readings, disagreeing, the file's text) as trade-evidence.py's `main` makes them."""
    kept, said = Kept(), io.StringIO()
    out, argv = ev.OUT, sys.argv
    ev.OUT, sys.argv = kept, argv[:1]
    try:
        with contextlib.redirect_stdout(said):
            code = ev.main()
    finally:
        ev.OUT, sys.argv = out, argv
    last = said.getvalue().strip().split("\n")[-1]
    words = last.split()
    if code != 0 or words[1:2] != ["readings"]:
        sys.exit(f"tools/trade-evidence.py did not print its count: {last!r}")
    return int(words[0]), int(last.split(", ")[1].split()[0]), kept.buffer.getvalue()


def domain_of(entry):
    stat = entry.get("hash") or ""
    return stat.split(".")[1] if stat.startswith("stat.") and stat.count(".") >= 2 else NO_HASH


def twin_of(entry):
    stat = ev.stat_of(entry)
    return "" if not stat else "local" if stat in table.LOCAL else "other"


def flags_of(entry):
    return ";".join(sorted(k for k, v in (entry.get("flags") or {}).items() if v is True))


def without(item, drop):
    """The item with every line `drop(array, entry, line)` holds of taken off: an entry
    all of whose lines go is gone, one of several lines keeps the others."""
    lines = {}
    for array, entries in item["lines"].items():
        if array == "pseudoMods":
            lines[array] = entries
            continue
        kept = []
        for e in entries:
            rows = [r for r in e["description"].split("\n") if not drop(array, e, r)]
            if len(rows) == len(e["description"].split("\n")):
                kept.append(e)
            elif rows:
                kept.append(dict(e, description="\n".join(rows)))
        lines[array] = kept
    return dict(item, lines=lines)


def verdict_of(named, breaks, only_out):
    if breaks and only_out:
        return "shown counted and shown left out"
    if breaks:
        return VERDICTS[0]
    if only_out:
        return VERDICTS[1]
    return VERDICTS[2] if named else VERDICTS[3]


class Tally:
    def __init__(self):
        self.lines = 0
        self.counted = 0
        self.items = set()
        self.readings = 0
        self.agree = 0
        self.breaks = 0
        self.only_out = 0
        self.same = 0

    def test(self, stands, left_out):
        self.readings += 1
        self.agree += stands
        self.breaks += stands and not left_out
        self.only_out += (not stands) and left_out
        self.same += stands == left_out

    def verdict(self):
        return verdict_of(self.lines, self.breaks, self.only_out)

    def cells(self):
        return [self.lines, self.counted, len(self.items), self.readings, self.agree, self.breaks,
                self.only_out, self.same, self.verdict()]


def main():
    if sys.argv[1:]:
        print(__doc__)
        return 2
    captures = json.loads(CAPTURES.read_text())["searches"]
    versions = table.latest()
    derived = table.derived()
    _, texts = table.stats()
    groups = [g["id"] for g in json.loads(TRADE_STATS.read_text())["result"] if g["id"] != "pseudo"]

    # The mapping: every captured line that is no pseudo's, by what it carries.
    row_templates = {t for v in versions.values() for (t, _) in v["rows"]}
    mapping = defaultdict(lambda: [0, set(), 0])  # (domain, field, array, flags) -> lines, items, named lines
    tables = expected.Tables()
    unsuffixed = {f"Adds # to # {kind.capitalize()} Damage" for kind in table.TYPES}
    ranged_lines = defaultdict(lambda: [0, set()])  # (template, domain, array, flags, twin, side)
    seen_items = set()
    for name, capture in captures.items():
        for item in capture["fetched"] or []:
            ident = json.dumps(item, sort_keys=True)
            if ident not in seen_items:
                seen_items.add(ident)
                side = tables.side({"base": item.get("baseType"), "rarity": item.get("rarity")})[0] or "open"
                for array, entries in item["lines"].items():
                    if array == "pseudoMods":
                        continue
                    for e in entries:
                        for row in e["description"].split("\n"):
                            if ev.template(row) in unsuffixed:
                                cell = ranged_lines[(ev.template(row), domain_of(e), array, flags_of(e),
                                                     twin_of(e), side)]
                                cell[0] += 1
                                cell[1].add(ident)
            for array, entries in item["lines"].items():
                if array == "pseudoMods":
                    continue
                for e in entries:
                    cell = mapping[(domain_of(e), e.get("domain") or "", array, flags_of(e))]
                    cell[0] += 1
                    cell[1].add(ident)
                    cell[2] += any(ev.template(r) in row_templates for r in e["description"].split("\n"))

    def agrees(pseudo, version, slots, item, values):
        """trade-evidence.py's question of one item, as its `main` asks it."""
        if pseudo in derived:
            rows_give = ev.read_of(*derived[pseudo], versions, item)
        else:
            rows_give = ev.given(version, slots, item)
        ok = values == ([] if rows_give is None else [rows_give])
        if not ok and version["cut"] and values and rows_give:
            over = values[0][0] - rows_give[0]
            ok = 0 <= over <= version["cut"] * ev.lines_counted(version, item)
        return ok

    readings = disagreeing = skipped = 0
    with_named = 0
    by_domain = defaultdict(Tally)
    by_pair = defaultdict(Tally)       # (pseudo, domain)
    by_line = defaultdict(Tally)       # (pseudo, template, domain, field, array, flags, twin)
    pseudos_read = set()
    for name, capture in captures.items():
        for pseudo, site_says_not in ev.asked(capture["returned_query"]):
            if pseudo not in versions:
                skipped += 1
                continue
            version = versions[pseudo]
            slots = texts[pseudo].count("#")
            sources = [versions[p] for p in derived[pseudo][1]] if pseudo in derived else [version]
            named = {t for v in sources for (t, _) in v["rows"]}
            for item in capture["fetched"]:
                readings += 1
                pseudos_read.add(pseudo)
                values = [v for e in item["lines"].get("pseudoMods", [])
                          if (v := ev.shown(texts[pseudo], e["description"])) is not None]
                stands = agrees(pseudo, version, slots, item, values)
                disagreeing += not stands
                ident = json.dumps(item, sort_keys=True)
                here = defaultdict(list)  # line key -> its lines under this reading
                for array, entries in item["lines"].items():
                    if array == "pseudoMods":
                        continue
                    for e in entries:
                        for row in e["description"].split("\n"):
                            t = ev.template(row)
                            if t not in named:
                                continue
                            counted = bool(ev.numbers(row)) and any(
                                table.weight_of(v, t, ev.stat_of(e)) is not None for v in sources)
                            here[(t, domain_of(e), e.get("domain") or "", array, flags_of(e),
                                  twin_of(e))].append(counted)
                with_named += bool(here)
                for domain in {k[1] for k in here}:
                    left = agrees(pseudo, version, slots,
                                  without(item, lambda a, e, r, d=domain: domain_of(e) == d), values)
                    for tally in (by_domain[domain], by_pair[(pseudo, domain)]):
                        tally.test(stands, left)
                        tally.items.add(ident)
                        for k, marks in here.items():
                            if k[1] == domain:
                                tally.lines += len(marks)
                                tally.counted += sum(marks)
                for k, marks in here.items():
                    left = agrees(pseudo, version, slots, without(
                        item, lambda a, e, r, k=k: (ev.template(r), domain_of(e), a, flags_of(e), twin_of(e))
                        == (k[0], k[1], k[3], k[4], k[5])), values)
                    tally = by_line[(pseudo,) + k]
                    tally.test(stands, left)
                    tally.items.add(ident)
                    tally.lines += len(marks)
                    tally.counted += sum(marks)

    theirs, theirs_disagree, theirs_file = evidence_run()
    committed = (TRACK / "data" / "evidence.csv").read_text()

    # What a domain is on a private item, as the captures show it.
    domains = list(dict.fromkeys(groups + sorted({k[0] for k in mapping})))
    carried = defaultdict(set)        # domain -> {(array, flags)}
    filed = defaultdict(Counter)      # (array, flags) -> {domain: lines}, every captured line
    filed_named = defaultdict(Counter)  # the same over the lines a row names
    for (domain, field, array, flags), (lines, items, named_lines) in mapping.items():
        carried[domain].add((array, flags))
        filed[(array, flags)][domain] += lines
        if named_lines:
            filed_named[(array, flags)][domain] += named_lines

    def settled(domain):
        if domain not in carried:
            return "not settled: no line of the domain captured"
        if domain == NO_HASH:
            return "not settled: the captured line carries no hash"
        shared = sorted({d for key in carried[domain] for d in filed[key] if d != domain})
        if len(carried[domain]) > 1:
            return "not settled: the domain's lines sit under several arrays or flags"
        if shared:
            return ("not settled: its array and flags also hold lines of " + ", ".join(shared))
        return "settled: one array and flags, holding this domain's lines alone"

    rows = []
    for (domain, field, array, flags), (lines, items, named_lines) in sorted(mapping.items()):
        rows.append(["mapping", "", "", "", domain, field, array, flags, "", "", lines, "", len(items), "", "", "", "",
                     "", "", f"every captured line that is no pseudo's; {named_lines} of them hold a template "
                     f"a row names; {settled(domain)}"])
    for domain in domains:
        if not any(k[0] == domain for k in mapping):
            rows.append(["mapping", "", "", "", domain, "", "", "", "", "", 0, "", 0, "", "", "", "", "", "",
                         settled(domain)])
    for domain in domains:
        tally = by_domain.get(domain) or Tally()
        pairs = [by_pair[k] for k in by_pair if k[1] == domain]
        rows.append(["domain", "", "", "", domain, "", "", "", "", ""] + tally.cells() + [
            f"{len(pairs)} pseudos carry a named line of it; shown counted under "
            f"{sum(1 for p in pairs if p.verdict() == VERDICTS[0])}; {settled(domain)}"])
    for (pseudo, domain), tally in sorted(by_pair.items()):
        rows.append(["pseudo-domain", pseudo, texts[pseudo], "", domain, "", "", "", "", ""] + tally.cells()
                    + [""])
    for (pseudo, t, domain, field, array, flags, twin), tally in sorted(by_line.items()):
        rows.append(["line", pseudo, texts[pseudo], t, domain, field, array, flags, twin, ""] + tally.cells() + [
            "left out: the lines of this template, domain, array, flags and twin alone"])
    for (t, domain, array, flags, twin, side), (lines, items) in sorted(ranged_lines.items()):
        rows.append(["ranged-line", "", "", t, domain, "", array, flags, twin, side, lines, "", len(items), "",
                     "", "", "", "", "", "every captured line of the template, an item counted once"])

    # Step 2: the copy.
    shipped = tomllib.loads(subprocess.run(["git", "show", TOTALS_AT], cwd=ROOT, check=True,
                                           capture_output=True, text=True).stdout)["total"]
    names = defaultdict(set)  # template -> the totals whose rows name it
    for total in shipped:
        for r in total["rows"]:
            names[r["template"]].add(total["name"])
    with CHANGES.open(newline="\n") as fh:
        ranged = [r for r in csv.DictReader(fh) if r["total"].startswith("adds_")
                  and r["change"] in ("add row", "twin")]
    for r in ranged:
        names[r["template"]].add(r["total"])
    ranged_totals = {r["total"] for r in ranged}
    shown_line = defaultdict(set)  # (template, array, flags) -> the verdicts of its `line` rows
    for (pseudo, t, domain, field, array, flags, twin), tally in by_line.items():
        shown_line[(t, array, flags)].add(tally.verdict())

    copy = defaultdict(lambda: [0, set()])
    copy_absent = not STORE.exists()
    passed_over = Counter()
    if not copy_absent:
        db = sqlite3.connect(f"file:{STORE}?mode=ro", uri=True)
        for item_id, body in db.execute(
                "select id, json from items where removed_at is null and realm = 'pc' order by id"):
            for array, entries in json.loads(body).items():
                if not array.endswith("Mods"):
                    continue
                if not isinstance(entries, list):
                    passed_over[array] += 1
                    continue
                for entry in entries:
                    text = entry.get("description") if isinstance(entry, dict) else entry
                    if not isinstance(text, str):
                        passed_over[array] += 1
                        continue
                    flags = expected.flags_shown(entry.get("flags")) if isinstance(entry, dict) else ""
                    for row in text.split("\n"):
                        t = expected.template(row)
                        if t in names:
                            cell = copy[(t, array, flags)]
                            cell[0] += 1
                            cell[1].add(item_id)

    def domains_for(array, flags):
        """The domains the captures file this array and these flags under, most lines first."""
        found = filed_named.get((array, flags)) or filed.get((array, flags))
        return [d for d, _ in sorted(found.items(), key=lambda kv: (-kv[1], kv[0]))] if found else []

    def domain_verdict(domain):
        return (by_domain.get(domain) or Tally()).verdict()

    by_verdict = defaultdict(lambda: [0, set()])
    by_where = defaultdict(lambda: [0, set()])
    by_template_verdict = defaultdict(lambda: [0, set()])
    for (t, array, flags), (lines, items) in sorted(copy.items()):
        found = domains_for(array, flags)
        verdicts = [domain_verdict(d) for d in found] or [VERDICTS[3]]
        first = verdicts[0]
        asked_here = shown_line.get((t, array, flags))
        of_template = (VERDICTS[0] if asked_here and VERDICTS[0] in asked_here
                       else VERDICTS[1] if asked_here and VERDICTS[1] in asked_here
                       else VERDICTS[2] if asked_here else VERDICTS[3])
        rows.append(["copy", "", "", t, "|".join(found) or "no captured line has this array and flags", "",
                     array, flags, "", "", lines, "", len(items), "", "", "", "", "", first,
                     f"this template in this array under these flags: {of_template}; named by "
                     f"{len(names[t])} totals, {len(names[t] & ranged_totals)} of them ranged"
                     + (f"; the array and flags also hold lines of {', '.join(found[1:])}: "
                        + ", ".join(verdicts[1:]) if len(found) > 1 else "")])
        for tally, key in ((by_verdict, first), (by_where, (array, flags, "|".join(found), first)),
                           (by_template_verdict, of_template)):
            tally[key][0] += lines
            tally[key][1] |= items

    with OUT.open("w", newline="") as fh:
        fh.write(f"# generated by scripts/evidence-domains.py from {INPUTS}\n")
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(HEADER)
        w.writerows(rows)

    print(f"readings made: {readings} of an item under a pseudo its search named, {disagreeing} disagreeing "
          f"as the rows stand; {skipped} askings of a pseudo with no rows under test skipped; "
          f"{len(pseudos_read)} pseudos read")
    print(f"tools/trade-evidence.py, its main run here and its file kept in memory: {theirs} readings, "
          f"{theirs_disagree} disagreeing — the count is "
          f"{'its' if (theirs, theirs_disagree) == (readings, disagreeing) else 'NOT its'}; its file is "
          f"{'data/evidence.csv byte for byte' if theirs_file == committed else 'NOT data/evidence.csv'}")
    print(f"readings holding a line a row names: {with_named}; holding none: {readings - with_named}")
    print("the mapping, every captured line that is no pseudo's — hash word / `domain` field / array / flags: "
          "lines, items, lines holding a template a row names:")
    for (domain, field, array, flags), (lines, items, named_lines) in sorted(mapping.items()):
        print(f"  {domain} / {field or '(none)'} / {array} / {flags or '(none)'}: {lines}, {len(items)}, "
              f"{named_lines}")
    print("each domain, how it maps to array and flags:")
    for domain in domains:
        print(f"  {domain}: {settled(domain)}")
    print("each domain — named lines (counted), items, readings, agreeing, broken left out, agreeing only "
          "left out, same either way: verdict:")
    for domain in domains:
        tally = by_domain.get(domain) or Tally()
        pairs = [by_pair[k] for k in by_pair if k[1] == domain]
        print(f"  {domain}: {tally.lines} ({tally.counted}), {len(tally.items)}, {tally.readings}, {tally.agree}, "
              f"{tally.breaks}, {tally.only_out}, {tally.same}: {tally.verdict()}; pseudos carrying it "
              f"{len(pairs)}, shown counted under {sum(1 for p in pairs if p.verdict() == VERDICTS[0])}")
    print("domains by verdict:")
    for v in VERDICTS:
        held = [d for d in domains if domain_verdict(d) == v]
        print(f"  {v}: {len(held)} — {', '.join(held) or 'none'}")
    print("pseudo and domain, by verdict (pairs holding a named line; a pair with none is not written):")
    for (domain, v), n in sorted(Counter((k[1], t.verdict()) for k, t in by_pair.items()).items()):
        print(f"  {domain}: {v}: {n} pseudos")
    family = Counter()
    for (pseudo, domain), tally in by_pair.items():
        if "_adds_" in pseudo:
            family[(domain, tally.verdict())] += 1
    print("the same over the ranged family's pseudos (`pseudo_adds_…`):")
    for (domain, v), n in sorted(family.items()):
        print(f"  {domain}: {v}: {n} pseudos")
    ranged_pseudos = sorted(p for p in versions if "_adds_" in p)
    print(f"the ranged family's {len(ranged_pseudos)} pseudos by domain — shown counted / same either way / "
          "no line captured:")
    for domain in domains:
        got = Counter(by_pair[(p, domain)].verdict() if (p, domain) in by_pair else VERDICTS[3]
                      for p in ranged_pseudos)
        print(f"  {domain}: {got[VERDICTS[0]]} / {got[VERDICTS[2]]} / {got[VERDICTS[3]]}")
    print("every pseudo and domain whose named lines read the same either way — readings, named lines, "
          "counted:")
    for (pseudo, domain), tally in sorted(by_pair.items()):
        if tally.verdict() == VERDICTS[2]:
            print(f"  {pseudo.removeprefix('pseudo.')} / {domain}: {tally.readings}, {tally.lines}, "
                  f"{tally.counted}")
    print("ranged lines, the test a template — unsuffixed or another row / domain / array / flags / the "
          "site's twin: pairs of pseudo and template by verdict:")
    fam_lines = Counter()
    for (pseudo, t, domain, field, array, flags, twin), tally in by_line.items():
        if "_adds_" in pseudo:
            kind = "unsuffixed" if t in unsuffixed else "other row"
            fam_lines[(kind, domain, array, flags or "(none)", twin or "(no hash)", tally.verdict())] += 1
    for k, n in sorted(fam_lines.items()):
        print(f"  {' / '.join(k[:5])}: {k[5]}: {n}")
    print("the unsuffixed lines captured, an item once — side / domain / array / flags / the site's twin: "
          "lines, items:")
    sides = defaultdict(lambda: [0, set()])
    for (t, domain, array, flags, twin, side), (lines, items) in ranged_lines.items():
        cell = sides[(side, domain, array, flags or "(none)", twin or "(no hash)")]
        cell[0] += lines
        cell[1] |= items
    for k, (lines, items) in sorted(sides.items()):
        print(f"  {' / '.join(k)}: {lines}, {len(items)}")
    against = sum(v[0] for k, v in sides.items() if (k[0] == "weapon") != (k[4] == "local") and k[0] != "open")
    print(f"unsuffixed lines whose twin is not the side's (a weapon's filed `other`, or `local` off a weapon): "
          f"{against}; on an item whose side is open: {sum(v[0] for k, v in sides.items() if k[0] == 'open')}")

    if copy_absent:
        print(f"ABSENT: {STORE.relative_to(ROOT)} — the copy's part is not written; every count of it is "
              "an open question")
    else:
        total_lines = sum(v[0] for v in copy.values())
        total_items = len(set().union(*(v[1] for v in copy.values()))) if copy else 0
        print(f"the copy: {len(names)} templates a row names ({len(shipped)} shipped totals, "
              f"{len(ranged_totals)} ranged); {total_lines} live pc lines of them on {total_items} items, in "
              f"{len(copy)} groups of template, array and flags; passed over as no text: "
              f"{dict(sorted(passed_over.items())) or 'none'}")
        print("the copy by array and flags — domain as the captures file it, verdict: lines, items:")
        for (array, flags, found, v), (lines, items) in sorted(by_where.items()):
            print(f"  {array} / {flags or '(none)'} — {found or 'no captured line has this array and flags'}, "
                  f"{v}: {lines}, {len(items)}")
        print("the copy's groups whose array and flags no captured line has — array / flags / template: "
              "lines, totals naming the template:")
        for (t, array, flags), (lines, items) in sorted(copy.items(), key=lambda kv: (kv[0][1], kv[0][2],
                                                                                    -kv[1][0], kv[0][0])):
            if not domains_for(array, flags):
                print(f"  {array} / {flags or '(none)'} / {t}: {lines}, {len(names[t])}")
        print("the copy by its domain's verdict: lines, items:")
        for v in VERDICTS:
            lines, items = by_verdict.get(v, [0, set()])
            print(f"  {v}: {lines}, {len(items)}")
        print("the copy by what the captures show of the very template in that array under those flags: "
              "lines, items:")
        for v in VERDICTS:
            lines, items = by_template_verdict.get(v, [0, set()])
            print(f"  {v}: {lines}, {len(items)}")
    print(f"wrote {OUT.relative_to(ROOT)}: {len(rows)} rows — "
          + ", ".join(f"{n} {p}" for p, n in sorted(Counter(r[0] for r in rows).items())))
    return 0


if __name__ == "__main__":
    sys.exit(main())
