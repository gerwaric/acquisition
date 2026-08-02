// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#pragma once

#include <QJsonObject>
#include <QString>

#include <expected>

namespace control {

struct ClientError
{
    QString code;
    QString message;
    bool request_may_have_been_sent{false};
};

std::expected<QJsonObject, ClientError> SendRequest(const QString &endpoint,
                                                    const QJsonObject &request,
                                                    int timeout_ms);

} // namespace control
