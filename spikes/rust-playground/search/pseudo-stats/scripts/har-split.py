#!/usr/bin/env python3
"""Split one browser recording of a sitting into its captures, each named by
the search it answers.

The owner opens the sheet's links with the browser's network panel recording,
then exports the panel once as a HAR file. This script reads that file and
writes ../raw/searches/<search>-search.json and <search>-fetch.json (further
pages -fetch-2.json, -3), as saving each response by hand would have. No name
is typed: a search response carries its query in its id, and the sheet says
which row that query is; a fetch belongs to the search whose result list holds
its items. Nothing here touches the network (C79; SURFACES.md: a person opens
each link).

    har-split.py <file.har>            # say what the recording holds, write nothing
    har-split.py <file.har> --write    # write the captures
    har-split.py <file.har> --write --replace   # and overwrite a capture that differs

**A HAR file can hold the session's cookie.** This script reads response
bodies and the site's rate-limit headers and nothing else, copies no request
and no other header, and says whether the file holds a cookie at all. Export
the recording sanitized where the browser offers it, keep it under raw/ (never
committed), and delete it once the captures are written.

What it refuses: a search response whose query is no row of the sheet (named,
not written); a row answered twice with different results (the later one kept
only under --replace); a capture already on disk with other bytes.
"""

import base64
import csv
import gzip
import json
import sys
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
RAW = TRACK / "raw" / "searches"
SHEET = TRACK / "data" / "search-sheet.csv"


def decode(search_id):
    padded = search_id + "=" * (-len(search_id) % 4)
    return json.loads(gzip.decompress(base64.urlsafe_b64decode(padded)))


def bare(node):
    """A query with every `"disabled": false` removed (the site drops them)."""
    if isinstance(node, dict):
        return {k: bare(v) for k, v in node.items() if not (k == "disabled" and v is False)}
    if isinstance(node, list):
        return [bare(v) for v in node]
    return node


def body(entry):
    content = entry["response"].get("content", {})
    text = content.get("text")
    if text is None:
        return None
    if content.get("encoding") == "base64":
        return base64.b64decode(text)
    return text.encode()


def limits(entry):
    return {
        h["name"].lower(): h["value"]
        for h in entry["response"].get("headers", [])
        if h["name"].lower().startswith("x-rate-limit") or h["name"].lower() == "retry-after"
    }


def holds_cookie(har):
    for entry in har["log"]["entries"]:
        for side in ("request", "response"):
            if entry[side].get("cookies"):
                return True
            for h in entry[side].get("headers", []):
                if h["name"].lower() in ("cookie", "set-cookie") and h["value"]:
                    return True
    return False


def read(har, sheet):
    """The recording's searches in order, each with its fetches."""
    found = []
    strays = []
    problems = []
    for entry in har["log"]["entries"]:
        url = entry["request"]["url"]
        if "/api/trade/search/" not in url and "/api/trade/fetch/" not in url:
            continue
        status = entry["response"]["status"]
        raw = body(entry)
        if status != 200 or raw is None:
            problems.append(f"{entry.get('startedDateTime')}  status {status}  {url.split('?')[0][:80]}"
                            + ("" if raw is not None else "  (no body recorded)"))
            continue
        answer = json.loads(raw)
        record = {"at": entry.get("startedDateTime"), "bytes": raw, "limits": limits(entry)}
        if "/api/trade/search/" in url:
            query = decode(answer["id"])
            names = [n for n, q in sheet.items() if bare(q) == bare(query)]
            record.update(name=names[0] if names else None, ids=set(answer.get("result", [])),
                          total=answer.get("total"), fetches=[])
            found.append(record)
        else:
            items = {r["id"] for r in answer.get("result", []) if r}
            owner = next((s for s in reversed(found) if items and items <= s["ids"]), None)
            (owner["fetches"] if owner else strays).append(record)
    return found, strays, problems


def main():
    args = sys.argv[1:]
    flags = {a for a in args if a.startswith("--")}
    files = [a for a in args if not a.startswith("--")]
    if len(files) != 1 or flags - {"--write", "--replace"}:
        print(__doc__)
        return 2
    har = json.loads(Path(files[0]).read_text())
    with SHEET.open(newline="\n") as f:
        sheet = {r["search"]: json.loads(r["query"]) for r in csv.DictReader(f)}
    found, strays, problems = read(har, sheet)

    print("the recording holds a cookie: " + (
        "YES — keep it under raw/ and delete it once the captures are written"
        if holds_cookie(har) else "no"))
    writes = {}
    refused = 0
    for s in found:
        if s["name"] is None:
            refused += 1
            print(f"REFUSED  {s['at']}  a search whose query is no row of the sheet (total {s['total']})")
            continue
        files_of = {f"{s['name']}-search.json": s["bytes"]}
        for page, fetch in enumerate(s["fetches"], start=1):
            files_of[f"{s['name']}-fetch.json" if page == 1 else f"{s['name']}-fetch-{page}.json"] = fetch["bytes"]
        if f"{s['name']}-search.json" in writes:
            same = writes[f"{s['name']}-search.json"] == s["bytes"]
            if same or "--replace" not in flags:
                refused += 0 if same else 1
                print(f"{'again  ' if same else 'REFUSED'}  {s['name']}  answered twice"
                      + ("" if same else ", with other results; the first is kept"))
                continue
        writes.update(files_of)
        print(f"{s['name']:<8} total {s['total']}; {len(s['fetches'])} fetch"
              + ("" if len(s["fetches"]) == 1 else "es")
              + ("" if s["fetches"] or not s["total"] else "  — results and no fetch recorded"))
    for stray in strays:
        print(f"REFUSED  {stray['at']}  a fetch belonging to no search in the recording")
    for p in problems:
        print(f"PROBLEM  {p}")
    missing = [n for n in sheet if f"{n}-search.json" not in writes and not (RAW / f"{n}-search.json").exists()]
    print(f"{len(found)} searches in the recording; rows of the sheet with no capture anywhere: "
          + (", ".join(missing) if missing else "none"))

    seen = [s["limits"] for s in found if s["limits"]] + [
        f["limits"] for s in found for f in s["fetches"] if f["limits"]]
    if seen:
        print("the site's rate-limit headers, first and last response carrying them:")
        for which in (seen[0], seen[-1]):
            print("    " + "; ".join(f"{k}={v}" for k, v in sorted(which.items())))
    if any("retry-after" in l for l in seen) or any("429" in p for p in problems):
        print("THE SITE ASKED THE SITTING TO SLOW DOWN: see the problems above")

    if "--write" not in flags:
        print("nothing written (say --write)")
        return 1 if refused or strays or problems else 0
    RAW.mkdir(parents=True, exist_ok=True)
    written = 0
    for name, data in writes.items():
        path = RAW / name
        if path.exists() and path.stat().st_size and path.read_bytes() != data:
            if "--replace" not in flags:
                print(f"KEPT     {name}: a capture with other bytes is on disk (say --replace)")
                continue
        if not path.exists() or path.read_bytes() != data:
            path.write_bytes(data)
            written += 1
    print(f"{written} files written under {RAW.relative_to(TRACK.parent.parent)}")
    return 1 if refused or strays or problems else 0


if __name__ == "__main__":
    sys.exit(main())
