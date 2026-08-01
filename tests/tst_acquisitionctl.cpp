#include <QProcess>
#include <QtTest>

class AcquisitionCtlTest : public QObject
{
    Q_OBJECT
private slots:
    void rejectsOutOfRangeLimit();
    void rejectsOptionsForOtherCommands();
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
    QVERIFY(result.standard_error.contains("items command"));
}

QTEST_GUILESS_MAIN(AcquisitionCtlTest)
#include "tst_acquisitionctl.moc"
