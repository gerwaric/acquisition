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
#include <QDesktopServices>
#include <QDir>
#include <QFontDatabase>
#include <QLocale>
#include <QMessageBox>
#include <QSettings>

#include "QsLog.h"
#include "QsLogDest.h"

#include <clocale>

#include "application.h"
#include "fatalerror.h"
#include "filesystem.h"
#include "shop.h"
#include "util.h"
#include "version_defines.h"
#include "testmain.h"

// This is needed for Visual Studio 2022.
#include "boost/config.hpp"
#ifdef BOOST_NO_EXCEPTIONS
#include <boost/throw_exception.hpp>
void boost::throw_exception(std::exception const& e) {
    throw e;
}
#endif

#ifdef _DEBUG
constexpr QsLogging::Level DEFAULT_LOGLEVEL = QsLogging::DebugLevel;
#else
constexpr QsLogging::Level DEFAULT_LOGLEVEL = QsLogging::InfoLevel;
#endif

#ifdef Q_OS_WINDOWS
bool checkManifest() {

    // Get the directory where the application is running from.
    const QString path = QGuiApplication::applicationDirPath();
    const QDir dir(path);

    // These are the Windows libraries we expect to find in the
    // directory alongside the application executable.
    const QStringList expected_dlls = {
        "D3Dcompiler_47.dll",
        "opengl32sw.dll",
        "Qt6Core.dll",
        "Qt6Gui.dll",
        "Qt6HttpServer.dll",
        "Qt6Network.dll",
        "Qt6Sql.dll",
        "Qt6Svg.dll",
        "Qt6Test.dll",
        "Qt6WebSockets.dll",
        "Qt6Widgets.dll"
    };

    // Build the list of libraries that were present but unexpected.
    // The .dll extension is windows-specific, but so far Windows
    // is the only platform affected by this issue.
    QStringList unexpected_dlls;
    for (const auto& dll : dir.entryList({ "*.dll" })) {
        if (!expected_dlls.contains(dll, Qt::CaseInsensitive)) {
            unexpected_dlls.push_back(dll);
        };
    };

    // Do nothing if nothing unexpected was found.
    if (unexpected_dlls.isEmpty()) {
        return true;
    };

    // Create a warning message for the dialog box.
    QStringList msg = {
        "Unexpected libraries found in '" + path + "':",
        "",
        "\t" + unexpected_dlls.join(", "),
        ""
    };
    if (unexpected_dlls.contains("msvcp140.dll", Qt::CaseInsensitive)) {
        msg.append("Acquisition may crash. "
            "Please consider deleteing the above files and installing the "
            "MSVC runtime that comes with acquisition. You can do this from "
            "the installer, or you can run 'vc_redist.x64.exe' from the "
            "acquisition program directory");
    } else {
        msg.append("Acquisition may crash. "
            "Please consider moving or deleting these files.");
    };


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
    return (clicked == ignore);
}
#endif // Q_OS_WINDOWS

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

#ifdef Q_OS_WINDOWS
    // Check for unexpected files, especially DLLs on Windows, where
    // this can cause unexpected crashes. We have to do this after
    // constructing the application object because we might need to 
    // present a warning dialog box to the user.
    if (!checkManifest()) {
        return EXIT_FAILURE;
    };
#endif

    // Setup the default user directory.
    Filesystem::Init();

    QFontDatabase::addApplicationFont(":/fonts/Fontin-SmallCaps.ttf");

    QCommandLineParser parser;
    QCommandLineOption option_test("test");
    QCommandLineOption option_data_dir("data-dir", "Where to save Acquisition data.", "data-dir");
    QCommandLineOption option_log_level("log-level", "How much to log.", "log-level");
    QCommandLineOption option_crash("crash-test");
    parser.addOption(option_test);
    parser.addOption(option_data_dir);
    parser.addOption(option_log_level);
    parser.addOption(option_crash);
    parser.process(a);

    // Setup the data dir, which is where the log will be written.
    if (parser.isSet(option_data_dir)) {
        const QString datadir = QString(parser.value(option_data_dir));
        Filesystem::SetUserDir(datadir);
    };
    const QString sLogPath = QString(QDir(Filesystem::UserDir()).filePath("log.txt"));

    // The filename here needs to match the one in the application
    // object constructor. This is a design flaw. We need this object
    // very briefly for the log level setting.
    //
    // Perhaps it should live here and be passed into the Application
    // object construtor?
    QSettings settings(Filesystem::UserDir() + "/settings.ini", QSettings::IniFormat);

    // Determine the logging level.
    //
    // Start with the compile-time default for this build, but allow it to be
    // superceded by a command-line argument. If there's no command-line argument
    // check the settings.ini file.
    QsLogging::Level loglevel = DEFAULT_LOGLEVEL;
    if (parser.isSet(option_log_level)) {
        loglevel = Util::TextToLogLevel(parser.value(option_log_level));
    } else if (settings.contains("log_level")) {
        loglevel = Util::TextToLogLevel(settings.value("log_level").toString());
    };

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
    QLOG_INFO() << "Logging level is" << logger.loggingLevel();

    QLOG_TRACE() << "Checking for SSL support...";
    if (!QSslSocket::supportsSsl()) {
        FatalError("SSL support is missing. On Linux, the LD_LIBRARY_PATH must include OpenSSL 3.x shared libraries.");
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
    Application app;

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
