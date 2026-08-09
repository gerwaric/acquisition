#!/usr/bin/env python3
"""CTest entry point: generate a fixture-lane store and prove determinism.

Generates the same (profile, seed, RePoE fixtures) twice into --out-dir,
compares the two databases row-for-row and the two manifests byte-for-byte,
and leaves the result as userstore-<account>.db (+ .manifest.json) for
tst_synthdata to validate through the application's repo layer. Runs
offline from the checked-in fixtures/ subset; no network.

Usage: selftest.py --out-dir <build dir> [--account 'SYNTH#0000']
"""

import argparse
import json
import pathlib
import sqlite3
import sys

import generate

TABLES = ["stashes", "characters", "item_buyouts", "location_buyouts"]


def dump(path):
    db = sqlite3.connect(path)
    out = {t: sorted(map(repr, db.execute(f"SELECT * FROM {t}")))
           for t in TABLES}
    out["user_version"] = db.execute("PRAGMA user_version").fetchone()[0]
    db.close()
    return out


def main():
    here = pathlib.Path(__file__).resolve().parent
    ap = argparse.ArgumentParser()
    ap.add_argument("--out-dir", required=True)
    ap.add_argument("--account", default="SYNTH#0000")
    ap.add_argument("--items", type=int, default=2000)
    ap.add_argument("--seed", type=int, default=42)
    args = ap.parse_args()

    out = pathlib.Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    final = out / f"userstore-{args.account}.db"
    second = out / "selftest-repeat.db"
    gen_args = ["--profile", str(here / "fresh-account.json"),
                "--items", str(args.items), "--seed", str(args.seed),
                "--coverage", "--repoe-dir", str(here / "fixtures")]
    generate.main(gen_args + ["--sqlite", str(final)])
    generate.main(gen_args + ["--sqlite", str(second)])

    if dump(final) != dump(second):
        sys.exit("DETERMINISM FAILURE: two runs with identical inputs"
                 " produced different databases")
    m1 = (final.parent / (final.name + ".manifest.json")).read_bytes()
    m2 = (second.parent / (second.name + ".manifest.json")).read_bytes()
    if m1 != m2:
        sys.exit("DETERMINISM FAILURE: manifests differ between runs")
    second.unlink()
    (second.parent / (second.name + ".manifest.json")).unlink()

    probes = len(json.loads(m1)["probes"])
    print(f"selftest ok: deterministic; {final} ready with {probes} probes")


if __name__ == "__main__":
    main()
