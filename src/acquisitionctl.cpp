// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include <QCommandLineParser>
#include <QCoreApplication>
#include <QDir>
#include <QElapsedTimer>
#include <QJsonDocument>
#include <QTextStream>
#include <QThread>
#include <QUuid>

#include <algorithm>

#include "control/controlendpoint.h"
#include "control/controlprotocol.h"
#include "control/localcontrolclient.h"
#include "version_defines.h"

namespace {

    void PrintJson(const QJsonObject &object)
    {
        QTextStream(stdout) << QJsonDocument(object).toJson(QJsonDocument::Compact) << Qt::endl;
    }

    int PrintClientError(const control::ClientError &error)
    {
        const QJsonObject object{{"protocol", control::PROTOCOL_VERSION},
                                 {"ok", false},
                                 {"error",
                                  QJsonObject{{"code", error.code}, {"message", error.message}}}};
        PrintJson(object);
        if (error.code == "not_running") {
            return 3;
        }
        if (error.code == "wait_timeout") {
            return 7;
        }
        return 4;
    }

    int ResultExitCode(const QString &command, const QJsonObject &response)
    {
        if (!response.value("ok").toBool(false)) {
            return 2;
        }
        const QJsonObject result = response.value("result").toObject();
        if (command == "refresh.start" && !result.value("accepted").toBool()) {
            return 5;
        }
        return 0;
    }

} // namespace

int main(int argc, char *argv[])
{
    QCoreApplication app(argc, argv);
    QCoreApplication::setApplicationName("acquisitionctl");
    QCoreApplication::setApplicationVersion(APP_VERSION_STRING);

    // Shared helper uses GenericDataLocation + "acquisition" explicitly, so
    // the parser's application name cannot redirect the GUI data directory.
    const QString default_data_dir = control::DefaultDataDirectory().absolutePath();

    QCommandLineParser parser;
    parser.setApplicationDescription("Control a running Acquisition application.");
    parser.addHelpOption();
    parser.addVersionOption();
    parser.addPositionalArgument(
        "command",
        "status, tabs, items, item <id>, or refresh <start|status|wait> [operation-id].");

    QCommandLineOption data_dir_option("data-dir", "Acquisition data directory.", "data-dir");
    data_dir_option.setDefaultValue(default_data_dir);
    QCommandLineOption limit_option("limit", "Maximum entries in a tabs/items page (1-100).", "count");
    QCommandLineOption cursor_option("cursor", "Continue a tabs/items page.", "cursor");
    QCommandLineOption tab_option("tab", "Filter items by stable display-tab id.", "id");
    QCommandLineOption kind_option("kind", "Location kind for --tab: stash or character.", "kind");
    QCommandLineOption timeout_option(
        "timeout",
        "Stop observing refresh wait after this many seconds; zero waits indefinitely.",
        "seconds",
        "0");
    parser.addOption(data_dir_option);
    parser.addOption(limit_option);
    parser.addOption(cursor_option);
    parser.addOption(tab_option);
    parser.addOption(kind_option);
    parser.addOption(timeout_option);
    parser.addOption(QCommandLineOption("json", "Emit machine-readable JSON (the default)."));
    parser.process(app);
    if (parser.value(data_dir_option).isEmpty()) {
        QTextStream(stderr) << "acquisitionctl: --data-dir must not be empty" << Qt::endl;
        return 2;
    }

    const QStringList positional = parser.positionalArguments();
    if (positional.isEmpty()) {
        QTextStream(stderr) << "acquisitionctl: a command is required; use --help for usage"
                            << Qt::endl;
        return 2;
    }

    const bool has_item_options = parser.isSet(limit_option) || parser.isSet(cursor_option)
                                  || parser.isSet(tab_option) || parser.isSet(kind_option);
    QString command = positional.front();
    QJsonObject params;
    bool wait_for_refresh = false;
    int wait_timeout_seconds = 0;

    if (command == "status") {
        if (positional.size() != 1) {
            QTextStream(stderr) << "acquisitionctl: status does not accept arguments"
                                << Qt::endl;
            return 2;
        }
        if (has_item_options || parser.isSet(timeout_option)) {
            QTextStream(stderr) << "acquisitionctl: those options do not apply to status"
                                << Qt::endl;
            return 2;
        }
    } else if (command == "tabs") {
        if (positional.size() != 1 || parser.isSet(timeout_option)
            || parser.isSet(tab_option) || parser.isSet(kind_option)) {
            QTextStream(stderr) << "acquisitionctl: invalid tabs arguments; use --help for usage"
                                << Qt::endl;
            return 2;
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
            if (parser.isSet(limit_option) || parser.value(cursor_option).isEmpty()) {
                QTextStream(stderr)
                    << "acquisitionctl: tab cursor must be non-empty and used without --limit"
                    << Qt::endl;
                return 2;
            }
            params.insert("cursor", parser.value(cursor_option));
        }
    } else if (command == "items") {
        if (positional.size() != 1 || parser.isSet(timeout_option)) {
            QTextStream(stderr) << "acquisitionctl: invalid items arguments; use --help for usage"
                                << Qt::endl;
            return 2;
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
            if (parser.value(cursor_option).isEmpty()) {
                QTextStream(stderr) << "acquisitionctl: --cursor must not be empty" << Qt::endl;
                return 2;
            }
            params.insert("cursor", parser.value(cursor_option));
        }
        if (parser.isSet(tab_option) != parser.isSet(kind_option)) {
            QTextStream(stderr) << "acquisitionctl: --tab and --kind must be provided together"
                                << Qt::endl;
            return 2;
        }
        if (parser.isSet(tab_option)) {
            const QString tab_id = parser.value(tab_option);
            const QString kind = parser.value(kind_option);
            if (tab_id.isEmpty()) {
                QTextStream(stderr) << "acquisitionctl: --tab must not be empty" << Qt::endl;
                return 2;
            }
            if (kind != "stash" && kind != "character") {
                QTextStream(stderr) << "acquisitionctl: --kind must be stash or character"
                                    << Qt::endl;
                return 2;
            }
            params.insert("tab_id", tab_id);
            params.insert("kind", kind);
        }
    } else if (command == "item") {
        if (positional.size() != 2 || positional.at(1).isEmpty()) {
            QTextStream(stderr) << "acquisitionctl: item requires a non-empty id" << Qt::endl;
            return 2;
        }
        if (has_item_options || parser.isSet(timeout_option)) {
            QTextStream(stderr) << "acquisitionctl: those options do not apply to item"
                                << Qt::endl;
            return 2;
        }
        params.insert("id", positional.at(1));
    } else if (command == "refresh") {
        if (has_item_options || positional.size() < 2 || positional.size() > 3) {
            QTextStream(stderr) << "acquisitionctl: invalid refresh arguments; use --help for usage"
                                << Qt::endl;
            return 2;
        }
        const QString action = positional.at(1);
        if (action == "start") {
            if (positional.size() != 2 || parser.isSet(timeout_option)) {
                QTextStream(stderr) << "acquisitionctl: refresh start does not accept arguments"
                                    << Qt::endl;
                return 2;
            }
            command = "refresh.start";
        } else if (action == "status" || action == "wait") {
            if (positional.size() != 3 || positional.at(2).isEmpty()) {
                QTextStream(stderr) << "acquisitionctl: refresh " << action
                                    << " requires a non-empty operation id" << Qt::endl;
                return 2;
            }
            command = "refresh.status";
            params.insert("operation_id", positional.at(2));
            wait_for_refresh = action == "wait";
            if (!wait_for_refresh && parser.isSet(timeout_option)) {
                QTextStream(stderr) << "acquisitionctl: --timeout applies only to refresh wait"
                                    << Qt::endl;
                return 2;
            }
            if (wait_for_refresh) {
                bool timeout_ok = false;
                wait_timeout_seconds = parser.value(timeout_option).toInt(&timeout_ok);
                if (!timeout_ok || wait_timeout_seconds < 0) {
                    QTextStream(stderr) << "acquisitionctl: --timeout must be zero or greater"
                                        << Qt::endl;
                    return 2;
                }
            }
        } else {
            QTextStream(stderr) << "acquisitionctl: unknown refresh action: " << action
                                << Qt::endl;
            return 2;
        }
    } else {
        QTextStream(stderr) << "acquisitionctl: unknown command: " << command << Qt::endl;
        return 2;
    }

    const QString endpoint = control::EndpointName(QDir(parser.value(data_dir_option)));
    const auto send = [&endpoint](const QString &request_command,
                                  const QJsonObject &request_params,
                                  int timeout_ms,
                                  QString request_id = {}) {
        if (request_id.isEmpty()) {
            request_id = QUuid::createUuid().toString(QUuid::WithoutBraces);
        }
        const QJsonObject request{{"protocol", control::PROTOCOL_VERSION},
                                  {"request_id", request_id},
                                  {"command", request_command},
                                  {"params", request_params}};
        return control::SendRequest(endpoint, request, timeout_ms);
    };

    // refresh start is always in this branch; wait_for_refresh is set only by
    // the separate `refresh wait <operation-id>` status-polling action.
    if (!wait_for_refresh) {
        const QString request_id = QUuid::createUuid().toString(QUuid::WithoutBraces);
        auto response = send(command, params, 2000, request_id);
        if (!response && command == "refresh.start"
            && (response.error().code == "timeout"
                || response.error().code == "transport_error")) {
            response = send(command, params, 2000, request_id);
        }
        if (!response) {
            if (command == "refresh.start"
                && (response.error().code == "timeout"
                    || response.error().code == "transport_error")) {
                QJsonObject unconfirmed = control::Error(
                    request_id,
                    "start_unconfirmed",
                    "refresh start was not confirmed; query the operation id before retrying");
                QJsonObject error = unconfirmed.value("error").toObject();
                error.insert("operation_id", request_id);
                unconfirmed.insert("error", error);
                PrintJson(unconfirmed);
                return 4;
            }
            return PrintClientError(response.error());
        }
        PrintJson(*response);
        return ResultExitCode(command, *response);
    }

    QElapsedTimer wait_timer;
    wait_timer.start();
    while (true) {
        int request_timeout_ms = 2000;
        if (wait_timeout_seconds > 0) {
            const qint64 remaining = qint64(wait_timeout_seconds) * 1000 - wait_timer.elapsed();
            if (remaining <= 0) {
                return PrintClientError(
                    control::ClientError{"wait_timeout", "stopped waiting; the refresh continues"});
            }
            request_timeout_ms = int(std::min<qint64>(remaining, request_timeout_ms));
        }
        auto response = send("refresh.status", params, request_timeout_ms);
        if (!response) {
            if (wait_timeout_seconds > 0
                && wait_timer.elapsed() >= qint64(wait_timeout_seconds) * 1000) {
                return PrintClientError(
                    control::ClientError{"wait_timeout", "stopped waiting; the refresh continues"});
            }
            return PrintClientError(response.error());
        }
        if (!response->value("ok").toBool(false)) {
            PrintJson(*response);
            return 2;
        }
        const QJsonObject operation = response->value("result")
                                          .toObject()
                                          .value("operation")
                                          .toObject();
        const QString state = operation.value("state").toString();
        if (state == "completed" || state == "failed") {
            PrintJson(*response);
            if (state == "failed") {
                return 5;
            }
            const bool clean = operation.value("outcome").toObject().value("clean").toBool();
            return clean ? 0 : 6;
        }
        int sleep_ms = 500;
        if (wait_timeout_seconds > 0) {
            const qint64 remaining = qint64(wait_timeout_seconds) * 1000 - wait_timer.elapsed();
            if (remaining <= 0) {
                return PrintClientError(
                    control::ClientError{"wait_timeout", "stopped waiting; the refresh continues"});
            }
            sleep_ms = int(std::min<qint64>(remaining, sleep_ms));
        }
        QThread::msleep(static_cast<unsigned long>(sleep_ms));
    }
}
