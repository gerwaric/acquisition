/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include <windows.h>

#include "QsLog/QsLog.h"

#include "fatalerror.h"
#include "version_defines.h"

// Local function prototypes
void checkApplicationDirectory(const QStringList& libraries);
void checkRuntimeVersion(const QStringList& libraries);
QVersionNumber getModuleVersion(const QString& name);

#ifdef QT_DEBUG
constexpr bool debug = true;
#else
constexpr bool debug = false;
#endif

static const QString DLL(const QString& name) {
    return name + (debug ? "d.dll" : ".dll");
}

void checkMicrosoftRuntime()
{
    QLOG_INFO() << "Checking Microsoft Visual C++ Runtime...";
    QLOG_INFO() << "Built with MSVC runtime" << MSVC_RUNTIME_VERSION;

    const QStringList libraries = {
        DLL("msvcp140"),
        DLL("vcruntime140"),
        DLL("vcruntime140_1") };

    QLOG_DEBUG() << "Checking MSVC runtime libraries:" << libraries.join(", ");

    checkApplicationDirectory(libraries);
    checkRuntimeVersion(libraries);
}

void checkApplicationDirectory(const QStringList& libraries)
{
    // Get the directory where the application is running from.
    const QString path = QGuiApplication::applicationDirPath();
    const QDir dir(path);

    QLOG_DEBUG() << "Checking application directory for unexpected MSVC libraries.";
    QLOG_DEBUG() << "Application directory:" << path;

    QStringList found;
    for (const auto& dll : libraries) {
        if (dir.exists(dll)) {
            found.append(dll);
        };
    };
    QLOG_DEBUG() << "Found unexpected MSVC libraries:" << found.join(", ");

    if (!found.isEmpty()) {

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
            QLOG_FATAL() << "Aborting.";
            abort();
        };

    };
}

void checkRuntimeVersion(const QStringList& libraries)
{
    QLOG_DEBUG() << "Checking MSVC runtime version.";

    const QVersionNumber build_version = QVersionNumber::fromString(MSVC_RUNTIME_VERSION).normalized();
    if (build_version.isNull()) {
        FatalError("Unable to parse MSVC runtime version form build constants");
    };
    QLOG_DEBUG() << "MSVC build version:" << build_version;

    for (const auto& lib : libraries) {
        
        const QVersionNumber lib_version = getModuleVersion(lib);
        if (lib_version.isNull()) {
            FatalError("Could not determine module version: " + lib);
        };
        QLOG_TRACE() << "Found" << lib << "version" << lib_version;

        if ((lib_version.majorVersion() < build_version.majorVersion()) ||
            (lib_version.minorVersion() < build_version.minorVersion()))
        {
            QLOG_ERROR() << "Found" << lib
                << "version" << lib_version.toString()
                << "but build version is" << build_version.toString();

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
    QLOG_TRACE() << "Getting module version for" << dll;

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
    QLOG_TRACE() << dll << "module path is" << path;

    // Get the DLL version.
    DWORD dummy = 0;
    const DWORD versionInfoSize = GetFileVersionInfoSize(path, &dummy);
    if (versionInfoSize == 0) {
        FatalError("Cannot get version info size for '" + dll + "'");
    };
    QLOG_TRACE() << dll << "module info size is" << versionInfoSize;

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

    const WORD major = HIWORD(fileInfo->dwFileVersionMS);
    const WORD minor = LOWORD(fileInfo->dwFileVersionMS);
    const WORD patch = HIWORD(fileInfo->dwFileVersionLS);
    const WORD tweak = LOWORD(fileInfo->dwFileVersionLS);

    QLOG_TRACE() << dll << "module versions are" << major << minor << patch << tweak;

    return QVersionNumber({ major, minor, patch, tweak }).normalized();
}
