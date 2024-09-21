# Acquisition
Acquisition is an inventory management tool for [Path of Exile](https://www.pathofexile.com/) that has been around for over a decade:

- You can download all of your stash tabs and character inventories for offline search.

- You can list items for trade using forum shop threads, which are indexed by the official trade site. This allows you to list items in remove-only tabs as well as character invetories, which are not otherwise indexed on the official trade site.

Acquisition can run on Windows, macOS, Linux.

You can download setup packages from [the releases page](https://github.com/gerwaric/acquisition/releases).

## Building Acquisition

Acquisition is written in C++ and uses the Qt widget toolkit. It was originally a qmake project, but has been migrated to cmake.

### Qt

Acquisition should be buildable with Qt Creator (Community) on any platform that supports Qt 6.5.3 LTS.

Acquisition depends on the following Qt modules, which should be installed from the Qt Maintenance Tool:
- Qt HTTP Server
- Qt WebSockets

### Microsoft Windows

On Windows you can also build Acquisition with Visual Studio 2022 and the Qt Visual Studio Tools extension.

Windows releases are currently built with:
- Windows 11
- Qt Creator 14.0.0 (Community) with Qt 6.5.3 LTS using the compiler from Visual Studio 2022 (for release builds)
- Visual Studio 2022 with Qt Visual Studio Tools 3.2.0 (for editing, debugging, and testing) and Qt Vs CMake Tools 1.1
- Inno Setup 6.2.2 (for installer creation)

**NOTE**: v0.9.9 is the last version of Acquisition that runs on Windows 7 and 8.

### Apple macOS

macOS releases are currently built with:
- macOS Sonoma 14.6 on Intel silicon
- Qt Creator 14.0.0 (Community) with Qt 6.5.3 for macOS
- XCode 15.4

### Linux

Linux releases are distributed as an AppImage and built with:
- Linut Mint 20 Cinnamon
- Qt Creator 14.0.1 with Qt 6.5.3 GCC 64bit 
- OpenSSL 3.1.7
- linuxdeploy

You will need to have OpenSSL version 3.1.7 or later available on your LD_LIBRARY_PATH to use the Linux AppImage. This is because linuxdeploy blacklists OpenSSL, which blocks the libraries from being included.

If you're building acquisition yourself, make sure the OPENSSL_ROOT_DIR environment variable is set, either within the Qt project settings or via some other method.

### SAST Tools

[PVS-Studio](https://pvs-studio.com/en/pvs-studio/?utm_source=website&utm_medium=github&utm_campaign=open_source) - static analyzer for C, C++, C#, and Java code.

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
