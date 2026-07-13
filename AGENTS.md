# Repository Guidelines

## Project Map
- Acquisition is a C++23 Qt Widgets application for Path of Exile stash, inventory, and forum shop management.
- `src/` contains the application code. Important subdirectories include `src/datastore/`, `src/legacy/`, `src/poe/`, `src/ratelimit/`, `src/repoe/`, `src/ui/`, and `src/util/`.
- `assets/` contains bundled icons and Path of Exile-style UI images.
- `deps/qdarkstyle/` is the vendored qdarkstyle dependency. Other third-party libraries are fetched by CMake.
- Release packaging is defined in `.github/workflows/`.

## Build And Run
- Read `BUILD.md` before changing build, packaging, or platform setup.
- Configure and build with `cmake -S . -B build` and `cmake --build build`.
- CMake requires Qt 6.11+ and the Qt Network Authorization module. Linux also requires OpenSSL.
- Run locally with `./build/acquisition --data-dir /tmp/acq-data` to avoid touching a user's real Acquisition data.
- Run the checked-in Qt Test suite with `ctest --test-dir build --output-on-failure` after building.

## Design Cleanup (in progress)
- An interior design cleanup is underway; read `docs/cleanup/plan.md` before making structural changes to core code.
- Known design/correctness problems are cataloged in `docs/cleanup/findings.md`. Check it before fixing something that looks broken — it may already be assigned to a phase.

## Development Guidance
- Use `.clang-format` for C++ formatting when touching formatted source, respect existing `// clang-format off/on` blocks, and preserve nearby style for naming and code organization.
- Prefer existing Qt and local helper patterns over new abstractions.
- Keep generated logs, `settings.ini`, OAuth tokens, and other user-local state out of commits.
- For release changes, compare `CMakeLists.txt`, `installer.iss`, and the relevant workflow so version and packaging assumptions stay aligned.
