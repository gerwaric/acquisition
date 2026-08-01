// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#pragma once

#include <QByteArray>
#include <QJsonObject>
#include <QObject>
#include <QPointer>
#include <QString>

#include <deque>
#include <functional>
#include <optional>

#include "control/controlprotocol.h"
#include "refreshoutcome.h"

class BuyoutManager;
class ItemsManager;
class ItemsManagerWorker;

namespace control {

class ControlService : public QObject
{
    Q_OBJECT
public:
    enum class RefreshReadiness { Initializing, Ready, Busy };

    explicit ControlService(QString application_version, QObject *parent = nullptr);

    void SetNeedsLogin();
    void AttachSession(ItemsManager &items_manager,
                       ItemsManagerWorker *worker,
                       BuyoutManager &buyout_manager,
                       const QString &account,
                       const QString &league);
    void ConfigureRefresh(std::function<RefreshReadiness()> readiness,
                          std::function<void()> start);

    QJsonObject Handle(const Request &request);
    QString InstanceId() const { return m_instance_id; }

private:
    enum class ServiceState { Starting, NeedsLogin, LoadingCache, Ready };

    QJsonObject Status() const;
    QJsonObject Tabs(const QString &request_id, const QJsonObject &params) const;
    QJsonObject Items(const QString &request_id, const QJsonObject &params) const;
    QJsonObject Item(const QString &request_id, const QJsonObject &params) const;
    QJsonObject StartRefresh(const QString &request_id);
    QJsonObject RefreshStatus(const QString &request_id, const QJsonObject &params) const;
    QJsonObject NotReady(const QString &request_id) const;
    void OnRefreshFinished(const RefreshOutcome &outcome);
    void OnSessionDestroyed();
    void TrimRefreshHistory();
    static QString StateName(ServiceState state);

    struct RefreshJob
    {
        QString id;
        QString state;
        QString progress;
        QString started_at;
        QString finished_at;
        std::optional<QJsonObject> outcome;
    };

    static QJsonObject RefreshJobJson(const RefreshJob &job);

    QString m_application_version;
    QString m_instance_id;
    QByteArray m_cursor_key;
    QString m_account;
    QString m_league;
    ServiceState m_state{ServiceState::Starting};
    // Application currently forbids replacing a live UserSession and destroys
    // this service first. QPointers plus OnSessionDestroyed also keep the
    // boundary safe if that lifecycle policy changes later.
    QPointer<ItemsManager> m_items_manager;
    QPointer<ItemsManagerWorker> m_worker;
    QPointer<BuyoutManager> m_buyout_manager;
    quint64 m_inventory_revision{0};
    std::function<RefreshReadiness()> m_refresh_readiness;
    std::function<void()> m_start_refresh;
    std::deque<RefreshJob> m_refresh_jobs;
    QString m_active_refresh_id;
    static constexpr qsizetype MAXIMUM_REFRESH_HISTORY = 32;
};

} // namespace control
