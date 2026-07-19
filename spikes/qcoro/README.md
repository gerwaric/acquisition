# Phase-0 QCoro spike

Running-code evidence for the network-redesign spec's phasing step 0
(`docs/design/network-redesign.md`). Findings are recorded as review
round **S1** in `docs/design/network-redesign-reviews.md`; the spec's
D2/D6/shutdown/dependency sections cite them inline.

Standalone throwaway project — not part of the acquisition build.

```sh
cmake -S spikes/qcoro -B build-spike -DCMAKE_PREFIX_PATH=$HOME/Qt/6.11.1/macos
cmake --build build-spike
./build-spike/qcoro_spike            # exit code = number of failed CHECKs
```

Optional verification passes:

```sh
cmake -S spikes/qcoro -B build-spike-asan -DSPIKE_ASAN=ON \
      -DCMAKE_PREFIX_PATH=$HOME/Qt/6.11.1/macos
cmake --build build-spike-asan && ./build-spike-asan/qcoro_spike
leaks --atExit -- ./build-spike/qcoro_spike   # macOS leak accounting
```

The binary prints CHECK lines (internal validity of each experiment)
and FINDING lines (the observed QCoro v0.13.0 semantics). Four frame
leaks at exit are deliberate — they demonstrate S1-1/S1-2's
detached-frame behavior.

Headline result (S1-1): destroying a `QCoro::Task` handle while the
coroutine is suspended does **not** destroy the frame — it detaches
it. Shutdown safety comes from queued awaiter delivery plus the dead
event loop, not from frame destruction.
