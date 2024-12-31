/*
    Copyright (C) 2014-2024 Acquisition Contributors

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
#include <QImage>
#include <QString>

#include <string>

class QNetworkAccessManager;
class QNetworkReply;

class ImageCache : public QObject {
    Q_OBJECT
public:
    explicit ImageCache(
        QNetworkAccessManager& network_manager,
        const QString& directory);
    bool contains(const std::string& url) const;
    QImage load(const std::string& url) const;
public slots:
    void fetch(const std::string& url);
    void onFetched();
signals:
    void imageReady(const std::string& url);
private:
    QString getImagePath(const std::string& url) const;
    QNetworkAccessManager& m_network_manager;
    QString m_directory;
};
