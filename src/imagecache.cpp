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

#include "imagecache.h"

#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QImageReader>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QString>

#include <util/spdlog_qt.h>
#include <util/util.h>

#include "network_info.h"

ImageCache::ImageCache(QNetworkAccessManager &network_manager, const QString &directory)
    : m_network_manager(network_manager)
    , m_directory(directory)
{
    if (!QDir(m_directory).exists()) {
        QDir().mkpath(m_directory);
    }
}

bool ImageCache::contains(const QString &url) const
{
    const QString filename = getImagePath(url);
    const QFile file(filename);
    return file.exists();
}

void ImageCache::fetch(const QString &url)
{
    if (contains(url)) {
        spdlog::debug("ImageCache: already contains {}", url);
        emit imageReady(url);
    } else {
        spdlog::debug("ImageCache: fetching {}", url);
        QNetworkRequest request = QNetworkRequest(QUrl(url));
        request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
        QNetworkReply *reply = m_network_manager.get(request);
        connect(reply, &QNetworkReply::finished, this, &ImageCache::onFetched);
    }
}

void ImageCache::onFetched()
{
    QNetworkReply *reply = qobject_cast<QNetworkReply *>(sender());
    const QString url = reply->url().toString();
    if (reply->error() != QNetworkReply::NoError) {
        spdlog::error("ImageCache: failed to fetch image: {}: {}", reply->errorString(), url);
        return;
    }
    spdlog::debug("ImageCatch: fetched {}", url);
    QImageReader image_reader(reply);
    const QImage image = image_reader.read();
    image.save(getImagePath(url));
    emit imageReady(url);
}

QImage ImageCache::load(const QString &url) const
{
    const QString filename = getImagePath(url);
    const QFile file(filename);
    if (file.exists()) {
        return QImage(getImagePath(url));
    } else {
        return QImage();
    }
}

QString ImageCache::getImagePath(const QString &url) const
{
    return m_directory + QDir::separator() + Util::Md5(url) + ".png";
}
