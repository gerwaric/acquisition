# Synthetic userstore generation

Generates synthetic Acquisition userstore databases — realistic in
shape, RePoE-legal in mod content, identifying of nobody — for scale
testing, search coverage, and demos. Principle: **commit generators,
not data**; datasets are produced locally from RePoE plus a
statistics-only profile.

Design, rationale, and limits: `docs/redesign/topics/synthetic-data.md`.

```sh
# Extract a statistics-only profile from a local database:
python3 profile.py path/to/userstore-ACCOUNT.db my-profile.json

# Generate (deterministic per seed + profile + RePoE version):
python3 generate.py --profile fresh-account.json --items 100000 \
    --seed 42 --coverage --sqlite "out/userstore-SYNTH#0000.db"

# Validate through the app's own datastore layer:
ACQ_SYNTH_DATA_DIR=out ACQ_SYNTH_ACCOUNT='SYNTH#0000' \
    ./build/tests/tst_synthdata
```
