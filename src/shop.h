// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QNetworkReply>
#include <QObject>
#include <QUrlQuery>

#include <map>

#include <ui/mainwindow.h>

#include "buyout.h"

class QSettings;

class Application;
class BuyoutManager;
class DataStore;
class ItemsManager;
class NetworkManager;
class RateLimiter;

class Shop : public QObject
{
    Q_OBJECT
public:
    explicit Shop(QSettings &settings,
                  NetworkManager &network_manager,
                  RateLimiter &rate_limiter,
                  DataStore &datastore,
                  ItemsManager &items_manager,
                  BuyoutManager &buyout_manager);
    void SetThread(const QStringList &threads);
    void SetAutoUpdate(bool update);
    void SetShopTemplate(const QString &shop_template);
    void UpdateShopData();
    void CopyToClipboard();
    void ExpireShopData();
    void SubmitShopToForum(bool force = false);
    bool auto_update() const { return m_auto_update; }

    QStringList threads() const { return m_threads; }
    QStringList shop_data() const { return m_shop_data; }
    QString shop_template() const { return m_shop_template; }

public slots:
    void OnEditPageFinished();
    void OnShopSubmitted(QUrlQuery query, QNetworkReply *reply);

signals:
    void StashesIndexed();
    void StatusUpdate(ProgramState state, const QString &status);

private:
    void UpdateStashIndex(bool force);
    void OnStashIndexReceived(bool force, QNetworkReply *reply);
    void OnStashIndexUpdated(bool force);

    void SubmitSingleShop();
    void SubmitNextShop(const QString &title, const QString &hash);
    QString ShopEditUrl(int idx);
    QString SpoilerBuyout(Buyout &bo);

    QSettings &m_settings;
    NetworkManager &m_network_manager;
    RateLimiter &m_rate_limiter;
    DataStore &m_datastore;
    ItemsManager &m_items_manager;
    BuyoutManager &m_buyout_manager;

    std::map<QString, unsigned int> m_tab_index;
    QStringList m_threads;
    QStringList m_shop_data;
    QString m_shop_hash;
    QString m_shop_template;
    bool m_shop_data_outdated;
    bool m_auto_update;
    bool m_submitting;
    int m_requests_completed;

    static const QRegularExpression error_regex;
    static const QRegularExpression ratelimit_regex;
};
