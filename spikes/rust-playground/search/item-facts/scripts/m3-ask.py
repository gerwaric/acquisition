#!/usr/bin/env python3
"""M3 and M4 of search/BUILD-PLAN.md ("Measurements the build owes"): a CLI
ask, process start to exit, `--json` to /dev/null, over a sqlite `.backup`
copy of a facts file, for every acceptance query built so far, and `~` over
all displayed text.

    cargo build --workspace && cargo build --release -p acquisition-cli \
        && cargo build -p acquisition-search --example copy-as-store
    python3 search/item-facts/scripts/m3-ask.py search/item-facts/raw/<copy>.db

The copy is written anew under raw/m3/ and wrapped as a store directory by
the `copy-as-store` example, so the owner's store is never opened (the
plan's rule 7). First: the first ask after the copy is written, as seen —
this machine's file cache is not controlled. Warm: the median of ten
consecutive asks. Release is judged against 500 ms; debug is what the seat
feels. Never in the gate: the input is raw/.
"""
import os, statistics, subprocess, sys, time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
LIFE = '"# to maximum Life"'
OQ1 = 'base:ring rarity=rare (line(template:resistance) or line(template:strength))'
ASKS = [
    ("the empty query", [""]),
    ("OQ1", [OQ1]),
    ("OQ1 refined", ['base:ring rarity=rare line(template:resistance is:fractured)']),
    ("OQ2", ['name="Ashes of the Stars"']),
    ("OQ3 a mod", ['line(template:explode)']),
    ("OQ3 a base", ['base="Two-Stone Ring"']),
    ("OQ4", ['base:staff "spell skill"']),
    ("OQ7", [LIFE]),
    ("AQ2", [f'({OQ1}) sum(line(template:resistance).arg1)>=60']),
    ("AQ5", [f'{LIFE}>=90', "--sort", f"line({LIFE}).arg1", "--desc", "--limit", "10"]),
    ("M4 a phrase", ['"explode"']),
    ("M4 ~ over all displayed text", ['text~"explo(de|sion)s?"']),
    ("M4 ~ anchored, a class", ['text~"^adds [0-9]+ to [0-9]+"']),
    # step 5: the seat's counts, the crossed table, the sum, the vocabulary
    ("AQ1 --count tab,league,rarity", ["", "--count", "tab,league,rarity"]),
    ("OQ7 --count tab", [LIFE, "--count", "tab"]),
    ("--cross league,tab", ["rarity=unique", "--cross", "league,tab"]),
    ("--count base --sum stack", ["frame=currency", "--count", "base", "--sum", "stack"]),
    ("the vocabulary, twice narrowed", ["rarity=rare base:ring", "--count", "line:resist,life"]),
    ("the vocabulary whole", ["", "--count", "line"]),
    # step 6: the class table
    ("OQ1 as worded (class:ring)", ['class:ring rarity=rare (line(template:resistance) or line(template:strength))']),
    ("OQ5 by class and level", ['(class:boots or class:gloves or class:helmet) (reqlevel=..30 or -has:reqlevel)']),
    ("--count class", ["", "--count", "class"]),
    # step 7: the computed values
    ("AQ2 as worded (pseudo.total_res)", ['(class:ring rarity=rare (line(template:resistance) or line(template:strength))) pseudo.total_res>=60']),
    ("the worked example whole", ['league=Standard class=Rings rarity=rare "# to maximum Life">=90 pseudo.total_res>=60']),
    ("pseudo.dps sorted", ["pseudo.dps>=100", "--sort", "pseudo.dps", "--desc", "--limit", "10"]),
    ("--count class --sum pseudo.total_res", ["rarity=rare", "--count", "class", "--sum", "pseudo.total_res"]),
    # step 8: the sockets
    ("OQ3 socket colours", ["sockets.red>=2"]),
    ("OQ3 within one link group", ["linked(red>=3 green>=1)"]),
    ("links sorted", ["links>=5", "--sort", "links", "--desc", "--limit", "10"]),
    ("--count links", ["", "--count", "links"]),
    # step 9: the effective price, joined from the listing state
    ("has:priced", ["has:priced"]),
    ("OQ6 a unique by name, priced", ['name="Ashes of the Stars" has:priced']),
    ("--count price.currency --sum price.amount", ["has:priced", "--count", "price.currency", "--sum", "price.amount"]),
    ("price.amount sorted", ["price.amount>=1", "--sort", "price.amount", "--desc", "--limit", "10"]),
    # the first seat (2026-09-26, V9): the six cheap asks it saw at 695-715 ms,
    # as journaled (runs/seat-2026-09-26/asks/NNN.txt), ask 3 the empty query
    ("seat 3: the empty query", [""]),
    ("seat 24: frame=quest --count base,container", ["frame=quest", "--count", "base,container", "--limit", "10"]),
    ("seat 49: class=rings --limit 0", ["class=rings", "--limit", "0"]),
    ("seat 53: OQ2 --count line:reservation,attributes", ['name="Ashes of the Stars"', "--count", "line:reservation,attributes"]),
    ("seat 60: --count line~^adds # to # cold", ["", "--count", "line~^adds # to # cold", "--limit", "20"]),
    ("seat 62: a bare phrase, \"Kaom\"", ['"Kaom"', "--limit", "2"]),
    # step 9c2: the site's other totals, and a total below nothing
    ("pseudo.total_mana", ["pseudo.total_mana>=100"]),
    ("a total below nothing", ["pseudo.total_attack_speed<0"]),
    ("the type's damage with attack skills", ["pseudo.increased_lightning_attack_damage>=30"]),
    # step 9c3: the four that are a reading of other totals, each costing the rows of the totals it reads
    ("pseudo.count_res", ["pseudo.count_res>=3"]),
    ("pseudo.count_ele_res", ["pseudo.count_ele_res=3"]),
    ("pseudo.total_all_ele_res", ["pseudo.total_all_ele_res>=30"]),
    ("pseudo.total_all_attributes", ["pseudo.total_all_attributes>=10"]),
    # step 9c4: a mod displayed over several rows, named by a row
    ("a mod named by a row", ['"#% to all Elemental Resistances"<0']),
    ("Thread of Hope's counts", ['name="Thread of Hope" pseudo.count_res=3 pseudo.count_ele_res=3']),
]

def main():
    source = sys.argv[1]
    realm = sys.argv[2] if len(sys.argv) > 2 else "pc"
    work = os.path.join(os.path.dirname(os.path.abspath(source)), "m3")
    subprocess.run(["rm", "-rf", work], check=True)
    os.makedirs(work)
    copy = os.path.join(work, "copy.db")
    subprocess.run(["sqlite3", source, f".backup '{copy}'"], check=True)
    subprocess.run([os.path.join(ROOT, "target/debug/examples/copy-as-store"), copy, os.path.join(work, "store")],
                   check=True, stdout=subprocess.DEVNULL)
    env = {k: v for k, v in os.environ.items() if k not in ("ACQ_ACCOUNT", "ACQ_GGG")}
    env.update(ACQ_PROVIDER="mock", ACQ_NO_KEYRING="1", ACQ_NO_SPAWN="1", ACQ_STORE_DIR=os.path.join(work, "store"))

    def ask(profile, args):
        started = time.perf_counter()
        done = subprocess.run([os.path.join(ROOT, "target", profile, "acq"), "--json", "search", "--realm", realm, *args],
                              env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        elapsed = (time.perf_counter() - started) * 1000
        if done.returncode != 0:
            sys.exit(f"{args}: {done.stderr.decode()}")
        return elapsed

    print(f"first ask after the copy was written, release, the empty query: {ask('release', ['']):.0f} ms")
    print(f"{'ask':<32}{'release':>10}{'debug':>10}   (ms, the median of ten)")
    for name, args in ASKS:
        medians = [statistics.median(ask(profile, args) for _ in range(10)) for profile in ("release", "debug")]
        print(f"{name:<32}{medians[0]:>10.0f}{medians[1]:>10.0f}")

main()
