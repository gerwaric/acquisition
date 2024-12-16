/*
    Copyright 2024 Gerwaric

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

#include "startupcheck.h"

#include <QDesktopServices>
#include <QDir>
#include <QGuiApplication>
#include <QMessageBox>
#include <QPushButton>
#include <QUrl>
#include <QVersionNumber>

#include <QtGlobal>


#ifdef Q_OS_WINDOWS
#include "windows.h"
#endif

#include "fatalerror.h"
#include "version_defines.h"

#ifdef Q_OS_WINDOWS
bool checkModuleVersion(const QString& name, QVersionNumber build_version, QStringList& message) {

#ifdef QT_DEBUG
    constexpr bool debug = true;
#else
    constexpr bool debug = false;
#endif
    const QString dll = name + ((debug) ? "d.dll" : ".dll");

    // Load the MSVC module.
    HMODULE hModule = GetModuleHandleA(dll.toLocal8Bit().constData());
    if (!hModule) {
        message.append("Unable to get module handle: " + dll);
        return false;
    };

    // Get the path to the DLL.
    WCHAR path[MAX_PATH];
    if (GetModuleFileName(hModule, path, MAX_PATH) == 0) {
        message.append("Unable to get module path: " + dll);
        return false;
    };

    // Get the DLL version.
    DWORD dummy = 0;
    DWORD versionInfoSize = GetFileVersionInfoSize(path, &dummy);
    if (versionInfoSize == 0) {
        message.append("Unable to get version info size: " + dll);
        return false;
    };

    // Allocate memory for version information
    std::vector<char> versionInfo(versionInfoSize);
    if (!GetFileVersionInfo(path, 0, versionInfoSize, versionInfo.data())) {
        message.append("Unable to get version info: " + dll);
        return false;
    };

    // Query the version value
    VS_FIXEDFILEINFO* fileInfo = nullptr;
    UINT size = 0;
    if (!VerQueryValue(versionInfo.data(), L"\\", reinterpret_cast<LPVOID*>(&fileInfo), &size) || (size == 0)) {
        message.append("Unable to query module version: " + dll);
        return false;
    };

    const QVersionNumber loaded_version = QVersionNumber({
        static_cast<int>(HIWORD(fileInfo->dwFileVersionMS)),
        static_cast<int>(LOWORD(fileInfo->dwFileVersionMS)),
        static_cast<int>(HIWORD(fileInfo->dwFileVersionLS)),
        static_cast<int>(LOWORD(fileInfo->dwFileVersionLS)) }).normalized();

    message.append("Found " + dll + " version " + loaded_version.toString());

    if (loaded_version != build_version) {
        message.back().append(" (expected version " + build_version.toString() + ")");
        return false;
    };

    return true;
}
#endif

#ifdef Q_OS_WINDOWS
bool checkMSVC() {

    const QVersionNumber build_version = QVersionNumber::fromString(MSVC_RUNTIME_VERSION).normalized();
    if (build_version.isNull()) {
		FatalError("Unable to parse MSVC runtime version form build constants");
    };

    const QStringList libraries = {
        "msvcp140",
        "vcruntime140",
        "vcruntime140_1"};

    QStringList errors;

    bool ok = true;
    for (const auto& lib : libraries) {
        if (!checkModuleVersion(lib, build_version, errors)) {
            ok = false;
        };
    };

    if (!ok) {
        // Construct a warning dialog box.
        QMessageBox msgbox;
        msgbox.setWindowTitle("Acquisition");
        msgbox.setText(errors.join("\n"));
        msgbox.setIcon(QMessageBox::Warning);
        const auto* help = msgbox.addButton("Help", QMessageBox::NoRole);
        const auto* quit = msgbox.addButton("Quit", QMessageBox::NoRole);
        const auto* ok = msgbox.addButton("Continue", QMessageBox::NoRole);
        Q_UNUSED(quit);

        // Get and react to the user input.
        msgbox.exec();
        const auto& clicked = msgbox.clickedButton();
        if (clicked == help) {
            //QDesktopServices::openUrl(QUrl::fromLocalFile(path));
        };
        return (clicked == ok);
    };
    return true;
}
#endif


bool startupCheck()
{
#ifdef Q_OS_WINDOWS
    if (!checkMSVC()) {
        return false;
    };
#endif
    return true;
}
