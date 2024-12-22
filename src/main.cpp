/*
    Copyright 2014 Ilya Zhuravlev

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

#include "logindialog.h"

#include <QApplication>
#include <QCommandLineParser>
#include <QDir>
#include <QFontDatabase>
#include <QLocale>
#include <QSettings>
#include <QStandardPaths>

#include "QsLog.h"
#include "QsLogDest.h"

#include <clocale>

#include "application.h"
#include "crashpad.h"
#include "fatalerror.h"
#include "shop.h"
#include "util.h"
#include "version_defines.h"
#include "testmain.h"

#ifdef Q_OS_WINDOWS
#include "checkmsvc.h"
#endif

#ifdef _DEBUG
constexpr QsLogging::Level DEFAULT_LOGLEVEL = QsLogging::DebugLevel;
#else
constexpr QsLogging::Level DEFAULT_LOGLEVEL = QsLogging::InfoLevel;
#endif

int main(int argc, char* argv[])
{
    // Make sure resources from the static qdarkstyle library are available.
    Q_INIT_RESOURCE(darkstyle);
    Q_INIT_RESOURCE(lightstyle);

    QLocale::setDefault(QLocale::C);
    std::setlocale(LC_ALL, "C");

    // Holds the date and time of the current build based on __DATE__ and __TIME__ macros.
    const QString BUILD_TIMESTAMP = QString(__DATE__ " " __TIME__).simplified();
    const QDateTime BUILD_DATE = QLocale("en_US").toDateTime(BUILD_TIMESTAMP, "MMM d yyyy hh:mm:ss");

    QApplication a(argc, argv);

    QFontDatabase::addApplicationFont(":/fonts/Fontin-SmallCaps.ttf");

    QCommandLineOption option_test("test");
    option_test.setDescription("Run tests and exit.");

    QCommandLineOption option_data_dir("data-dir");
    option_data_dir.setDescription("Where to save Acquisition data.");
    option_data_dir.setValueName("data-dir");
    option_data_dir.setDefaultValue(QStandardPaths::writableLocation(QStandardPaths::AppLocalDataLocation));
    
    QCommandLineOption option_log_level("log-level");
    option_log_level.setDescription("How much to log.");
    option_log_level.setValueName("log-level");
    
    QCommandLineOption option_crash("crash-test");
    option_crash.setDescription("Trigger a crash dump at startup for testing.");

    QCommandLineParser parser;
    parser.addOption(option_test);
    parser.addOption(option_data_dir);
    parser.addOption(option_log_level);
    parser.addOption(option_crash);
    parser.process(a);

    // Setup the data dir, which is where the log will be written.
    const QDir appDataDir = QDir(parser.value(option_data_dir));
    const QString sLogPath(appDataDir.filePath("log.txt"));
    QSettings settings(appDataDir.filePath("settings.ini"), QSettings::IniFormat);

    // Determine the logging level.
    QsLogging::Level loglevel = DEFAULT_LOGLEVEL;
    if (parser.isSet(option_log_level)) {
         // First priority is the command line argument.
        loglevel = Util::TextToLogLevel(parser.value(option_log_level));
    } else if (settings.contains("log_level")) {
        // Second priority is the settings file.
        loglevel = Util::TextToLogLevel(settings.value("log_level").toString());
    };

    // Setup the logger.
    QsLogging::MaxSizeBytes logsize(10 * 1024 * 1024);
    QsLogging::MaxOldLogCount logcount(0);
    QsLogging::DestinationPtr fileDestination(
        QsLogging::DestinationFactory::MakeFileDestination(sLogPath, QsLogging::EnableLogRotation, logsize, logcount));
    QsLogging::DestinationPtr debugDestination(
        QsLogging::DestinationFactory::MakeDebugOutputDestination());

    QsLogging::Logger& logger = QsLogging::Logger::instance();
    logger.setLoggingLevel(loglevel);
    logger.addDestination(debugDestination);
    logger.addDestination(fileDestination);

    // Start the log with basic info
    QLOG_INFO() << "-------------------------------------------------------------------------------";
    QLOG_INFO().noquote() << a.applicationName() << a.applicationVersion() << "( version code" << VERSION_CODE << ")";
    QLOG_INFO().noquote() << "Built with Qt" << QT_VERSION_STR << "on" << BUILD_DATE.toString();
    QLOG_INFO().noquote() << "Running on Qt" << qVersion();
    QLOG_INFO() << "Logging level is" << logger.loggingLevel();

#ifdef Q_OS_WINDOWS
    // On Windows, it's possible there are incompatible versions of the MSVC runtime
    // DLLs that can cause unexpected crashes, so acquisition does some extra work
    // to try and detect this. It's not foolproof, however.
    checkMicrosoftRuntime();
#endif

    // Start crash reporting.
    if (!settings.contains("report_crashes")) {
        settings.setValue("report_crashes", true);
    };
    if (settings.value("report_crashes").toBool()) {
        initializeCrashpad(appDataDir.absolutePath(), APP_PUBLISHER, APP_NAME, APP_VERSION_STRING);
    };

    // Check SSL.
    QLOG_TRACE() << "Checking for SSL support...";
    if (!QSslSocket::supportsSsl()) {
#ifdef Q_OS_LINUX
        FatalError("SSL support is missing. Make sure OpenSSL 3.x shared libaries are on the LD_LIBRARY_PATH.");
#else
        FatalError("SSL support is missing.");
#endif
    };
    QLOG_TRACE() << "SSL Library Build Version: " << QSslSocket::sslLibraryBuildVersionString();
    QLOG_TRACE() << "SSL Library Version: " << QSslSocket::sslLibraryVersionString();

    // Check for test mode.
    if (parser.isSet(option_test)) {
        QLOG_INFO() << "Running test suite...";
        return test_main();
    };

    // Run the main application, starting with the login dialog.
    QLOG_INFO() << "Running application...";

    // Construct an instance of Application.
    Application app(appDataDir);

    // Trigger an optional crash.
    if (parser.isSet(option_crash)) {
        QLOG_TRACE() << "main(): a forced crash was requested";
        const int choice = QMessageBox::critical(nullptr, "FATAL ERROR",
            "Acquisition wants to abort.",
            QMessageBox::StandardButton::Abort | QMessageBox::StandardButton::Cancel,
            QMessageBox::StandardButton::Abort);
        if (choice == QMessageBox::StandardButton::Abort) {
            QLOG_FATAL() << "Forcing acquisition to crash.";
            abort();
        };
    };

    // Starting the application creates and shows the login dialog.
    QLOG_TRACE() << "main(): starting the application";
    app.Start();

    // Start the main event loop.
    QLOG_TRACE() << "main(): starting the event loop";
    return a.exec();
}
