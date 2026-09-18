#!/usr/bin/env python3
"""How each unique in the owner's store fits the variants Path of Building labels for its name.

Inputs:  the two store backups in raw/, and the unique files of the PathOfBuilding clone at 16de4b82.
Output:  data/unique-fit.csv — one row per unique item, keyed by a hash prefix, never the GGG id.
"""

import collections
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402
from variants import OLD  # noqa: E402

FRAME_MATCHED = {"Unique"}
FRAME_APART = {"SupporterFoil"}
NAME_PREFIX = re.compile(r"^(?:<<[^>]*>>)+")
FIT_ARRAYS = ("implicitMods", "explicitMods")


def displayed(item):
    """The item's implicit and explicit lines — the pool the fit is measured over."""
    out = []
    for array, texts, _ in common.mod_arrays(item):
        if array in FIT_ARRAYS:
            out.extend(texts)
    return out


def flags_of(item):
    counts = collections.Counter()
    for array, _, flags in common.mod_arrays(item):
        if array in FIT_ARRAYS:
            counts.update(flags)
    return counts


def fit(ip, var_lines):
    """Does the variant fit? `ip` is the item's tokenized lines.

    Both ways, as the brief defines it: every displayed line matches some line of the variant, and
    every line of the variant is matched by some displayed line. Lines are matched by cover, not
    one-to-one, so a line repeated on one side is satisfied by a single line on the other.
    Returns (fits, [variant lines the item does not display]).
    """
    vp = [common.tokenize(t) for t in var_lines]
    for tpl, specs in ip:
        same = [s for t, s in vp if t == tpl]
        if not same or not any(common.numbers_fit(specs, s) for s in same):
            return False, None
    missing = [text for text, (tpl, specs) in zip(var_lines, vp)
               if not any(t == tpl and common.numbers_fit(s, specs) for t, s in ip)]
    return (not missing), missing


def why_none(item_lines, ip, per_variant):
    """The first reason no variant fits, in the brief's order, against every candidate variant.

    `per_variant` is [(label, variant lines, tokenized, missing), …].
    """
    union = [p for _, _, vp, _ in per_variant for p in vp]
    for text, (tpl, specs) in zip(item_lines, ip):
        if not any(t == tpl for t, _ in union):
            return "no-text-match", text
    for text, (tpl, specs) in zip(item_lines, ip):
        if not any(t == tpl and common.numbers_fit(specs, s) for t, s in union):
            return "number-outside-range", text
    best = min(per_variant, key=lambda v: (len(v[3] or v[1]), v[0]))
    return "variant-line-not-displayed", (best[3] or best[1])[0]


SWAP = {"increased": "reduced", "reduced": "increased", "more": "less", "less": "more"}
WORD = re.compile(r"[A-Za-z]+")


def swapped(tpl):
    return WORD.sub(lambda m: SWAP.get(m.group(0), m.group(0)), tpl)


def depluralised(tpl):
    return WORD.sub(lambda m: m.group(0)[:-1] if m.group(0).endswith("s") else m.group(0), tpl)


def main():
    commit = common.require_pob_commit()
    entries = [e for e in common.read_entries() if e.refusal is None]
    by_name = collections.defaultdict(list)
    for e in entries:
        by_name[e.name].append(e)

    items = common.read_items()
    apart = collections.Counter()
    pop = []
    for it in items:
        j = it.json
        frame = j.get("frameTypeId")
        if frame in FRAME_APART or j.get("isRelic"):
            if frame in FRAME_APART or frame in FRAME_MATCHED:
                apart[f"relic or foil frame type ({frame})"] += 1
            continue
        if frame not in FRAME_MATCHED:
            continue
        if not j.get("identified", True):
            apart["unidentified unique"] += 1
            continue
        if it.realm == "poe2":
            apart["realm poe2"] += 1
            continue
        pop.append(it)

    rows = []
    outcome = collections.Counter()
    either = 0
    reasons = collections.Counter()
    cross = collections.Counter()
    by_league = collections.defaultdict(collections.Counter)
    name_unbound = collections.Counter()
    flag_items = collections.Counter()
    near = collections.Counter()
    for it in pop:
        j = it.json
        name = NAME_PREFIX.sub("", j.get("name") or "").strip()
        cand = by_name.get(name, [])
        lines = displayed(j)
        descriptor, quality = common.catalyst_quality(j)
        others = sorted({a for a, t, _ in common.mod_arrays(j) if a not in FIT_ARRAYS and t})
        fl = flags_of(j)
        if fl:
            for k in fl:
                flag_items[k] += 1
        n_variants = sum(len(e.labels) for e in cand)
        ip = [common.tokenize(t) for t in lines]
        fitting, per_variant = [], []
        for e in cand:
            for vi, label in enumerate(e.labels, 1):
                vl = common.variant_lines(e, vi)
                ok, missing = fit(ip, vl)
                if ok:
                    fitting.append(label)
                per_variant.append((label, vl, [common.tokenize(t) for t in vl], missing))
        if not cand:
            out = "unbound"
            name_unbound[name] += 1
        elif len(fitting) == 1:
            out = "one"
        elif len(fitting) > 1:
            out = "several"
        else:
            out = "none"
        is_either = "no"
        if fitting:
            olds = [bool(OLD.search(l)) for l in fitting]
            if any(olds) and not all(olds):
                is_either = "yes"
                either += 1
        outcome[out] += 1
        by_league[it.league][out] += 1
        cat = "yes" if descriptor else "no"
        cross[(out, cat)] += 1
        reason, line = ("", "")
        if out == "none":
            reason, line = why_none(lines, ip, per_variant)
            reasons[reason] += 1
            if any("Has Alt Variant" in h for e in cand for h in e.headers):
                near["the entry carries a `Has Alt Variant` header"] += 1
            if reason == "no-text-match":
                union = {t for _, _, vp, _ in per_variant for t, _ in vp}
                tpl = common.tokenize(line)[0]
                if swapped(tpl) in union:
                    near["the line matches a variant line once `increased`/`reduced` are exchanged"] += 1
                elif depluralised(tpl) in {depluralised(u) for u in union}:
                    near["the line matches a variant line once a trailing plural `s` is ignored"] += 1
        rows.append([common.row_key(it.id), name, j.get("baseType", ""), it.league,
                     len(cand), n_variants, ";".join(sorted(set(fitting))), out, is_either,
                     cat, descriptor or "", quality if quality is not None else "",
                     ";".join(others), ";".join(sorted(fl)), reason, line])
    common.write_csv("unique-fit.csv",
                     ["row_key", "name", "base", "league", "entries_bound", "variants_considered",
                      "fitting_labels", "outcome", "either", "catalyst_quality", "catalyst_descriptor",
                      "catalyst_percent", "other_arrays", "fit_line_flags", "none_reason", "none_line"],
                     rows)

    print(f"PathOfBuilding {commit}; entries bound from {len(by_name):,} distinct names")
    print(f"  items in both stores (deduplicated by GGG id): {len(items):,}")
    print("  counted apart, not matched:")
    for k, c in apart.most_common():
        print(f"    {c:6,d}  {k}")
    print(f"  population matched: {len(pop):,}")
    print("  outcome:")
    for k in ("one", "several", "none", "unbound"):
        print(f"    {outcome[k]:6,d}  {k}")
    print(f"  either (the fitting set holds an old and a not-old variant): {either:,}")
    print("  `none` reasons:")
    for k, c in reasons.most_common():
        print(f"    {c:6,d}  {k}")
    print("  of the `none` rows, how many sit near a fit:")
    for k, c in near.most_common():
        print(f"    {c:6,d}  {k}")
    print("  catalyst quality × outcome:")
    for k in ("one", "several", "none", "unbound"):
        print(f"    {k:>8}: carries it {cross[(k, 'yes')]:,}   does not {cross[(k, 'no')]:,}")
    n_out = sum(1 for r in rows if r[7] == "none" and r[14] == "number-outside-range")
    n_out_cat = sum(1 for r in rows if r[7] == "none" and r[14] == "number-outside-range" and r[9] == "yes")
    print(f"  `none` rows whose reason is a number outside every range: {n_out:,}; "
          f"of those, carrying catalyst quality: {n_out_cat:,}")
    print("  by league:")
    for lg in sorted(by_league):
        c = by_league[lg]
        print(f"    {lg:<18} one {c['one']:,}  several {c['several']:,}  none {c['none']:,}  unbound {c['unbound']:,}")
    print(f"  distinct names that bind to no entry: {len(name_unbound):,}; the commonest:")
    for n, c in name_unbound.most_common(12):
        print(f"    {c:6,d}  {n}")
    print("  items whose implicit or explicit lines carry a flag (the fit reads them like any line):")
    for k, c in flag_items.most_common():
        print(f"    {c:6,d}  {k}")
    print("  items carrying a line outside implicit and explicit, among the population:")
    arr = collections.Counter()
    for r in rows:
        for a in filter(None, r[12].split(";")):
            arr[a] += 1
    for a, c in arr.most_common():
        print(f"    {c:6,d}  {a}")
    ashes = [r for r in rows if r[1] == "Ashes of the Stars"]
    lines_by_key = {common.row_key(it.id): displayed(it.json) for it in pop}
    print(f"  Ashes of the Stars (S165) in the population: {len(ashes)}")
    for e in by_name.get("Ashes of the Stars", []):
        print(f"    entry {e.file} #{e.index} — {e.base}; variants {e.labels}")
        for vi, label in enumerate(e.labels, 1):
            print(f"      {label}: {common.variant_lines(e, vi)}")
    for r in sorted(ashes, key=lambda r: (r[7], r[0])):
        print(f"    {r[0]}  {r[3]:<16} {r[7]:<8} fits [{r[6]}]  either {r[8]}  "
              f"catalyst {r[9]} {r[10]} {r[11]}  other [{r[12]}]  {r[14]} {r[15]!r}")
        print(f"      displays: {lines_by_key[r[0]]}")
    apart_ashes = [it for it in items
                   if NAME_PREFIX.sub("", it.json.get("name") or "").strip() == "Ashes of the Stars"
                   and (it.json.get("frameTypeId") in FRAME_APART or it.json.get("isRelic")
                        or it.realm == "poe2" or not it.json.get("identified", True))]
    print(f"  Ashes of the Stars counted apart: {len(apart_ashes)}")
    for it in apart_ashes:
        print(f"    {common.row_key(it.id)}  {it.league:<16} frame {it.json.get('frameTypeId')}  "
              f"isRelic {it.json.get('isRelic')}  displays: {displayed(it.json)}")




if __name__ == "__main__":
    sys.exit(main())
