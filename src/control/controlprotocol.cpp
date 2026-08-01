// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/controlprotocol.h"

#include <QJsonDocument>
#include <QJsonParseError>

#include <limits>

namespace control {

namespace {

    constexpr qsizetype FRAME_HEADER_BYTES = 4;

    ProtocolError InvalidRequest(const QString &message)
    {
        return ProtocolError{"invalid_request", message};
    }

} // namespace

FrameDecoder::FrameDecoder(qsizetype maximum_payload)
    : m_maximum_payload(maximum_payload)
{}

std::expected<QVector<QByteArray>, ProtocolError> FrameDecoder::Feed(const QByteArray &bytes)
{
    m_buffer.append(bytes);
    QVector<QByteArray> frames;

    while (true) {
        if (m_expected_payload < 0) {
            if (m_buffer.size() < FRAME_HEADER_BYTES) {
                break;
            }
            const auto *header = reinterpret_cast<const unsigned char *>(m_buffer.constData());
            const quint32 length = (quint32(header[0]) << 24U) | (quint32(header[1]) << 16U)
                                   | (quint32(header[2]) << 8U) | quint32(header[3]);
            m_buffer.remove(0, FRAME_HEADER_BYTES);
            if (length == 0) {
                return std::unexpected(
                    ProtocolError{"invalid_frame", "frame payload must not be empty"});
            }
            if (length > quint32(m_maximum_payload)) {
                return std::unexpected(
                    ProtocolError{"frame_too_large", "frame payload exceeds the configured limit"});
            }
            m_expected_payload = qsizetype(length);
        }

        if (m_buffer.size() < m_expected_payload) {
            break;
        }
        frames.push_back(m_buffer.first(m_expected_payload));
        m_buffer.remove(0, m_expected_payload);
        m_expected_payload = -1;
    }

    return frames;
}

QByteArray EncodeFrame(const QJsonObject &object)
{
    const QByteArray payload = QJsonDocument(object).toJson(QJsonDocument::Compact);
    if (payload.size() > qsizetype(std::numeric_limits<quint32>::max())) {
        return {};
    }

    const quint32 length = quint32(payload.size());
    QByteArray frame;
    frame.resize(FRAME_HEADER_BYTES);
    frame[0] = char((length >> 24U) & 0xffU);
    frame[1] = char((length >> 16U) & 0xffU);
    frame[2] = char((length >> 8U) & 0xffU);
    frame[3] = char(length & 0xffU);
    frame.append(payload);
    return frame;
}

std::expected<QJsonObject, ProtocolError> DecodeObject(const QByteArray &payload)
{
    QJsonParseError parse_error;
    const QJsonDocument document = QJsonDocument::fromJson(payload, &parse_error);
    if (parse_error.error != QJsonParseError::NoError) {
        return std::unexpected(ProtocolError{"invalid_json", parse_error.errorString()});
    }
    if (!document.isObject()) {
        return std::unexpected(InvalidRequest("the JSON root must be an object"));
    }
    return document.object();
}

std::expected<Request, ProtocolError> DecodeRequest(const QByteArray &payload)
{
    auto decoded = DecodeObject(payload);
    if (!decoded) {
        return std::unexpected(decoded.error());
    }
    const QJsonObject &object = *decoded;
    const QJsonValue protocol = object.value("protocol");
    if (!protocol.isDouble() || protocol.toInt(-1) != PROTOCOL_VERSION) {
        return std::unexpected(
            ProtocolError{"unsupported_version", "the request protocol version is not supported"});
    }

    const QJsonValue request_id = object.value("request_id");
    if (!request_id.isString() || request_id.toString().isEmpty()
        || request_id.toString().size() > 128) {
        return std::unexpected(InvalidRequest("request_id must be a non-empty short string"));
    }

    const QJsonValue command = object.value("command");
    if (!command.isString() || command.toString().isEmpty()) {
        return std::unexpected(InvalidRequest("command must be a non-empty string"));
    }

    QJsonObject params;
    if (object.contains("params")) {
        if (!object.value("params").isObject()) {
            return std::unexpected(InvalidRequest("params must be an object"));
        }
        params = object.value("params").toObject();
    }

    return Request{request_id.toString(), command.toString(), params};
}

std::expected<QJsonObject, ProtocolError> DecodeResponse(const QByteArray &payload,
                                                         const QString &request_id)
{
    auto decoded = DecodeObject(payload);
    if (!decoded) {
        return std::unexpected(decoded.error());
    }
    const QJsonObject &object = *decoded;
    if (!object.value("protocol").isDouble()
        || object.value("protocol").toInt(-1) != PROTOCOL_VERSION) {
        return std::unexpected(
            ProtocolError{"unsupported_version", "the response protocol version is not supported"});
    }
    if (!object.value("request_id").isString()
        || object.value("request_id").toString() != request_id) {
        return std::unexpected(
            ProtocolError{"invalid_response", "the response request_id does not match the request"});
    }
    if (!object.value("ok").isBool()) {
        return std::unexpected(
            ProtocolError{"invalid_response", "the response ok field must be a boolean"});
    }
    if (object.value("ok").toBool()) {
        if (!object.value("result").isObject()) {
            return std::unexpected(
                ProtocolError{"invalid_response", "a successful response must contain a result object"});
        }
    } else {
        const QJsonValue error = object.value("error");
        if (!error.isObject() || !error.toObject().value("code").isString()
            || error.toObject().value("code").toString().isEmpty()
            || !error.toObject().value("message").isString()) {
            return std::unexpected(
                ProtocolError{"invalid_response", "an error response must contain code and message"});
        }
    }
    return object;
}

QJsonObject Success(const QString &request_id, const QJsonObject &result)
{
    return QJsonObject{{"protocol", PROTOCOL_VERSION},
                       {"request_id", request_id},
                       {"ok", true},
                       {"result", result}};
}

QJsonObject Error(const QString &request_id, const QString &code, const QString &message)
{
    return QJsonObject{{"protocol", PROTOCOL_VERSION},
                       {"request_id", request_id},
                       {"ok", false},
                       {"error", QJsonObject{{"code", code}, {"message", message}}}};
}

} // namespace control
