#!/usr/bin/env python3
"""The store crate's read surface: every public read a frontend can call,
what it returns, what it is keyed on, what it costs, and who calls it.

Inputs:
  crates/acquisition-store/src/{lib.rs,snapshot.rs,annotations.rs,index.rs}
      — the symbols must exist, or this script fails rather than print a
        stale table
  crates/acquisition-{cli,mcp,plan,daemon}/src   — callers, grepped
  data/query-plans.txt (scripts/query-plans.py)  — the cost evidence

The per-read annotations (returns, key, cost class, verb) are in ENTRIES
below: each cost class is the plan in data/query-plans.txt, each verb the
name CLI-REFERENCE.md / MCP-REFERENCE.md give it (both generated from the
binaries' own help).

Output: data/read-surface.md.
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TRACK = Path(__file__).resolve().parents[1]
STORE = ROOT / "crates" / "acquisition-store" / "src"
FRONTENDS = {
    "cli": ROOT / "crates" / "acquisition-cli" / "src",
    "mcp": ROOT / "crates" / "acquisition-mcp" / "src",
    "plan": ROOT / "crates" / "acquisition-plan" / "src",
    "daemon": ROOT / "crates" / "acquisition-daemon" / "src",
}
OUT = TRACK / "data" / "read-surface.md"

# symbol, file, returns, keyed on, cost class, verbs/tools
ENTRIES = [
    # --- facts: the item reads ---
    ("Store::search", "lib.rs", "`Vec<ItemRow>` (each with the whole body parsed)",
     "text substring; optional realm, league; live-or-all; limit",
     "**full scan of `items`** + temp b-tree sort + one `serde_json` parse per returned row",
     "`acq items search`, MCP `search_items`"),
    ("Store::item", "lib.rs", "`Option<ItemRow>`", "item id (PK)",
     "one row by index + one body parse", "`acq items show`, MCP `get_item`"),
    ("Store::events_since", "lib.rs", "`Vec<EventRow>` (kind, from/to location, name)",
     "unix seconds, oldest first; limit",
     "index seek on `item_events_at` + one PK lookup into `items` per event",
     "`acq store events`, MCP `item_events`, `acq refresh --apply`"),
    # --- facts: the container reads ---
    ("Store::tabs", "lib.rs", "`Vec<TabRow>` in listing order",
     "(realm, league); live only",
     "PK-prefix seek on `tabs` + per tab an indexed count over `items` and a `json_extract`; temp b-tree for the order",
     "`acq tabs`, MCP `tabs`"),
    ("Store::characters", "lib.rs", "`Vec<CharacterRow>`",
     "optional realm, optional league; live only",
     "scan of `characters` + per character an indexed count over `items` and a `json_each` of `_split`",
     "`acq store characters`, MCP `characters`"),
    ("Store::holds", "lib.rs", "`bool`",
     "a `FactAddress`: item id, character id, (realm, id), (realm, parent, id)",
     "one indexed existence check (a substash seeks on `realm` and filters — `parent` is not indexed)",
     "`acq price set` (C64: intent is never gated by facts)"),
    ("Store::status", "lib.rs", "`Status` (path, bytes, 12 row counts)", "nothing",
     "12 counts, one scan per table (`items` twice) + two `json_each` aggregations over live characters",
     "`acq store status`, MCP `store_status`, `acq refresh` before/after"),
    ("Store::refused_list", "lib.rs", "`Vec<Refused>` without bodies", "newest first; limit",
     "scan of `refused` (a table no basis query reads, C30)", "`acq store refused`"),
    ("Store::refused", "lib.rs", "`Option<Refused>` with the body verbatim", "`refused` row id",
     "one row", "`acq store refused <id>`"),
    # --- the neutral snapshots ---
    ("Store::refresh_snapshot", "snapshot.rs",
     "`RefreshSnapshot`: two `ListingBasis`, `Vec<TabSnapshot>`, `Vec<CharacterSnapshot>`, the sync-policy row",
     "(realm, league) + an `Annotations` handle bound to the same uuid",
     "one read transaction: two **scans of `responses`** for the bases, the tab and character reads above, one JSON parse per listed tab and per character",
     "`acq refresh`, `acq shop`, MCP `refresh_plan`, `acquisition-plan`"),
    ("Store::pricing_snapshot", "snapshot.rs",
     "`PricingSnapshot`: the same tabs and characters + `Vec<ItemSnapshot>` + every `buyout` row",
     "(realm, league) + the bound `Annotations` handle",
     "the refresh snapshot's reads **plus one pass over the league's live items**, two `json_extract` per item, no body returned",
     "`acq price`, `acq shop`"),
    ("Store::orphaned_item_annotations", "lib.rs",
     "`Vec<AnnotationRow>` whose item is gone or removed", "an `Annotations` handle",
     "one PK lookup into `items` per item annotation", "*nothing — no frontend calls it*"),
    # --- intent ---
    ("Annotations::get", "annotations.rs", "`Option<AnnotationRow>` (value, revision, provenance)",
     "(scope, key, kind)", "one row", "`acq policy show`, `acq price show`, MCP `sync_policy`"),
    ("Annotations::get_as", "annotations.rs", "`Option<(AnnotationRow, K)>`, strictly parsed (C66)",
     "(scope, key) + the kind's Rust type", "one row + one typed parse", "`acquisition-plan`"),
    ("Annotations::list", "annotations.rs", "`Vec<AnnotationRow>`, tombstones excluded",
     "optional scope, optional kind", "scan of `annotations` (measured 35 ms at 10k rows — the doc comment)",
     "`pricing_snapshot`, inside the crate — `acq price list` reaches its rows only through that snapshot"),
    # --- the account index and the world ---
    ("Index::load / resolve / entries", "index.rs", "`AccountEntry` rows from `accounts.json`",
     "username, `#discriminator`, or uuid", "one small JSON file, no database",
     "every frontend's account resolution, MCP `accounts`"),
    ("world::store_dir", "world.rs", "the provider's directory `PathBuf`", "provider name",
     "no I/O", "every frontend before opening a store"),
]

# A bare method name that other types also carry: the exact pattern that
# finds this crate's call and no other (checked by hand against the hits).
PATTERNS = {
    # `t.status()` in reference_cmd is the currency table's; the daemon's
    # `.status(` calls are the rate limiter's and the rails'.
    "Store::status": r"(store|st)\.status\(\)",
    # `.get(` is on every map in the workspace; an annotation get takes
    # three arguments, the last a *_KIND constant.
    "Annotations::get": r"\.get\([^)]*_KIND",
    "Annotations::get_as": r"\.get_as::<",
    "Index::load / resolve / entries": r"Index::load\(",
    "world::store_dir": r"store_dir\(",
    # `.list(` is the daemon's job list too; an annotation list takes
    # two Option arguments.
    "Annotations::list": r"\.list\((None|Some)",
}

# Reads that exist but are not a search consumer's business, listed so the
# table's silence about them is deliberate.
OUT_OF_SCOPE = {
    "jobs::JobDb / Persisted": "the persisted job queue (`daemon.db`) — the daemon's, and `acq jobs` when no daemon runs",
    "Store::record": "the write door, the daemon's alone (C28)",
    "Store::rebuild": "re-extracts every derived column from each row's own json (`acq store rebuild`)",
    "Annotations::put / delete / export": "the intent write door (C65, C66) and its `VACUUM INTO` backup",
}


def check_symbols():
    """Every annotated symbol must still exist where it is claimed."""
    missing = []
    for sym, file, *_ in ENTRIES:
        method = sym.split("::")[-1].split(" / ")[0]
        text = (STORE / file).read_text()
        if not re.search(r"pub fn " + re.escape(method) + r"\b", text):
            missing.append(f"{sym} (looked for `pub fn {method}` in {file})")
    if missing:
        sys.exit("read-surface.py is stale:\n  " + "\n  ".join(missing))


def test_ranges(path):
    """Line ranges of every `#[cfg(test)] mod … { … }` in a file. A file
    can carry several, and a `#[cfg(test)]` on a helper is not one — so
    brace-count the module, never take "after the first attribute"."""
    src = Path(path).read_text().splitlines()
    ranges = []
    i = 0
    while i < len(src):
        if src[i].startswith("#[cfg(test)]") and i + 1 < len(src) and src[i + 1].lstrip().startswith("mod "):
            depth, j = 0, i + 1
            while j < len(src):
                depth += src[j].count("{") - src[j].count("}")
                if depth <= 0 and "{" in "".join(src[i + 1:j + 1]):
                    break
                j += 1
            ranges.append((i + 1, j + 1))
            i = j
        i += 1
    return ranges


def callers(sym):
    """Which frontend crates call it today, grepped: doc comments dropped,
    and a hit inside a `#[cfg(test)] mod` marked as test-only — "the tests
    call it" is not "a frontend calls it"."""
    method = sym.split("::")[-1].split(" / ")[0]
    # Default: a method call on some receiver. A bare `name(` is
    # someone else's free function (the daemon has its own `refused`).
    pattern = PATTERNS.get(sym, r"\." + method + r"\(")
    found = []
    for name, path in FRONTENDS.items():
        r = subprocess.run(
            ["grep", "-rn", "-E", pattern, str(path)],
            capture_output=True, text=True,
        )
        prod, test = False, False
        cache = {}
        for line in r.stdout.splitlines():
            file, lineno, body = line.split(":", 2)
            if "///" in body or "//!" in body:
                continue
            if file not in cache:
                cache[file] = test_ranges(file)
            n = int(lineno)
            if any(lo <= n <= hi for lo, hi in cache[file]):
                test = True
            else:
                prod = True
        if prod:
            found.append(name)
        elif test:
            found.append(name + " (tests only)")
    return found


def main():
    check_symbols()
    rows = []
    for sym, file, returns, key, cost, verbs in ENTRIES:
        grepped = callers(sym)
        rows.append((sym, file, returns, key, cost, verbs, grepped))

    lines = [
        "# generated by scripts/read-surface.py from "
        "crates/acquisition-store/src/{lib.rs,snapshot.rs,annotations.rs,index.rs,world.rs},",
        "# the frontend crates (callers grepped) and data/query-plans.txt (the cost column).",
        "",
        "| Read | Returns | Keyed on | Cost | Called by today | crates (grepped) |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for sym, file, returns, key, cost, verbs, grepped in rows:
        crates = ", ".join(grepped) if grepped else "**none**"
        lines.append(
            f"| `{sym}`<br>`{file}` | {returns} | {key} | {cost} | {verbs} | {crates} |"
        )
    lines += ["", "Deliberately not in the table (not a search consumer's reads):", ""]
    for sym, why in OUT_OF_SCOPE.items():
        lines.append(f"- `{sym}` — {why}")
    lines.append("")
    OUT.write_text("\n".join(lines))
    print(f"wrote {OUT.relative_to(TRACK)}: {len(rows)} reads")
    for sym, *_rest, grepped in rows:
        if not grepped:
            print(f"  uncalled: {sym}")


if __name__ == "__main__":
    main()
