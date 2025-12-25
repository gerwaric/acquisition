# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Core C++/Qt application code (UI in `src/ui/`, utilities in `src/util/`, Path of Exile API types in `src/poe/`).
- `test/`: Qt Test-based unit/integration tests (e.g., `test/testitem.cpp`, `test/testitemsmanager.cpp`).
- `assets/`: Icons and branding assets used by platform bundles.
- `deps/`: Vendored dependencies (Crashpad, qdarkstyle, etc.).
- `resources.qrc`: Qt resource collection for bundled assets.
- Build scripts: `deploy-*.sh` and `deploy-*.cmd` for platform packaging.

## Build, Test, and Development Commands
- `cmake -S . -B build`: Configure the CMake project (requires Qt 6.10+ and OpenSSL on Linux).
- `cmake --build build`: Build the `acquisition` executable.
- `./build/acquisition --test`: Run the Qt test suite via the application entry point.
- `./build/acquisition --data-dir /tmp/acq-data`: Run with a custom data directory.
- `./build/acquisition --log-level debug`: Increase logging verbosity for debugging.

## Coding Style & Naming Conventions
- Indentation: 4 spaces; braces on the next line (see `src/main.cpp`).
- C++ standard: C++23 (see `CMakeLists.txt`).
- Naming: files are lowercase; classes and methods are PascalCase; locals are typically `lower_snake_case`. Follow nearby file conventions when unsure.
- Formatting: no enforced formatter in repo; preserve existing style and `// clang-format off/on` blocks.

## Testing Guidelines
- Framework: Qt Test (`Qt::Test` linked in `CMakeLists.txt`).
- Test files live in `test/` and are named `test*.{h,cpp}`.
- Use `QTest::qExec` patterns in `test/testmain.cpp` when adding new suites.

## Commit & Pull Request Guidelines
- Commit messages are short, imperative summaries (e.g., "Add ...", "Remove ...", "Bump version ...").
- PRs should include: a concise description, platform tested (Windows/macOS/Linux), and any UI screenshots for visual changes.
- Link related issues or release tags when applicable.

## Configuration & Security Notes
- Local state is stored in a platform-specific data directory (see `--data-dir` in `README.md`).
- Avoid committing user credentials, OAuth tokens, or generated logs (`settings.ini`, `log.txt`).
