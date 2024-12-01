# Acquisition
Acquisition is an inventory management tool for [Path of Exile](https://www.pathofexile.com/) that has been around for over a decade:

- You can download all of your stash tabs and character inventories for offline search.

- You can list items for trade using forum shop threads, which are indexed by the official trade site. This allows you to list items in remove-only tabs as well as character invetories, which are not otherwise indexed on the official trade site.

Acquisition can run on Windows, macOS, Linux.

You can download setup packages from [the releases page](https://github.com/gerwaric/acquisition/releases).

## Building Acquisition

Acquisition is written in C++ and uses the Qt widget toolkit. It was originally a qmake project, but has been migrated to cmake.

### Qt

Acquisition should be buildable with Qt Creator (Community) on any platform that supports Qt 6.8.0.

Acquisition depends on the following Qt modules, which should be installed from the Qt Maintenance Tool:
- Qt HTTP Server
- Qt WebSockets

### Microsoft Windows

On Windows you can also build Acquisition with Visual Studio 2022 and the Qt Visual Studio Tools extension.

Windows releases are currently built with:
- Windows 11
- Qt Creator 14.0.2 (Community) with Qt 6.8.0 using MSVC 2022 64-bit
- Visual Studio 2022 with Qt Visual Studio Tools 3.3.0 (for editing, debugging, and testing) and Qt Vs CMake Tools 1.1
- Inno Setup 6.3.3 for installer creation

### Apple macOS

macOS releases are currently built with:
- macOS Sonoma 14.7.1 on Intel silicon
- Qt Creator 14.0.2 with Qt 6.8.0 for macOS
- XCode 15

### Linux

Linux releases are distributed as an AppImage and built with:
- Linut Mint 20 Cinnamon
- Qt Creator 14.0.2 with Qt 6.8.0
- OpenSSL 3.0.15 as provided by the Qt Maintenance Tool
- linuxdeploy

In order run acquisition, your LD_LIBRARY_PATH must include a directory that has OpenSSL 3.x libraries. If you are building acquisition, you can use the shared libraries provided by Qt, which are refernces in CMakeLists.txt. Otherwise you may have to download and install a compatible version of OpenSSL yourself unless your distribution provides one. This is because linuxdeploy blacklists the OpenSSL libraries for sercurity reasons.

### SAST Tools

[PVS-Studio](https://pvs-studio.com/en/pvs-studio/?utm_source=website&utm_medium=github&utm_campaign=open_source) - static analyzer for C, C++, C#, and Java code.

<a href="https://scan.coverity.com/projects/gerwaric-acquisition">
  <img alt="Coverity Scan Build Status"
       src="https://scan.coverity.com/projects/31083/badge.svg"/>
</a>

## Command line

`acquisition [--data-dir <path>] [--log-level <level>] [--test]`

`--data-dir <path>`:
	Set the path where Acquisition should save its data.
	The default on Windows is `%localappdata%\acquisition`.
	The default on macOS is `~/Library/Application Support/acquisition`.
	The default on Linux is `~/.local/share/acquisition`.

`--log-level <level>`:
	Controls the amount of detail in the log.
	Options are `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`, and `OFF`.
 	This option is case-insensitive.
	The default level for release builds is `INFO`.
	The default level for debug builds is `TRACE`.

`--test`:
	run tests. Zero exit code on success, other values indicate errors.

## Reporting issues

If you're having problems with Acquisition, please check the issues page: https://github.com/gerwaric/acquisition/issues
