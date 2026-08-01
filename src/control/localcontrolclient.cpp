// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/localcontrolclient.h"

#include <QDeadlineTimer>
#include <QLocalSocket>

#include <algorithm>
#include <limits>

#include "control/controlprotocol.h"

namespace control {

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
        return std::unexpected(ClientError{"not_running", socket.errorString()});
    }

    const QByteArray frame = EncodeFrame(request);
    if (socket.write(frame) != frame.size()
        || (socket.bytesToWrite() > 0 && !socket.waitForBytesWritten(remaining()))) {
        return std::unexpected(ClientError{"transport_error", socket.errorString()});
    }

    FrameDecoder decoder(MAX_RESPONSE_BYTES);
    while (socket.state() == QLocalSocket::ConnectedState || socket.bytesAvailable() > 0) {
        if (socket.bytesAvailable() == 0 && !socket.waitForReadyRead(remaining())) {
            if (deadline.hasExpired()) {
                return std::unexpected(ClientError{"timeout", "timed out waiting for the response"});
            }
            return std::unexpected(
                ClientError{"transport_error", "the server disconnected without a response"});
        }
        auto frames = decoder.Feed(socket.readAll());
        if (!frames) {
            return std::unexpected(ClientError{frames.error().code, frames.error().message});
        }
        if (!frames->isEmpty()) {
            auto response = DecodeResponse(frames->front(), request_id);
            if (!response) {
                return std::unexpected(ClientError{response.error().code, response.error().message});
            }
            return *response;
        }
    }

    return std::unexpected(ClientError{"transport_error", "the server disconnected without a response"});
}

} // namespace control
