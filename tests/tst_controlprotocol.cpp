#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QTemporaryDir>
#include <QtTest>

#ifndef Q_OS_WIN
#include <pwd.h>
#include <sys/un.h>
#include <unistd.h>
#endif

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
    void canonicalizesMissingDirectoryThroughSymlink();
    void unixRootSelection();
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
    const QStringList endpoint_candidates = control::EndpointNames(QDir(first.path()));
    QVERIFY(!first_endpoint.isEmpty());
    QVERIFY(!endpoint_candidates.isEmpty());
    QCOMPARE(endpoint_candidates.front(), first_endpoint);
    QStringList unique_candidates = endpoint_candidates;
    unique_candidates.removeDuplicates();
    QCOMPARE(endpoint_candidates.size(), unique_candidates.size());
    QVERIFY(first_endpoint != control::EndpointName(QDir(second.path())));
    const QString lock_path = control::EndpointLockPath(QDir(first.path()));
    QVERIFY(QFileInfo(lock_path).isAbsolute());
#ifndef Q_OS_WIN
    const QByteArray old_runtime = qgetenv("XDG_RUNTIME_DIR");
    const QByteArray old_home = qgetenv("HOME");
    qputenv("XDG_RUNTIME_DIR", second.path().toUtf8());
    qputenv("HOME", second.path().toUtf8());
    QCOMPARE(control::EndpointName(QDir(first.path())), first_endpoint);
    if (old_runtime.isNull()) {
        qunsetenv("XDG_RUNTIME_DIR");
    } else {
        qputenv("XDG_RUNTIME_DIR", old_runtime);
    }
    if (old_home.isNull()) {
        qunsetenv("HOME");
    } else {
        qputenv("HOME", old_home);
    }

    QCOMPARE(lock_path, endpoint_candidates.back() + ".lock");
    for (const QString &endpoint : endpoint_candidates) {
        QVERIFY(QFile::encodeName(endpoint).size()
                < qsizetype(sizeof(sockaddr_un::sun_path)));
    }
#ifdef Q_OS_MACOS
    const size_t runtime_size = confstr(_CS_DARWIN_USER_TEMP_DIR, nullptr, 0);
    QVERIFY(runtime_size > 0);
    QByteArray runtime_buffer(qsizetype(runtime_size), '\0');
    QVERIFY(confstr(_CS_DARWIN_USER_TEMP_DIR,
                    runtime_buffer.data(),
                    runtime_size)
            > 0);
    const passwd *user = getpwuid(getuid());
    QVERIFY(user && user->pw_dir);
    const QString expected_root = control::detail::SelectUnixControlRoot(
        QString::fromLocal8Bit(runtime_buffer.constData()),
        QString::fromLocal8Bit(user->pw_dir));
    QCOMPARE(QFileInfo(first_endpoint).absolutePath(), QDir(expected_root).absolutePath());
#elif defined(Q_OS_LINUX)
    const QString runtime_root = QString("/run/user/%1").arg(qulonglong(getuid()));
    const passwd *user = getpwuid(getuid());
    QVERIFY(user && user->pw_dir);
    const QString expected_root = control::detail::SelectUnixControlRoot(
        runtime_root,
        QString::fromLocal8Bit(user->pw_dir));
    QCOMPARE(QFileInfo(first_endpoint).absolutePath(), QDir(expected_root).absolutePath());
#endif
#endif
}

void ControlProtocolTest::canonicalizesMissingDirectoryThroughSymlink()
{
#ifndef Q_OS_WIN
    QTemporaryDir parent;
    QTemporaryDir target;
    QVERIFY(parent.isValid());
    QVERIFY(target.isValid());
    const QString link = parent.filePath("linked-parent");
    QVERIFY(QFile::link(target.path(), link));
    const QDir missing(QDir(link).filePath("missing/data"));

    const QString before = control::CanonicalDataDirectory(missing);
    QVERIFY(QDir().mkpath(missing.absolutePath()));
    QCOMPARE(control::CanonicalDataDirectory(missing), before);
    QCOMPARE(before,
             QDir(QFileInfo(target.path()).canonicalFilePath()).filePath("missing/data"));
#else
    QSKIP("Symlink creation is not generally available to Windows tests");
#endif
}

void ControlProtocolTest::unixRootSelection()
{
#ifndef Q_OS_WIN
    QTemporaryDir runtime("/tmp/acq-runtime-XXXXXX");
    QTemporaryDir home("/tmp/acq-home-XXXXXX");
    QTemporaryDir insecure_home("/tmp/acq-insecure-XXXXXX");
    QVERIFY(runtime.isValid());
    QVERIFY(home.isValid());
    QVERIFY(insecure_home.isValid());
    const auto private_permissions = QFileDevice::ReadOwner | QFileDevice::WriteOwner
                                     | QFileDevice::ExeOwner;
    QVERIFY(QFile::setPermissions(runtime.path(), private_permissions));
    QVERIFY(QFile::setPermissions(home.path(), private_permissions));

    QCOMPARE(control::detail::SelectUnixControlRoot(runtime.path(), home.path()),
             QDir(runtime.path()).filePath("acquisition-control"));
    QCOMPARE(control::detail::SelectUnixControlRoot(runtime.filePath("missing"), home.path()),
             QDir(home.path()).filePath(".acquisition-control"));

    QVERIFY(QFile::setPermissions(insecure_home.path(),
                                  private_permissions | QFileDevice::WriteGroup
                                      | QFileDevice::ExeGroup));
    QCOMPARE(control::detail::SelectUnixControlRoot(runtime.path(),
                                                    insecure_home.path()),
             QDir(runtime.path()).filePath("acquisition-control"));
    QVERIFY(control::detail::SelectUnixControlRoot(runtime.filePath("missing"),
                                                   insecure_home.path())
                .isEmpty());

    QString short_name_home = home.path();
    while (QFile::encodeName(QDir(short_name_home).filePath(
                                 ".acquisition-control/socket-0000000000000000"))
               .size()
           < qsizetype(sizeof(sockaddr_un::sun_path))) {
        short_name_home = QDir(short_name_home).filePath("abc");
    }
    QVERIFY(QDir().mkpath(short_name_home));
    QCOMPARE(control::detail::SelectUnixControlRoot(runtime.filePath("missing"),
                                                    short_name_home),
             QDir(short_name_home).filePath(".acq-control"));

    QString long_home = short_name_home;
    while (QFile::encodeName(QDir(long_home).filePath(
                                 ".acq-control/socket-0000000000000000"))
               .size()
           < qsizetype(sizeof(sockaddr_un::sun_path))) {
        long_home = QDir(long_home).filePath("abc");
    }
    QVERIFY(QDir().mkpath(long_home));
    QVERIFY(control::detail::SelectUnixControlRoot(runtime.filePath("missing"), long_home)
                .isEmpty());
#else
    QSKIP("Unix endpoint roots are not used on Windows");
#endif
}

QTEST_GUILESS_MAIN(ControlProtocolTest)
#include "tst_controlprotocol.moc"
