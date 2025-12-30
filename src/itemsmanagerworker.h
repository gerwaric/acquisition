/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include <QNetworkCookie>
#include <QNetworkRequest>
#include <QObject>
#include <QString>

#include <queue>
#include <set>

#include <poe/types/character.h>
#include <poe/types/stashtab.h>
#include <ui/mainwindow.h>
#include <util/util.h>

#include "item.h"

class QNetworkReply;
class QSettings;
class QSignalMapper;
class QTimer;

class BuyoutManager;
class DataStore;
class ItemLocation;
class NetworkManager;
class RateLimiter;
class RePoE;

struct ItemsRequest
{
    int id{-1};
    QString endpoint;
    QNetworkRequest network_request;
    ItemLocation location;
};

struct ItemsReply
{
    QNetworkReply *network_reply;
    ItemsRequest request;
};

class ItemsManagerWorker : public QObject
{
    Q_OBJECT
public:
    explicit ItemsManagerWorker(QSettings &m_settings,
                                BuyoutManager &buyout_manager,
                                DataStore &datastore,
                                RateLimiter &rate_limiter,
                                QObject *parent = nullptr);
    void UpdateRequest(TabSelection type, const std::vector<ItemLocation> &locations);

signals:
    void ItemsRefreshed(const Items &items,
                        const std::vector<ItemLocation> &tabs,
                        bool initial_refresh);
    void StatusUpdate(ProgramState state, const QString &status);

    void characterListReceived(const std::vector<poe::Character> &characters, const QString &realm);
    void characterReceived(const poe::Character &character, const QString &realm);
    void stashListReceived(const std::vector<poe::StashTab> &stashes,
                           const QString &realm,
                           const QString &league);
    void stashReceived(const poe::StashTab &stash, const QString &realm, const QString &league);

public slots:
    void OnRePoEReady();
    void Update(Util::TabSelection type,
                const std::vector<ItemLocation, std::allocator<ItemLocation>> &tab_names
                = std::vector<ItemLocation, std::allocator<ItemLocation>>());

private slots:
    void OnOAuthStashListReceived(QNetworkReply *reply);
    void OnOAuthStashReceived(QNetworkReply *reply, const ItemLocation &location);
    void OnOAuthCharacterListReceived(QNetworkReply *reply);
    void OnOAuthCharacterReceived(QNetworkReply *reply, const ItemLocation &location);

private:
    bool isInitialized() const { return m_initialized; }
    bool isUpdating() const { return m_updating; };
    void ParseItemMods();
    void RemoveUpdatingTabs(const std::set<QString> &tab_ids);
    void RemoveUpdatingItems(const std::set<QString> &tab_ids);
    void QueueRequest(const QString &endpoint,
                      const QNetworkRequest &request,
                      const ItemLocation &location);
    void FetchItems();

    void OAuthRefresh();
    QNetworkRequest MakeOAuthStashListRequest(const QString &realm, const QString &league);
    QNetworkRequest MakeOAuthStashRequest(const QString &realm,
                                          const QString &league,
                                          const QString &stash_id,
                                          const QString &substash_id = "");
    QNetworkRequest MakeOAuthCharacterListRequest(const QString &realm);
    QNetworkRequest MakeOAuthCharacterRequest(const QString &realm, const QString &name);

    typedef std::pair<QString, QString> TabSignature;
    typedef std::vector<TabSignature> TabsSignatureVector;
    TabsSignatureVector CreateTabsSignatureVector(const rapidjson::Value &tabs);

    void SendStatusUpdate();
    void ParseItems(rapidjson::Value &value,
                    const ItemLocation &base_location,
                    rapidjson_allocator &alloc);
    bool TabsChanged(rapidjson::Document &doc,
                     QNetworkReply *network_reply,
                     const ItemLocation &location);
    void FinishUpdate();

    bool IsOAuthTabValid(rapidjson::Value &tab);
    void ProcessOAuthTab(rapidjson::Value &tab, int &count, rapidjson_allocator &alloc);

    QSettings &m_settings;
    DataStore &m_datastore;
    BuyoutManager &m_buyout_manager;
    RateLimiter &m_rate_limiter;

    QString m_realm;
    QString m_league;
    QString m_account;

    std::vector<ItemLocation> m_tabs;
    std::queue<ItemsRequest> m_queue;

    // m_tabs_signature captures <"n", "id"> from JSON tab list, used as consistency check
    TabsSignatureVector m_tabs_signature;

    Items m_items;

    size_t m_stashes_needed{0};
    size_t m_stashes_received{0};

    size_t m_characters_needed{0};
    size_t m_characters_received{0};

    std::set<QString> m_tab_id_index;

    volatile bool m_initialized{false};
    volatile bool m_updating{false};

    bool m_cancel_update{false};
    bool m_updateRequest{false};
    TabSelection m_type;
    std::vector<ItemLocation> m_locations;
    std::set<ItemLocation> m_requested_locations;

    int m_queue_id{0};
    QString m_selected_character;

    int m_first_stash_request_index;
    QString m_first_character_request_name;

    bool m_need_stash_list;
    bool m_need_character_list;

    bool m_has_stash_list;
    bool m_has_character_list;

    bool m_update_tab_contents;
};
