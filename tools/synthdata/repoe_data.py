"""Fetch and cache RePoE data files.

Uses the same source the application does (src/repoe/repoe.cpp:
https://repoe-fork.github.io), plus mods.min.json, which the app does not
need but the generator does for legal mod tiers and value ranges. Files are
cached under tools/synthdata/.cache/<version>/ keyed by the upstream
version.txt so a game update invalidates the cache naturally.
"""

import json
import pathlib
import urllib.request

BASE_URL = "https://repoe-fork.github.io"
FILES = [
    "base_items.min.json",
    "item_classes.min.json",
    "mods.min.json",
    "stat_translations.min.json",
]

CACHE_ROOT = pathlib.Path(__file__).resolve().parent / ".cache"


def _fetch(path):
    with urllib.request.urlopen(f"{BASE_URL}/{path}", timeout=60) as r:
        return r.read()


def load():
    """Return {filename-stem: parsed json} for all FILES, from cache or web."""
    version = _fetch("version.txt").decode().strip()
    cache = CACHE_ROOT / version
    cache.mkdir(parents=True, exist_ok=True)
    data = {}
    for name in FILES:
        f = cache / name
        if not f.exists():
            f.write_bytes(_fetch(name))
        stem = name.split(".")[0]
        data[stem] = json.loads(f.read_text())
    data["version"] = version
    return data


if __name__ == "__main__":
    d = load()
    print("version:", d["version"])
    for k in ("base_items", "item_classes", "mods", "stat_translations"):
        print(f"{k}: {len(d[k])} entries")
