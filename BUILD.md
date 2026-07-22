# Building Acquisition

Acquisition is a C++23 Qt Widgets application. It can be built on Windows,
macOS, and Linux with CMake and Qt 6.

The CMake project currently requires Qt 6.11 or newer with these Qt modules:

- Core
- Gui
- Network
- Network Authorization
- Sql
- Test
- Widgets

## Compiler Requirements

Acquisition uses C++ coroutines (via QCoro), which need more than a compiler
that accepts `-std=c++23`. CMake checks these floors explicitly and fails the
configure step below them:

- GCC 13 or newer (earlier GCC miscompiles coroutines with captures in
  temporary lambdas)
- Clang 15 or newer
- AppleClang 15 or newer (Xcode 15.2)
- MSVC 19.40 or newer (Visual Studio 2022 17.10)

The Clang, AppleClang, and MSVC minimums are the ones QCoro v0.13.0 documents;
the GCC floor is stricter for the miscompilation above.

## Third-Party Dependencies

CMake fetches third-party libraries (sentry-native, glaze, cpp-semver, spdlog,
and QCoro) at configure time via FetchContent; the vendored qdarkstyle lives in
`deps/`. QCoro is pinned exactly at v0.13.0 — a hard floor, not a preference:
the coroutine semantics acquisition relies on are verified at that release (see
`docs/design/network-redesign.md`, "Dependency: QCoro"). Its examples and tests
are kept out of the build with `QCORO_BUILD_EXAMPLES=OFF` and
`QCORO_BUILD_TESTING=OFF`; the global `BUILD_TESTING` flag is never touched, so
acquisition's own test suite stays enabled.

For day-to-day development, Qt Creator Community is the easiest setup on all
supported platforms. Open the repository as a CMake project, configure a Qt 6.11+
desktop kit, and build the `acquisition` target.

## Command Line Build

From the repository root:

```sh
cmake -S . -B build
cmake --build build
```

For a release-style local build:

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --parallel
```

The executable is written to the CMake build directory. You can run it with a
temporary data directory while testing:

```sh
./build/acquisition.app/Contents/MacOS/acquisition --data-dir /tmp/acq-data --log-level debug
```

## Tests

The checked-in tests use Qt Test and are registered with CTest. After building,
run them from the repository root:

```sh
ctest --test-dir build --output-on-failure
```

GUI-dependent tests are configured by CMake to use Qt's offscreen platform.

## Sanitizer Builds (test/CI only)

The `ACQ_SANITIZE` CMake option builds the checked-in test suite with a runtime
sanitizer. It is a test/CI-only knob: it is OFF by default and release packaging
never sets it, so shipped binaries are never sanitized. Use a throwaway build
tree so the normal `build/` results stay untouched, and build only test targets:

```sh
cmake -S . -B build-asan -DACQ_SANITIZE=address -DCMAKE_BUILD_TYPE=RelWithDebInfo
cmake --build build-asan --target tst_workerintegration
ASAN_OPTIONS=abort_on_error=1 ctest --test-dir build-asan \
    -R tst_workerintegration --output-on-failure
```

This gives the phase-5 shutdown pins (`I-SHUT-*`, `I-LEAK-BOUND` in
`tst_workerintegration.cpp`) use-after-free teeth: if any event-loop turn ran
after owner destruction, a detached QCoro frame would resume into a freed
worker/hub and AddressSanitizer would report heap-use-after-free.

Do **not** build or run the `acquisition` app target under a sanitizer. The app
links crashpad, whose signal handlers are installed only by `sentry_init()` in
`src/main.cpp` (never in the test binaries), so a sanitized app can conflict with
the sanitizer's own handlers. The test binaries do not run crashpad and are safe.

On macOS, ASAN performs no leak detection (LSAN is unavailable there), so this
knob covers use-after-free only. An LSAN Linux-CI job for `I-LEAK-BOUND`
leak-boundedness (see `docs/design/network-redesign-phase5-verification.md` §6)
is a separate follow-up.

## Code Scanning

Static analysis is run by `.github/workflows/codeql.yml` using GitHub CodeQL.
The workflow performs a manual Linux CMake build so CodeQL sees the same Qt,
compiler, generated sources, and include paths used by the application build.

The workflow runs on pushes and pull requests to `main`, on a weekly schedule,
and by `workflow_dispatch`.

## Linux

Linux builds require OpenSSL 3.x support at build and runtime. On Ubuntu-like
systems, the GitHub Actions workflow installs these packages:

```sh
sudo apt-get install -y \
  patchelf openssl libfuse2 \
  libcurl4-openssl-dev \
  libgl-dev \
  libssl-dev \
  libvulkan-dev \
  libxcb-cursor0 \
  libxcb-cursor-dev \
  zlib1g-dev
```

Other distributions may need equivalent packages. If Qt cannot find OpenSSL, set
`OPENSSL_ROOT_DIR` or adjust your library path for your local installation.

## Release Packaging

Release artifacts are built by GitHub Actions:

- `.github/workflows/build-linux.yml` builds the Linux AppImage.
- `.github/workflows/build-macos.yml` builds the macOS DMG.
- `.github/workflows/build-windows.yml` builds the Windows installer.

The workflows run on `workflow_dispatch` and on tags matching `v*`. Tag builds
create draft GitHub releases and attach the platform artifacts. They currently
install Qt 6.11.1 with the `qtnetworkauth` module.

## Platform Notes

Windows release packaging uses MSVC 2022, `windeployqt`, the Visual C++
Redistributable, and Inno Setup via `installer.iss`.

macOS release packaging uses `macdeployqt` to produce a DMG. The current workflow
runs on `macos-latest` and names the uploaded artifact as an arm64 DMG.

Linux release packaging uses `linuxdeploy` and `linuxdeploy-plugin-qt` to produce
an AppImage.
