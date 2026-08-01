#include <QElapsedTimer>
#include <QJsonDocument>
#include <QLocalSocket>
#include <QTemporaryDir>
#include <QtTest>

#include "control/controlprotocol.h"
#include "control/localcontrolserver.h"

class LocalControlServerTest : public QObject
{
    Q_OBJECT
private slots:
    void roundTrip();
    void fragmentedRequest();
    void liveEndpointIsNotReplaced();
    void malformedClientDoesNotAffectAnother();
    void validationErrorsPreserveRequestId();
    void servesOnlyFirstPipelinedRequest();
    void ignoresTrailingPartialFrame();
    void idleConnectionTimesOut();
    void connectionLimitIsEnforced();
};

namespace {

    QJsonObject request(const QString &id = "request")
    {
        return QJsonObject{{"protocol", 1}, {"request_id", id}, {"command", "status"}};
    }

    QJsonObject readResponse(QLocalSocket &socket)
    {
        control::FrameDecoder decoder(control::MAX_RESPONSE_BYTES);
        QElapsedTimer timer;
        timer.start();
        while (timer.elapsed() < 1000) {
            QCoreApplication::processEvents();
            if (socket.bytesAvailable() > 0) {
                auto frames = decoder.Feed(socket.readAll());
                if (!frames) {
                    return {};
                }
                if (!frames->isEmpty()) {
                    const auto object = control::DecodeObject(frames->front());
                    return object ? *object : QJsonObject{};
                }
            }
            QTest::qWait(1);
        }
        return {};
    }

    void connectClient(QLocalSocket &socket, const QString &endpoint)
    {
        socket.connectToServer(endpoint);
        QTRY_COMPARE_WITH_TIMEOUT(socket.state(), QLocalSocket::ConnectedState, 1000);
    }

} // namespace

void LocalControlServerTest::roundTrip()
{
    QTemporaryDir dir;
    QVERIFY(dir.isValid());
    control::LocalControlServer server;
    server.SetHandler([](const control::Request &request) {
        return control::Success(request.request_id, QJsonObject{{"state", "ready"}});
    });
    QVERIFY2(server.Listen(QDir(dir.path())), qPrintable(server.ErrorString()));

    QLocalSocket socket;
    connectClient(socket, server.Endpoint());
    socket.write(control::EncodeFrame(request()));

    const QJsonObject response = readResponse(socket);
    QVERIFY(response.value("ok").toBool());
    QCOMPARE(response.value("request_id").toString(), "request");
    QCOMPARE(response.value("result").toObject().value("state").toString(), "ready");
}

void LocalControlServerTest::fragmentedRequest()
{
    QTemporaryDir dir;
    control::LocalControlServer server;
    int calls = 0;
    server.SetHandler([&](const control::Request &request) {
        ++calls;
        return control::Success(request.request_id);
    });
    QVERIFY(server.Listen(QDir(dir.path())));

    QLocalSocket socket;
    connectClient(socket, server.Endpoint());
    const QByteArray frame = control::EncodeFrame(request());
    for (char byte : frame) {
        socket.write(QByteArray(1, byte));
        QCoreApplication::processEvents();
    }

    const QJsonObject response = readResponse(socket);
    QVERIFY(response.value("ok").toBool());
    QCOMPARE(calls, 1);
}

void LocalControlServerTest::liveEndpointIsNotReplaced()
{
    QTemporaryDir dir;
    control::LocalControlServer first;
    first.SetHandler([](const control::Request &request) {
        return control::Success(request.request_id);
    });
    QVERIFY(first.Listen(QDir(dir.path())));

    control::LocalControlServer second;
    QVERIFY(!second.Listen(QDir(dir.path())));
    QVERIFY(first.IsListening());

    QLocalSocket socket;
    connectClient(socket, first.Endpoint());
    socket.write(control::EncodeFrame(request()));
    QVERIFY(readResponse(socket).value("ok").toBool());

    first.Close();
    QVERIFY2(second.Listen(QDir(dir.path())), qPrintable(second.ErrorString()));
}

void LocalControlServerTest::malformedClientDoesNotAffectAnother()
{
    QTemporaryDir dir;
    control::LocalControlServer server;
    server.SetHandler([](const control::Request &request) {
        return control::Success(request.request_id);
    });
    QVERIFY(server.Listen(QDir(dir.path())));

    QLocalSocket malformed;
    connectClient(malformed, server.Endpoint());
    malformed.write(QByteArray(4, '\0'));
    const QJsonObject error = readResponse(malformed);
    QVERIFY(!error.value("ok").toBool(true));

    QLocalSocket valid;
    connectClient(valid, server.Endpoint());
    valid.write(control::EncodeFrame(request("valid")));
    const QJsonObject response = readResponse(valid);
    QVERIFY(response.value("ok").toBool());
    QCOMPARE(response.value("request_id").toString(), "valid");
}

void LocalControlServerTest::validationErrorsPreserveRequestId()
{
    QTemporaryDir dir;
    control::LocalControlServer server;
    QVERIFY(server.Listen(QDir(dir.path())));

    QLocalSocket socket;
    connectClient(socket, server.Endpoint());
    const QJsonObject invalid{{"protocol", 2},
                              {"request_id", "version-request"},
                              {"command", "status"}};
    socket.write(control::EncodeFrame(invalid));
    const QJsonObject response = readResponse(socket);
    QCOMPARE(response.value("request_id").toString(), "version-request");
    QCOMPARE(response.value("error").toObject().value("code").toString(),
             "unsupported_version");
}

void LocalControlServerTest::servesOnlyFirstPipelinedRequest()
{
    QTemporaryDir dir;
    control::LocalControlServer server;
    int calls = 0;
    server.SetHandler([&](const control::Request &request) {
        ++calls;
        return control::Success(request.request_id);
    });
    QVERIFY(server.Listen(QDir(dir.path())));

    QLocalSocket socket;
    connectClient(socket, server.Endpoint());
    socket.write(control::EncodeFrame(request("one")) + control::EncodeFrame(request("two")));
    const QJsonObject response = readResponse(socket);
    QVERIFY(response.value("ok").toBool());
    QCOMPARE(response.value("request_id").toString(), "one");
    QCOMPARE(calls, 1);
}

void LocalControlServerTest::ignoresTrailingPartialFrame()
{
    QTemporaryDir dir;
    control::LocalControlServer server;
    int calls = 0;
    server.SetHandler([&](const control::Request &request) {
        ++calls;
        return control::Success(request.request_id);
    });
    QVERIFY(server.Listen(QDir(dir.path())));

    QLocalSocket socket;
    connectClient(socket, server.Endpoint());
    socket.write(control::EncodeFrame(request()) + QByteArray(1, '\0'));
    const QJsonObject response = readResponse(socket);
    QVERIFY(response.value("ok").toBool());
    QCOMPARE(response.value("request_id").toString(), "request");
    QCOMPARE(calls, 1);
}

void LocalControlServerTest::idleConnectionTimesOut()
{
    QTemporaryDir dir;
    control::LocalControlServer server(nullptr, 20);
    QVERIFY(server.Listen(QDir(dir.path())));

    QLocalSocket socket;
    connectClient(socket, server.Endpoint());
    socket.write("\0", 1);
    QTRY_COMPARE_WITH_TIMEOUT(socket.state(), QLocalSocket::UnconnectedState, 1000);
}

void LocalControlServerTest::connectionLimitIsEnforced()
{
    QTemporaryDir dir;
    control::LocalControlServer server(nullptr, 5000, 1);
    server.SetHandler([](const control::Request &request) {
        return control::Success(request.request_id);
    });
    QVERIFY(server.Listen(QDir(dir.path())));

    QLocalSocket first;
    connectClient(first, server.Endpoint());
    QTRY_VERIFY_WITH_TIMEOUT(server.IsListening(), 1000);

    QLocalSocket second;
    second.connectToServer(server.Endpoint());
    QTRY_COMPARE_WITH_TIMEOUT(second.state(), QLocalSocket::UnconnectedState, 500);

    first.write(control::EncodeFrame(request()));
    QVERIFY(readResponse(first).value("ok").toBool());
}

QTEST_GUILESS_MAIN(LocalControlServerTest)
#include "tst_localcontrolserver.moc"
