# Acquisition [![Build Status](https://travis-ci.org/xyzz/acquisition.svg?branch=master)](https://travis-ci.org/xyzz/acquisition) [![Build status](https://ci.appveyor.com/api/projects/status/yutua4cn9cjv6wym?svg=true)](https://ci.appveyor.com/project/xyzz/acquisition)

Acquisition is an inventory management tool for [Path of Exile](https://www.pathofexile.com/).

It is written in C++, uses Qt widget toolkit and runs on Windows and Linux.

Check the [website](http://get.acquisition.today) for screenshots and video tutorials. You can download Windows setup packages from [the releases page](https://github.com/xyzz/acquisition/releases).

## Compiling/developing Acquisition

### Windows

On Windows you can use either Visual Studio or MinGW version of Qt Creator. Alternatively you can also use Visual Studio with Qt Add-in.

Make sure to have the [Windows 10 SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-10-sdk/) installed as well.

### Linux

Either open `acquisition.pro` in Qt Creator and build or do `qmake && make`.

## Command line arguments

`--data-dir <path>`: set the path where Acquisition should save its data. By default it's `%localappdata%\acquisition` on Windows and `~/.local/share/acquisition` on Linux.

`--test`: run tests. Zero exit code on success, other values indicate errors.

### Changelog

## v0.9.9

Application Changes:
- New dynamic rate limiting based on HTTP reply headers
- Only tabs marked or selected for updating are requested (this replaces the previous tab caching approach)
- Remove-only stash tabs are no longer refresh-locked
- The status bar updates while acquisition is loading saved tabs and items at startup

Development Changes:
- Added a build expiration option defined at compile time (for pre-release builds)
- Updated deployment and installer scripts

Toolchain:
- QT Creator 11.0.3 with QT 5.15.2 and MSVC 2019 64-bit (for release builds)
- Visual Studio 2019 with QT Visual Studio Tools 3.0.1 (for editing, debugging, and testing)
- Open SSL 1.1.1v (64-bit)
- Inno Setup 6.2.2 (for installer creation)