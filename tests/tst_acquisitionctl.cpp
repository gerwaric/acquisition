#include <QJsonDocument>
#include <QProcess>
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
    void validatesRefreshArguments();
    void identifiesItselfInParserErrors();
    void statusRoundTrip();
};

namespace {

    struct Result
    {
        QProcess::ExitStatus status;
        int code;
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
        return Result{process.exitStatus(), process.exitCode(), process.readAllStandardError()};
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
    QVERIFY(result.standard_error.contains("do not apply"));
}

void AcquisitionCtlTest::validatesRefreshArguments()
{
    const Result missing_id = run({"refresh", "status"});
    QCOMPARE(missing_id.status, QProcess::NormalExit);
    QCOMPARE(missing_id.code, 2);

    const Result negative_timeout = run({"refresh", "wait", "operation", "--timeout", "-1"});
    QCOMPARE(negative_timeout.status, QProcess::NormalExit);
    QCOMPARE(negative_timeout.code, 2);
    QVERIFY(negative_timeout.standard_error.contains("zero or greater"));

    const Result ignored_timeout = run({"refresh", "status", "operation", "--timeout", "1"});
    QCOMPARE(ignored_timeout.status, QProcess::NormalExit);
    QCOMPARE(ignored_timeout.code, 2);
    QVERIFY(ignored_timeout.standard_error.contains("applies only"));
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

QTEST_GUILESS_MAIN(AcquisitionCtlTest)
#include "tst_acquisitionctl.moc"
