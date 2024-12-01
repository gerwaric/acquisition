#include "networkmanager.h"

#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>

#include <QsLog/QsLog.h>

#include "network_info.h"

NetworkManager::NetworkManager()
{
    network_manager_ = std::make_unique<QNetworkAccessManager>();

    network_info_ = QNetworkInformation::instance();

    connect(network_info_, &QNetworkInformation::reachabilityChanged, this,
        [=](QNetworkInformation::Reachability reachability) {
            QLOG_INFO() << "NetworkManager: reachability changed to" << reachability;
            offline_ = (reachability != QNetworkInformation::Reachability::Online);
        });
}

NetworkManager::~NetworkManager() {};

QNetworkReply* NetworkManager::get(const QNetworkRequest& request) {
    return offline_ ? nullptr : network_manager_->get(prepare(request));
}

QNetworkReply* NetworkManager::get(const QNetworkRequest& request, QIODevice* data) {
    return offline_ ? nullptr : network_manager_->get(prepare(request), data);
}

QNetworkReply* NetworkManager::get(const QNetworkRequest& request, const QByteArray& data) {
    return offline_ ? nullptr : network_manager_->get(prepare(request), data);
}

QNetworkReply* NetworkManager::head(const QNetworkRequest& request) {
    return offline_ ? nullptr : network_manager_->head(prepare(request));
}

QNetworkReply* NetworkManager::post(const QNetworkRequest& request, QIODevice* data) {
    return offline_ ? nullptr : network_manager_->post(prepare(request), data);
}

QNetworkReply* NetworkManager::post(const QNetworkRequest& request, QHttpMultiPart* multiPart) {
    return offline_ ? nullptr : network_manager_->post(prepare(request), multiPart);
}

QNetworkReply* NetworkManager::post(const QNetworkRequest& request, const QByteArray& data) {
    return offline_ ? nullptr : network_manager_->post(prepare(request), data);
}

QNetworkReply* NetworkManager::post(const QNetworkRequest& request, std::nullptr_t nptr) {
    return offline_ ? nullptr : network_manager_->post(prepare(request), nptr);
}

QNetworkRequest NetworkManager::prepare(const QNetworkRequest& request) {
    QNetworkRequest outgoing_request(request);
    outgoing_request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    return outgoing_request;
}
