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
#include <QDir>
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

#include <crashpad/client/crash_report_database.h>
#include <crashpad/client/crashpad_client.h>
#include <crashpad/client/settings.h>
#include <mini_chromium/base/files/file_path.h>

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

constexpr const int STARTUP_TIMEOUT_MS = 8000;

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
    const QString& appDataDir,
    const char* dbName,
    const char* appName,
    const char* appVersion)
{
    static CrashpadClient* client = nullptr;
    if (client != nullptr) {
        QLOG_WARN() << "Crashpad has already been initialized";
        return false;
    };
    QLOG_INFO() << "Initializing Crashpad";

    const QDir dataDir(appDataDir);
    if (!dataDir.exists()) {
        QLOG_ERROR() << "Crashpad: app data director does not exist:" << appDataDir;
        return false;
    };

    // Make sure the executable exists.
    const QString crashpadHandler = QCoreApplication::applicationDirPath() + "/" + CRASHPAD_HANDLER;
    const QFileInfo appInfo(crashpadHandler);
    if (!appInfo.exists()) {
        QLOG_ERROR() << "Crashpad: the handler does not exist:" << crashpadHandler;
        return false;
    };

    QLOG_DEBUG() << "Crashpad: app data =" << appDataDir;
    QLOG_DEBUG() << "Crashpad: database =" << dbName;
    QLOG_DEBUG() << "Crashpad: application =" << appName;
    QLOG_DEBUG() << "Crashpad: version =" << appVersion;
    QLOG_DEBUG() << "Crashpad: handler =" << crashpadHandler;

    // Convert paths to base::FilePath
    const FilePath handlerPath(StdPath(crashpadHandler));
    const FilePath crashpadDirPath(StdPath(appDataDir + "/crashpad"));
    const FilePath& reportsDirPath = crashpadDirPath;
    const FilePath& metricsDirPath = crashpadDirPath;

    // Configure url with your BugSplat database
    const std::string url = "https://" + std::string(dbName) + ".bugsplat.com/post/bp/crash/crashpad.php";

    // Metadata that will be posted to BugSplat
    const std::map<std::string, std::string> annotations = {
        { "format", "minidump" }, // Required: Crashpad setting to save crash as a minidump
        { "database", dbName },   // Required: BugSplat database
        { "product", appName },   // Required: BugSplat appName
        { "version", appVersion } // Required: BugSplat appVersion
    };

    // Disable crashpad rate limiting so that all crashes have dmp files
    const std::vector<std::string> arguments = {
        "--no-rate-limit"
    };
    const bool restartable = true;
    const bool asynchronous_start = true;


    // Attachments to be uploaded alongside the crash - default bundle size limit is 20MB
    const QString buyoutData = appDataDir + "/export/buyouts.tgz";
    QFile buyoutDataFile(buyoutData);
    if (buyoutDataFile.exists()) {
        buyoutDataFile.remove();
    };
    const std::vector<FilePath> attachments = {
        FilePath(StdPath(buyoutData))
    };

    // Log the crashpad initialization settings 
    QLOG_DEBUG() << "Crashpad: starting the crashpad client";
    QLOG_TRACE() << "Crashpad: handler =" << handlerPath.value();
    QLOG_TRACE() << "Crashpad: reportsDir =" << reportsDirPath.value();
    QLOG_TRACE() << "Crashpad: metricsDir =" << metricsDirPath.value();
    QLOG_TRACE() << "Crashpad: url =" << url;
    for (const auto& pair : annotations) {
        QLOG_TRACE() << "Crashpad: annotations[" << pair.first << "] =" << pair.second;
    };
    for (size_t i = 0; i < arguments.size(); ++i) {
        QLOG_TRACE() << "Crashpad: arguments[" << i << "] =" << arguments[i];
    };
    QLOG_TRACE() << "Crashpad: restartable =" << restartable;
    QLOG_TRACE() << "Crashpad: asynchronous_start =" << asynchronous_start;
    for (size_t i = 0; i < attachments.size(); ++i) {
        QLOG_TRACE() << "Crashpad: attachments[" << i << "] =" << attachments[i].value();
    };

    // Initialize crashpad database
    auto database = CrashReportDatabase::Initialize(reportsDirPath);
    if (database == NULL) {
        QLOG_ERROR() << "Crashpad: failed to initialize the crash report database.";
        return false;
    };
    QLOG_TRACE() << "Crashpad: database initialized";

    // Enable automated crash uploads
    auto settings = database->GetSettings();
    if (settings == NULL) {
        QLOG_ERROR() << "Crashpad: failed to get database settings.";
        return false;
    };
    settings->SetUploadsEnabled(true);
    QLOG_TRACE() << "Crashpad: upload enabled";

    // Create the client and start the handler
    client = new CrashpadClient();
    const bool started = client->StartHandler(handlerPath,
        reportsDirPath, metricsDirPath, url, annotations,
        arguments, restartable, asynchronous_start, attachments);
    if (!started) {
        QLOG_ERROR() << "Crashpad: unable to start the handler";
        delete(client);
        client = nullptr;
        return false;
    };
    QLOG_TRACE() << "Crashpad: handler started";

    // Wait for the handler to initialize
    const bool done = client->WaitForHandlerStart(STARTUP_TIMEOUT_MS);
    if (!done) {
        QLOG_ERROR() << "Crashpad: handler startup timed out";
        delete(client);
        client = nullptr;
        return false;
    };

    QLOG_INFO() << "Crashpad: initialization complete";
    return true;
}
