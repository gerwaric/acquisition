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

#include "QsLog.h"

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
    const QString& dbName,
    const QString& appName,
    const QString& appVersion)
{
    static CrashpadClient* client = nullptr;

    if (client != nullptr) {
        QLOG_ERROR() << "Crashpad has already been initialized";
        return false;
    };

    QLOG_INFO() << "Initalizing Crashpad";
    QLOG_TRACE() << "  database =" << dbName;
    QLOG_TRACE() << "  application =" << appName;
    QLOG_TRACE() << "  version =" << appVersion;

    const QString appExe = QCoreApplication::applicationDirPath() + "/" + CRASHPAD_HANDLER;
    const QString appDataDir = QStandardPaths::writableLocation(QStandardPaths::AppLocalDataLocation) + "/" + CRASHPAD_DIR;

    const QFileInfo appInfo(appExe);
    if (!appInfo.exists()) {
        QLOG_ERROR() << "Crashpad: the crash handler executable is missing:" << appExe;
        return false;
    };

    const QFileInfo dirInfo(appDataDir);
    if (dirInfo.exists() && !dirInfo.isDir()) {
        QLOG_ERROR() << "Crashpad: the data directory is a file:" << appDataDir;
        return false;
    };

    const FilePath handler(StdPath(appExe));
    const FilePath dataDir(StdPath(appDataDir));
    const FilePath& reportsDir = dataDir;
    const FilePath& metricsDir = dataDir;

    // Configure url with your BugSplat database
    const std::string url = "https://" + dbName.toStdString() + ".bugsplat.com/post/bp/crash/crashpad.php";

    // Metadata that will be posted to BugSplat
    const std::map<std::string, std::string> annotations{ {
        {"format", "minidump"},                 // Required: Crashpad setting to save crash as a minidump
        {"database", dbName.toStdString()},     // Required: BugSplat database
        {"product", appName.toStdString()},     // Required: BugSplat appName
        {"version", appVersion.toStdString()}   // Required: BugSplat appVersion
    } };

    // Initialize crashpad database
    auto database = CrashReportDatabase::Initialize(reportsDir);
    if (database == NULL) {
        QLOG_ERROR() << "Crashpad: failed to initialize the crash reports database.";
        return false;
    };

    // Enable automated crash uploads
    auto settings = database->GetSettings();
    if (settings == NULL) {
        QLOG_ERROR() << "Crashpad: failed to get settings.";
        return false;
    };
    settings->SetUploadsEnabled(true);

    // Disable crashpad rate limiting so that all crashes have dmp files
    const std::vector<std::string> arguments{ "--no-rate-limit" };

    const bool restartable = true;
    const bool asynchronous_start = true;

    // Attachments to be uploaded alongside the crash - default bundle size limit is 20MB
    const std::vector<FilePath> attachments;

    QLOG_TRACE() << "Crashpad: starting handler";
    QLOG_TRACE() << "  hander =" << handler.value();
    QLOG_TRACE() << "  database =" << reportsDir.value();
    QLOG_TRACE() << "  metrics_dir =" << metricsDir.value();
    QLOG_TRACE() << "  url =" << url;
    for (auto& pair : annotations) {
        QLOG_TRACE() << "  annotations[" << pair.first << "] =" << pair.second;
    };
    for (auto i = 0; i < arguments.size(); ++i) {
        QLOG_TRACE() << "  arguments[" << i << "] =" << arguments[i];
    };
    QLOG_TRACE() << "  restartable =" << restartable;
    QLOG_TRACE() << "  asynchronous_start =" << asynchronous_start;
    for (auto i = 0; i < attachments.size(); ++i) {
        QLOG_TRACE() << "  attachments[" << i << "] =" << attachments[i].value();
    };

    // Start crash handler
    client = new CrashpadClient();
    bool status = client->StartHandler(handler,
        reportsDir, metricsDir, url, annotations, arguments,
        restartable, asynchronous_start, attachments);

    if (status) {
        QLOG_TRACE() << "Crashpad is initialized.";
    } else {
        QLOG_ERROR() << "Crashpad failed to initialize the handler.";
        client = nullptr;
    };
    return status;
}

