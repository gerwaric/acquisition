#!/usr/bin/env python3
"""How the census's multi-number templates split for the ranged-line rule
(search/DESIGN.md, the language reference, "Slots"). Reads
../data/mod-templates.csv; prints one row per case and the checks the rule
leans on. Run: python3 search/item-facts/scripts/ranged-split.py [-v]"""
import collections
import csv
import pathlib
import re
import sys

CENSUS = pathlib.Path(__file__).resolve().parent.parent / "data" / "mod-templates.csv"
PAIR = re.compile(r"#%? to #")
PAREN = re.compile(r"\(#-#\)")

CASES = {
    "A": "one `# to #`, no other number",
    "B": "one `# to #` and other numbers",
    "C": "two or more `# to #`",
    "D": "several numbers, no `# to #`",
    "E": "`(#-#)`: a description of possible rolls",
}


def classify(template):
    if template.count("#") < 2:
        return None
    if PAREN.search(template):
        return "E"
    pairs = len(PAIR.findall(template))
    if pairs == 0:
        return "D"
    if pairs >= 2:
        return "C"
    return "A" if template.count("#") == 2 else "B"


def main():
    # newline="\n": 260 templates carry a bare CR (search/README.md, traps)
    with open(CENSUS, newline="\n") as fh:
        fh.readline()
        rows = list(csv.DictReader(fh))
    by_case = collections.defaultdict(list)
    for row in rows:
        case = classify(row["template"])
        if case:
            by_case[case].append(row)
    print(f"{len(rows)} census rows, {sum(map(len, by_case.values()))} with two or more numbers")
    for case, label in CASES.items():
        found = by_case[case]
        templates = {r["template"] for r in found}
        lines = sum(int(r["lines"]) for r in found)
        print(f"{case}  {len(templates):4d} templates  {lines:5d} lines  {label}")

    # the checks the rule leans on
    inverted = 0
    for row in by_case["A"]:
        m = re.search(r"([+-]?\d+(?:\.\d+)?)%? to ([+-]?\d+(?:\.\d+)?)", row["example"])
        if not m or float(m.group(1)) > float(m.group(2)):
            inverted += 1
    print(f"A: examples where low > high or the pair is unreadable: {inverted}")
    pair_first = sum(1 for r in by_case["B"] if PAIR.search(r["template"]).start() == r["template"].index("#"))
    print(f"B: the pair holds the first two numbers in {pair_first} of {len(by_case['B'])}")
    prefixed = sum(1 for r in by_case["E"] if re.match(r"^[A-Za-z ]+: ", r["template"]))
    arrays = sorted({r["array"] for r in by_case["E"]})
    print(f"E: a slot prefix on {prefixed} of {len(by_case['E'])}; arrays {arrays}")
    print(f"most numbers in one template: {max(r['template'].count('#') for r in rows)}")

    if "-v" in sys.argv:
        for case in "BC":
            print(f"\n== {case}")
            for row in sorted(by_case[case], key=lambda r: -int(r["lines"])):
                print(f"{int(row['lines']):5d}  {row['template']!r}")


if __name__ == "__main__":
    main()
