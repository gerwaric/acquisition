// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/localcontrolserver.h"

#include <QAbstractSocket>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QLocalSocket>
#include <QLocale>
#include <QtNumeric>
#include <QTimer>

#include <exception>

#ifdef Q_OS_WIN
#include <qt_windows.h>
#else
#include <cerrno>
#include <fcntl.h>
#include <sys/file.h>
#include <sys/stat.h>
#include <unistd.h>
#endif

#include "control/controlendpoint.h"

namespace control {

namespace {

    bool AddEstimatedBytes(qsizetype &used, qsizetype amount, qsizetype maximum)
    {
        if (amount < 0 || used > maximum - amount) {
            return false;
        }
        used += amount;
        return true;
    }

    bool EstimateJsonString(const QString &text, qsizetype &used, qsizetype maximum)
    {
        if (!AddEstimatedBytes(used, 2, maximum)) {
            return false;
        }
        for (qsizetype index = 0; index < text.size(); ++index) {
            const ushort code = text.at(index).unicode();
            qsizetype bytes = 0;
            if (code == '"' || code == '\\' || code == '\b' || code == '\f'
                || code == '\n' || code == '\r' || code == '\t') {
                bytes = 2;
            } else if (code < 0x20 || code == 0x2028 || code == 0x2029) {
                bytes = 6;
            } else if (QChar::isHighSurrogate(code) && index + 1 < text.size()
                       && QChar::isLowSurrogate(text.at(index + 1).unicode())) {
                bytes = 4;
                ++index;
            } else if (QChar::isSurrogate(code)) {
                // Qt replaces an unpaired surrogate; six bytes remains a safe
                // upper bound regardless of replacement escaping.
                bytes = 6;
            } else if (code < 0x80) {
                bytes = 1;
            } else if (code < 0x800) {
                bytes = 2;
            } else {
                bytes = 3;
            }
            if (!AddEstimatedBytes(used, bytes, maximum)) {
                return false;
            }
        }
        return true;
    }

    bool EstimateJson(const QJsonValue &value, qsizetype &used, qsizetype maximum)
    {
        switch (value.type()) {
        case QJsonValue::Null:
        case QJsonValue::Undefined:
            return AddEstimatedBytes(used, 4, maximum);
        case QJsonValue::Bool:
            return AddEstimatedBytes(used, value.toBool() ? 4 : 5, maximum);
        case QJsonValue::Double: {
            const double number = value.toDouble();
            const qsizetype bytes = qIsFinite(number)
                                        ? QByteArray::number(number,
                                                             'g',
                                                             QLocale::FloatingPointShortest)
                                              .size()
                                        : 4; // QJsonDocument writes non-finite values as null.
            return AddEstimatedBytes(used, bytes, maximum);
        }
        case QJsonValue::String:
            return EstimateJsonString(value.toString(), used, maximum);
        case QJsonValue::Array: {
            if (!AddEstimatedBytes(used, 2, maximum)) {
                return false;
            }
            const QJsonArray array = value.toArray();
            for (qsizetype index = 0; index < array.size(); ++index) {
                if ((index > 0 && !AddEstimatedBytes(used, 1, maximum))
                    || !EstimateJson(array.at(index), used, maximum)) {
                    return false;
                }
            }
            return true;
        }
        case QJsonValue::Object: {
            if (!AddEstimatedBytes(used, 2, maximum)) {
                return false;
            }
            const QJsonObject object = value.toObject();
            qsizetype index = 0;
            for (auto entry = object.begin(); entry != object.end(); ++entry, ++index) {
                if ((index > 0 && !AddEstimatedBytes(used, 1, maximum))
                    || !EstimateJsonString(entry.key(), used, maximum)
                    || !AddEstimatedBytes(used, 1, maximum)
                    || !EstimateJson(entry.value(), used, maximum)) {
                    return false;
                }
            }
            return true;
        }
        }
        return false;
    }

    class BindLock
    {
    public:
        ~BindLock()
        {
#ifdef Q_OS_WIN
            if (m_acquired) {
                ReleaseMutex(m_handle);
            }
            if (m_handle) {
                CloseHandle(m_handle);
            }
#else
            if (m_fd >= 0) {
                ::flock(m_fd, LOCK_UN);
                ::close(m_fd);
            }
#endif
        }

        bool TryLock(const QString &path, const QString &endpoint)
        {
#ifdef Q_OS_WIN
            const QString name = "Global\\acquisition-control-bind-" + endpoint;
            m_handle = CreateMutexW(nullptr,
                                    FALSE,
                                    reinterpret_cast<LPCWSTR>(name.utf16()));
            if (!m_handle) {
                return false;
            }
            const DWORD result = WaitForSingleObject(m_handle, 0);
            m_acquired = result == WAIT_OBJECT_0 || result == WAIT_ABANDONED_0;
            return m_acquired;
#else
            Q_UNUSED(endpoint);
            const QByteArray encoded = QFile::encodeName(path);
            m_fd = ::open(encoded.constData(), O_CREAT | O_RDWR, S_IRUSR | S_IWUSR);
            if (m_fd < 0) {
                return false;
            }
            if (::flock(m_fd, LOCK_EX | LOCK_NB) != 0) {
                ::close(m_fd);
                m_fd = -1;
                return false;
            }
            return true;
#endif
        }

    private:
#ifdef Q_OS_WIN
        HANDLE m_handle{nullptr};
        bool m_acquired{false};
#else
        int m_fd{-1};
#endif
    };

#ifndef Q_OS_WIN
    bool SecureControlDirectory(const QString &path)
    {
        const QByteArray encoded = QFile::encodeName(path);
        if (::mkdir(encoded.constData(), S_IRWXU) != 0 && errno != EEXIST) {
            return false;
        }
        struct stat status {};
        if (::lstat(encoded.constData(), &status) != 0 || !S_ISDIR(status.st_mode)
            || status.st_uid != getuid()) {
            return false;
        }
        return ::chmod(encoded.constData(), S_IRWXU) == 0;
    }
#endif

} // namespace

std::optional<qsizetype> JsonSizeWithinLimit(const QJsonValue &value, qsizetype maximum)
{
    qsizetype estimated = 0;
    if (!EstimateJson(value, estimated, maximum)) {
        return std::nullopt;
    }
    return estimated;
}

LocalControlServer::LocalControlServer(QObject *parent,
                                       int request_timeout_ms,
                                       qsizetype maximum_connections)
    : QObject(parent)
    , m_request_timeout_ms(request_timeout_ms)
    , m_maximum_connections(maximum_connections)
{
    m_server.setSocketOptions(QLocalServer::UserAccessOption);
    connect(&m_server, &QLocalServer::newConnection, this, &LocalControlServer::AcceptConnections);
}

LocalControlServer::~LocalControlServer()
{
    Close();
}

bool LocalControlServer::Listen(const QDir &data_directory)
{
    Close();
    m_endpoint = EndpointName(data_directory);
    m_error_string.clear();
    m_owner_conflict = false;
    if (m_endpoint.isEmpty()) {
        m_error_string = "a private per-user runtime directory is unavailable";
        return false;
    }
#ifndef Q_OS_WIN
    const QString endpoint_directory = QFileInfo(m_endpoint).absolutePath();
    if (!SecureControlDirectory(endpoint_directory)) {
        m_error_string = "the private control directory could not be secured";
        return false;
    }
#endif

    const QString lock_path = EndpointLockPath(data_directory);
    // The lock parent is created on every platform, including Windows where
    // it lives under the data directory rather than beside the named pipe.
    if (lock_path.isEmpty()
        || !QDir().mkpath(QFileInfo(lock_path).absolutePath())) {
        m_error_string = "a writable control-lock directory is unavailable";
        return false;
    }
    BindLock bind_lock;
    if (!bind_lock.TryLock(lock_path + ".native", m_endpoint)) {
        m_owner_conflict = true;
        m_error_string = "another Acquisition process is binding the control endpoint";
        return false;
    }

    // QLocalServer may remove an existing Unix socket before bind, so probe
    // before listen as well as before any explicit stale-endpoint removal.
    QLocalSocket probe;
    probe.connectToServer(m_endpoint);
    if (probe.waitForConnected(250)) {
        probe.abort();
        m_owner_conflict = true;
        m_error_string = "another Acquisition process owns the control endpoint";
        return false;
    }
    if (probe.error() != QLocalSocket::ServerNotFoundError
        && probe.error() != QLocalSocket::ConnectionRefusedError) {
        m_owner_conflict = true;
        m_error_string = "control endpoint ownership could not be confirmed: "
                         + probe.errorString();
        return false;
    }

    if (m_server.listen(m_endpoint)) {
        return true;
    }
    if (m_server.serverError() != QAbstractSocket::AddressInUseError) {
        m_error_string = m_server.errorString();
        return false;
    }

    QLocalSocket contender;
    contender.connectToServer(m_endpoint);
    if (contender.waitForConnected(250)) {
        contender.abort();
        m_owner_conflict = true;
        m_error_string = "another Acquisition process owns the control endpoint";
        return false;
    }
    if (contender.error() != QLocalSocket::ServerNotFoundError
        && contender.error() != QLocalSocket::ConnectionRefusedError) {
        m_owner_conflict = true;
        m_error_string = "control endpoint ownership could not be confirmed: "
                         + contender.errorString();
        return false;
    }
    if (!QLocalServer::removeServer(m_endpoint) || !m_server.listen(m_endpoint)) {
        m_error_string = m_server.errorString();
        return false;
    }
    return true;
}

void LocalControlServer::Close()
{
    for (auto &[socket, connection] : m_connections) {
        Q_UNUSED(connection);
        disconnect(socket, nullptr, this, nullptr);
        socket->abort();
        socket->deleteLater();
    }
    m_connections.clear();
    m_server.close();
}

void LocalControlServer::AcceptConnections()
{
    while (QLocalSocket *socket = m_server.nextPendingConnection()) {
        if (qsizetype(m_connections.size()) >= m_maximum_connections) {
            socket->abort();
            socket->deleteLater();
            continue;
        }
        socket->setReadBufferSize(MAX_REQUEST_BYTES + 5);
        m_connections.try_emplace(socket);
        connect(socket, &QLocalSocket::readyRead, this, [this, socket] { ReadFrom(socket); });
        connect(socket, &QLocalSocket::disconnected, this, [this, socket] { Drop(socket); });
        QTimer::singleShot(m_request_timeout_ms, socket, [this, socket] {
            if (m_connections.contains(socket)) {
                socket->abort();
            }
        });
    }
}

void LocalControlServer::ReadFrom(QLocalSocket *socket)
{
    const auto connection = m_connections.find(socket);
    if (connection == m_connections.end()) {
        return;
    }

    if (connection->second.handled) {
        socket->readAll();
        socket->disconnectFromServer();
        return;
    }
    auto frames = connection->second.decoder.FeedFirst(socket->readAll());
    if (!frames) {
        connection->second.handled = true;
        Send(socket, Error("", frames.error().code, frames.error().message));
        socket->disconnectFromServer();
        return;
    }
    if (frames->isEmpty()) {
        return;
    }
    // One request is served per connection. Any bytes after the first complete
    // frame are ignored whether the transport delivered them together or in a
    // later notification; disconnecting after the response prevents a second
    // command from being dispatched.
    connection->second.handled = true;

    QString request_id;
    if (const auto object = DecodeObject(frames->front()); object) {
        request_id = object->value("request_id").toString();
    }
    auto request = DecodeRequest(frames->front());
    if (!request) {
        Send(socket, Error(request_id, request.error().code, request.error().message));
        socket->disconnectFromServer();
        return;
    }

    if (!m_handler) {
        Send(socket,
             Error(request->request_id,
                   "service_unavailable",
                   "the application control service is not ready"));
        socket->disconnectFromServer();
        return;
    }

    try {
        Send(socket, m_handler(*request));
    } catch (const std::exception &error) {
        Send(socket, Error(request->request_id, "internal_error", error.what()));
    } catch (...) {
        Send(socket,
             Error(request->request_id,
                   "internal_error",
                   "the control request failed unexpectedly"));
    }
    socket->disconnectFromServer();
}

void LocalControlServer::Drop(QLocalSocket *socket)
{
    const auto connection = m_connections.find(socket);
    if (connection == m_connections.end()) {
        return;
    }
    m_connections.erase(connection);
    socket->deleteLater();
}

bool LocalControlServer::Send(QLocalSocket *socket, const QJsonObject &response)
{
    QByteArray frame;
    if (JsonSizeWithinLimit(response, MAX_RESPONSE_BYTES)) {
        frame = EncodeFrame(response);
    }
    if (frame.isEmpty() || frame.size() - 4 > MAX_RESPONSE_BYTES) {
        frame = EncodeFrame(Error(response.value("request_id").toString(),
                                  "response_too_large",
                                  "the response exceeds the configured limit"));
    }
    if (socket->bytesToWrite() + frame.size() > MAX_RESPONSE_BYTES + 4) {
        socket->abort();
        return false;
    }
    return socket->write(frame) == frame.size();
}

} // namespace control
