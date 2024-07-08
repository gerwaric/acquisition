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
#include <QMessageBox>

#include "QsLog.h"
#include "QsLogDest.h"

#include <clocale>

#include "application.h"
#include "filesystem.h"
#include "mainwindow.h"
#include "shop.h"
#include "updatechecker.h"
#include "util.h"
#include "version_defines.h"
#include "testmain.h"

#ifdef _DEBUG
constexpr QsLogging::Level DEFAULT_LOGLEVEL = QsLogging::TraceLevel;
#else
constexpr QsLogging::Level DEFAULT_LOGLEVEL = QsLogging::InfoLevel;
#endif

#ifdef Q_OS_LINUX
constexpr const char* SSL_ERROR = "OpenSSL 3.x was not found; check LD_LIBRARY_PATH if you have a custom installation.";
#else
constexpr const char* SSL_ERROR = "SSL is not supported. This is unexpected.";
#endif


int main(int argc, char* argv[])
{
    // Make sure resources from the static qdarkstyle library are available.
    Q_INIT_RESOURCE(darkstyle);
    Q_INIT_RESOURCE(lightstyle);

    QLocale::setDefault(QLocale::C);
    std::setlocale(LC_ALL, "C");

    // Holds the date and time of the current build based on __DATE__ and __TIME__ macros.
    const QString BUILD_TIMESTAMP = QStringLiteral(__DATE__ " " __TIME__).simplified();
    const QDateTime BUILD_DATE = QLocale("en_US").toDateTime(BUILD_TIMESTAMP, "MMM d yyyy hh:mm:ss");

    QApplication a(argc, argv);
    Filesystem::Init();

    QFontDatabase::addApplicationFont(":/fonts/Fontin-SmallCaps.ttf");

    QCommandLineParser parser;
    QCommandLineOption option_test("test");
    QCommandLineOption option_data_dir("data-dir", "Where to save Acquisition data.", "data-dir");
    QCommandLineOption option_log_level("log-level", "How much to log.", "log-level");
    parser.addOption(option_test);
    parser.addOption(option_data_dir);
    parser.addOption(option_log_level);
    parser.process(a);

    // Start by assumign the default log level.
    QsLogging::Level loglevel = DEFAULT_LOGLEVEL;
    bool valid_loglevel = true;

    // Process --log-level if it was present on the command line.
    if (parser.isSet(option_log_level)) {
        const QString logging_option = parser.value(option_log_level).toUpper();
        if (logging_option == "TRACE") {
            loglevel = QsLogging::TraceLevel;
        } else if (logging_option == "DEBUG") {
            loglevel = QsLogging::DebugLevel;
        } else if (logging_option == "INFO") {
            loglevel = QsLogging::InfoLevel;
        } else if (logging_option == "WARN") {
            loglevel = QsLogging::WarnLevel;
        } else if (logging_option == "ERROR") {
            loglevel = QsLogging::ErrorLevel;
        } else if (logging_option == "FATAL") {
            loglevel = QsLogging::FatalLevel;
        } else if (logging_option == "OFF") {
            loglevel = QsLogging::OffLevel;
        } else {
            valid_loglevel = false;
        };
    };

    // Setup the data dir, which is where the log will be written.
    if (parser.isSet(option_data_dir)) {
        const QString datadir = QString(parser.value(option_data_dir));
        Filesystem::SetUserDir(datadir);
    };
    const QString sLogPath = QString(QDir(Filesystem::UserDir()).filePath("log.txt"));

    // Setup the logger.
    QsLogging::Logger& logger = QsLogging::Logger::instance();
    QsLogging::MaxSizeBytes logsize(10 * 1024 * 1024);
    QsLogging::MaxOldLogCount logcount(0);
    QsLogging::DestinationPtr fileDestination(
        QsLogging::DestinationFactory::MakeFileDestination(sLogPath, QsLogging::EnableLogRotation, logsize, logcount));
    QsLogging::DestinationPtr debugDestination(
        QsLogging::DestinationFactory::MakeDebugOutputDestination());
    logger.setLoggingLevel(loglevel);
    logger.addDestination(debugDestination);
    logger.addDestination(fileDestination);

    // Start the log with basic info
    QLOG_INFO() << "-------------------------------------------------------------------------------";
    QLOG_INFO().noquote() << a.applicationName() << a.applicationVersion() << "( version code" << VERSION_CODE << ")";
    QLOG_INFO().noquote() << "Built with Qt" << QT_VERSION_STR << "on" << BUILD_DATE.toString();
    QLOG_INFO().noquote() << "Running on Qt" << qVersion();
    if (valid_loglevel == false) {
        QLOG_ERROR() << "Called with invalid log level:" << parser.value(option_log_level);
        QLOG_ERROR() << "Valid options are: TRACE, DEBUG, INFO, WARN, ERROR, FATAL, and OFF (case insensitive)";
        return EXIT_FAILURE;
    };
    QLOG_INFO() << "Logging level is" << logger.loggingLevel();

    if (!QSslSocket::supportsSsl()) {
        QLOG_FATAL() << QString(SSL_ERROR);
        QMessageBox msg(nullptr);
        msg.setWindowTitle("Acquisition [" + QString(APP_VERSION_STRING) + "]");
        msg.setText(SSL_ERROR);
        msg.addButton(QMessageBox::Abort);
        msg.exec();
        return EXIT_FAILURE;
    };
    QLOG_DEBUG() << "SSL Library Build Version: " << QSslSocket::sslLibraryBuildVersionString();
    QLOG_DEBUG() << "SSL Library Version: " << QSslSocket::sslLibraryVersionString();

    // Check for test mode.
    if (parser.isSet(option_test)) {
        QLOG_INFO() << "Running test suite...";
        return test_main();
    };

    // Run the main application, starting with the login dialog.
    QLOG_INFO() << "Running application...";

    // Construct an instance of Application.
    Application app;

    // Use the application objects to contruct a login diaglog.
    LoginDialog login(
        app.settings(),
        app.network_manager(),
        app.oauth_manager());

    // Connect to the update signal in case an update is detected before the main window is open.
    QObject::connect(&app.update_checker(), &UpdateChecker::UpdateAvailable, &app.update_checker(), &UpdateChecker::AskUserToUpdate);

    QObject::connect(&login, &LoginDialog::LoginComplete, &login,
        [&](POE_API api) {

            // Disconnect from the update signal so that only the main window gets it from now on.
            QObject::disconnect(&app.update_checker(), &UpdateChecker::UpdateAvailable, nullptr, nullptr);

            // Call init login to setup the shop, items manager, and other objects.
            app.InitLogin(api);

            // Prepare to show the main window now that everything is initialized.
            MainWindow* mw = new MainWindow(
                app.settings(),
                app.network_manager(),
                app.rate_limiter(),
                app.data(),
                app.oauth_manager(),
                app.items_manager(),
                app.buyout_manager(),
                app.currency_manager(),
                app.update_checker(),
                app.shop());

            login.close();
            mw->show();
        });

    // Start the initial check for updates.
    app.update_checker().CheckForUpdates();

    // Show the login dialog.
    login.show();

    // Start the main event loop.
    return a.exec();
}
