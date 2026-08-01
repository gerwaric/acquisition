// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/localcontrolserver.h"

#include <QAbstractSocket>
#include <QDir>
#include <QFileInfo>
#include <QLocalSocket>
#include <QLockFile>
#include <QTimer>

#include <exception>

#include "control/controlendpoint.h"

namespace control {

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
    if (!QDir().mkpath(QFileInfo(m_endpoint).absolutePath())) {
        m_error_string = "the private control runtime directory could not be created";
        return false;
    }
#endif

    const QString lock_path = EndpointLockPath(data_directory);
    if (lock_path.isEmpty()
        || !QDir().mkpath(QFileInfo(lock_path).absolutePath())) {
        m_error_string = "a writable control-lock directory is unavailable";
        return false;
    }
    m_endpoint_lock = std::make_unique<QLockFile>(lock_path);
    // A live process may own control for days; stale ownership is PID-based,
    // never age-based.
    m_endpoint_lock->setStaleLockTime(0);
    if (!m_endpoint_lock->tryLock(0)) {
        m_owner_conflict = true;
        m_error_string = "another Acquisition process owns the control endpoint";
        m_endpoint_lock.reset();
        return false;
    }

    if (m_server.listen(m_endpoint)) {
        return true;
    }

    if (m_server.serverError() != QAbstractSocket::AddressInUseError
        || !QLocalServer::removeServer(m_endpoint) || !m_server.listen(m_endpoint)) {
        m_error_string = m_server.errorString();
        m_endpoint_lock.reset();
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
    m_endpoint_lock.reset();
}

void LocalControlServer::AcceptConnections()
{
    while (QLocalSocket *socket = m_server.nextPendingConnection()) {
        if (qsizetype(m_connections.size()) >= m_maximum_connections) {
            socket->abort();
            socket->deleteLater();
            continue;
        }
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

    const QByteArray bytes = socket->readAll();
    connection->second.received_bytes += bytes.size();
    if (connection->second.received_bytes > MAX_REQUEST_BYTES + 4 || connection->second.handled) {
        Send(socket, Error("", "invalid_frame", "one request is allowed per connection"));
        socket->disconnectFromServer();
        return;
    }

    auto frames = connection->second.decoder.Feed(bytes);
    if (!frames) {
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
    QByteArray frame = EncodeFrame(response);
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
