#!/usr/bin/env python3
"""Write the pages a sitting is run from: search/pseudo-stats/raw/sitting/<round>-<n>.html.

A page is a batch of the sheet's links, each a plain link with one named
target, so that every search lands in the one tab whose network panel is
recording. The owner clicks each link: a page opens nothing by itself, and
its script marks what was clicked and counts the seconds since, no more. No
link is composed here — each is copied from search/pseudo-stats/data/search-sheet.csv
once it is seen to decode to its row's query — and nothing here touches the
network (C79; SURFACES.md, the trade site's rows: access method `browser`).

    tools/trade-pages.py h             # round h's rows with no capture yet, 25 a page
    tools/trade-pages.py h --again     # every row of round h, captured or not
    tools/trade-pages.py h --size 20   # another batch
    tools/trade-pages.py h --check     # round h's pages on disk against the sheet; writes nothing

A round is the letter its rows' names open with (`h` for h001–h053). A row
has a capture when raw/searches/<row>-search.json holds anything, so the
same command after a sitting writes pages for what is still missing. A
round's earlier pages are removed before its pages are written.

**The tab is one the page opened.** A link's named target reaches only a tab
related to the page holding the link; a tab the owner opened by hand cannot
be reached. So a page's first link opens the tab, blank, and the panel is
set up there before the first search.

What is checked, at every write and by `--check`: every link on a page is a
row of the sheet, under its own name, with the sheet's link; the link
decodes to the row's query; every link has the one target; no row is there
twice.
"""

import base64
import csv
import html
import json
import re
import sys
import zlib
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1] / "search" / "pseudo-stats"
SHEET = TRACK / "data" / "search-sheet.csv"
RAW = TRACK / "raw" / "searches"
PAGES = TRACK / "raw" / "sitting"

SIZE = 25
PACE = 10  # seconds a link; the skill's pace, half of what the site allows
TARGET = "sitting"

STYLE = """
body { font: 16px/1.5 system-ui, sans-serif; max-width: 44rem; margin: 1.5rem auto; padding: 0 1rem; }
h1 { font-size: 1.3rem; }
#pace { position: sticky; top: 0; background: #fff; border-bottom: 1px solid #ccc; padding: .5rem 0;
        font-variant-numeric: tabular-nums; }
#pace.wait { color: #a00; }
ol.steps li { margin: .3rem 0; }
ul.searches { list-style: none; padding: 0; display: grid; grid-template-columns: repeat(5, 1fr); gap: .5rem; }
ul.searches a { display: block; padding: .7rem 0; text-align: center; border: 1px solid #888;
                border-radius: .3rem; text-decoration: none; color: #024; }
ul.searches a:visited { color: #888; }
ul.searches li.next a { border: 3px solid #024; font-weight: bold; }
ul.searches li.done a { background: #ddd; color: #666; }
"""

# Marks and counts; opens nothing and stops nothing. The browser follows the
# link the owner clicked.
SCRIPT = """
const PACE = %d;
const pace = document.getElementById('pace');
const items = Array.from(document.querySelectorAll('ul.searches li'));
let last = null;
function next() {
  items.forEach(li => li.classList.remove('next'));
  const li = items.find(li => !li.classList.contains('done'));
  if (li) li.classList.add('next');
}
items.forEach(li => li.querySelector('a').addEventListener('click', () => {
  li.classList.add('done');
  last = Date.now();
  next();
}));
setInterval(() => {
  const left = items.filter(li => !li.classList.contains('done')).length;
  if (last === null) { pace.textContent = left + ' to open'; return; }
  const since = Math.floor((Date.now() - last) / 1000);
  pace.className = since < PACE ? 'wait' : '';
  pace.textContent = (since < PACE ? 'wait ' + (PACE - since) + ' s' : since + ' s since the last link')
    + ' — ' + (left ? left + ' to open' : 'all opened: export the recording');
}, 250);
next();
"""

LINK = re.compile(r'<a class="search" href="([^"]*)" target="([^"]*)" title="[^"]*">([^<]*)</a>')


def decode(link_text):
    token = link_text.rsplit("/", 1)[1]
    raw = base64.urlsafe_b64decode(token + "=" * (-len(token) % 4))
    return json.loads(zlib.decompress(raw, 16 + zlib.MAX_WBITS))


def read_sheet():
    with SHEET.open(newline="") as fh:
        return list(csv.DictReader(fh))


def captured(name):
    path = RAW / f"{name}-search.json"
    return path.exists() and path.stat().st_size > 0


def page(name, rows, number, count, following):
    e = html.escape
    links = "\n".join(
        f'  <li><a class="search" href="{e(r["link"])}" target="{TARGET}" '
        f'title="{e(r["decides"])}">{e(r["search"])}</a></li>'
        for r in rows
    )
    after = (f'open <a href="{e(following)}">the next page</a>' if following
             else "this is the round's last page")
    return f"""<!doctype html>
<meta charset="utf-8">
<title>{e(name)}: {e(rows[0]["search"])}–{e(rows[-1]["search"])}</title>
<style>{STYLE}</style>
<h1>Sitting {e(name)} — page {number} of {count}, {e(rows[0]["search"])} to {e(rows[-1]["search"])}, {len(rows)} searches</h1>
<ol class="steps">
  <li><a href="about:blank" target="{TARGET}">Open the sitting tab</a> if it is not open: signed in,
    its network panel open and keeping its log.</li>
  <li>Click each search, a plain click, one every {PACE} seconds.</li>
  <li>Export the panel as a HAR file with its content into <code>raw/searches/</code>, clear the log,
    and {after}.</li>
</ol>
<p id="pace">{len(rows)} to open</p>
<ul class="searches">
{links}
</ul>
<script>{SCRIPT % PACE}</script>
"""


def check(sheet, name, expected=None):
    """A round's pages on disk against the sheet; `expected` is the rows a write meant to put there."""
    by_name = {r["search"]: r for r in sheet}
    found = []
    bad = 0
    files = sorted(PAGES.glob(f"{name}-*.html"), key=lambda p: int(p.stem.rsplit("-", 1)[1]))
    for path in files:
        text = path.read_text()
        links = LINK.findall(text)
        if len(links) != text.count('class="search"'):
            bad += 1
            print(f"UNREAD  {path.name}: a search link this check cannot read")
        for href, target, search in links:
            href, search = html.unescape(href), html.unescape(search)
            found.append(search)
            row = by_name.get(search)
            if row is None:
                bad += 1
                print(f"NO ROW  {path.name}: {search} is no row of the sheet")
            elif href != row["link"]:
                bad += 1
                print(f"DIFFER  {path.name}: {search} is not the sheet's link")
            elif decode(href) != json.loads(row["query"]):
                bad += 1
                print(f"DIFFER  {path.name}: {search} does not decode to its query")
            if target != TARGET:
                bad += 1
                print(f"TARGET  {path.name}: {search} opens in `{target}`")
    for search in sorted({n for n in found if found.count(n) > 1}):
        bad += 1
        print(f"TWICE   {search}")
    if expected is not None and found != expected:
        bad += 1
        print(f"DIFFER  the pages hold {len(found)} searches, the round asked for {len(expected)}, or in another order")
    print(f"{len(files)} pages, {len(found)} links read against the sheet, {bad} faults")
    return 1 if bad else 0


def main():
    args = sys.argv[1:]
    flags = {a for a in args if a.startswith("--")}
    words = [a for a in args if not a.startswith("--")]
    size = SIZE
    if "--size" in flags:
        at = args.index("--size")
        if at + 1 >= len(args) or not args[at + 1].isdigit() or int(args[at + 1]) < 1:
            print(__doc__)
            return 2
        size = int(args[at + 1])
        words.remove(args[at + 1])
    if len(words) != 1 or not words[0].isalpha() or flags - {"--again", "--size", "--check"}:
        print(__doc__)
        return 2
    name = words[0]
    sheet = read_sheet()
    if "--check" in flags:
        return check(sheet, name)

    of_round = [r for r in sheet if re.fullmatch(re.escape(name) + r"\d+", r["search"])]
    if not of_round:
        print(f"no row of the sheet is named {name}<number>")
        return 1
    rows = of_round if "--again" in flags else [r for r in of_round if not captured(r["search"])]
    print(f"round {name}: {len(of_round)} rows in the sheet, {len(of_round) - len(rows)} left out as captured")
    PAGES.mkdir(parents=True, exist_ok=True)
    for old in PAGES.glob(f"{name}-*.html"):
        old.unlink()
    if not rows:
        print("nothing to write: every row has a capture (say --again)")
        return 0
    batches = [rows[i:i + size] for i in range(0, len(rows), size)]
    for number, batch in enumerate(batches, start=1):
        following = f"{name}-{number + 1}.html" if number < len(batches) else None
        path = PAGES / f"{name}-{number}.html"
        path.write_text(page(name, batch, number, len(batches), following))
        print(f"{path.relative_to(TRACK.parent.parent)}  {batch[0]['search']}–{batch[-1]['search']}  {len(batch)} searches")
    return check(sheet, name, [r["search"] for r in rows])


if __name__ == "__main__":
    sys.exit(main())
