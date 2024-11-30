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

bool NetworkManager::offline() {
    return (QNetworkInformation::instance()->reachability() != QNetworkInformation::Reachability::Online);
}

QNetworkRequest& NetworkManager::prepare(QNetworkRequest& request) {
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    return request;
}

QNetworkReply* NetworkManager::get(QNetworkRequest& request) {
    return online_ ? network_manager_->get(prepare(request)) : nullptr;
}

QNetworkReply* NetworkManager::get(QNetworkRequest& request, QIODevice* data) {
    return online_ ? network_manager_->get(prepare(request), data) : nullptr;
};

QNetworkReply* NetworkManager::get(QNetworkRequest& request, const QByteArray& data) {
    return online_ ? network_manager_->get(prepare(request), data) : nullptr;
};




QNetworkReply* NetworkManager::get(QNetworkRequest& request, ...)
{
    if (network_info_->reachability() != QNetworkInformation::Reachability::Online) {
        QLOG_ERROR() << "NetworkManager: network is not online";
        return nullptr;
    };
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    return network_manager_->get(request);
}

QNetworkReply* NetworkManager::head(QNetworkRequest& request, ...)
{
    if (network_info_->reachability() != QNetworkInformation::Reachability::Online) {
        QLOG_ERROR() << "NetworkManager: network is not online";
        return nullptr;
    };
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    return network_manager_->head(request);
}

QNetworkReply* NetworkManager::post(QNetworkRequest& request, ...)
{
    if (network_info_->reachability() != QNetworkInformation::Reachability::Online) {
        QLOG_ERROR() << "NetworkManager: network is not online";
        return nullptr;
    };
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    return network_manager_->post(request);
}


void NetworkManager::onReachabilityChanged(QNetworkInformation::Reachability reachability)
{
    reachability_ = reachability;
}
