#!/usr/bin/env python3
"""bothy — a linter and reader for a markdown decision registry.

    python3 -I tools/bothy.py check                 lint the registry; exit 1 on a fault
    python3 -I tools/bothy.py list [decisions|parks] [--area NAME]
    python3 -I tools/bothy.py show C<n>             one entry, what it cites, what cites it, where code holds it
    python3 -I tools/bothy.py pack --area store     what to read before touching an area
    python3 -I tools/bothy.py pack --path crates/acquisition-store/src/lib.rs
    python3 -I tools/bothy.py --root DIR ...        the tree to read (default: the script's parent directory)

The registry is `CONTEXT.md` (invariants, principles, the cross-cutting
rulings and parks, the index of area files) and `decisions/<area>.md`
(one file per area: rulings, then "Parked"). The grammar is the one
`CONTEXT.md`, "Decisions — the registry", states: one bullet per
decision, `- **C<n> — title.** ruling *Why:* ... *Details:* ...
*Pinned:* ... *Evidence:* ...`; one bullet per park, `item → home.
Trigger: ...`. This script reads that grammar and never writes.

`check` holds what `tools/docs-check.sh` does not: the index in
`CONTEXT.md` agrees with the files; a cross-reference inside the
registry resolves; a park has a home and a trigger and is not struck
through; a path the registry names exists; a module doc that names an
area names one that exists. docs-check keeps the byte budgets, the
entry length, the citations from code and the uncited ids. The two
do not overlap; a fault has one reporter.

Faults print as `FAIL  <kind>  <file>:<line>  <what is wrong and what to do>`.
Notes print as `note  ...` and never fail. Standard library only.
`tools/bothy-breakers.sh` proves each refusal on a fixture; run it when
a check is added here.
"""
import argparse
import os
import re
import sys
from collections import defaultdict

DEC_RE = re.compile(r"^- \*\*(C\d+)\s+—\s+(.*?)\*\*\s*(.*)$")
DEC_START = re.compile(r"^- \*\*C\d")
PRIN_RE = re.compile(r"^- \*\*(P\d+)\.\*\*\s*(.*)$")
INV_RE = re.compile(r"^(\d+)\. \*\*(.*?)\*\*")
INDEX_RE = re.compile(r"^\| `decisions/(\w+)\.md` \| (.*?) \| (.*?) \|\s*$")
ID_RE = re.compile(r"\b([CP])(\d+)\b")
MARKER_RE = re.compile(r"\*(Why|Details|Pinned|Evidence):\*")
TRIGGER_RE = re.compile(r"\bTrigger\b[^.:]{0,80}:")
HAS_TRIGGER = re.compile(r"\btrigger\b", re.I)   # a parking-lot bullet writes "…, trigger X; …, trigger Y"
TICK_RE = re.compile(r"`([^`\s]+)`")
# A path the registry names: has a `/`, and its last component has a file
# extension or its first component is a directory of the tree. Not an API
# route (`/profile`), not a key shape (`wanted/lot`), not a note cited by
# number (`brainstorming-notes/10`, which docs-check resolves), and not
# history (`search/BUILD-PLAN.md` at `453a4429`).
PATH_EXT = re.compile(r"\.(rs|md|sh|py|sql|json|toml|csv|txt)$")
NOTE_CITE = re.compile(r"^brainstorming-notes[/ ]\d+$")
HISTORY = re.compile(r"` at `[0-9a-f]{7,}`")
MODDOC_AREA = re.compile(r"decisions/(\w+)\.md")
CODE_EXT = (".rs", ".sh", ".py", ".sql", ".toml")


class Registry:
    def __init__(self, root):
        self.root = root
        self.faults = []      # (kind, file, line, message)
        self.notes = []
        self.decisions = {}   # id -> dict
        self.order = []       # ids in file order
        self.parks = []       # dict
        self.principles = {}  # P<n> -> text
        self.invariants = []  # (n, text)
        self.index = {}       # area -> (line, desc, [ids])
        self.areas = {}       # area -> {file, title, intro, sections: [(heading, line)], park_heading}
        self.orientation = ""
        self.cross_heading = ""
        self.cross_parks_heading = ""
        self.cross_park_intro = ""
        self._read_context()
        for name in sorted(os.listdir(self.path("decisions"))) if os.path.isdir(self.path("decisions")) else []:
            if name.endswith(".md"):
                self._read_area(name[:-3])
        self.code = None

    def path(self, rel):
        return os.path.join(self.root, rel)

    def fail(self, kind, file, line, msg):
        self.faults.append((kind, file, line, msg))

    # ---- reading ---------------------------------------------------------
    def _lines(self, rel):
        with open(self.path(rel), encoding="utf-8") as fh:
            return [l.rstrip("\n") for l in fh]

    def _read_context(self):
        rel = "CONTEXT.md"
        if not os.path.exists(self.path(rel)):
            self.fail("missing", rel, 0, "no CONTEXT.md: the registry starts there")
            return
        section, sub = "", ""
        in_orient = in_parked = False
        orient = []
        for n, line in enumerate(self._lines(rel), 1):
            if line.startswith("## "):
                section, sub = line[3:], ""
                in_orient = section.startswith("Orientation")
                in_parked = section.startswith("Parked")
                if in_parked:
                    self.cross_parks_heading = line
                continue
            if line.startswith("### "):
                sub = line[4:]
                if sub.startswith("Cross-cutting"):
                    self.cross_heading = line
                continue
            if in_orient and line.strip():
                orient.append(line)
            m = INDEX_RE.match(line)
            if m:
                ids = re.findall(r"C\d+", m.group(3))
                self.index[m.group(1)] = (n, m.group(2), ids)
                continue
            m = PRIN_RE.match(line)
            if m:
                self.principles[m.group(1)] = m.group(2)
                continue
            if section.startswith("Invariants"):
                m = INV_RE.match(line)
                if m:
                    self.invariants.append((int(m.group(1)), line))
            if in_parked and not line.startswith("- ") and line.strip() and not self.cross_park_intro:
                self.cross_park_intro = line
            self._entry(rel, "cross-cutting", n, line, section, in_parked)
        self.orientation = "\n".join(orient)

    def _read_area(self, area):
        rel = f"decisions/{area}.md"
        lines = self._lines(rel)
        meta = {"file": rel, "title": "", "intro": "", "sections": [], "park_heading": "", "park_intro": ""}
        self.areas[area] = meta
        section = ""
        in_parked = False
        for n, line in enumerate(lines, 1):
            if line.startswith("# ") and not meta["title"]:
                meta["title"] = line[2:]
                continue
            if line.startswith("## "):
                section = line[3:]
                in_parked = section.startswith("Parked")
                if in_parked:
                    meta["park_heading"] = line
                else:
                    meta["sections"].append((line, n))
                continue
            if not section and line.strip() and not meta["intro"]:
                meta["intro"] = line
            if in_parked and not line.startswith("- ") and line.strip() and not meta["park_intro"]:
                meta["park_intro"] = line
            self._entry(rel, area, n, line, section, in_parked)

    def _entry(self, rel, area, n, line, section, in_parked):
        if in_parked and line.startswith("- "):
            body = line[2:]
            park = {"file": rel, "line": n, "area": area, "text": line, "body": body,
                    "item": None, "home": None, "trigger": None,
                    "cites": sorted({a + b for a, b in ID_RE.findall(body)})}
            if "→" in body:
                item, rest = body.split("→", 1)
                park["item"] = item.strip()
                tm = TRIGGER_RE.search(rest)
                if tm:
                    park["home"] = rest[: tm.start()].strip(" .;")
                    park["trigger"] = rest[tm.end():].strip()
                elif HAS_TRIGGER.search(rest):
                    park["home"], park["trigger"] = rest.strip(), "(stated per item)"
                else:
                    park["home"] = rest.strip()
            self.parks.append(park)
            return
        if in_parked:
            return
        m = DEC_RE.match(line)
        if DEC_START.match(line) and not m:
            self.fail("grammar", rel, n,
                      "starts like a decision but is not `- **C<n> — title.** ruling ...`; "
                      "fix the bullet or move the text out of the registry")
            return
        if not m:
            return
        cid, title, body = m.group(1), m.group(2), m.group(3)
        cites = sorted({a + b for a, b in ID_RE.findall(body)} - {cid})
        entry = {"id": cid, "file": rel, "line": n, "area": area, "section": section,
                 "title": title, "body": body, "text": line, "cites": cites,
                 "markers": [mk for mk in MARKER_RE.findall(body)],
                 "dated": bool(re.search(r"\d{4}-\d{2}-\d{2}", line))}
        if cid in self.decisions:
            first = self.decisions[cid]
            self.fail("duplicate", rel, n,
                      f"{cid} is also at {first['file']}:{first['line']}; an id is never reused, "
                      "so one of these takes the next free id")
            return
        self.decisions[cid] = entry
        self.order.append(cid)

    # ---- the code --------------------------------------------------------
    def read_code(self):
        """Where each id is named in the code and tools, and which module docs
        name which area. Walks `crates/` (not target) and `tools/`."""
        if self.code is not None:
            return self.code
        held = defaultdict(list)      # id -> [(file, line)]
        moddoc = defaultdict(list)    # area -> [file]
        files = []                    # every file under crates/ and tools/
        for top in ("crates", "tools"):
            for dirpath, dirs, names in os.walk(self.path(top)):
                dirs[:] = [d for d in dirs if d not in ("target", "__pycache__")]
                for name in names:
                    full = os.path.join(dirpath, name)
                    rel = os.path.relpath(full, self.root)
                    files.append(rel)
                    if not name.endswith(CODE_EXT):
                        continue
                    try:
                        with open(full, encoding="utf-8", errors="replace") as fh:
                            for n, line in enumerate(fh, 1):
                                for cid in re.findall(r"\bC\d+\b", line):
                                    held[cid].append((rel, n))
                                if n <= 200 and name.endswith(".rs"):
                                    for area in MODDOC_AREA.findall(line):
                                        if rel not in moddoc[area]:
                                            moddoc[area].append(rel)
                    except OSError:
                        pass
        self.code = {"held": held, "moddoc": moddoc, "files": files,
                     "by_base": defaultdict(list)}
        for rel in files:
            self.code["by_base"][os.path.basename(rel)].append(rel)
        return self.code

    def git_root(self):
        d = self.root
        while d != os.path.dirname(d):
            if os.path.exists(os.path.join(d, ".git")):
                return d
            d = os.path.dirname(d)
        return self.root

    def is_path(self, tok):
        """Whether a backticked token is a path this check should resolve."""
        if "/" not in tok or tok.startswith(("/", "http", "~", "-", "../")) or re.search(r"[<>*{}$]", tok):
            return False
        if NOTE_CITE.match(tok):
            return False
        tok = tok.split("#")[0].split(":")[0]
        first, last = tok.split("/")[0], tok.rsplit("/", 1)[-1]
        return bool(PATH_EXT.search(last)) or os.path.isdir(self.path(first)) or os.path.isdir(os.path.join(self.git_root(), first))

    def resolve_path(self, tok):
        """A path the registry names, as the registry writes it: relative to
        the tree, to `crates/`, to one crate, or to the repository root."""
        tok = tok.split("#")[0].split(":")[0]
        bases = [self.root, self.path("crates"), self.git_root()]
        if os.path.isdir(self.path("crates")):   # `tests/wire.rs`, written after the crate's name
            bases += [self.path(os.path.join("crates", c)) for c in os.listdir(self.path("crates"))]
        return any(os.path.exists(os.path.join(base, tok)) for base in bases)

    def area_of_path(self, rel):
        """The area a file belongs to: the module doc names `decisions/<area>.md`
        near the top, else the area whose intro names the file or its crate."""
        full = self.path(rel)
        found = []
        if os.path.exists(full):
            with open(full, encoding="utf-8", errors="replace") as fh:
                for n, line in enumerate(fh, 1):
                    if n > 200:
                        break
                    for area in MODDOC_AREA.findall(line):
                        if area in self.areas and area not in found:
                            found.append(area)
        if found:
            return found
        base = os.path.basename(rel)
        crate = rel.split("/")[1] if rel.startswith("crates/") and rel.count("/") > 1 else None
        for area, meta in self.areas.items():
            toks = TICK_RE.findall(meta["intro"])
            if any(t.endswith(base) for t in toks) or (crate and any(t.startswith(crate) for t in toks)):
                found.append(area)
        return found


# ---- check -----------------------------------------------------------------
def check(reg, show_notes=True):
    # the index agrees with the files
    for area, (n, _, ids) in reg.index.items():
        if area not in reg.areas:
            reg.fail("index", "CONTEXT.md", n, f"the index names decisions/{area}.md, which does not exist")
            continue
        in_file = [i for i in reg.order if reg.decisions[i]["area"] == area]
        for i in ids:
            if i not in reg.decisions:
                reg.fail("index", "CONTEXT.md", n, f"the index lists {i} for decisions/{area}.md, and no file holds {i}")
            elif reg.decisions[i]["area"] != area:
                reg.fail("index", "CONTEXT.md", n,
                         f"the index lists {i} under decisions/{area}.md, but it is in {reg.decisions[i]['file']}:{reg.decisions[i]['line']}")
        for i in in_file:
            if i not in ids:
                reg.fail("index", reg.decisions[i]["file"], reg.decisions[i]["line"],
                         f"{i} is not in CONTEXT.md's index row for decisions/{area}.md; add it there")
    for area, meta in reg.areas.items():
        if area not in reg.index:
            reg.fail("index", meta["file"], 1, f"decisions/{area}.md has no row in CONTEXT.md's index table; add one")
    # every cross-reference resolves
    for i in reg.order:
        d = reg.decisions[i]
        for c in d["cites"]:
            if c[0] == "C" and c not in reg.decisions:
                reg.fail("cite", d["file"], d["line"], f"{i} names {c}, which no file defines; fix the id or add the decision")
            if c[0] == "P" and c not in reg.principles:
                reg.fail("cite", d["file"], d["line"], f"{i} names {c}, which CONTEXT.md's principles do not define")
    for p in reg.parks:
        for c in p["cites"]:
            if c[0] == "C" and c not in reg.decisions:
                reg.fail("cite", p["file"], p["line"], f"a park names {c}, which no file defines")
            if c[0] == "P" and c not in reg.principles:
                reg.fail("cite", p["file"], p["line"], f"a park names {c}, which CONTEXT.md's principles do not define")
    # every park is item → home, with a trigger, and not struck through
    for p in reg.parks:
        where = (p["file"], p["line"])
        if p["item"] is None:
            reg.fail("park", *where, "a park is `item → home. Trigger: ...`; this bullet has no `→`")
        elif p["trigger"] is None:
            reg.fail("park", *where, "this park states no trigger; deferral with no trigger is re-argued every time it is read")
        if "~~" in p["body"]:
            reg.fail("park", *where, "a struck-through park is deleted, not kept (CONTEXT.md, \"Parked\"); delete the bullet")
    # every path the registry names exists
    seen = set()
    for rel in ["CONTEXT.md"] + [m["file"] for m in reg.areas.values()]:
        if not os.path.exists(reg.path(rel)):
            continue
        for n, line in enumerate(reg._lines(rel), 1):
            for m in TICK_RE.finditer(line):
                tok = m.group(1)
                if not reg.is_path(tok) or (rel, tok) in seen or HISTORY.match(line, m.end() - 1):
                    continue
                seen.add((rel, tok))
                if not reg.resolve_path(tok):
                    reg.fail("path", rel, n, f"`{tok}` is named here and exists nowhere under the tree, crates/ or the repository; "
                             "point at the file that holds it now, or cite history as `<path>` at `<commit>`")
    # a module doc that names an area names one that exists
    code = reg.read_code()
    for area, files in code["moddoc"].items():
        if area not in reg.areas:
            for f in files:
                reg.fail("routing", f, 1, f"names decisions/{area}.md, which does not exist; name the area's file")
    # notes: soft shape, reported and never failed
    no_why = [i for i in reg.order if "Why" not in reg.decisions[i]["markers"]]
    no_date = [i for i in reg.order if not reg.decisions[i]["dated"]]
    unrouted = [a for a in reg.areas if a not in code["moddoc"]]
    if no_why:
        reg.notes.append(f"{len(no_why)} decisions carry no *Why:* — {', '.join(no_why)}")
    if no_date:
        reg.notes.append(f"{len(no_date)} decisions carry no date — {', '.join(no_date)}")
    if unrouted:
        reg.notes.append("no module doc names " + ", ".join(f"decisions/{a}.md" for a in unrouted)
                         + " — `pack --path` reaches the area only through its intro's file list")
    # report
    for kind, file, line, msg in sorted(reg.faults, key=lambda f: (f[1], f[2], f[0])):
        print(f"FAIL  {kind:<9} {file}:{line}  {msg}")
    if show_notes:
        for note in reg.notes:
            print(f"note  {note}")
    if reg.faults:
        print(f"{len(reg.faults)} fault(s) in the registry")
        return 1
    print(f"ok    registry  {len(reg.decisions)} decisions in {1 + len(reg.areas)} files, {len(reg.parks)} parks, "
          f"{len(reg.principles)} principles, {len(reg.invariants)} invariants: the index matches the files, "
          "every cross-reference resolves, every park has a home and a trigger, every path named exists")
    return 0


# ---- list, show ------------------------------------------------------------
def short(s, n=96):
    return s if len(s) <= n else s[: n - 1] + "…"


def cmd_list(reg, what, area):
    if what in ("decisions", "all"):
        for i in reg.order:
            d = reg.decisions[i]
            if area and d["area"] != area:
                continue
            print(f"{i:<5} {d['area']:<14} {short(d['title'])}")
    if what in ("parks", "all"):
        for p in reg.parks:
            if area and p["area"] != area:
                continue
            item = p["item"] or p["body"]
            print(f"park  {p['area']:<14} {short(item)}  [{p['file']}:{p['line']}]")


def cmd_show(reg, cid):
    code = reg.read_code()
    if cid in reg.decisions:
        d = reg.decisions[cid]
        print(f"{d['file']}:{d['line']}  ({d['area']}; {d['section']})\n\n{d['text']}\n")
        if d["cites"]:
            print("Names: " + ", ".join(
                f"{c} ({short(reg.decisions[c]['title'], 60)})" if c in reg.decisions else
                f"{c} ({short(reg.principles.get(c, '?'), 60)})" for c in d["cites"]))
    elif cid in reg.principles:
        print(f"CONTEXT.md  {cid}. {reg.principles[cid]}\n")
    else:
        print(f"no entry {cid}; `list` shows the ids", file=sys.stderr)
        return 2
    named_by = [i for i in reg.order if cid in reg.decisions[i]["cites"]]
    parks_by = [p for p in reg.parks if cid in p["cites"]]
    if named_by or parks_by:
        print("Named by: " + ", ".join(named_by + [f"a park at {p['file']}:{p['line']}" for p in parks_by]))
    held = code["held"].get(cid, [])
    if held:
        print(f"Held in code ({len(held)} places): " + ", ".join(f"{f}:{n}" for f, n in held[:12]) + (" …" if len(held) > 12 else ""))
    else:
        print("Held in code: nowhere — no test, doc comment or tool names it")
    return 0


# ---- pack ------------------------------------------------------------------
def cmd_pack(reg, area, path):
    code = reg.read_code()
    if path:
        areas = reg.area_of_path(path)
        if not areas:
            print(f"no area governs {path}: no module doc near its top names a decisions/<area>.md and no area's intro names it. "
                  f"Areas: {', '.join(sorted(reg.areas))}. Use --area.", file=sys.stderr)
            return 2
        head = f"# What to read before touching `{path}`"
    else:
        if area not in reg.areas:
            print(f"no area {area}; areas: {', '.join(sorted(reg.areas))}", file=sys.stderr)
            return 2
        areas = [area]
        head = f"# What to read before touching `{area}`"
    out = [head, "", "Every line below is verbatim from `CONTEXT.md` and `decisions/<area>.md`, which are the authority; "
           "this is a view of them, assembled by `tools/bothy.py pack`.", ""]
    if reg.orientation:
        out += ["## Orientation", "", reg.orientation, ""]
    if reg.invariants:
        out += ["## Invariants (never break these)", ""] + [t for _, t in reg.invariants] + [""]
    if reg.principles:
        out += ["## Principles (cited as P<n>)", ""] + [f"- **{k}.** {v}" for k, v in reg.principles.items()] + [""]
    cross = [reg.decisions[i]["text"] for i in reg.order if reg.decisions[i]["area"] == "cross-cutting"]
    if cross:
        out += [reg.cross_heading.replace("###", "##") if reg.cross_heading else "## Cross-cutting rulings", ""] + cross + [""]
    for a in areas:
        meta = reg.areas[a]
        title = re.sub(r"^Decisions\s+—\s+", "", meta["title"])
        out += [f"## Rulings for `{a}` — {title}", "", meta["intro"], ""]
        current = None
        for i in reg.order:
            d = reg.decisions[i]
            if d["area"] != a:
                continue
            if d["section"] != current:
                current = d["section"]
                out += [f"### {current}", ""]
            out.append(d["text"])
        out.append("")
        parks = [p for p in reg.parks if p["area"] == a]
        if parks:
            out += [(meta["park_heading"] or "## Parked").replace("## ", f"## `{a}`: ", 1), ""]
            if meta["park_intro"]:
                out += [meta["park_intro"], ""]
            out += [p["text"] for p in parks] + [""]
    cross_parks = [p["text"] for p in reg.parks if p["area"] == "cross-cutting"]
    if cross_parks:
        out += [reg.cross_parks_heading or "## Parked (cross-cutting)", ""]
        if reg.cross_park_intro:
            out += [reg.cross_park_intro, ""]
        out += cross_parks + [""]
    out += ["## Where each ruling is held (file:line that names it; the mechanism lives there)", ""]
    for a in areas:
        for i in reg.order:
            d = reg.decisions[i]
            if d["area"] != a:
                continue
            held = code["held"].get(i, [])
            firsts, seen = [], set()
            for f, n in held:
                if f not in seen:
                    seen.add(f)
                    firsts.append(f"{f}:{n}")
            out.append(f"- {i}: " + (", ".join(firsts[:6]) + (" …" if len(firsts) > 6 else "") if firsts else "nothing names it yet"))
    out.append("")
    mods = sorted({f for a in areas for f in code["moddoc"].get(a, [])})
    if mods:
        out += ["## Module docs that name this area (read the one you will change)", ""] + [f"- {m}" for m in mods] + [""]
    print("\n".join(out))
    return 0


# ---- main ------------------------------------------------------------------
def main(argv):
    ap = argparse.ArgumentParser(prog="bothy", description=__doc__.split("\n\n")[0], add_help=True)
    ap.add_argument("--root", help="the tree to read (default: the script's parent directory)")
    sub = ap.add_subparsers(dest="verb", required=True)
    c = sub.add_parser("check", help="lint the registry; exit 1 on a fault")
    c.add_argument("--no-notes", action="store_true", help="print faults only")
    l = sub.add_parser("list", help="one line per decision and park")
    l.add_argument("what", nargs="?", default="all", choices=["all", "decisions", "parks"])
    l.add_argument("--area")
    s = sub.add_parser("show", help="one entry, what it names, what names it, where code holds it")
    s.add_argument("id")
    p = sub.add_parser("pack", help="what to read before touching an area or a file")
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument("--area")
    g.add_argument("--path")
    args = ap.parse_args(argv)
    root = args.root or os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
    root = os.path.abspath(root)
    reg = Registry(root)
    if args.verb == "check":
        return check(reg, show_notes=not args.no_notes)
    if reg.faults:
        check(reg, show_notes=False)
        print("fix the registry before reading it", file=sys.stderr)
        return 1
    if args.verb == "list":
        return cmd_list(reg, args.what, args.area) or 0
    if args.verb == "show":
        return cmd_show(reg, args.id)
    if args.verb == "pack":
        return cmd_pack(reg, args.area, args.path)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
