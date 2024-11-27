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
#include <QString>
#include <QCryptographicHash>

#include "util.h"

ImageCache::ImageCache(const QString& directory)
    : directory_(directory)
{
    if (!QDir(directory_).exists())
        QDir().mkpath(directory_);
}

bool ImageCache::Exists(const std::string& url) const {
    QString path = GetPath(url);
    QFile file(path);
    return file.exists();
}

QImage ImageCache::Get(const std::string& url) const {
    return QImage(QString(GetPath(url)));
}

void ImageCache::Set(const std::string& url, const QImage& image) {
    image.save(QString(GetPath(url)));
}

QString ImageCache::GetPath(const std::string& url) const {
    return directory_ + "/" + QString::fromStdString(Util::Md5(url)) + ".png";
}
