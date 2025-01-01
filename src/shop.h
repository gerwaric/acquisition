/*
    Copyright (C) 2014-2024 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#pragma once

#include <QNetworkReply>
#include <QObject>
#include <QUrlQuery>

#include <map>

#include "buyout.h"
#include "item.h"
#include "ui/mainwindow.h"

class QNetworkAccessManager;
class QSettings;

class Application;
class BuyoutManager;
class DataStore;
class ItemsManager;
class RateLimiter;

struct AugmentedItem {
    Item* item{ nullptr };
    Buyout bo;
    bool operator<(const AugmentedItem& other) const {
        if (bo.type != other.bo.type) {
            return bo.type < other.bo.type;
        } else if (bo.currency != other.bo.currency) {
            return bo.currency < other.bo.currency;
        } else if (bo.value != other.bo.value) {
            return bo.value < other.bo.value;
        } else {
            return false;
        };
    };
};

class Shop : public QObject {
    Q_OBJECT
public:
    explicit Shop(
        QSettings& settings,
        QNetworkAccessManager& network_manager,
        RateLimiter& rate_limiter,
        DataStore& datastore,
        ItemsManager& items_manager,
        BuyoutManager& buyout_manager);
    void SetThread(const QStringList& threads);
    void SetAutoUpdate(bool update);
    void SetShopTemplate(const QString& shop_template);
    void Update();
    void CopyToClipboard();
    void ExpireShopData();
    void SubmitShopToForum(bool force = false);
    bool auto_update() const { return m_auto_update; }
    const QStringList& threads() const { return m_threads; }
    const QStringList& shop_data() const { return m_shop_data; }
    const QString& shop_template() const { return m_shop_template; }
public slots:
    void UpdateStashIndex();
    void OnStashTabIndexReceived(QNetworkReply* reply);
    void OnEditPageFinished();
    void OnShopSubmitted(QUrlQuery query, QNetworkReply* reply);
signals:
    void StashesIndexed();
    void StatusUpdate(ProgramState state, const QString& status);
private:
    void SubmitSingleShop();
    void SubmitNextShop(const QString title, const QString hash);
    QString ShopEditUrl(size_t idx);
    QString SpoilerBuyout(Buyout& bo);

    QSettings& m_settings;
    QNetworkAccessManager& m_network_manager;
    RateLimiter& m_rate_limiter;
    DataStore& m_datastore;
    ItemsManager& m_items_manager;
    BuyoutManager& m_buyout_manager;

    std::map<QString, unsigned int> m_tab_index;
    QStringList m_threads;
    QStringList m_shop_data;
    QString m_shop_hash;
    QString m_shop_template;
    bool m_shop_data_outdated;
    bool m_auto_update;
    bool m_submitting;
    bool m_indexing;
    size_t m_requests_completed;

    static const QRegularExpression error_regex;
    static const QRegularExpression ratelimit_regex;
};
