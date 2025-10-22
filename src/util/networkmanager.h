/*
    Copyright (C) 2014-2025 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/


#pragma once

#include <QObject>
#include <QNetworkAccessManager>

class QNetworkDiskCache;

class NetworkManager : public QNetworkAccessManager
{
    Q_OBJECT

public:
    explicit NetworkManager(QObject *parent = nullptr);

    void setBearerToken(const QString &token);

    static void logRequest(const QNetworkRequest &request);
    static void logReply(const QNetworkReply *reply);

protected:
    QNetworkReply *createRequest(QNetworkAccessManager::Operation op,
                                 const QNetworkRequest &originalRequest,
                                 QIODevice *outgoingData = nullptr) override;

private:
    QNetworkDiskCache* m_diskCache;
    QByteArray m_bearerToken;

    using AttributeGetter = std::function<QVariant(QNetworkRequest::Attribute)>;

    static void logAttributes(const QString &name, AttributeGetter attrs);
    static void logHeaders(const QString &name, const QHttpHeaders &headers);
};
