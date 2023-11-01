# Acquisition
Acquisition is an inventory management tool for [Path of Exile](https://www.pathofexile.com/).

You can download all of your stash tabs and character inventories for offline search.

You can list items for trade using forum shop threads. These threads are indexed by the official trade site. This allows you to list items in remove-only tabs as well as character invetories, which are not normally listed on the official trade site.

Acquisition runs on Windows and macOS, and probably Linux with some effort.

You can download setup packages from [the releases page](https://github.com/gerwaric/acquisition/releases).

## Compiling or Developing Acquisition

Acquisition is written in C++ and uses the Qt widget toolkit.

Acquisition can be built on all supported platforms with Qt Creator.

### Windows

On Windows you can also use Visual Studio with the Qt Visual Studio Tools extension.

v0.9.10 was built with the following tools:
- Qt Creator 11.0.3 with Qt 6.5.3 LTS and MSVC 2019 64-bit (for release builds)
- Visual Studio 2019 with Qt Visual Studio Tools 3.0.1 (for editing, debugging, and testing)
- Inno Setup 6.2.2 (for installer creation)

### macOS

v0.9.10 was built on an M1 Mac running macOS Ventura 13.6 with the following tools:
- Qt Creator 11.0.3 with Qt 6.5.3 for macOS
- XCode 15.0.1

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