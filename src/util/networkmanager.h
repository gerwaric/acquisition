// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

#pragma once

#include <QObject>
#include <QNetworkAccessManager>

class QNetworkDiskCache;

class NetworkManager : public QNetworkAccessManager
{
    Q_OBJECT

public:
    explicit NetworkManager();

    void setPoesessid(const QByteArray &poesessid);
    void setBearerToken(const QByteArray &token);

    static void logRequest(const QNetworkRequest &request);
    static void logReply(const QNetworkReply *reply);
    static void logReplyErrors(QNetworkReply *reply, const char *context);

signals:
    void sessionIdChanged(const QByteArray &poesessid);

protected:
    QNetworkReply *createRequest(QNetworkAccessManager::Operation op,
                                 const QNetworkRequest &originalRequest,
                                 QIODevice *outgoingData = nullptr) override;

private:
    QNetworkDiskCache* m_diskCache;
    QByteArray m_poesessid;
    QByteArray m_bearerToken;

    using AttributeGetter = std::function<QVariant(QNetworkRequest::Attribute)>;

    static void logAttributes(const QString &name, AttributeGetter attrs);
    static void logHeaders(const QString &name, const QHttpHeaders &headers);
};
