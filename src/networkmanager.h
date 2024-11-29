#pragma once

#include <QObject>
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
    explicit NetworkManager(QObject* parent);
    ~NetworkManager();
public slots:
    NetworkReply* get(QNetworkRequest& request);
    NetworkReply* head(QNetworkRequest& request);
    NetworkReply* post(QNetworkRequest& request, QByteArray&)
    void onReachabilityChanged(QNetworkInformation::Reachability reachability);
signals:
private:
    QNetworkInformation* network_info_;
    QNetworkInformation::Reachability reachability_;
    QNetworkAccessManager* network_manager_;
};
