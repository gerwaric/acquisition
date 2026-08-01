// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/controlservice.h"

#include <QUuid>

#include "buyoutmanager.h"
#include "itemsmanager.h"
#include "itemsmanagerworker.h"

namespace control {

ControlService::ControlService(QString application_version, QObject *parent)
    : QObject(parent)
    , m_application_version(std::move(application_version))
    , m_instance_id(QUuid::createUuid().toString(QUuid::WithoutBraces))
{}

void ControlService::SetNeedsLogin()
{
    if (!m_items_manager) {
        m_state = ServiceState::NeedsLogin;
    }
}

void ControlService::AttachSession(ItemsManager &items_manager,
                                   ItemsManagerWorker &worker,
                                   BuyoutManager &buyout_manager,
                                   const QString &account,
                                   const QString &league)
{
    m_items_manager = &items_manager;
    m_worker = &worker;
    m_buyout_manager = &buyout_manager;
    m_account = account;
    m_league = league;
    m_state = ServiceState::LoadingCache;

    connect(&items_manager, &ItemsManager::ItemsRefreshed, this, [this](bool initial_refresh) {
        ++m_inventory_revision;
        if (initial_refresh) {
            m_state = ServiceState::Ready;
        }
    });
    connect(&items_manager, &ItemsManager::TabRefreshed, this, [this] { ++m_inventory_revision; });
    connect(&items_manager,
            &ItemsManager::ChildrenReconciled,
            this,
            [this] { ++m_inventory_revision; });
    connect(&buyout_manager,
            &BuyoutManager::BuyoutsChanged,
            this,
            [this] { ++m_inventory_revision; });
}

QJsonObject ControlService::Handle(const Request &request)
{
    if (request.command == "status") {
        return Success(request.request_id, Status());
    }
    return Error(request.request_id,
                 "unknown_command",
                 QString("unknown control command: %1").arg(request.command));
}

QJsonObject ControlService::Status() const
{
    QJsonObject result{{"application_version", m_application_version},
                       {"instance_id", m_instance_id},
                       {"service_state", StateName(m_state)},
                       {"inventory_revision", QString::number(m_inventory_revision)}};

    if (m_items_manager) {
        result.insert("account", m_account);
        result.insert("league", m_league);
        result.insert("item_count", int(m_items_manager->items().size()));
        result.insert("location_count", int(m_items_manager->locationInventory().entries().size()));
    }

    if (m_worker) {
        switch (m_worker->updateReadiness()) {
        case ItemsManagerWorker::UpdateReadiness::Initializing:
            result.insert("refresh_state", "initializing");
            break;
        case ItemsManagerWorker::UpdateReadiness::Ready:
            result.insert("refresh_state", "idle");
            break;
        case ItemsManagerWorker::UpdateReadiness::Busy:
            result.insert("refresh_state", "updating");
            break;
        }
    } else {
        result.insert("refresh_state", "unavailable");
    }

    return result;
}

QString ControlService::StateName(ServiceState state)
{
    switch (state) {
    case ServiceState::Starting:
        return "starting";
    case ServiceState::NeedsLogin:
        return "needs_login";
    case ServiceState::LoadingCache:
        return "loading_cache";
    case ServiceState::Ready:
        return "ready";
    }
    return "starting";
}

} // namespace control
