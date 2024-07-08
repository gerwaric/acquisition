#pragma once

#include <QImage>
#include <string>

class MainWindow;

class ImageCache {
public:
    explicit ImageCache(const QString& directory);
    bool Exists(const std::string& url) const;
    QImage Get(const std::string& url) const;
    void Set(const std::string& url, const QImage& image);
private:
    QString GetPath(const std::string& url) const;
    QString directory_;
};
