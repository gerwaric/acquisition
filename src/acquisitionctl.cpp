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
    parser.addPositionalArgument("command", "status, tabs, items, or item <id>.");

    QCommandLineOption data_dir_option("data-dir", "Acquisition data directory.", "data-dir");
    data_dir_option.setDefaultValue(default_data_dir);
    QCommandLineOption limit_option("limit", "Maximum items in one page (1-100).", "count");
    QCommandLineOption cursor_option("cursor", "Continue an items page.", "cursor");
    QCommandLineOption tab_option("tab", "Filter items by stable display-tab id.", "id");
    QCommandLineOption kind_option("kind", "Location kind for --tab: stash or character.", "kind");
    parser.addOption(data_dir_option);
    parser.addOption(limit_option);
    parser.addOption(cursor_option);
    parser.addOption(tab_option);
    parser.addOption(kind_option);
    parser.addOption(QCommandLineOption("json", "Emit machine-readable JSON (the default)."));
    parser.process(app);

    const QStringList positional = parser.positionalArguments();
    if (positional.isEmpty()) {
        QTextStream(stderr) << "acquisitionctl: a command is required" << Qt::endl;
        parser.showHelp(2);
    }

    const QString command = positional.front();
    const bool has_item_options = parser.isSet(limit_option) || parser.isSet(cursor_option)
                                  || parser.isSet(tab_option) || parser.isSet(kind_option);
    QJsonObject params;
    if (command == "status" || command == "tabs") {
        if (positional.size() != 1) {
            parser.showHelp(2);
        }
        if (has_item_options) {
            QTextStream(stderr) << "acquisitionctl: item options require the items command"
                                << Qt::endl;
            return 2;
        }
    } else if (command == "items") {
        if (positional.size() != 1) {
            parser.showHelp(2);
        }
        if (parser.isSet(limit_option)) {
            bool ok = false;
            const int limit = parser.value(limit_option).toInt(&ok);
            if (!ok || limit < 1 || limit > 100) {
                QTextStream(stderr) << "acquisitionctl: --limit must be between 1 and 100"
                                    << Qt::endl;
                return 2;
            }
            params.insert("limit", limit);
        }
        if (parser.isSet(cursor_option)) {
            if (parser.isSet(limit_option) || parser.isSet(tab_option)
                || parser.isSet(kind_option)) {
                QTextStream(stderr)
                    << "acquisitionctl: --cursor cannot be combined with item filters"
                    << Qt::endl;
                return 2;
            }
            params.insert("cursor", parser.value(cursor_option));
        }
        if (parser.isSet(tab_option)) {
            params.insert("tab_id", parser.value(tab_option));
        }
        if (parser.isSet(kind_option)) {
            params.insert("kind", parser.value(kind_option));
        }
        if (parser.isSet(tab_option) != parser.isSet(kind_option)) {
            QTextStream(stderr) << "acquisitionctl: --tab and --kind must be provided together"
                                << Qt::endl;
            return 2;
        }
    } else if (command == "item") {
        if (positional.size() != 2) {
            QTextStream(stderr) << "acquisitionctl: item requires an id" << Qt::endl;
            return 2;
        }
        if (has_item_options) {
            QTextStream(stderr) << "acquisitionctl: item options require the items command"
                                << Qt::endl;
            return 2;
        }
        params.insert("id", positional.at(1));
    } else {
        QTextStream(stderr) << "acquisitionctl: unknown command: " << command << Qt::endl;
        return 2;
    }

    const QString request_id = QUuid::createUuid().toString(QUuid::WithoutBraces);
    const QJsonObject request{{"protocol", control::PROTOCOL_VERSION},
                              {"request_id", request_id},
                              {"command", command},
                              {"params", params}};

    auto response = control::SendRequest(control::EndpointName(QDir(parser.value(data_dir_option))),
                                         request,
                                         2000);
    if (!response) {
        return PrintClientError(response.error());
    }

    QTextStream(stdout) << QJsonDocument(*response).toJson(QJsonDocument::Compact) << Qt::endl;
    return response->value("ok").toBool(false) ? 0 : 2;
}
