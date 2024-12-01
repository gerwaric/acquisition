#pragma once

#include <QObject>
#include <QNetworkAccessManager>
#include <QNetworkInformation>

class QNetworkAccessManager;
class QNetworkReply;
class QNetworkRequest;

class NetworkManager : public QObject {
    Q_OBJECT
public:
    NetworkManager();
    ~NetworkManager();

    QNetworkReply* get(const QNetworkRequest& request);
    QNetworkReply* get(const QNetworkRequest& request, QIODevice* data);
    QNetworkReply* get(const QNetworkRequest& request, const QByteArray& data);

    QNetworkReply* head(const QNetworkRequest& request);

    QNetworkReply* post(const QNetworkRequest& request, QIODevice* data);
    QNetworkReply* post(const QNetworkRequest& request, QHttpMultiPart* multiPart);
    QNetworkReply* post(const QNetworkRequest& request, const QByteArray& data);
    QNetworkReply* post(const QNetworkRequest& request, std::nullptr_t nptr);

private:
    QNetworkRequest prepare(const QNetworkRequest& request);
    std::unique_ptr<QNetworkAccessManager> network_manager_;
    QNetworkInformation* network_info_;
    bool offline_{ true };
};
