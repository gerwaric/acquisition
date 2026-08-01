#include <QFileInfo>
#include <QJsonDocument>
#include <QStandardPaths>
#include <QTemporaryDir>
#include <QtTest>

#include "control/controlendpoint.h"
#include "control/controlprotocol.h"

class ControlProtocolTest : public QObject
{
    Q_OBJECT
private slots:
    void fragmentedFrame();
    void multipleFrames();
    void rejectsInvalidLengths();
    void validatesRequests();
    void responseEnvelope();
    void endpointIsUserScoped();
};

void ControlProtocolTest::fragmentedFrame()
{
    const QJsonObject object{{"protocol", 1}, {"request_id", "one"}, {"command", "status"}};
    const QByteArray encoded = control::EncodeFrame(object);

    for (qsizetype split = 0; split < encoded.size(); ++split) {
        control::FrameDecoder decoder(control::MAX_REQUEST_BYTES);
        auto first = decoder.Feed(encoded.first(split));
        QVERIFY(first);
        QVERIFY(first->isEmpty());
        auto second = decoder.Feed(encoded.sliced(split));
        QVERIFY(second);
        QCOMPARE(second->size(), 1);
        QCOMPARE(control::DecodeObject(second->front()).value(), object);
    }
}

void ControlProtocolTest::multipleFrames()
{
    const QJsonObject first{{"value", 1}};
    const QJsonObject second{{"value", 2}};
    control::FrameDecoder decoder(control::MAX_REQUEST_BYTES);
    auto decoded = decoder.Feed(control::EncodeFrame(first) + control::EncodeFrame(second));
    QVERIFY(decoded);
    QCOMPARE(decoded->size(), 2);
    QCOMPARE(control::DecodeObject(decoded->at(0)).value(), first);
    QCOMPARE(control::DecodeObject(decoded->at(1)).value(), second);
}

void ControlProtocolTest::rejectsInvalidLengths()
{
    control::FrameDecoder empty_decoder(control::MAX_REQUEST_BYTES);
    const QByteArray empty_header(4, '\0');
    auto empty = empty_decoder.Feed(empty_header);
    QVERIFY(!empty);
    QCOMPARE(empty.error().code, "invalid_frame");

    control::FrameDecoder large_decoder(16);
    const QByteArray large_header{"\0\0\0\x11", 4};
    auto large = large_decoder.Feed(large_header);
    QVERIFY(!large);
    QCOMPARE(large.error().code, "frame_too_large");
}

void ControlProtocolTest::validatesRequests()
{
    const QByteArray valid = QJsonDocument(
                                 QJsonObject{{"protocol", 1},
                                             {"request_id", "request"},
                                             {"command", "status"},
                                             {"params", QJsonObject{}}})
                                 .toJson(QJsonDocument::Compact);
    const auto request = control::DecodeRequest(valid);
    QVERIFY(request);
    QCOMPARE(request->request_id, "request");
    QCOMPARE(request->command, "status");

    auto malformed = control::DecodeRequest("{");
    QVERIFY(!malformed);
    QCOMPARE(malformed.error().code, "invalid_json");

    auto wrong_version = control::DecodeRequest(
        R"({"protocol":2,"request_id":"request","command":"status"})");
    QVERIFY(!wrong_version);
    QCOMPARE(wrong_version.error().code, "unsupported_version");

    auto fractional_version = control::DecodeRequest(
        R"({"protocol":1.5,"request_id":"request","command":"status"})");
    QVERIFY(!fractional_version);
    QCOMPARE(fractional_version.error().code, "unsupported_version");

    auto missing_id = control::DecodeRequest(R"({"protocol":1,"command":"status"})");
    QVERIFY(!missing_id);
    QCOMPARE(missing_id.error().code, "invalid_request");
}

void ControlProtocolTest::responseEnvelope()
{
    const QJsonObject success = control::Success("request", QJsonObject{{"ready", true}});
    QCOMPARE(success.value("protocol").toInt(), 1);
    QCOMPARE(success.value("request_id").toString(), "request");
    QVERIFY(success.value("ok").toBool());
    QVERIFY(success.value("result").toObject().value("ready").toBool());

    const QJsonObject error = control::Error("request", "bad", "no");
    QVERIFY(!error.value("ok").toBool(true));
    QCOMPARE(error.value("error").toObject().value("code").toString(), "bad");

    const QByteArray encoded_success = QJsonDocument(success).toJson(QJsonDocument::Compact);
    QVERIFY(control::DecodeResponse(encoded_success, "request"));
    auto wrong_id = control::DecodeResponse(encoded_success, "different");
    QVERIFY(!wrong_id);
    QCOMPARE(wrong_id.error().code, "invalid_response");

    auto missing_envelope = control::DecodeResponse(R"({"ok":true})", "request");
    QVERIFY(!missing_envelope);
    QCOMPARE(missing_envelope.error().code, "unsupported_version");

    auto fractional_version = control::DecodeResponse(
        R"({"protocol":1.5,"request_id":"request","ok":true,"result":{}})",
        "request");
    QVERIFY(!fractional_version);
    QCOMPARE(fractional_version.error().code, "unsupported_version");
}

void ControlProtocolTest::endpointIsUserScoped()
{
    QTemporaryDir first;
    QTemporaryDir second;
    QVERIFY(first.isValid());
    QVERIFY(second.isValid());
    const QString first_endpoint = control::EndpointName(QDir(first.path()));
    QVERIFY(!first_endpoint.isEmpty());
    QVERIFY(first_endpoint != control::EndpointName(QDir(second.path())));
    const QString lock_path = control::EndpointLockPath(QDir(first.path()));
    QVERIFY(QFileInfo(lock_path).isAbsolute());
#ifndef Q_OS_WIN
    QCOMPARE(QFileInfo(first_endpoint).absolutePath(),
             QStandardPaths::writableLocation(QStandardPaths::RuntimeLocation));
#endif
}

QTEST_GUILESS_MAIN(ControlProtocolTest)
#include "tst_controlprotocol.moc"
