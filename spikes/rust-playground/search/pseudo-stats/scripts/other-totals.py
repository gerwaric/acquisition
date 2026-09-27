#!/usr/bin/env python3
"""Where each of the site's pseudo stats stands before the plan's step 9c2:
search/pseudo-stats/data/other-totals.csv.

The step's scope is "the sum-like pseudos 9c left `unresolved`". This script
says which those are, by three reads and no list of names:

  1. under test already — the pseudo is one tools/trade_rows.py holds rows
     for, whatever the first pass called it;
  2. the first pass's class — search/pseudo-stats/data/pseudo-classes.csv,
     unchanged since 2026-09-18;
  3. for an `unresolved` row, the read its note names as the one that closes
     it: a search whose pseudo is read against the item's lines is a sum of
     lines to ask the site about; a capture of a logbook or a tablet, or the
     export's base ranges, is another question.

Every one of the site's pseudo stats gets a row, so the counts sum to the
whole. An owner's cut is a row of LEFT_OUT, in his words, never an edit of
the rule.

    search/pseudo-stats/scripts/other-totals.py    # write the file, print the counts

Reads search/trade-query/data/stats-2026-09-12.json (through tools/trade_rows.py),
search/pseudo-stats/data/pseudo-classes.csv, search/item-facts/data/mod-templates.csv.
"""

import csv
import sys
from collections import Counter, defaultdict
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
ROOT = TRACK.parent.parent
sys.path.insert(0, str(ROOT / "tools"))
import trade_rows as table  # noqa: E402

CLASSES = TRACK / "data" / "pseudo-classes.csv"
TEMPLATES = TRACK.parent / "item-facts" / "data" / "mod-templates.csv"
OUT = TRACK / "data" / "other-totals.csv"

# What the first pass wrote, word for word, on a row a search of lines closes.
SUM_OF_LINES = ("run one trade search on this id and read the returned item's "
                "pseudoMods against its lines")

# The owner's cuts: {pseudo id: his words, dated}.
LEFT_OUT = {}


def corpus():
    """{template: items}, over every array of the owner's census."""
    items = defaultdict(int)
    with TEMPLATES.open(newline="\n") as f:
        f.readline()  # the census names its script and inputs on its first line
        for r in csv.DictReader(f):
            items[r["template"].split("\n")[0].replace("\r", "")] += int(r["items"])
    return items


def main():
    if sys.argv[1:]:
        print(__doc__)
        return 2
    ids, texts = table.stats()
    under = {h[-1]["pseudo"]: name for name, h in table.versions().items()}
    held = corpus()
    with CLASSES.open(newline="\n") as f:
        classes = list(csv.DictReader(f))
    if {r["id"] for r in classes} != set(texts):
        raise SystemExit("the first pass and the site's stats name different pseudos")
    out = []
    stands = Counter()
    for r in classes:
        own = table.norm(r["text"])
        first = ids.get(own, [])
        forms = [i for prefix in table.PRESENCE for i in ids.get(prefix + own, [])]
        local = [i for i in first if i in table.LOCAL]
        if r["id"] in under:
            where, why = "under test", f"tools/trade_rows.py, {under[r['id']]}"
        elif r["class"] != "unresolved":
            where, why = "classed", f"the first pass: {r['class']}"
        elif not r["note"].startswith(SUM_OF_LINES):
            where, why = "another read", r["note"].split(";")[0]
        elif r["id"] in LEFT_OUT:
            where, why = "left out", LEFT_OUT[r["id"]]
        else:
            where, why = "in scope", "unresolved; the first pass names a search read against the item's lines"
        stands[where] += 1
        scoped = where == "in scope"
        out.append([r["id"], r["text"], r["class"], where, why,
                    own if scoped and first else "",
                    len(first) if scoped else "",
                    len(local) if scoped else "",
                    len(forms) if scoped else "",
                    held.get(own, 0) if scoped and first else ""])
    with OUT.open("w", newline="") as f:
        sheet = csv.writer(f, lineterminator="\n")
        sheet.writerow(["id", "text", "first_pass_class", "stands", "why", "own_text_template",
                        "own_text_ids", "local_twin_ids", "eldritch_form_ids", "corpus_items"])
        sheet.writerows(out)
    print(f"{len(out)} pseudo stats written to {OUT.relative_to(ROOT)}: "
          + ", ".join(f"{n} {k}" for k, n in sorted(stands.items())))
    scoped = [r for r in out if r[3] == "in scope"]
    with_text = [r for r in scoped if r[6]]
    print(f"in scope: {len(scoped)} — {len(with_text)} whose own text a line displays, "
          f"{len(scoped) - len(with_text)} whose text no line displays")
    twins = [r for r in scoped if r[7]]
    print(f"{len(twins)} of them with a `(Local)` twin among the ids of its own text")
    print(f"a first round: {2 * len(with_text) + len(scoped) - len(with_text)} searches — a complete "
          "and a sound check for each pseudo with rows, one search of what carries it for each without")
    for r in scoped:
        rows = (f"{r[6]} ids, {r[7]} of them a `(Local)` twin, {r[8]} eldritch, {r[9]} corpus items"
                if r[6] else "no line displays it")
        print(f"    {r[0].removeprefix('pseudo.')} · {r[1]} · {rows}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
