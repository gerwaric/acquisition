// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QImage>
#include <QObject>
#include <QString>

class QNetworkReply;

class NetworkManager;

class ImageCache : public QObject
{
    Q_OBJECT
public:
    explicit ImageCache(NetworkManager &network_manager, const QString &directory);
    bool contains(const QString &url) const;
    QImage load(const QString &url) const;
public slots:
    void fetch(const QString &url);
    void onFetched();
signals:
    void imageReady(const QString &url);

private:
    QString getImagePath(const QString &url) const;
    NetworkManager &m_network_manager;
    QString m_directory;
};
