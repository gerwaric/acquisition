# Acquisition
Acquisition is an inventory management tool for [Path of Exile](https://www.pathofexile.com/).

You can download all of your stash tabs and character inventories for offline search.

You can list items for trade using forum shop threads. These threads are indexed by the official trade site. This allows you to list items in remove-only tabs as well as character invetories, which are not normally listed on the official trade site.

Acquisition runs on Windows, macOS, Linux.

You can download setup packages from [the releases page](https://github.com/gerwaric/acquisition/releases).

## Building Acquisition

Acquisition is written in C++ and uses the Qt widget toolkit.

### Qt

Acquisition should be buildable with Qt Creator (Community) on any platform that supports Qt 6.5.

Acquisition depends on the following Qt modules, which should be installed from the Qt Maintenance Tool:
- Qt HTTP Server
- Qt WebSockets

### Microsoft Windows

On Windows you can also build Acquisition with Visual Studio 2019 and the Qt Visual Studio Tools extension.

Windows releases are currently built with:
- Windows 11
- Qt Creator 11.0.3 (Community) with Qt 6.5.3 LTS and MSVC 2019 64-bit (for release builds)
- Visual Studio 2019 with Qt Visual Studio Tools 3.0.2 (for editing, debugging, and testing)
- Inno Setup 6.2.2 (for installer creation)

**NOTE**: v0.9.9 is the last version of Acquisition that runs on Windows 7 and 8.

### Apple macOS

macOS releases are currently built with:
- macOS Ventura 13.6.1 on Apple M1 silicon
- Qt Creator 11.0.3 (Community) with Qt 6.5.3 for macOS
- XCode 15.0.1

### Linux

Linux releases are currently built with:
- Linut Mint 20 Cinnamon (based on Ubuntu Focal)
- Qt Creator 11.0.3 (Community) with Qt 6.5.3 GCC 64bit 
- OpenSSL 3.1.4 (manually built and installed from source)
- linuxdeployqt (for AppImage creation)

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