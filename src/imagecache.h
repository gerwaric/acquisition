#pragma once

#include <QImage>
#include <string>

class MainWindow;

class ImageCache {
public:
	explicit ImageCache(const QString& directory);
	bool Exists(const std::string& url);
	QImage Get(const std::string& url);
	void Set(const std::string& url, const QImage& image);
private:
	QString GetPath(const std::string& url);
	QString directory_;
};
