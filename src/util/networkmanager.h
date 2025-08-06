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

#include <QNetworkAccessManager>
#include <QNetworkInformation>
#include <QObject>

class QNetworkAccessManager;
class QNetworkReply;
class QNetworkRequest;

class NetworkManager : public QObject
{
    Q_OBJECT
public:
    NetworkManager();
    ~NetworkManager();

    QNetworkReply *get(const QNetworkRequest &request);
    QNetworkReply *get(const QNetworkRequest &request, QIODevice *data);
    QNetworkReply *get(const QNetworkRequest &request, const QByteArray &data);

    QNetworkReply *head(const QNetworkRequest &request);

    QNetworkReply *post(const QNetworkRequest &request, QIODevice *data);
    QNetworkReply *post(const QNetworkRequest &request, QHttpMultiPart *multiPart);
    QNetworkReply *post(const QNetworkRequest &request, const QByteArray &data);
    QNetworkReply *post(const QNetworkRequest &request, std::nullptr_t nptr);

private:
    QNetworkRequest prepare(const QNetworkRequest &request);
    std::unique_ptr<QNetworkAccessManager> m_network_manager;
    QNetworkInformation *m_network_info;
    bool m_offline{true};
};
