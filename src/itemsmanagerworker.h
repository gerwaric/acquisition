// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QNetworkCookie>
#include <QNetworkRequest>
#include <QObject>
#include <QString>

#include <queue>
#include <set>

#include "item.h"
#include "ui/mainwindow.h"
#include "util/util.h"

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

namespace poe {
    struct Character;
    struct StashTab;
} // namespace poe

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
    void Update(Util::TabSelection type, const std::vector<ItemLocation> &tab_names = {});

private slots:
    void OnStashListReceived(QNetworkReply *reply);
    void OnStashReceived(QNetworkReply *reply, const ItemLocation &location);
    void OnCharacterListReceived(QNetworkReply *reply);
    void OnCharacterReceived(QNetworkReply *reply, const ItemLocation &location);

private:
    bool isInitialized() const { return m_initialized; }
    bool isUpdating() const { return m_updating; };
    void ParseItemMods();
    void ParseItems(const poe::Character &character, ItemLocation location);
    void ParseItems(const poe::StashTab &stash, ItemLocation location);
    void RemoveUpdatingTabs(const std::set<QString> &tab_ids);
    void RemoveUpdatingItems(const std::set<QString> &tab_ids);
    void QueueRequest(const QString &endpoint,
                      const QNetworkRequest &request,
                      const ItemLocation &location);
    void FetchItems();

    void OAuthRefresh();

    typedef std::pair<QString, QString> TabSignature;
    typedef std::vector<TabSignature> TabsSignatureVector;
    TabsSignatureVector CreateTabsSignatureVector(const std::vector<poe::StashTab> &tabs);

    void SendStatusUpdate();
    void ParseItems(const std::vector<poe::Item> &items, const ItemLocation &base_location);
    void FinishUpdate();

    void ProcessTab(const poe::StashTab &tab, int &count);

    QSettings &m_settings;
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
