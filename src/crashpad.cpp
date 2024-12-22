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

#include "crashpad.h"

#include <QCoreApplication>
#include <QStandardPaths>
#include <QFileInfo>

#include <filesystem>
#include <map>

// These platform-specific defines and includes are needed for chromium.

#if defined(Q_OS_WINDOWS)
#define NOMINMAX
#include <windows.h>
#endif

#if defined(Q_OS_MAC)
#include <mach-o/dyld.h>
#endif

#if defined(Q_OS_LINUX)
#include <unistd.h>
#define MIN(x, y) (((x) < (y)) ? (x) : (y))
#endif

#include "crashpad/client/crash_report_database.h"
#include "crashpad/client/crashpad_client.h"
#include "crashpad/client/settings.h"
#include "mini_chromium/base/files/file_path.h"

#include <QsLog/QsLog.h>

using base::FilePath;
using crashpad::CrashpadClient;
using crashpad::CrashReportDatabase;

constexpr const char* CRASHPAD_DIR = "crashpad";
#if defined(Q_OS_WINDOWS)
constexpr const char* CRASHPAD_HANDLER = "crashpad_handler.exe";
#else
constexpr const char* CRASHPAD_HANDLER = "crashpad_handler";
#endif
constexpr const char* ATTACHMENT_TXT = "attachment.txt";

// https://stackoverflow.com/questions/53744285/how-to-convert-between-boostfilesystempath-and-qstring

std::filesystem::path StdPath(const QString& path)
{
#if defined(Q_OS_WINDOWS)
    auto* wptr = reinterpret_cast<const wchar_t*>(path.utf16());
    return std::filesystem::path(wptr, wptr + path.size());
#else
    return std::filesystem::path(path.toStdString());
#endif
}

bool initializeCrashpad(
    const QString& dataDir,
    const QString& dbName,
    const QString& appName,
    const QString& appVersion)
{
    QLOG_TRACE() << "initializeCrashpad() entered";
    static CrashpadClient* client = nullptr;

    if (client != nullptr) {
        QLOG_ERROR() << "Crashpad has already been initialized";
        return false;
    };

    QLOG_INFO() << "Initalizing Crashpad";
    QLOG_TRACE() << "initializeCrashpad() database =" << dbName;
    QLOG_TRACE() << "initializeCrashpad() application =" << appName;
    QLOG_TRACE() << "initializeCrashpad() version =" << appVersion;

    const QString appExe = QCoreApplication::applicationDirPath() + "/" + CRASHPAD_HANDLER;
    QLOG_TRACE() << "initializeCrashpad() appExe =" << appExe;

    // Make sure the executable exists.
    const QFileInfo appInfo(appExe);
    if (!appInfo.exists()) {
        QLOG_ERROR() << "The crash handler is missing:" << appExe;
        return false;
    };

    // Make sure the data directory exists.
    const QFileInfo dirInfo(dataDir);
    if (dirInfo.exists() && !dirInfo.isDir()) {
        QLOG_ERROR() << "The data directory does not exist:" << dataDir;
        return false;
    };

    // Convert paths to base::FilePath
    const FilePath handler(StdPath(appExe));
    const FilePath crashpadDir(StdPath(dataDir));
    const FilePath& reportsDir = crashpadDir;
    const FilePath& metricsDir = crashpadDir;
    QLOG_TRACE() << "initializeCrashpad() handler =" << handler.value();
    QLOG_TRACE() << "initializeCrashpad() crashpadDir =" << crashpadDir.value();
    QLOG_TRACE() << "initializeCrashpad() reportsDir =" << reportsDir.value();
    QLOG_TRACE() << "initializeCrashpad() metricsDir =" << metricsDir.value();

    // Configure url with your BugSplat database
    const std::string url = "https://" + dbName.toStdString() + ".bugsplat.com/post/bp/crash/crashpad.php";
    QLOG_TRACE() << "initializeCrashpad() url =" << url;

    // Metadata that will be posted to BugSplat
    const std::map<std::string, std::string> annotations{ {
        {"format", "minidump"},                 // Required: Crashpad setting to save crash as a minidump
        {"database", dbName.toStdString()},     // Required: BugSplat database
        {"product", appName.toStdString()},     // Required: BugSplat appName
        {"version", appVersion.toStdString()}   // Required: BugSplat appVersion
    } };
    for (auto& pair : annotations) {
        QLOG_TRACE() << "initializeCrashpad() annotations[" << pair.first << "] =" << pair.second;
    };

    // Initialize crashpad database
    QLOG_TRACE() << "initializeCrashpad() calling CrashReportDatabase::Initialize(" << reportsDir.value() << ")";
    auto database = CrashReportDatabase::Initialize(reportsDir);
    if (database == NULL) {
        QLOG_ERROR() << "Crashpad: failed to initialize the crash reports database.";
        return false;
    };

    // Enable automated crash uploads
    QLOG_TRACE() << "initializeCrashpad() getting settings from the crash report database";
    auto settings = database->GetSettings();
    if (settings == NULL) {
        QLOG_ERROR() << "Crashpad: failed to get settings.";
        return false;
    };
    QLOG_TRACE() << "initializeCrashpad() calling SetUploadsEnabled( true )";
    settings->SetUploadsEnabled(true);

    // Disable crashpad rate limiting so that all crashes have dmp files
    const std::vector<std::string> arguments{ "--no-rate-limit" };
    for (auto i = 0; i < arguments.size(); ++i) {
        QLOG_TRACE() << "initializeCrashpad() arguments[" << i << "] =" << arguments[i];
    };

    const bool restartable = true;
    const bool asynchronous_start = true;
    QLOG_TRACE() << "initializeCrashpad() restartable =" << restartable;
    QLOG_TRACE() << "initializeCrashpad() asynchronous_start =" << asynchronous_start;

    // Attachments to be uploaded alongside the crash - default bundle size limit is 20MB
    const std::vector<FilePath> attachments;
    for (auto i = 0; i < attachments.size(); ++i) {
        QLOG_TRACE() << "initializeCrashpad() attachments[" << i << "] =" << attachments[i].value();
    };

    // Start crash handler
    QLOG_TRACE() << "initializeCrashpad() creating a new CrashpadClient";
    client = new CrashpadClient();
    bool status = client->StartHandler(handler,
        reportsDir, metricsDir, url, annotations, arguments,
        restartable, asynchronous_start, attachments);

    if (status) {
        QLOG_TRACE() << "initializeCrashpad() crashpad is initialized.";
    } else {
        QLOG_ERROR() << "Crashpad failed to initialize the handler.";
        client = nullptr;
    };
    return status;
}
