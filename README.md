# Acquisition

Acquisition is an inventory management tool for [Path of Exile](https://www.pathofexile.com/) that has been around for over a decade:

- You can download all of your stash tabs and character inventories for offline search.

- You can list items for trade using forum shop threads, which are indexed by the official trade site. This allows you to list items in remove-only tabs as well as character inventories, which are not otherwise indexed on the official trade site.

Acquisition can run on Windows, macOS, Linux.

You can download setup packages from [the releases page](https://github.com/gerwaric/acquisition/releases).

## Building Acquisition

Acquisition is written in C++ and uses the Qt widget toolkit. It was originally a qmake project, but has been migrated to cmake.

See [BUILD.md](BUILD.md) for more detailed build and release packaging guidance.

## Command line

`acquisition [--data-dir <path>] [--log-level <level>]`

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
	The default level for debug builds is `DEBUG`.

### Local control CLI

`acquisitionctl` inspects and controls an Acquisition GUI that is already
running for the same data directory. It never opens the application's databases
or starts a second synchronizer. Output is versioned JSON.

```sh
acquisitionctl status --json
acquisitionctl tabs --json
acquisitionctl items --limit 50 --json
acquisitionctl item <item-id> --json
acquisitionctl refresh start --json
acquisitionctl refresh status <operation-id> --json
acquisitionctl refresh wait <operation-id> --timeout 300 --json
```

Use `--data-dir <path>` for both executables when Acquisition uses a non-default
location. An accepted refresh belongs to the GUI and continues if
`acquisitionctl` disconnects. Existing automatic-shop settings remain in effect.
Run `acquisitionctl --help` for pagination and tab-filter options.

## Reporting issues

If you're having problems with Acquisition, please check the issues page: https://github.com/gerwaric/acquisition/issues

You can also contact me on Discord as gerwaric.

## SAST Tools

[PVS-Studio](https://pvs-studio.com/en/pvs-studio/?utm_source=website&utm_medium=github&utm_campaign=open_source) - static analyzer for C, C++, C#, and Java code.

<a href="https://scan.coverity.com/projects/gerwaric-acquisition">
  <img alt="Coverity Scan Build Status"
       src="https://scan.coverity.com/projects/31083/badge.svg"/>
</a>
