#include <QtTest>

#include "control/controlservice.h"

class ControlServiceTest : public QObject
{
    Q_OBJECT
private slots:
    void reportsPreLoginStatus();
    void rejectsUnknownCommand();
};

void ControlServiceTest::reportsPreLoginStatus()
{
    control::ControlService service("test-version");
    service.SetNeedsLogin();
    const QJsonObject response = service.Handle(control::Request{"request", "status", {}});

    QVERIFY(response.value("ok").toBool());
    QCOMPARE(response.value("request_id").toString(), "request");
    const QJsonObject result = response.value("result").toObject();
    QCOMPARE(result.value("application_version").toString(), "test-version");
    QCOMPARE(result.value("service_state").toString(), "needs_login");
    QCOMPARE(result.value("refresh_state").toString(), "unavailable");
    QVERIFY(!result.value("instance_id").toString().isEmpty());
}

void ControlServiceTest::rejectsUnknownCommand()
{
    control::ControlService service("test-version");
    const QJsonObject response = service.Handle(control::Request{"request", "nope", {}});

    QVERIFY(!response.value("ok").toBool(true));
    QCOMPARE(response.value("error").toObject().value("code").toString(), "unknown_command");
}

QTEST_GUILESS_MAIN(ControlServiceTest)
#include "tst_controlservice.moc"
