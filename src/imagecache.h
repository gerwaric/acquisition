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
    QNetworkAccessManager& network_manager_;
    QString directory_;
};
