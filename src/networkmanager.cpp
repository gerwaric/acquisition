#include "networkmanager.h"

#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>

#include <QsLog/QsLog.h>

#include "network_info.h"

NetworkManager::NetworkManager(QObject* parent)
    : QObject(parent)
{
    network_info_ = QNetworkInformation::instance();
    network_manager_ = new QNetworkAccessManager(this);

    connect(network_info_, &QNetworkInformation::reachabilityChanged, this, &NetworkManager::onReachabilityChanged);
}

NetworkManager::~NetworkManager() {};

QNetworkReply* NetworkManager::get(QNetworkRequest& request)
{
    if (reachability_ != QNetworkInformation::Reachability::Online) {
        QLOG_ERROR() << "NetworkManager: network is not online";
        return nullptr;
    };
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    return network_manager_->get(request);
}

QNetworkReply* NetworkManager::head(QNetworkRequest& request)
{
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    return network_manager_->head(request);
}


void NetworkManager::onReachabilityChanged(QNetworkInformation::Reachability reachability)
{
    reachability_ = reachability;
}
