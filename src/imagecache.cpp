/*
    Copyright 2014 Ilya Zhuravlev

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

#include <QDir>
#include <QFile>
#include <QImageReader>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QString>
#include <QCryptographicHash>

#include <QsLog/QsLog.h>

#include "network_info.h"
#include "util.h"

ImageCache::ImageCache(
    QNetworkAccessManager& network_manager,
    const QString& directory)
    : network_manager_(network_manager)
    , directory_(directory)
{
    if (!QDir(directory_).exists()) {
        QDir().mkpath(directory_);
    };
}

bool ImageCache::contains(const std::string& url) const {
    const QString filename = getImagePath(url);
    const QFile file(filename);
    return file.exists();
}

void ImageCache::fetch(const std::string& url) {
    if (contains(url)) {
        QLOG_DEBUG() << "ImageCache: already contains" << url;
        emit imageReady(url);
    } else {
        QLOG_DEBUG() << "ImageCache: fetching" << url;
        QNetworkRequest request = QNetworkRequest(QUrl(QString::fromStdString(url)) );
        request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
        QNetworkReply* reply = network_manager_.get(request);
        connect(reply, &QNetworkReply::finished, this, &ImageCache::onFetched);
    };
}

void ImageCache::onFetched() {
    QNetworkReply* reply = qobject_cast<QNetworkReply*>(sender());
    const std::string url = reply->url().toString().toStdString();
    if (reply->error() != QNetworkReply::NoError) {
        QLOG_ERROR() << "ImageCache: failed to fetch image:" << reply->errorString() << ":" << url;
        return;
    };
    QLOG_DEBUG() << "ImageCatch: fetched" << url;
    QImageReader image_reader(reply);
    const QImage image = image_reader.read();
    image.save(getImagePath(url));
    emit imageReady(url);
}

QImage ImageCache::load(const std::string& url) const {
    const QString filename = getImagePath(url);
    const QFile file(filename);
    if (file.exists()) {
        return QImage(getImagePath(url));
    } else {
        return QImage();
    };
}

QString ImageCache::getImagePath(const std::string& url) const {
    return directory_ + QDir::separator() + QString::fromStdString(Util::Md5(url)) + ".png";
}
