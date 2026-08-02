// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/localcontrolclient.h"

#include <QDeadlineTimer>
#include <QLocalSocket>

#include <algorithm>
#include <limits>

#ifdef Q_OS_WIN
#include <qt_windows.h>
#endif

#include "control/controlprotocol.h"

namespace control {

namespace {

#ifdef Q_OS_WIN
    bool ReadTokenUser(HANDLE token, QByteArray &buffer)
    {
        DWORD size = 0;
        GetTokenInformation(token, TokenUser, nullptr, 0, &size);
        if (GetLastError() != ERROR_INSUFFICIENT_BUFFER || size == 0) {
            return false;
        }
        buffer.resize(int(size));
        return GetTokenInformation(token, TokenUser, buffer.data(), size, &size);
    }

    bool ServerBelongsToCurrentUser(const QLocalSocket &socket)
    {
        ULONG server_pid = 0;
        const HANDLE pipe = reinterpret_cast<HANDLE>(socket.socketDescriptor());
        if (!GetNamedPipeServerProcessId(pipe, &server_pid)) {
            return false;
        }

        HANDLE server_process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, server_pid);
        if (!server_process) {
            return false;
        }
        HANDLE server_token = nullptr;
        HANDLE current_token = nullptr;
        const bool opened = OpenProcessToken(server_process, TOKEN_QUERY, &server_token)
                            && OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &current_token);
        CloseHandle(server_process);
        if (!opened) {
            if (server_token) {
                CloseHandle(server_token);
            }
            if (current_token) {
                CloseHandle(current_token);
            }
            return false;
        }

        QByteArray server_user;
        QByteArray current_user;
        const bool read = ReadTokenUser(server_token, server_user)
                          && ReadTokenUser(current_token, current_user);
        CloseHandle(server_token);
        CloseHandle(current_token);
        if (!read) {
            return false;
        }
        const auto *server = reinterpret_cast<const TOKEN_USER *>(server_user.constData());
        const auto *current = reinterpret_cast<const TOKEN_USER *>(current_user.constData());
        return EqualSid(server->User.Sid, current->User.Sid);
    }
#endif

} // namespace

std::expected<QJsonObject, ClientError> SendRequest(const QString &endpoint,
                                                    const QJsonObject &request,
                                                    int timeout_ms)
{
    const QString request_id = request.value("request_id").toString();
    if (request_id.isEmpty()) {
        return std::unexpected(ClientError{"invalid_request", "request_id is required"});
    }

    QDeadlineTimer deadline(timeout_ms);
    const auto remaining = [&deadline]() {
        return int(std::clamp<qint64>(deadline.remainingTime(),
                                     0,
                                     std::numeric_limits<int>::max()));
    };

    QLocalSocket socket;
    socket.connectToServer(endpoint, QIODevice::ReadWrite);
    if (!socket.waitForConnected(remaining())) {
        const bool absent = socket.error() == QLocalSocket::ServerNotFoundError
                            || socket.error() == QLocalSocket::ConnectionRefusedError;
        return std::unexpected(
            ClientError{absent ? "not_running" : "connect_error", socket.errorString()});
    }
#ifdef Q_OS_WIN
    if (!ServerBelongsToCurrentUser(socket)) {
        socket.abort();
        return std::unexpected(
            ClientError{"server_identity", "the control server belongs to another user"});
    }
#endif

    const QByteArray frame = EncodeFrame(request);
    if (socket.write(frame) != frame.size()
        || (socket.bytesToWrite() > 0 && !socket.waitForBytesWritten(remaining()))) {
        return std::unexpected(ClientError{"transport_error", socket.errorString(), true});
    }

    FrameDecoder decoder(MAX_RESPONSE_BYTES);
    while (socket.state() == QLocalSocket::ConnectedState || socket.bytesAvailable() > 0) {
        if (socket.bytesAvailable() == 0 && !socket.waitForReadyRead(remaining())) {
            if (deadline.hasExpired()) {
                return std::unexpected(
                    ClientError{"timeout", "timed out waiting for the response", true});
            }
            return std::unexpected(ClientError{
                "transport_error", "the server disconnected without a response", true});
        }
        auto frames = decoder.Feed(socket.readAll());
        if (!frames) {
            return std::unexpected(
                ClientError{frames.error().code, frames.error().message, true});
        }
        if (!frames->isEmpty()) {
            auto response = DecodeResponse(frames->front(), request_id);
            if (!response) {
                return std::unexpected(
                    ClientError{response.error().code, response.error().message, true});
            }
            return *response;
        }
    }

    return std::unexpected(ClientError{
        "transport_error", "the server disconnected without a response", true});
}

} // namespace control
