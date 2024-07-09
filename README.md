# Acquisition
Acquisition is an inventory management tool for [Path of Exile](https://www.pathofexile.com/):

- You can download all of your stash tabs and character inventories for offline search.

- You can list items for trade using forum shop threads, which are indexed by the official trade site. This allows you to list items in remove-only tabs as well as character invetories, which are not otherwise indexed on the official trade site.

Acquisition can run on Windows, macOS, Linux.

You can download setup packages from [the releases page](https://github.com/gerwaric/acquisition/releases).

## Building Acquisition

Acquisition is written in C++ and the Qt widget toolkit. It was original a qmake project, but it now uses cmake.

## SAST Tools

[PVS-Studio](https://pvs-studio.com/en/pvs-studio/?utm_source=website&utm_medium=github&utm_campaign=open_source) - static analyzer for C, C++, C#, and Java code.

### Qt

Acquisition should be buildable with Qt Creator (Community) on any platform that supports Qt 6.5.3 LTS.

Acquisition depends on the following Qt modules, which should be installed from the Qt Maintenance Tool:
- Qt HTTP Server
- Qt WebSockets

### Microsoft Windows

On Windows you can also build Acquisition with Visual Studio 2019 and the Qt Visual Studio Tools extension.

Windows releases are currently built with:
- Windows 11
- Qt Creator 13.0.0 (Community) with Qt 6.5.3 LTS and MSVC 2019 64-bit (for release builds)
- Visual Studio 2022 with Qt Visual Studio Tools 3.0.2 (for editing, debugging, and testing) and Qt Vs CMake 1.1
- Inno Setup 6.2.2 (for installer creation)

**NOTE**: v0.9.9 is the last version of Acquisition that runs on Windows 7 and 8.

### Apple macOS

**NOTE: macOS builds are unavailable because I don't have Apple hardware to build on right now --gerwaric**

macOS releases are currently built with:
- macOS Ventura 13.6.1 on Apple M1 silicon
- Qt Creator 11.0.3 (Community) with Qt 6.5.3 for macOS
- XCode 15.0.1

### Linux

Linux releases are currently built with:
- Linut Mint 20 Cinnamon (based on Ubuntu Focal) running in a VirtualBox VM
- Qt Creator 13.0.0 (Community) with Qt 6.5.3 GCC 64bit 
- OpenSSL 3.1.5 (manually built and installed from source)
- linuxdeploy (for AppImage creation)

You will need to have OpenSSL installed and available on your LD_LIBRARY_PATH to use the Linux AppImage. This is because linuxdeploy blacklists OpenSSL, which blocks the libraries from being included.

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
