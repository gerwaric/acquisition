// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#pragma once

#include <QJsonObject>
#include <QObject>
#include <QString>

#include "control/controlprotocol.h"

class BuyoutManager;
class ItemsManager;
class ItemsManagerWorker;

namespace control {

class ControlService : public QObject
{
    Q_OBJECT
public:
    explicit ControlService(QString application_version, QObject *parent = nullptr);

    void SetNeedsLogin();
    void AttachSession(ItemsManager &items_manager,
                       ItemsManagerWorker &worker,
                       BuyoutManager &buyout_manager,
                       const QString &account,
                       const QString &league);

    QJsonObject Handle(const Request &request);
    QString InstanceId() const { return m_instance_id; }

private:
    enum class ServiceState { Starting, NeedsLogin, LoadingCache, Ready };

    QJsonObject Status() const;
    static QString StateName(ServiceState state);

    QString m_application_version;
    QString m_instance_id;
    QString m_account;
    QString m_league;
    ServiceState m_state{ServiceState::Starting};
    // Application forbids replacing a live UserSession, and owns this service
    // after m_session so the service is destroyed first. These pointers cannot
    // outlive their session under that enforced lifecycle invariant.
    ItemsManager *m_items_manager{nullptr};
    ItemsManagerWorker *m_worker{nullptr};
    BuyoutManager *m_buyout_manager{nullptr};
    quint64 m_inventory_revision{0};
};

} // namespace control
