#pragma once

#include <QObject>
#include <QNetworkAccessManager>
#include <QNetworkInformation>

class QNetworkAccessManager;
class QNetworkReply;
class QNetworkRequest;

class NetworkReply : public QObject {
    Q_OBJECT
signals:
    void finished(QNetworkReply* reply);
};

class NetworkManager : public QObject {
    Q_OBJECT
public:
    NetworkManager();
    ~NetworkManager();

    QNetworkReply* get(QNetworkRequest& request);
    QNetworkReply* get(QNetworkRequest& request, QIODevice* data);
    QNetworkReply* get(QNetworkRequest& request, const QByteArray& data);

    QNetworkReply* head(const QNetworkRequest& request);

    QNetworkReply* post(const QNetworkRequest& request, QIODevice* data);
    QNetworkReply* post(const QNetworkRequest& request, QHttpMultiPart* multiPart);
    QNetworkReply* post(const QNetworkRequest& request, const QByteArray& data);
    QNetworkReply* post(const QNetworkRequest& request, std::nullptr_t nptr);

private:
    bool offline() const;
    QNetworkRequest& prepare(QNetworkRequest& request);
    std::unique_ptr<QNetworkAccessManager> network_manager_;
    QNetworkInformation* network_info_;
};
