// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include <QCommandLineParser>
#include <QCoreApplication>
#include <QDir>
#include <QJsonDocument>
#include <QTextStream>
#include <QUuid>

#include "control/controlendpoint.h"
#include "control/controlprotocol.h"
#include "control/localcontrolclient.h"
#include "version_defines.h"

namespace {

    int PrintClientError(const control::ClientError &error)
    {
        const QJsonObject object{{"protocol", control::PROTOCOL_VERSION},
                                 {"ok", false},
                                 {"error",
                                  QJsonObject{{"code", error.code}, {"message", error.message}}}};
        QTextStream(stdout) << QJsonDocument(object).toJson(QJsonDocument::Compact) << Qt::endl;
        return error.code == "not_running" ? 3 : 4;
    }

} // namespace

int main(int argc, char *argv[])
{
    QCoreApplication app(argc, argv);
    QCoreApplication::setApplicationName(APP_NAME);
    QCoreApplication::setApplicationVersion(APP_VERSION_STRING);

    const QString default_data_dir = control::DefaultDataDirectory().absolutePath();

    QCommandLineParser parser;
    parser.setApplicationDescription("Control a running Acquisition application.");
    parser.addHelpOption();
    parser.addVersionOption();
    parser.addPositionalArgument("command", "Command to run (currently: status).");

    QCommandLineOption data_dir_option("data-dir", "Acquisition data directory.", "data-dir");
    data_dir_option.setDefaultValue(default_data_dir);
    parser.addOption(data_dir_option);
    parser.addOption(QCommandLineOption("json", "Emit machine-readable JSON (the default)."));
    parser.process(app);

    const QStringList positional = parser.positionalArguments();
    if (positional.size() != 1 || positional.front() != "status") {
        QTextStream(stderr) << "acquisitionctl: expected command 'status'" << Qt::endl;
        parser.showHelp(2);
    }

    const QString request_id = QUuid::createUuid().toString(QUuid::WithoutBraces);
    const QJsonObject request{{"protocol", control::PROTOCOL_VERSION},
                              {"request_id", request_id},
                              {"command", positional.front()},
                              {"params", QJsonObject{}}};

    auto response = control::SendRequest(control::EndpointName(QDir(parser.value(data_dir_option))),
                                         request,
                                         2000);
    if (!response) {
        return PrintClientError(response.error());
    }

    QTextStream(stdout) << QJsonDocument(*response).toJson(QJsonDocument::Compact) << Qt::endl;
    return response->value("ok").toBool(false) ? 0 : 2;
}
