#include <QJsonDocument>
#include <QProcess>
#include <QStringList>
#include <QTemporaryDir>
#include <QtTest>

#include "control/controlprotocol.h"
#include "control/localcontrolserver.h"

class AcquisitionCtlTest : public QObject
{
    Q_OBJECT
private slots:
    void rejectsOutOfRangeLimit();
    void rejectsOptionsForOtherCommands();
    void validatesItemArguments();
    void validatesRefreshArguments();
    void invalidUsageDoesNotWriteStdout();
    void identifiesItselfInParserErrors();
    void statusRoundTrip();
    void waitTimeoutSendsOnlyStatusRequests();
};

namespace {

    struct Result
    {
        QProcess::ExitStatus status;
        int code;
        QByteArray standard_output;
        QByteArray standard_error;
    };

    Result run(const QStringList &arguments)
    {
        QProcess process;
        process.start(ACQUISITIONCTL_PATH, arguments);
        if (!process.waitForFinished(5000)) {
            process.kill();
            process.waitForFinished();
        }
        return Result{process.exitStatus(),
                      process.exitCode(),
                      process.readAllStandardOutput(),
                      process.readAllStandardError()};
    }

} // namespace

void AcquisitionCtlTest::rejectsOutOfRangeLimit()
{
    const Result result = run({"items", "--limit", "101"});
    QCOMPARE(result.status, QProcess::NormalExit);
    QCOMPARE(result.code, 2);
    QVERIFY(result.standard_error.contains("between 1 and 100"));
}

void AcquisitionCtlTest::rejectsOptionsForOtherCommands()
{
    const Result result = run({"tabs", "--tab", "stash", "--kind", "stash"});
    QCOMPARE(result.status, QProcess::NormalExit);
    QCOMPARE(result.code, 2);
    QVERIFY(result.standard_error.contains("invalid tabs arguments"));
}

void AcquisitionCtlTest::validatesItemArguments()
{
    const Result invalid_tab_limit = run({"tabs", "--limit", "0"});
    QCOMPARE(invalid_tab_limit.code, 2);
    QVERIFY(invalid_tab_limit.standard_output.isEmpty());

    const Result empty_cursor = run({"items", "--cursor", ""});
    QCOMPARE(empty_cursor.code, 2);
    QVERIFY(empty_cursor.standard_output.isEmpty());

    const Result empty_tab = run({"items", "--tab", "", "--kind", "stash"});
    QCOMPARE(empty_tab.code, 2);
    QVERIFY(empty_tab.standard_error.contains("--tab must not be empty"));

    const Result invalid_kind = run({"items", "--tab", "tab", "--kind", "invalid"});
    QCOMPARE(invalid_kind.code, 2);
    QVERIFY(invalid_kind.standard_error.contains("stash or character"));

    const Result empty_id = run({"item", ""});
    QCOMPARE(empty_id.code, 2);
    QVERIFY(empty_id.standard_error.contains("non-empty id"));
}

void AcquisitionCtlTest::validatesRefreshArguments()
{
    const Result missing_id = run({"refresh", "status"});
    QCOMPARE(missing_id.status, QProcess::NormalExit);
    QCOMPARE(missing_id.code, 2);

    const Result empty_id = run({"refresh", "status", ""});
    QCOMPARE(empty_id.code, 2);
    QVERIFY(empty_id.standard_error.contains("non-empty operation id"));

    const Result negative_timeout = run({"refresh", "wait", "operation", "--timeout", "-1"});
    QCOMPARE(negative_timeout.status, QProcess::NormalExit);
    QCOMPARE(negative_timeout.code, 2);
    QVERIFY(negative_timeout.standard_error.contains("zero or greater"));

    const Result ignored_timeout = run({"refresh", "status", "operation", "--timeout", "1"});
    QCOMPARE(ignored_timeout.status, QProcess::NormalExit);
    QCOMPARE(ignored_timeout.code, 2);
    QVERIFY(ignored_timeout.standard_error.contains("applies only"));
}

void AcquisitionCtlTest::invalidUsageDoesNotWriteStdout()
{
    const Result missing_command = run({});
    QCOMPARE(missing_command.code, 2);
    QVERIFY(missing_command.standard_output.isEmpty());
    QVERIFY(missing_command.standard_error.contains("command is required"));

    const Result missing_id = run({"refresh", "status"});
    QCOMPARE(missing_id.code, 2);
    QVERIFY(missing_id.standard_output.isEmpty());
    QVERIFY(missing_id.standard_error.contains("requires a non-empty operation id"));
}

void AcquisitionCtlTest::identifiesItselfInParserErrors()
{
    const Result result = run({"--definitely-invalid"});
    QCOMPARE(result.status, QProcess::NormalExit);
    QCOMPARE(result.code, 1);
    QVERIFY(result.standard_error.startsWith("acquisitionctl:"));
}

void AcquisitionCtlTest::statusRoundTrip()
{
    QTemporaryDir dir;
    QVERIFY(dir.isValid());
    control::LocalControlServer server;
    QString received_command;
    server.SetHandler([&received_command](const control::Request &request) {
        received_command = request.command;
        if (request.command != "status") {
            return control::Error(request.request_id, "wrong_command", "expected status");
        }
        return control::Success(request.request_id, QJsonObject{{"service_state", "ready"}});
    });
    QVERIFY2(server.Listen(QDir(dir.path())), qPrintable(server.ErrorString()));

    QProcess process;
    process.start(ACQUISITIONCTL_PATH, {"--data-dir", dir.path(), "status"});
    QTRY_COMPARE_WITH_TIMEOUT(process.state(), QProcess::NotRunning, 5000);
    QCOMPARE(process.exitStatus(), QProcess::NormalExit);
    QCOMPARE(process.exitCode(), 0);
    QCOMPARE(received_command, "status");
    QJsonParseError error;
    const QJsonDocument document = QJsonDocument::fromJson(process.readAllStandardOutput(), &error);
    QCOMPARE(error.error, QJsonParseError::NoError);
    QVERIFY(document.object().value("ok").toBool());
    QCOMPARE(document.object()
                 .value("result")
                 .toObject()
                 .value("service_state")
                 .toString(),
             "ready");
}

void AcquisitionCtlTest::waitTimeoutSendsOnlyStatusRequests()
{
    QTemporaryDir dir;
    QVERIFY(dir.isValid());
    control::LocalControlServer server;
    QStringList received_commands;
    server.SetHandler([&received_commands](const control::Request &request) {
        received_commands.push_back(request.command);
        if (request.command != "refresh.status"
            || request.params.value("operation_id").toString() != "operation") {
            return control::Error(request.request_id,
                                  "wrong_command",
                                  "expected refresh status");
        }
        return control::Success(
            request.request_id,
            QJsonObject{{"operation", QJsonObject{{"id", "operation"}, {"state", "running"}}}});
    });
    QVERIFY2(server.Listen(QDir(dir.path())), qPrintable(server.ErrorString()));

    QProcess process;
    process.start(ACQUISITIONCTL_PATH,
                  {"--data-dir", dir.path(), "refresh", "wait", "operation", "--timeout", "1"});
    QTRY_COMPARE_WITH_TIMEOUT(process.state(), QProcess::NotRunning, 3000);
    QCOMPARE(process.exitStatus(), QProcess::NormalExit);
    QCOMPARE(process.exitCode(), 7);
    QVERIFY(!received_commands.isEmpty());
    for (const QString &command : received_commands) {
        QCOMPARE(command, "refresh.status");
    }
    const QJsonObject output = QJsonDocument::fromJson(process.readAllStandardOutput()).object();
    QCOMPARE(output.value("error").toObject().value("code").toString(), "wait_timeout");
}

QTEST_GUILESS_MAIN(AcquisitionCtlTest)
#include "tst_acquisitionctl.moc"
