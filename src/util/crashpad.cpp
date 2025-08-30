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

#include "crashpad.h"

#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
#include <QStandardPaths>

#include <filesystem>
#include <map>

// These platform-specific defines and includes are needed for chromium.

#if defined(Q_OS_WINDOWS)
#define NOMINMAX
#include <windows.h>
#endif

#if defined(Q_OS_LINUX)
#include <unistd.h>
#define MIN(x, y) (((x) < (y)) ? (x) : (y))
#endif

#include <crashpad/client/crash_report_database.h>
#include <crashpad/client/crashpad_client.h>
#include <crashpad/client/settings.h>
#include <mini_chromium/base/files/file_path.h>

#if defined(Q_OS_WINDOWS)
#include <base/strings/utf_string_conversions.h>
#endif

#include <util/spdlog_qt.h>

using base::FilePath;
using crashpad::CrashpadClient;
using crashpad::CrashReportDatabase;

constexpr const char *CRASHPAD_DIR = "crashpad";
#if defined(Q_OS_WINDOWS)
constexpr const char *CRASHPAD_HANDLER = "crashpad_handler.exe";
#else
constexpr const char *CRASHPAD_HANDLER = "crashpad_handler";
#endif

inline std::filesystem::path to_path(const QString &path)
{
#if defined(Q_OS_WINDOWS)
    return std::filesystem::path(path.toStdWString());
#else
    return std::filesystem::path(path.toStdString());
#endif
}

inline std::string to_string(const FilePath &path)
{
#if defined(Q_OS_WINDOWS)
    return base::WideToUTF8(path.value());
#else
    return path.value();
#endif
}

bool initializeCrashpad(const QString &appDataDir,
                        const char *dbName,
                        const char *appName,
                        const char *appVersion)
{
    static CrashpadClient *client = nullptr;
    if (client != nullptr) {
        spdlog::warn("Crashpad has already been initialized");
        return false;
    };
    spdlog::info("Initializing Crashpad");

    const QDir dataDir(appDataDir);
    if (!dataDir.exists()) {
        spdlog::error("Crashpad: app data director does not exist: {}", appDataDir);
        return false;
    }

    // Make sure the executable exists.
    const QString crashpadHandler = QCoreApplication::applicationDirPath() + "/" + CRASHPAD_HANDLER;
    const QFileInfo appInfo(crashpadHandler);
    if (!appInfo.exists()) {
        spdlog::error("Crashpad: the handler does not exist: {}", crashpadHandler);
        return false;
    }

    spdlog::debug("Crashpad: app data = {}", appDataDir);
    spdlog::debug("Crashpad: database = {}", dbName);
    spdlog::debug("Crashpad: application = {}", appName);
    spdlog::debug("Crashpad: version = {}", appVersion);
    spdlog::debug("Crashpad: handler = {}", crashpadHandler);

    // Convert paths to base::FilePath
    const FilePath handlerPath(to_path(crashpadHandler));
    const FilePath crashpadDirPath(to_path(appDataDir + "/crashpad"));
    const FilePath &reportsDirPath = crashpadDirPath;
    const FilePath &metricsDirPath = crashpadDirPath;

    // Configure url with your BugSplat database
    const std::string url = "https://" + std::string(dbName)
                            + ".bugsplat.com/post/bp/crash/crashpad.php";

    // Metadata that will be posted to BugSplat
    const std::map<std::string, std::string> annotations = {
        {"format", "minidump"}, // Required: Crashpad setting to save crash as a minidump
        {"database", dbName},   // Required: BugSplat database
        {"product", appName},   // Required: BugSplat appName
        {"version", appVersion} // Required: BugSplat appVersion
    };

    // Disable crashpad rate limiting so that all crashes have dmp files
    const std::vector<std::string> arguments = {"--no-rate-limit"};
    const bool restartable = true;
    const bool asynchronous_start = true;

    // Attachments to be uploaded alongside the crash - default bundle size limit is 20MB
    const QString buyoutData = appDataDir + "/export/buyouts.tgz";
    QFile buyoutDataFile(buyoutData);
    if (buyoutDataFile.exists()) {
        buyoutDataFile.remove();
    }
    const std::vector<FilePath> attachments = {FilePath(to_path(buyoutData))};

    // Log the crashpad initialization settings
    spdlog::debug("Crashpad: starting the crashpad client");
    spdlog::trace("Crashpad: handler = {}", to_string(handlerPath));
    spdlog::trace("Crashpad: reportsDir = {}", to_string(reportsDirPath));
    spdlog::trace("Crashpad: metricsDir = {}", to_string(metricsDirPath));
    spdlog::trace("Crashpad: url = {}", url);
    for (const auto &pair : annotations) {
        spdlog::trace("Crashpad: annotations[{}] = {}", pair.first, pair.second);
    }
    for (size_t i = 0; i < arguments.size(); ++i) {
        spdlog::trace("Crashpad: arguments[{}] = {}", i, arguments[i]);
    }
    spdlog::trace("Crashpad: restartable = {}", restartable);
    spdlog::trace("Crashpad: asynchronous_start = {}", asynchronous_start);
    for (size_t i = 0; i < attachments.size(); ++i) {
        spdlog::trace("Crashpad: attachments[{}] = {}", i, to_string(attachments[i]));
    }

    // Initialize crashpad database
    auto database = CrashReportDatabase::Initialize(reportsDirPath);
    if (database == NULL) {
        spdlog::error("Crashpad: failed to initialize the crash report database.");
        return false;
    }
    spdlog::trace("Crashpad: database initialized");

    // Enable automated crash uploads
    auto settings = database->GetSettings();
    if (settings == NULL) {
        spdlog::error("Crashpad: failed to get database settings.");
        return false;
    }
    settings->SetUploadsEnabled(true);
    spdlog::trace("Crashpad: upload enabled");

    // Create the client and start the handler
    client = new CrashpadClient();
    const bool started = client->StartHandler(handlerPath,
                                              reportsDirPath,
                                              metricsDirPath,
                                              url,
                                              annotations,
                                              arguments,
                                              restartable,
                                              asynchronous_start,
                                              attachments);
    if (!started) {
        spdlog::error("Crashpad: unable to start the handler");
        delete (client);
        client = nullptr;
        return false;
    }
    spdlog::debug("Crashpad: handler started");
    return true;
}
