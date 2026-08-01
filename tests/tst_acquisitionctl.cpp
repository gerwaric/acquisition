#include <QProcess>
#include <QtTest>

class AcquisitionCtlTest : public QObject
{
    Q_OBJECT
private slots:
    void rejectsOutOfRangeLimit();
    void rejectsOptionsForOtherCommands();
    void validatesRefreshArguments();
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

QTEST_GUILESS_MAIN(AcquisitionCtlTest)
#include "tst_acquisitionctl.moc"
