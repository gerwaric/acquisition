# Phase-0 QCoro spike

Running-code evidence for the network-redesign spec's phasing step 0
(`docs/design/network-redesign.md`). Findings are recorded as review
rounds **S1** (semantics, `main.cpp`) and **S2** (batch-scale
measurement, `batch.cpp`) in `docs/design/network-redesign-reviews.md`;
the spec's D2/D6/shutdown/dependency sections cite them inline.

Standalone throwaway project — not part of the acquisition build.

```sh
cmake -S spikes/qcoro -B build-spike -DCMAKE_PREFIX_PATH=$HOME/Qt/6.11.1/macos
cmake --build build-spike
./build-spike/qcoro_spike            # exit code = number of failed CHECKs
```

## Batch-scale measurement (`qcoro_batch`)

Second binary, same project: peak memory and abort cost for the full
promise/future/frame/token population at the 2,000-tab scale (S2). It
is a separate executable so it gets a fresh process — clean memory
baselines, away from `qcoro_spike`'s deliberate leaks. Build it
Release for quotable numbers:

```sh
cmake -S spikes/qcoro -B build-spike-release -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_PREFIX_PATH=$HOME/Qt/6.11.1/macos
cmake --build build-spike-release --target qcoro_batch
./build-spike-release/qcoro_batch            # default scales: 200 (warm-up), 2000
./build-spike-release/qcoro_batch 2000 10000 # explicit scales
```

Headline numbers (Release, Qt 6.11.1/macOS): ~5.8 KB heap per entry,
linear through 10,000 (≈11 MB standing population at 2,000 tabs);
full-abort cost ~2–4 ms at 2,000 (Canceled burst + queued
resumptions + sweep — the ~22 ms per-pump sleep wake from S1-5
dominates); heap returns to baseline afterward and `leaks --atExit`
reports zero leaks.

Optional verification passes:

```sh
cmake -S spikes/qcoro -B build-spike-asan -DSPIKE_ASAN=ON \
      -DCMAKE_PREFIX_PATH=$HOME/Qt/6.11.1/macos
cmake --build build-spike-asan && ./build-spike-asan/qcoro_spike
./build-spike-asan/qcoro_batch                # S2 validity checks under ASAN
leaks --atExit -- ./build-spike/qcoro_spike   # macOS leak accounting
leaks --atExit -- ./build-spike-release/qcoro_batch  # expects ZERO leaks
```

The binary prints CHECK lines (internal validity of each experiment)
and FINDING lines (the observed QCoro v0.13.0 semantics). Seven
deliberate coroutine-frame leaks remain at exit — five top-level
task frames (E6a + four E8 tasks) plus two inner task frames
(`QCoro::sleepFor`'s and `qCoro().takeResult()`'s) — demonstrating
S1-1/S1-2's detach behavior; each frame transitively retains its
awaiter state (`QFutureWatcher`, context QObjects, `QFuture` handles
keeping promise shared state alive, sleep timers). The in-process
sentinel checks are the authoritative leak accounting; what `leaks`
roots varies by run: the reply frame is a stable root, the timer
frames surface as root cycles, and the future frames usually stay
hidden behind Qt thread-data reachability (pending cancel call-out
events). Treat the tool as a cross-check on the rooted subset, not
as the count.

Headline result (S1-1): destroying a `QCoro::Task` handle while the
coroutine is suspended does **not** destroy the frame — it detaches
it. Shutdown safety comes from queued awaiter delivery plus the dead
event loop, not from frame destruction.
