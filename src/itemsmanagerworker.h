// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QNetworkCookie>
#include <QNetworkRequest>
#include <QObject>
#include <QPointer>
#include <QString>
#include <QStringList>

#include <atomic>
#include <map>
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
    // Ids whose contents exist in the datastore — a strict subset of
    // tab_id_index, since a tab can be listed (metadata saved) without its
    // items ever having been fetched (F55).
    std::set<QString> contents_known;
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

    // Authoritative-list signals (F53): emitted only for a fresh top-level
    // list (never for ProcessTab's folder-children re-emits of
    // stashListReceived, which are partial and must not drive deletion) so
    // the datastore can drop rows the server no longer lists.
    void stashListReplaced(const std::vector<poe::StashTab> &stashes,
                           const QString &realm,
                           const QString &league);
    void characterListReplaced(const std::vector<poe::Character> &characters, const QString &realm);
    // Emitted when a parent stash reply arrives (F53): the reply is
    // authoritative for the parent's children, so the datastore can drop
    // child rows it no longer lists. child_ids is empty when child
    // fetching is disabled in the settings.
    void stashChildrenReplaced(const QString &parent_id,
                               const QStringList &child_ids,
                               const QString &realm,
                               const QString &league);

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
    void RebaseItemLocations(ItemLocationType type);
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

    void ProcessTab(const poe::StashTab &tab, int &count);

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

    // Ids whose contents have actually been fetched — seeded from the
    // datastore at startup, extended on every successful stash/character
    // reply, never consumed by list receipt alone. The always-fetch check
    // keys on this instead of list membership, so a new tab whose first
    // fetch failed (or never ran) stays "new" until a fetch lands (F55).
    std::set<QString> m_contents_known;

    // Child fetches still outstanding per Map/Unique parent: the parent
    // joins m_contents_known only when its last enabled child fetch lands,
    // so a failed child fetch leaves the parent "new" and the children get
    // retried — they never appear in a top-level list, so nothing else
    // would ever refetch them (F55).
    std::map<QString, int> m_pending_children;

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
