/*
    Copyright (C) 2014-2025 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "checkmsvc.h"

#include <QtGlobal>
#include <QDesktopServices>
#include <QDir>
#include <QGuiApplication>
#include <QMessageBox>
#include <QPushButton>
#include <QUrl>
#include <QVersionNumber>

#include <util/spdlog_qt.h>

#include "fatalerror.h"
#include "version_defines.h"

#ifdef Q_OS_WINDOWS
#include <windows.h>
#else
// Define stubs so we can include this file on non-Windows platforms,
// which helps with development when a windows machine isn't available.
constexpr unsigned MAX_PATH = 256;
using DWORD = unsigned long;
using WCHAR = wchar_t;
using LPCWSTR = const WCHAR*;
using LPVOID = void;
using HMODULE = void*;
using DWORD = unsigned long;
using UINT = unsigned int;
struct VS_FIXEDFILEINFO {
    DWORD dwFileVersionMS{ 0 };
    DWORD dwFileVersionLS{ 0 };
};
void* GetModuleHandle(...) { spdlog::error("MSVC checks not implemented."); return nullptr; }
int GetModuleFileName(...) { spdlog::error("MSVC checks not implemented.");  return 0; }
DWORD GetFileVersionInfoSize(...) { spdlog::error("MSVC checks not implemented.");  return 0; }
bool GetFileVersionInfo(...) { spdlog::error("MSVC checks not implemented.");  return false; }
unsigned int HIWORD(...) { spdlog::error("MSVC checks not implemented.");  return 0; };
unsigned int LOWORD(...) { spdlog::error("MSVC checks not implemented.");  return 0; };
bool VerQueryValue(...) { spdlog::error("MSVC checks not implemented.");  return false; };
#endif

// Local function prototypes
void checkApplicationDirectory(const QStringList& libraries);
void checkRuntimeVersion(const QStringList& libraries);
QVersionNumber getModuleVersion(const QString& name);

#ifdef QT_DEBUG
constexpr bool debug = true;
#else
constexpr bool debug = false;
#endif

static QString DLL(const QString& name) {
    return name + (debug ? "d.dll" : ".dll");
}

void checkMicrosoftRuntime()
{
#ifndef Q_OS_WINDOWS
    // Do nothing on Linux or macOS, but we still want this file to be included
    // in the build we can check for things like linting errors when developing
    // on other platforms.
    return;
#endif

    spdlog::info("Checking Microsoft Visual C++ Runtime...");
	spdlog::info("Built with MSVC runtime {}", MSVC_RUNTIME_BUILD_VERSION);
	spdlog::info("Requires MSVC runtime {}", MSVC_RUNTIME_MINIMUM_VERSION);

    const QStringList libraries = {
        DLL("msvcp140"),
        DLL("vcruntime140"),
        DLL("vcruntime140_1") };

    spdlog::debug("Checking MSVC runtime libraries: {}", libraries.join(", "));

    checkApplicationDirectory(libraries);
    checkRuntimeVersion(libraries);
}

void checkApplicationDirectory(const QStringList& libraries)
{
    // Get the directory where the application is running from.
    const QString path = QGuiApplication::applicationDirPath();
    const QDir dir(path);

    spdlog::debug("Checking application directory for unexpected MSVC libraries.");
    spdlog::debug("Application directory: {}", path);

    QStringList found;
    for (const auto& dll : libraries) {
        if (dir.exists(dll)) {
            found.append(dll);
        };
    };

    if (!found.isEmpty()) {

		spdlog::debug("Found {} unexpected MSVC libraries: {}", found.size(), found.join(", "));

        QStringList msg;
        msg.append("The application directory contains one or more MSVC runtime dlls:");
        msg.append("");
        for (const auto& filename : found) {
            msg.append("\t" + filename);
        };
        msg.append("");
        msg.append("Please delete these files and restart acquisition; they may cause unexpected crashes.");

        // Construct a warning dialog box.
        QMessageBox msgbox;
        msgbox.setWindowTitle("Acquisition");
        msgbox.setText(msg.join("\n"));
        msgbox.setIcon(QMessageBox::Warning);
        const auto* open = msgbox.addButton("Open folder and quit", QMessageBox::NoRole);
        const auto* quit = msgbox.addButton("Quit", QMessageBox::NoRole);
        const auto* ignore = msgbox.addButton("Ignore and continue", QMessageBox::NoRole);
        Q_UNUSED(quit);

        // Get and react to the user input.
        msgbox.exec();
        const auto& clicked = msgbox.clickedButton();
        if (clicked == open) {
            QDesktopServices::openUrl(QUrl::fromLocalFile(path));
        };
        if (clicked != ignore) {
            spdlog::critical("Aborting.");
            abort();
        };

    };
}

void checkRuntimeVersion(const QStringList& libraries)
{
    spdlog::debug("Checking MSVC runtime version.");

	const QVersionNumber build_version = QVersionNumber::fromString(MSVC_RUNTIME_BUILD_VERSION).normalized();
	const QVersionNumber required_version = QVersionNumber::fromString(MSVC_RUNTIME_MINIMUM_VERSION).normalized();
	if (required_version.isNull()) {
        FatalError("Unable to parse MSVC runtime version form build constants");
    };
	spdlog::debug("MSVC runtime build version: {}", build_version.toString());
	spdlog::debug("MSVC runtime minimum version: {}", required_version.toString());

    for (const auto& lib : libraries) {
        
        const QVersionNumber lib_version = getModuleVersion(lib);
        if (lib_version.isNull()) {
            FatalError("Could not determine module version: " + lib);
        };
        spdlog::trace("Found {} version {}", lib, lib_version.toString());

		if ((lib_version.majorVersion() < required_version.majorVersion()) ||
			(lib_version.minorVersion() < required_version.minorVersion()))
        {
            spdlog::error("Found {} version {} but build version is {}", lib, lib_version.toString(), required_version.toString());

            const QString msg =
                "The Microsoft Visual C++ Runtime needs to be updated."
                "\n\n"
                "Please re-install acquisition with this option selected.";

            QMessageBox::critical(nullptr, "Acquisition", msg);
            abort();
        };
    };
}

QVersionNumber getModuleVersion(const QString& dll)
{
    spdlog::trace("Getting module version for {}", dll);

    // Load the MSVC module.
    const std::wstring wstr = dll.toStdWString();
    const LPCWSTR name = wstr.c_str();
    const HMODULE hModule = GetModuleHandle(name);
    if (!hModule) {
        FatalError("Cannot get module handle for '" + dll + "'");
    };

    // Get the path to the DLL.
    WCHAR path[MAX_PATH];
    if (GetModuleFileName(hModule, path, MAX_PATH) == 0) {
        FatalError("Cannot get module file name for '" + dll + "'");
    };
    spdlog::trace("{} module path is {}", dll, QString::fromWCharArray(path));

    // Get the DLL version.
    DWORD dummy = 0;
    const DWORD versionInfoSize = GetFileVersionInfoSize(path, &dummy);
    if (versionInfoSize == 0) {
        FatalError("Cannot get version info size for '" + dll + "'");
    };
    spdlog::trace("{} module info size is {}", dll, static_cast<size_t>(versionInfoSize));

    // Allocate memory for version information
    std::vector<char> versionInfo(versionInfoSize);
    if (!GetFileVersionInfo(path, 0, versionInfoSize, versionInfo.data())) {
        FatalError("Cannot get version info for '" + dll + "'");
    };

    // Query the version value
    VS_FIXEDFILEINFO* fileInfo = nullptr;
    UINT size = 0;
    if (!VerQueryValue(versionInfo.data(), L"\\", reinterpret_cast<LPVOID*>(&fileInfo), &size) || (size == 0)) {
        FatalError("Unable to find the version of '" + dll + "'");
    };

    const int major = static_cast<int>(HIWORD(fileInfo->dwFileVersionMS));
    const int minor = static_cast<int>(LOWORD(fileInfo->dwFileVersionMS));
    const int patch = static_cast<int>(HIWORD(fileInfo->dwFileVersionLS));
    const int tweak = static_cast<int>(LOWORD(fileInfo->dwFileVersionLS));

    spdlog::trace("{} module versions are major={} minor={} patch={} tweak={}", dll, major, minor, patch, tweak);

    return QVersionNumber({ major, minor, patch, tweak }).normalized();
}
