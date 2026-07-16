// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QNetworkCookie>
#include <QNetworkRequest>
#include <QObject>
#include <QPointer>
#include <QString>

#include <atomic>
#include <queue>
#include <set>

#include "item.h"
#include "util/programstate.h"
#include "util/util.h"

class QNetworkReply;
class QSettings;
class QSignalMapper;
class QThread;
class QTimer;

class BuyoutManager;
class DataStore;
class ItemLocation;
class NetworkManager;
class RateLimitedReply;
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

struct ParseResult
{
    std::vector<ItemLocation> tabs;
    Items items;
    std::set<QString> tab_id_index;
};

class ItemsManagerWorker : public QObject
{
    Q_OBJECT
public:
    explicit ItemsManagerWorker(QSettings &m_settings,
                                BuyoutManager &buyout_manager,
                                RateLimiter &rate_limiter,
                                QObject *parent = nullptr);
    ~ItemsManagerWorker() override;

    void UpdateRequest(TabSelection type, const std::vector<ItemLocation> &locations);
    ParseResult ParseCachedItems(const QString &dataDir) const;

signals:
    void ItemsRefreshed(const Items &items,
                        const std::vector<ItemLocation> &tabs,
                        bool initial_refresh);
    void StatusUpdate(ProgramState state, const QString &status);
    void NotifyUser(const QString &message);

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
    enum class WorkerState { Initializing, Idle, Updating };

    bool isInitialized() const { return m_state != WorkerState::Initializing; }
    bool isUpdating() const { return m_state == WorkerState::Updating; }
    void StartParseThread();
    void OnParseCompleted(ParseResult result);
    void LoadItems(const poe::Character &character,
                   ItemLocation location,
                   ParseResult &result) const;
    void LoadItems(const poe::StashTab &stash, ItemLocation location, ParseResult &result) const;
    void RemoveItemsFetchedBy(const QString &fetch_id);
    void QueueRequest(const QString &endpoint,
                      const QNetworkRequest &request,
                      const ItemLocation &location);
    void FetchItems();
    void SubmitStashListRequest();
    void SubmitCharacterListRequest();
    void SubmitNextItemRequest();
    bool DiscardIfStale(int generation, RateLimitedReply *reply, QNetworkReply *network_reply);

    void Refresh();

    void SendStatusUpdate();
    void ParseItems(const std::vector<poe::Item> &items, const ItemLocation &base_location);
    void CheckUpdateFinished();
    void FinishUpdate();

    void ProcessTab(const poe::StashTab &tab, int &count, const std::set<QString> &previously_known);

    QSettings &m_settings;
    BuyoutManager &m_buyout_manager;
    RateLimiter &m_rate_limiter;

    QString m_realm;
    QString m_league;
    QString m_account;

    std::vector<ItemLocation> m_tabs;
    std::queue<ItemsRequest> m_queue;

    Items m_items;

    size_t m_stashes_needed{0};
    size_t m_stashes_received{0};

    size_t m_characters_needed{0};
    size_t m_characters_received{0};

    std::set<QString> m_tab_id_index;

    WorkerState m_state{WorkerState::Initializing};
    QPointer<QThread> m_parser_thread;
    std::atomic<bool> m_shutdown{false};

    // Incremented by every update; reply handlers discard replies whose
    // generation is not the currently running update's (F28).
    int m_update_generation{0};

    bool m_updateRequest{false};
    TabSelection m_type;
    std::vector<ItemLocation> m_locations;
    size_t m_request_failures{0};

    int m_queue_id{0};

    // The current update's content selection: refresh everything
    // (All/TabsOnly), or only the tabs/characters whose ids are listed.
    // Tabs not previously known (brand new on the server) are always
    // fetched.
    bool m_update_all;
    std::set<QString> m_tabs_to_update;

    bool m_need_stash_list;
    bool m_need_character_list;

    bool m_has_stash_list;
    bool m_has_character_list;

    bool m_update_tab_contents;
};
