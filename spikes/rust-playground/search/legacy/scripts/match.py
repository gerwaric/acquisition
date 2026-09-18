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

# `_strict` is the first pass's rule — one variant at a time, nothing rescaled. The unsuffixed
# columns are the final one: a selection of variants, rescaled by the item's catalyst.
OUTCOMES = ("one", "several", "none", "unbound")
HEADER = ["row_key", "name", "base", "league", "entries_bound", "variants_considered",
          "selections_considered", "fitting_labels", "outcome_strict", "either_strict",
          "fitting_selections", "outcome", "either", "moved_by", "lines_rescaled",
          "catalyst_quality", "catalyst_descriptor", "catalyst_percent", "other_arrays",
          "fit_line_flags", "none_reason", "none_line"]
C = {name: i for i, name in enumerate(HEADER)}


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


def fit(ip, vp):
    """Does the selection fit? `ip` is the item's tokenized lines, `vp` the selection's.

    Both ways, as the brief defines it: every displayed line matches some line of the selection, and
    every line of the selection is matched by some displayed line. Lines are matched by cover, not
    one-to-one, so a line repeated on one side is satisfied by a single line on the other.
    Returns (fits, [selection lines the item does not display]).
    """
    for tpl, specs in ip:
        same = [s for _, t, s in vp if t == tpl]
        if not same or not any(common.numbers_fit(specs, s) for s in same):
            return False, None
    missing = [text for text, tpl, specs in vp
               if not any(t == tpl and common.numbers_fit(s, specs) for t, s in ip)]
    return (not missing), missing


def why_none(item_lines, ip, per_selection):
    """The first reason no selection fits, in the brief's order, against every candidate selection.

    `per_selection` is [(label, [(text, template, specs), …], missing), …].
    """
    union = [(t, s) for _, vp, _ in per_selection for _, t, s in vp]
    for text, (tpl, specs) in zip(item_lines, ip):
        if not any(t == tpl for t, _ in union):
            return "no-text-match", text
    for text, (tpl, specs) in zip(item_lines, ip):
        if not any(t == tpl and common.numbers_fit(specs, s) for t, s in union):
            return "number-outside-range", text
    best = min(per_selection, key=lambda v: (len(v[2] if v[2] else v[1]), v[0]))
    return "variant-line-not-displayed", (best[2] or [t for t, _, _ in best[1]])[0]


# The four fits of one item: one variant or a selection, rescaled by its catalyst or not. Selections
# and their tokenized lines are shared by every item bound to the entry, so they are memoised.
_sel_cache = {}


def selection_pairs(entry, sel, index, quality):
    """[(text, template, specs), …] for a selection, under an item's catalyst kind and quality."""
    key = (id(entry), sel, index, quality)
    hit = _sel_cache.get(key)
    if hit is None:
        hit = []
        for text, tags in common.selection_lines(entry, sel):
            tpl, specs, _ = common.scale_line(text, tags, index, quality)
            hit.append((text, tpl, specs))
        _sel_cache[key] = hit
    return hit


def rescaled_lines(entry, index, quality):
    """How many of the entry's mod lines the item's catalyst scales."""
    n = 0
    for text, _, tags in entry.mods:
        if common.scale_line(text, tags, index, quality)[2]:
            n += 1
    return n


def outcome_of(cand, fitting):
    if not cand:
        return "unbound"
    return "one" if len(fitting) == 1 else "several" if len(fitting) > 1 else "none"


# No variant label holds ` | `, so a selection of several reads back unambiguously.
SEL_SEP = " | "


def is_either(fitting):
    """A fitting set holding an old and a not-old member. `fitting` is [(label, is old), …]."""
    olds = [old for _, old in fitting]
    return bool(olds) and any(olds) and not all(olds)


def labels_of(fitting):
    return ";".join(sorted({label for label, _ in fitting}))


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
    strict_outcome = collections.Counter()
    either = strict_either = 0
    reasons = collections.Counter()
    strict_reasons = collections.Counter()
    strict_reasons_cat = collections.Counter()
    cross = collections.Counter()
    strict_cross = collections.Counter()
    by_league = collections.defaultdict(collections.Counter)
    name_unbound = collections.Counter()
    flag_items = collections.Counter()
    near = collections.Counter()
    transition = collections.Counter()
    movers = collections.Counter()
    kind_apart = collections.Counter()
    alt_entries = collections.defaultdict(collections.Counter)
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
        index = common.catalyst_index(descriptor) if descriptor else None
        if index in common.PREFIX_SUFFIX_INDEX:
            # Sinistral and Dextral key on a mod line's prefix/suffix flag, which no unique file
            # line here carries: the item is counted apart and nothing of it is rescaled.
            kind_apart[descriptor] += 1
            index = None
        n_variants = sum(len(e.labels) for e in cand)
        n_sel = sum(len(common.selections(e)) for e in cand)
        ip = [common.tokenize(t) for t in lines]
        strict_fit, sel_fit, cat_fit, final_fit, per_selection = [], [], [], [], []
        per_variant = []
        for e in cand:
            for sel in common.selections(e):
                label = SEL_SEP.join(e.labels[v - 1] for v in sel)
                old = any(OLD.search(e.labels[v - 1]) for v in sel)
                plain = selection_pairs(e, sel, None, None)
                ok_plain, miss_plain = fit(ip, plain)
                if index is None:
                    scaled, ok_scaled, miss = plain, ok_plain, miss_plain
                else:
                    scaled = selection_pairs(e, sel, index, quality)
                    ok_scaled, miss = fit(ip, scaled)
                if len(sel) == 1:
                    per_variant.append((label, plain, miss_plain))
                    if ok_plain:
                        strict_fit.append((label, old))
                    if ok_scaled:
                        cat_fit.append((label, old))
                if ok_plain:
                    sel_fit.append((label, old))
                if ok_scaled:
                    final_fit.append((label, old))
                per_selection.append((label, scaled, miss))
        out_strict = outcome_of(cand, strict_fit)
        out = outcome_of(cand, final_fit)
        if not cand:
            name_unbound[name] += 1
        moved = ""
        if out != out_strict:
            by_sel = outcome_of(cand, sel_fit) != out_strict
            by_cat = outcome_of(cand, cat_fit) != out_strict
            moved = ("selection" if by_sel and not by_cat else
                     "catalyst" if by_cat and not by_sel else
                     "both" if not by_sel and not by_cat else "either fix alone")
            movers[moved] += 1
        strict_e = is_either(strict_fit)
        final_e = is_either(final_fit)
        either += final_e
        strict_either += strict_e
        outcome[out] += 1
        strict_outcome[out_strict] += 1
        transition[(out_strict, out)] += 1
        by_league[it.league][out] += 1
        cat = "yes" if descriptor else "no"
        cross[(out, cat)] += 1
        strict_cross[(out_strict, cat)] += 1
        rescaled = sum(rescaled_lines(e, index, quality) for e in cand) if index is not None else 0
        reason, line = ("", "")
        own_implicit = False
        if out_strict == "none":
            r_strict = why_none(lines, ip, per_variant)[0]
            strict_reasons[r_strict] += 1
            if descriptor:
                strict_reasons_cat[r_strict] += 1
        if out == "none":
            reason, line = why_none(lines, ip, per_selection)
            reasons[reason] += 1
            own_implicit = line in [t for a, ts, _ in common.mod_arrays(j)
                                    if a == "implicitMods" for t in ts]
            if any("Has Alt Variant" in h for e in cand for h in e.headers):
                near["the entry carries a `Has Alt Variant` header"] += 1
                if own_implicit:
                    near["— and the line it fails on is one of the item's own implicits"] += 1
        for e in cand:
            if any(h.startswith("Has Alt Variant") for h in e.headers):
                c = alt_entries[(e.file, e.index, e.name, len(e.labels), common.selection_size(e))]
                c[out] += 1
                if out == "none" and own_implicit:
                    c["none on one of the item's own implicits"] += 1
            if reason == "no-text-match":
                union = {t for _, vp, _ in per_selection for _, t, _ in vp}
                tpl = common.tokenize(line)[0]
                if swapped(tpl) in union:
                    near["the line matches a variant line once `increased`/`reduced` are exchanged"] += 1
                elif depluralised(tpl) in {depluralised(u) for u in union}:
                    near["the line matches a variant line once a trailing plural `s` is ignored"] += 1
        rows.append([common.row_key(it.id), name, j.get("baseType", ""), it.league,
                     len(cand), n_variants, n_sel,
                     labels_of(strict_fit), out_strict, "yes" if strict_e else "no",
                     labels_of(final_fit), out, "yes" if final_e else "no",
                     moved, rescaled,
                     cat, descriptor or "", quality if quality is not None else "",
                     ";".join(others), ";".join(sorted(fl)), reason, line])
    common.write_csv("unique-fit.csv", HEADER, rows)

    print(f"PathOfBuilding {commit}; entries bound from {len(by_name):,} distinct names")
    print(f"  items in both stores (deduplicated by GGG id): {len(items):,}")
    print("  counted apart, not matched:")
    for k, c in apart.most_common():
        print(f"    {c:6,d}  {k}")
    print(f"  population matched: {len(pop):,}")
    print("  outcome — strict (one variant, nothing rescaled) beside final (selections, rescaled):")
    for k in ("one", "several", "none", "unbound"):
        print(f"    {k:>8}: strict {strict_outcome[k]:6,d}   final {outcome[k]:6,d}")
    print(f"  either (the fitting set holds an old and a not-old member): "
          f"strict {strict_either:,}   final {either:,}")
    print("  strict → final (rows sum to the strict count, columns to the final one):")
    keys = ("one", "several", "none", "unbound")
    print(f"    {'':>10}" + "".join(f"{k:>10}" for k in keys) + f"{'row':>10}")
    for a in keys:
        print(f"    {a:>10}" + "".join(f"{transition[(a, b)]:10,d}" for b in keys)
              + f"{sum(transition[(a, b)] for b in keys):10,d}")
    print(f"    {'column':>10}" + "".join(f"{sum(transition[(a, b)] for a in keys):10,d}" for b in keys)
          + f"{sum(transition.values()):10,d}")
    print("  movers, by what moved them:")
    for k in ("catalyst", "selection", "both", "either fix alone"):
        print(f"    {movers[k]:6,d}  {k}")
    print("  `none` reasons, strict then final:")
    for k, c in reasons.most_common():
        print(f"    strict {strict_reasons[k]:6,d}   final {c:6,d}   {k}")
    print("  the commonest failing line of each `none` reason, under the final rule, by template:")
    for k, _ in reasons.most_common():
        group = collections.defaultdict(list)
        for r in rows:
            if r[C['none_reason']] == k:
                group[common.tokenize(r[C['none_line']])[0]].append(r)
        for tpl, rs in sorted(group.items(), key=lambda kv: -len(kv[1]))[:2]:
            names = collections.Counter(r[C['name']] for r in rs)
            print(f"    {len(rs):6,d}  {k}: {rs[0][C['none_line']]!r} "
                  f"({', '.join(f'{n} {v}' for n, v in names.most_common(2))})")
    print("  of the `none` rows, how many sit near a fit:")
    for k, c in near.most_common():
        print(f"    {c:6,d}  {k}")
    print("  catalyst quality × outcome, strict then final:")
    for k in ("one", "several", "none", "unbound"):
        print(f"    {k:>8}: strict {strict_cross[(k, 'yes')]:,}   final {cross[(k, 'yes')]:,}"
              f"   (without catalyst quality, final {cross[(k, 'no')]:,})")
    if kind_apart:
        print("  catalysed items whose kind has nothing on a unique's line to match (counted apart):")
        for k, c in kind_apart.most_common():
            print(f"    {c:6,d}  {k}")
    else:
        print("  catalysed items of the prefix/suffix kinds, which have nothing to match: 0")
    n_out = sum(1 for r in rows if r[C['outcome']] == "none" and r[C['none_reason']] == "number-outside-range")
    n_out_cat = sum(1 for r in rows if r[C['outcome']] == "none"
                    and r[C['none_reason']] == "number-outside-range" and r[C['catalyst_quality']] == "yes")
    print(f"  `none` rows whose reason is a number outside every range: "
          f"strict {strict_reasons['number-outside-range']:,}, final {n_out:,}; of those, carrying "
          f"catalyst quality: strict {strict_reasons_cat['number-outside-range']:,}, final {n_out_cat:,}")
    print("  by league, final:")
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
        for a in filter(None, r[C['other_arrays']].split(";")):
            arr[a] += 1
    for a, c in arr.most_common():
        print(f"    {c:6,d}  {a}")

    cat_rows = [r for r in rows if r[C['catalyst_quality']] == "yes"]
    print(f"  catalysed items in the population: {len(cat_rows)}; "
          f"lines rescaled, total {sum(r[C['lines_rescaled']] for r in cat_rows):,}")
    print("  every catalysed item that fitted under the strict rule, and what it fits after rescale:")
    for r in sorted(cat_rows, key=lambda r: (r[C['name']], r[C['row_key']])):
        if r[C['outcome_strict']] in ("one", "several"):
            print(f"    {r[C['row_key']]}  {r[C['name']]} ({r[C['base']]}), {r[C['league']]}, "
                  f"{r[C['catalyst_descriptor']]} +{r[C['catalyst_percent']]}%, "
                  f"{r[C['lines_rescaled']]} line(s) rescaled")
            print(f"      strict {r[C['outcome_strict']]} [{r[C['fitting_labels']]}]  →  "
                  f"final {r[C['outcome']]} [{r[C['fitting_selections']]}]"
                  + (f"  {r[C['none_reason']]} {r[C['none_line']]!r}" if r[C['outcome']] == "none" else ""))
    print("  catalysed items by strict → final outcome:")
    for k, c in collections.Counter((r[C['outcome_strict']], r[C['outcome']]) for r in cat_rows).most_common():
        print(f"    {c:6,d}  {k[0]} → {k[1]}")
    print("  catalysed items by kind (what the item's property shows), and lines rescaled:")
    kinds = collections.Counter(r[C['catalyst_descriptor']] for r in cat_rows)
    for k, c in kinds.most_common():
        print(f"    {c:6,d}  {k:<26} lines rescaled "
              f"{sum(r[C['lines_rescaled']] for r in cat_rows if r[C['catalyst_descriptor']] == k):,}")
    print("  catalysed items still `none` after the rescale:")
    for r in cat_rows:
        if r[C['outcome']] == "none":
            print(f"    {r[C['row_key']]}  {r[C['name']]} ({r[C['base']]}), {r[C['catalyst_descriptor']]} "
                  f"+{r[C['catalyst_percent']]}%, {r[C['lines_rescaled']]} rescaled — "
                  f"{r[C['none_reason']]} {r[C['none_line']]!r}")

    print(f"  the entries carrying a `Has Alt Variant` header that bind an item: {len(alt_entries)}")
    for (f, i, nm, nlab, size), c in sorted(alt_entries.items(),
                                            key=lambda kv: -sum(kv[1][k] for k in OUTCOMES)):
        print(f"    {f} #{i}  {nm}: {nlab} variants, {size} chosen at once, "
              f"{sum(c[k] for k in OUTCOMES):,} items — "
              + ", ".join(f"{k} {c[k]:,}" for k in OUTCOMES if c[k])
              + (f"; {c['none on one of the item\'s own implicits']:,} of the `none` fail on one of "
                 f"the item's own implicits" if c["none on one of the item's own implicits"] else ""))
    alt_rows = [r for r in rows if r[C['moved_by']] and r[C['selections_considered']] > r[C['variants_considered']]]
    print(f"  items on an alt-variant entry whose outcome moved: {len(alt_rows):,}")
    print("  the `none` rows still on an alt-variant entry, by reason:")
    still = collections.Counter(r[C['none_reason']] for r in rows if r[C['outcome']] == "none"
                                and r[C['selections_considered']] > r[C['variants_considered']])
    for k, c in still.most_common():
        print(f"    {c:6,d}  {k}")

    ashes = [r for r in rows if r[C['name']] == "Ashes of the Stars"]
    lines_by_key = {common.row_key(it.id): displayed(it.json) for it in pop}
    print(f"  Ashes of the Stars (S165) in the population: {len(ashes)}")
    for e in by_name.get("Ashes of the Stars", []):
        print(f"    entry {e.file} #{e.index} — {e.base}; variants {e.labels}")
        for vi, label in enumerate(e.labels, 1):
            print(f"      {label}: {common.variant_lines(e, vi)}")
        print("      what an Attribute catalyst at the +20% the three catalysed ones carry rescales:")
        for text, tags in common.variant_tagged_lines(e, 1):
            tpl, specs, did = common.scale_line(text, tags, common.catalyst_index("Attribute"), 20)
            if did:
                print(f"        {text!r}  →  {tpl} {[(int(lo), int(hi)) for lo, hi in specs]}")
    for r in sorted(ashes, key=lambda r: (r[C['outcome']], r[C['row_key']])):
        print(f"    {r[C['row_key']]}  {r[C['league']]:<16} strict {r[C['outcome_strict']]:<8} "
              f"[{r[C['fitting_labels']]}]  →  final {r[C['outcome']]:<8} [{r[C['fitting_selections']]}]  "
              f"either {r[C['either']]}  catalyst {r[C['catalyst_descriptor']]} {r[C['catalyst_percent']]}"
              f"  rescaled {r[C['lines_rescaled']]}  {r[C['none_reason']]} {r[C['none_line']]!r}")
        print(f"      displays: {lines_by_key[r[C['row_key']]]}")
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
