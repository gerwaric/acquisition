# Acquisition
Acquisition is an inventory management tool for [Path of Exile](https://www.pathofexile.com/).

It is written in C++, uses the Qt widget toolkit, and runs on Windows and macOS. It used to run on Linux as well.

You can download Windows and macOS setup packages from [the releases page](https://github.com/gerwaric/acquisition/releases).

## Compiling or Developing Acquisition

### Windows

On Windows you can use either QT Creator, or Visual Studio with the Qt Add-In.

v0.9.9 was built with the following tools:
- QT Creator 11.0.3 with QT 5.15.2 and MSVC 2019 64-bit (for release builds)
- Visual Studio 2019 with QT Visual Studio Tools 3.0.1 (for editing, debugging, and testing)
- Open SSL 1.1.1v (64-bit)
- Inno Setup 6.2.2 (for installer creation)

### macOS

v0.9.9 was built on an M1 MacBook Air running macOS Ventura 13.6.

But it's tricky. Acquisition uses Qt version 5, which you'll have to compile directly from source. It also requires an older macOS SDK, so you'll need to install an older XCode, specifically version 14.0.1. Once you have that working, you'll need to build the `qtbase`, `qtdeclarative`, and `qttools` modules.

*NOTE*: v0.9.9 is the first macOS release. Please report any issues with the installer.

### Linux

Either open `acquisition.pro` in Qt Creator and build or do `qmake && make`.

*WARNING*: This has not been tested lately.

## Command line arguments

`--data-dir <path>`: set the path where Acquisition should save its data. By default it's `%localappdata%\acquisition` on Windows, `~/.local/share/acquisition` on Linux, and `~/Library/Application Support/acquisition` on macOS.

`--log-level <level>`: controls the amount of detail in the log.
	Options are `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`, and `OFF`.
	The default level for release builds is `INFO`.
	The default level for debug builds is `TRACE`.
 	This option is case-insensitive.

`--test`: run tests. Zero exit code on success, other values indicate errors.

### Recent Changes

## v0.9.9

Application Changes:
- The status bar updates while acquisition is loading saved tabs and items at startup.
- New dynamic rate limiting based on HTTP reply headers fully complies with GGG's rate limiting policies.
- Remove-only stash tabs with in-game buyouts are no longer refresh-locked.
- Only tabs marked or selected for updating are requested (this replaces the previous tab caching).
- Better error handling and error prevention during shop forum updates.

Development Changes:
- Added a build expiration option defined at compile time (for pre-release builds).
- Updated deployment and installer scripts.
- Created a macOS build.

### Roadmap

## v0.10 (or maybe v1.0?)

- OAuth. Acquisition has been approved by GGG as a public client, which means reworking authentication, as well as moving away from the legacy API that Acquisition was designed around, so it comes with significant rework.
- Porting to Qt 6, which will make OAuth easier to implement and prevent issues when Qt 5 goes EOL in 2025.
- Various improvements under-the-hood.
