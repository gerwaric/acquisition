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

#include <ui/logindialog.h>
#include <util/fatalerror.h>
#include <util/logging.h>
#include <util/spdlog_qt.h>

#ifdef Q_OS_WINDOWS
#include <util/checkmsvc.h>
#endif

#include "application.h"
#include "shop.h"
#include "testmain.h"

constexpr const char *BUILD_TIMESTAMP = (__DATE__ " " __TIME__);

constexpr const char *SENTRY_DSN
    = "https://89d30fa945c751603c0dfdde2c574497@o4509396161855488.ingest.us.sentry.io/"
      "4510597980618752";

#ifdef QT_DEBUG
constexpr const char *DEFAULT_LOGGING_LEVEL = "debug";
#else
constexpr const char *DEFAULT_LOGGING_LEVEL = "info";
#endif

int main(int argc, char *argv[])
{
    // Make sure resources from the static qdarkstyle library are available.
    Q_INIT_RESOURCE(darkstyle);
    Q_INIT_RESOURCE(lightstyle);

    sentry_options_t *options = sentry_options_new();
    sentry_options_set_dsn(options, SENTRY_DSN);
    sentry_options_set_database_path(options, ".sentry-native");
    sentry_options_set_release(options, "acquisition@0.16.0");
    sentry_options_set_debug(options, 1);
    sentry_init(options);

    auto sentryClose = qScopeGuard([] { sentry_close(); });

    QLocale::setDefault(QLocale::C);
    std::setlocale(LC_ALL, "C");

    // Holds the date and time of the current build based on m___DATE_ and m___TIME_ macros.
    const QString build_timestamp = QString(BUILD_TIMESTAMP).simplified();
    const QDateTime build_date = QLocale("en_US").toDateTime(build_timestamp, "MMM d yyyy hh:mm:ss");

    QApplication a(argc, argv);

    QFontDatabase::addApplicationFont(":/fonts/Fontin-SmallCaps.ttf");

    QCommandLineOption option_test("test");
    option_test.setDescription("Run tests and exit.");

    QCommandLineOption option_data_dir("data-dir");
    option_data_dir.setDescription("Where to save Acquisition data.");
    option_data_dir.setValueName("data-dir");
    option_data_dir.setDefaultValue(
        QStandardPaths::writableLocation(QStandardPaths::AppLocalDataLocation));

    QCommandLineOption option_log_level("log-level");
    option_log_level.setDescription("How much to log.");
    option_log_level.setValueName("log-level");

    QCommandLineOption option_crash("crash-test");
    option_crash.setDescription("Trigger a crash dump at startup for testing.");

    QCommandLineOption option_validate_buyouts("validate-buyouts");
    option_validate_buyouts.setDefaultValue("Validate buyouts in the specified data file");
    option_validate_buyouts.setValueName("validate-buyouts");

    QCommandLineParser parser;
    parser.addHelpOption();
    parser.addVersionOption();
    parser.addOption(option_test);
    parser.addOption(option_data_dir);
    parser.addOption(option_log_level);
    parser.addOption(option_crash);
    parser.addOption(option_validate_buyouts);
    parser.process(a);

    // Setup the data dir, which is where the log will be written.
    const QDir appDataDir = QDir(parser.value(option_data_dir));
    const QString sLogPath(appDataDir.filePath("log.txt"));
    QSettings settings(appDataDir.filePath("settings.ini"), QSettings::IniFormat);

    // Determine the logging level. The command-line argument takes first priority.
    // If no command line argument is present, Acquistion will check for a logging
    // level in the settings file. Otherwise it will fallback to a default.
    QString logging_option;
    if (parser.isSet(option_log_level)) {
        logging_option = parser.value(option_log_level);
    } else if (settings.contains("log_level")) {
        logging_option = settings.value("log_level").toString();
    } else {
        logging_option = QString(DEFAULT_LOGGING_LEVEL);
    }
    const auto loglevel = spdlog::level::from_str(logging_option.toStdString());

    logging::init(sLogPath);

    // Start the log with basic info
    spdlog::set_level(spdlog::level::info);
    spdlog::info("-------------------------------------------------------------------------------");
    spdlog::info("{} {}", a.applicationName(), a.applicationVersion());
    spdlog::info("Built with Qt {} on {}", QT_VERSION_STR, build_date.toString());
    spdlog::info("Running on Qt {}", qVersion());
    spdlog::info("Logging level will be {}", loglevel);
    spdlog::set_level(loglevel);

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

    // Check for test mode.
    if (parser.isSet(option_test)) {
        spdlog::info("Running test suite...");
        return test_main(appDataDir.absolutePath());
    }

    if (parser.isSet(option_validate_buyouts)) {
        const QString filename = parser.value(option_validate_buyouts);
        spdlog::info("Validating buyouts: {}", filename);
    }

    // Run the main application, starting with the login dialog.
    spdlog::info("Running application...");

    // Construct an instance of Application.
    Application app;

    // Trigger an optional crash.
    if (parser.isSet(option_crash)) {
        spdlog::trace("main(): a forced crash was requested");
        const int choice = QMessageBox::critical(nullptr,
                                                 "FATAL ERROR",
                                                 "Acquisition wants to abort.",
                                                 QMessageBox::StandardButton::Abort
                                                     | QMessageBox::StandardButton::Cancel,
                                                 QMessageBox::StandardButton::Abort);
        if (choice == QMessageBox::StandardButton::Abort) {
            spdlog::critical("Forcing acquisition to crash.");
            abort();
        }
    }

    // Starting the application creates and shows the login dialog.
    QTimer::singleShot(0, &app, [&] { app.Start(appDataDir); });

    // Start the main event loop.
    spdlog::trace("main(): starting the event loop");
    return a.exec();
}
