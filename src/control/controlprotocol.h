// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#pragma once

#include <QByteArray>
#include <QJsonObject>
#include <QString>
#include <QVector>

#include <expected>

namespace control {

constexpr int PROTOCOL_VERSION = 1;
constexpr qsizetype MAX_REQUEST_BYTES = 64 * 1024;
constexpr qsizetype MAX_RESPONSE_BYTES = 4 * 1024 * 1024;

struct Request
{
    QString request_id;
    QString command;
    QJsonObject params;
};

struct ProtocolError
{
    QString code;
    QString message;
};

class FrameDecoder
{
public:
    explicit FrameDecoder(qsizetype maximum_payload);

    std::expected<QVector<QByteArray>, ProtocolError> Feed(const QByteArray &bytes);
    bool HasPartialFrame() const { return !m_buffer.isEmpty() || m_expected_payload >= 0; }

private:
    QByteArray m_buffer;
    qsizetype m_maximum_payload;
    qsizetype m_expected_payload{-1};
};

QByteArray EncodeFrame(const QJsonObject &object);
std::expected<QJsonObject, ProtocolError> DecodeObject(const QByteArray &payload);
std::expected<Request, ProtocolError> DecodeRequest(const QByteArray &payload);
std::expected<QJsonObject, ProtocolError> DecodeResponse(const QByteArray &payload,
                                                         const QString &request_id);

QJsonObject Success(const QString &request_id, const QJsonObject &result = {});
QJsonObject Error(const QString &request_id, const QString &code, const QString &message);

} // namespace control
