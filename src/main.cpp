// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include <QApplication>
#include <QCommandLineParser>
#include <QDir>
#include <QFontDatabase>
#include <QLocale>
#include <QSettings>
#include <QStandardPaths>
#include <QTimer>

#include <clocale>

#include <sentry.h>

#include "application.h"
#include "shop.h"
#include "ui/logindialog.h"
#include "util/fatalerror.h"
#include "util/logging.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/util.h"
#include "version_defines.h"

#ifdef Q_OS_WINDOWS
#include "util/checkmsvc.h"
#endif

constexpr const char *BUILD_TIMESTAMP = (__DATE__ " " __TIME__);

constexpr const char *SENTRY_DSN
    = "https://89d30fa945c751603c0dfdde2c574497@o4509396161855488.ingest.us.sentry.io/"
      "4510597980618752";

#ifdef QT_DEBUG
constexpr const char *DEFAULT_LOGGING_LEVEL = "debug";
#else
constexpr const char *DEFAULT_LOGGING_LEVEL = "info";
#endif

#ifdef Q_OS_WIN
constexpr const char *CRASHPAD_HANDLER = "crashpad_handler.exe";
#else
constexpr const char *CRASHPAD_HANDLER = "crashpad_handler";
#endif

int main(int argc, char *argv[])
{
    // Make sure resources from the static qdarkstyle library are available.
    Q_INIT_RESOURCE(darkstyle);
    Q_INIT_RESOURCE(lightstyle);

    QLocale::setDefault(QLocale::C);
    std::setlocale(LC_ALL, "C");

    QApplication a(argc, argv);

    // Holds the date and time of the current build based on m___DATE_ and m___TIME_ macros.
    // This needs to be done after creating QApplication, otherwise there can be unexecpted
    // behavior, e.g. QStandardPaths::AppLocalDataLocation not being as expected.
    const QString build_timestamp = QString(BUILD_TIMESTAMP).simplified();
    const QDateTime build_date = QLocale("en_US").toDateTime(build_timestamp, "MMM d yyyy hh:mm:ss");
    const QString default_data_dir = QStandardPaths::writableLocation(
        QStandardPaths::AppLocalDataLocation);

    QFontDatabase::addApplicationFont(":/fonts/Fontin-SmallCaps.ttf");

    QCommandLineOption option_data_dir("data-dir");
    option_data_dir.setDescription("Where to save Acquisition data.");
    option_data_dir.setValueName("data-dir");
    option_data_dir.setDefaultValue(default_data_dir);

    QCommandLineOption option_log_level("log-level");
    option_log_level.setDescription("How much to log.");
    option_log_level.setValueName("log-level");

    QCommandLineParser parser;
    parser.addHelpOption();
    parser.addVersionOption();
    parser.addOption(option_data_dir);
    parser.addOption(option_log_level);
    parser.process(a);

    // Setup the data dir, which is where the log will be written.
    const auto appDir = QDir(QCoreApplication::applicationDirPath());
    const auto appDataDir = QDir(parser.value(option_data_dir));

    // Configure Sentry event logging.
    const auto handlerPath = Util::toPathBytes(appDir.filePath(CRASHPAD_HANDLER));
    const auto sentryPath = Util::toPathBytes(appDataDir.absoluteFilePath("sentry-native-db"));

    sentry_options_t *options = sentry_options_new();
    sentry_options_set_dsn(options, SENTRY_DSN);
    sentry_options_set_handler_path(options, handlerPath.constData());
    sentry_options_set_database_path(options, sentryPath.constData());
    sentry_options_set_release(options, APP_NAME "@" APP_VERSION_STRING);
    sentry_options_set_enable_logs(options, 1);
    sentry_init(options);

    // Make sure sentry closes when the program terminates.
    auto sentryClose = qScopeGuard([] { sentry_close(); });

    // Setup logging.
    const QString logPath(appDataDir.filePath("log.txt"));
    logging::init(logPath);

    // Determine the logging level. The command-line argument takes first priority.
    // If no command line argument is present, Acquistion will check for a logging
    // level in the settings file. Otherwise it will fallback to a default.
    QSettings settings(appDataDir.filePath("settings.ini"), QSettings::IniFormat);
    QString logging_option;
    if (parser.isSet(option_log_level)) {
        logging_option = parser.value(option_log_level);
    } else {
        logging_option = settings.value("log_level", DEFAULT_LOGGING_LEVEL).toString();
    }
    const auto loglevel = spdlog::level::from_str(logging_option.toStdString());

    // Start the log with basic info
    spdlog::flush_every(std::chrono::seconds(2));
    spdlog::flush_on(spdlog::level::err);
    spdlog::set_level(spdlog::level::info);
    spdlog::info("-------------------------------------------------------------------------------");
    spdlog::info("{} {}", a.applicationName(), a.applicationVersion());
    spdlog::info("Built with Qt {} on {}", QT_VERSION_STR, build_date.toString());
    spdlog::info("Running on Qt {}", qVersion());
    spdlog::info("Logging level will be {}", loglevel);
    spdlog::set_level(loglevel);

    QObject::connect(&a, &QCoreApplication::aboutToQuit, [] {
        spdlog::shutdown(); // flushes and stops background threads
    });

#ifdef Q_OS_WINDOWS
    // On Windows, it's possible there are incompatible versions of the MSVC runtime
    // DLLs that can cause unexpected crashes, so acquisition does some extra work
    // to try and detect this. It's not foolproof, however.
    checkMicrosoftRuntime();
#endif

    // Check SSL.
    spdlog::trace("Checking for SSL support...");
    if (!QSslSocket::supportsSsl()) {
#ifdef Q_OS_LINUX
        FatalError("SSL support is missing. Make sure OpenSSL 3.x shared libaries are on the "
                   "LD_LIBRARY_PATH.");
#else
        FatalError("SSL support is missing.");
#endif
    }
    spdlog::trace("SSL Library Build Version: {}", QSslSocket::sslLibraryBuildVersionString());
    spdlog::trace("SSL Library Version: {}", QSslSocket::sslLibraryVersionString());

    // Run the main application, starting with the login dialog.
    spdlog::info("Running application...");

    // Construct an instance of Application.
    Application app(appDataDir);

    // Start the main event loop.
    spdlog::trace("main(): starting the event loop");
    return a.exec();
}
