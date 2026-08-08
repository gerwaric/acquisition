// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#pragma once

#include <QJsonObject>
#include <QLocalServer>
#include <QObject>
#include <QString>

#include <functional>
#include <map>

#include "control/controlprotocol.h"

class QDir;
class QLocalSocket;

namespace control {

class LocalControlServer : public QObject
{
    Q_OBJECT
public:
    using Handler = std::function<QJsonObject(const Request &)>;

    explicit LocalControlServer(QObject *parent = nullptr,
                                int request_timeout_ms = 5000,
                                qsizetype maximum_connections = 32);
    ~LocalControlServer() override;

    bool Listen(const QDir &data_directory);
    void Close();
    bool IsListening() const { return m_server.isListening(); }
    QString ErrorString() const { return m_error_string; }
    bool OwnerConflict() const { return m_owner_conflict; }
    QString Endpoint() const { return m_endpoint; }
    void SetHandler(Handler handler) { m_handler = std::move(handler); }

private:
    struct Connection
    {
        FrameDecoder decoder{MAX_REQUEST_BYTES};
        bool handled{false};
    };

    void AcceptConnections();
    void ReadFrom(QLocalSocket *socket);
    void Drop(QLocalSocket *socket);
    bool Send(QLocalSocket *socket, const QJsonObject &response);

    QLocalServer m_server;
    std::map<QLocalSocket *, Connection> m_connections;
    Handler m_handler;
    QString m_endpoint;
    QString m_error_string;
    int m_request_timeout_ms;
    qsizetype m_maximum_connections;
    bool m_owner_conflict{false};
};

} // namespace control
