# Building Acquisition

Acquisition is a C++23 Qt Widgets application. It can be built on Windows,
macOS, and Linux with CMake and Qt 6.

The CMake project currently requires Qt 6.11 or newer with these Qt modules:

- Core
- Gui
- Network
- Network Authorization
- Sql
- Widgets

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
./build/acquisition --data-dir /tmp/acq-data --log-level debug
```

There is no checked-in standalone test suite at the moment.

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
