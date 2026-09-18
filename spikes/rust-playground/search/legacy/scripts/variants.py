#!/usr/bin/env python3
"""Every variant Path of Building's unique files label, and every block the parse refused.

Input:  ../../../../PathOfBuilding/src/Data/Uniques/*.lua and Special/*.lua at 16de4b82.
Output: data/pob-variants.csv, data/pob-parse-refusals.csv
"""

import collections
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

# A variant is *old* when its label names a game version it precedes (the brief's definition).
OLD = re.compile(r"\bPre\s+\d+(?:\.\d+)*", re.I)
VERSION = re.compile(r"\d+\.\d+(?:\.\d+)*")


def label_shape(label):
    """The census bucket of a label: what axis it names, never what it means."""
    if label == "only":
        return "only (the entry has no Variant line)"
    if OLD.fullmatch(label.strip()):
        return "bare `Pre <version>`"
    if OLD.search(label):
        return "compound holding `Pre <version>`"
    if label.strip().lower() == "current":
        return "`Current`"
    if VERSION.search(label):
        return "names a version, not `Pre`"
    if re.fullmatch(r"[A-Za-z' ]+:[A-Za-z' ]+", label):
        return "`<thing>: <axis>`"
    if "(" in label and ")" in label:
        return "parenthesised qualifier"
    return "names an axis other than a version"


def main():
    commit = common.require_pob_commit()
    entries = common.read_entries()
    ok = [e for e in entries if e.refusal is None]
    refused = [e for e in entries if e.refusal is not None]

    rows = []
    shapes = collections.Counter()
    for e in ok:
        size = common.selection_size(e)
        for i, label in enumerate(e.labels, 1):
            lines = common.variant_tagged_lines(e, i)
            shape = label_shape(label)
            shapes[shape] += 1
            rows.append([e.file, e.index, e.name, e.base, i, label,
                         "yes" if OLD.search(label) else "no", shape, len(lines), size,
                         sum(1 for _, tags in lines if tags.get("tags"))])
    common.write_csv("pob-variants.csv",
                     ["file", "entry_index", "name", "base", "variant_index", "label", "old",
                      "label_shape", "variant_lines", "selection_size", "tagged_lines"], rows)

    rrows = [[e.file, e.index, e.name or "", e.refusal] for e in refused]
    common.write_csv("pob-parse-refusals.csv", ["file", "entry_index", "name", "refusal"], rrows)

    names = collections.Counter(e.name for e in ok)
    print(f"PathOfBuilding {commit}")
    print(f"  files read: {len(common.unique_files())}")
    print(f"  entries parsed: {len(ok)}   refused: {len(refused)}")
    print(f"  distinct entry names: {len(names)}   names with more than one entry: "
          f"{sum(1 for n, c in names.items() if c > 1)}")
    print(f"  variants: {len(rows)}   old: {sum(1 for r in rows if r[6] == 'yes')}   "
          f"not-old: {sum(1 for r in rows if r[6] == 'no')}")
    print(f"  entries with no Variant line (one variant, `only`): "
          f"{sum(1 for e in ok if e.labels == ['only'])}")
    print("  label shapes:")
    for shape, c in shapes.most_common():
        print(f"    {c:5d}  {shape}")
    print("  refusals by reason:")
    for reason, c in collections.Counter(re.sub(r"\[[^]]*\]", "[…]", e.refusal) for e in refused).most_common():
        print(f"    {c:5d}  {reason}")
    print("  entries carrying an alt-variant header (a second variant can be selected at once):")
    alt = collections.Counter()
    for e in ok:
        for h in e.headers:
            if h.startswith("Has Alt Variant"):
                alt[h] += 1
    for h, c in sorted(alt.items()):
        print(f"    {c:5d}  {h}")
    odd = [e for e in ok if "base not in Data/Bases" in e.headers]
    print(f"  entries whose base line is not a name `Data/Bases/*.lua` declares: {len(odd)}")
    for e in odd:
        print(f"      {e.file} #{e.index}  {e.name} — {e.base!r}")
    print(f"  entries with more than one base line (the base varies by variant): "
          f"{sum(1 for e in ok if ' / ' in e.base)}")
    print(f"  entries with a `Source:` header: {sum(1 for e in ok if 'Source' in e.headers)}   "
          f"with `League:`: {sum(1 for e in ok if 'League' in e.headers)}   "
          f"with `Upgrade:`: {sum(1 for e in ok if 'Upgrade' in e.headers)}")
    src = collections.Counter(v for e in ok for v in e.headers.get("Source", []))
    print("  the five commonest `Source:` values:")
    for v, c in src.most_common(5):
        print(f"    {c:5d}  {v}")
    tagged, numbered, answered = set(), 0, 0
    for e in ok:
        for text, _, tags in e.mods:
            if not tags.get("tags") or text in tagged:
                continue
            tagged.add(text)
            specs = common.tokenize(text)[1]
            if not specs:
                continue
            numbered += 1
            if all(p[1] > 0 for p in common.scalability_plan(text, specs)):
                answered += 1
    print(f"  distinct mod lines carrying a `{{tags:…}}` prefix, which a catalyst may rescale: "
          f"{len(tagged)}, of which {numbered} carry a number")
    print(f"    their scaling is answered by `Data/ModScalability.lua` for {answered} and falls "
          f"back to the old method for {numbered - answered}")


if __name__ == "__main__":
    sys.exit(main())
