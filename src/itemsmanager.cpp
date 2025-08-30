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

#include "itemsmanager.h"

#include <QMessageBox>
#include <QNetworkCookie>
#include <QSettings>

#include <datastore/datastore.h>
#include <ui/mainwindow.h>
#include <util/repoe.h>
#include <util/spdlog_qt.h>
#include <util/util.h>

#include "application.h"
#include "buyoutmanager.h"
#include "filters.h"
#include "item.h"
#include "itemsmanagerworker.h"
#include "modlist.h"
#include "shop.h"

ItemsManager::ItemsManager(QSettings &settings,
                           QNetworkAccessManager &network_manager,
                           RePoE &repoe,
                           BuyoutManager &buyout_manager,
                           DataStore &datastore,
                           RateLimiter &rate_limiter)
    : m_settings(settings)
    , m_network_manager(network_manager)
    , m_repoe(repoe)
    , m_buyout_manager(buyout_manager)
    , m_datastore(datastore)
    , m_rate_limiter(rate_limiter)
    , m_auto_update_timer(std::make_unique<QTimer>())
{
    spdlog::trace("ItemsManager::ItemsManager() entered");

    const int interval = m_settings.value("autoupdate_interval", 30).toInt();
    m_auto_update_timer->setSingleShot(false);
    m_auto_update_timer->setInterval(interval * 60 * 1000);
    connect(m_auto_update_timer.get(), &QTimer::timeout, this, &ItemsManager::OnAutoRefreshTimer);

    const bool autoupdate = m_settings.value("autoupdate", false).toBool();
    if (autoupdate) {
        m_auto_update_timer->start();
    }
}

ItemsManager::~ItemsManager() {}

void ItemsManager::Start(POE_API mode)
{
    spdlog::trace("ItemsManager::Start() entered");
    spdlog::trace("ItemsManager::Start() creating items manager worker");
    m_worker = std::make_unique<ItemsManagerWorker>(m_settings,
                                                    m_network_manager,
                                                    m_repoe,
                                                    m_buyout_manager,
                                                    m_datastore,
                                                    m_rate_limiter,
                                                    mode);
    connect(this, &ItemsManager::UpdateSignal, m_worker.get(), &ItemsManagerWorker::Update);
    connect(m_worker.get(), &ItemsManagerWorker::StatusUpdate, this, &ItemsManager::OnStatusUpdate);
    connect(m_worker.get(),
            &ItemsManagerWorker::ItemsRefreshed,
            this,
            &ItemsManager::OnItemsRefreshed);

    spdlog::trace("ItemsManager::Start() initializing the worker");
    m_worker->Init();
}

void ItemsManager::OnStatusUpdate(ProgramState state, const QString &status)
{
    emit StatusUpdate(state, status);
}

void ItemsManager::ApplyAutoTabBuyouts()
{
    spdlog::trace("ItemsManager::ApplyAutoTabBuyouts() entered");
    // Can handle everything related to auto-tab pricing here.
    // 1. First format we need to honor is ascendency pricing formats which is top priority and overrides other types
    // 2. Second priority is to honor manual user pricing
    // 3. Third priority it to apply pricing based on ideally user specified formats (doesn't exist yet)

    // Loop over all tabs, create buyout based on tab name which applies auto-pricing policies
    for (auto const &loc : m_buyout_manager.GetStashTabLocations()) {
        auto tab_label = loc.get_tab_label();
        Buyout buyout = m_buyout_manager.StringToBuyout(tab_label);
        if (buyout.IsActive()) {
            m_buyout_manager.SetTab(loc.GetUniqueHash(), buyout);
        }
    }

    // Need to compress tab buyouts here, as the tab names change we accumulate and save BO's
    // for tabs that no longer exist I think.
    m_buyout_manager.CompressTabBuyouts();
}

void ItemsManager::ApplyAutoItemBuyouts()
{
    spdlog::trace("ItemsManager::ApplyAutoItemBuyouts() entered");
    // Loop over all items, check for note field with pricing and apply
    for (auto const &item : m_items) {
        auto const &note = item->note();
        if (!note.isEmpty()) {
            Buyout buyout = m_buyout_manager.StringToBuyout(note);
            // This line may look confusing, buyout returns an active buyout if game
            // pricing was found or a default buyout (inherit) if it was not.
            // If there is a currently valid note we want to apply OR if
            // old note no longer is valid (so basically clear pricing)
            if (buyout.IsActive() || m_buyout_manager.Get(*item).IsGameSet()) {
                m_buyout_manager.Set(*item, buyout);
            }
        }
    }

    // Commenting this out for robustness (iss381) to make it as unlikely as possible that users
    // pricing data will be removed.  Side effect is that stale pricing data will pile up and
    // could be applied to future items with the same hash (which includes tab name).
    // bo.CompressItemBuyouts(m_items);
}

void ItemsManager::PropagateTabBuyouts()
{
    spdlog::trace("ItemsManager::PropagateTabBuyouts() entered");
    m_buyout_manager.ClearRefreshLocks();
    for (auto &item_ptr : m_items) {
        Item &item = *item_ptr;
        QString hash = item.location().GetUniqueHash();
        auto item_bo = m_buyout_manager.Get(item);
        auto tab_bo = m_buyout_manager.GetTab(hash);

        if (item_bo.IsInherited()) {
            if (tab_bo.IsActive()) {
                // Any propagation from tab price to item price should include this bit set
                tab_bo.inherited = true;
                tab_bo.last_update = QDateTime::currentDateTime();
                m_buyout_manager.Set(item, tab_bo);
            } else {
                // This effectively 'clears' buyout by setting back to 'inherit' state.
                m_buyout_manager.Set(item, Buyout());
            }
        }

        // If any savable bo's are set on an item or the tab then lock the refresh state.
        // Skip remove-only tabs because they are not editable, nor indexed for trade now.
        if (item.location().removeonly() == false) {
            if (m_buyout_manager.Get(item).RequiresRefresh() || tab_bo.RequiresRefresh()) {
                m_buyout_manager.SetRefreshLocked(item.location());
            }
        }
    }
}

void ItemsManager::OnItemsRefreshed(const Items &items,
                                    const std::vector<ItemLocation> &tabs,
                                    bool initial_refresh)
{
    spdlog::trace("ItemsManager::OnItemsRefreshed() entered");
    m_items = items;

    spdlog::debug("There are {} items and {} tabs after the refresh.", m_items.size(), tabs.size());
    int n = 0;
    for (const auto &item : items) {
        if (item->category().isEmpty()) {
            spdlog::trace("Unable to categorize {}", item->PrettyName());
            ++n;
        }
    }
    if (n > 0) {
        spdlog::debug("There are {} uncategorized items.", n);
    }

    m_buyout_manager.SetStashTabLocations(tabs);
    MigrateBuyouts();
    ApplyAutoTabBuyouts();
    ApplyAutoItemBuyouts();
    PropagateTabBuyouts();

    emit ItemsRefreshed(initial_refresh);
}

void ItemsManager::Update(TabSelection type, const std::vector<ItemLocation> &locations)
{
    spdlog::trace("ItemsManager::Update() entered");
    if (!isInitialized()) {
        // tell ItemsManagerWorker to run an Update() after it's finished updating mods
        m_worker->UpdateRequest(type, locations);
        spdlog::debug("Update deferred until item mods parsing is complete");
        QMessageBox::information(
            nullptr,
            "Acquisition",
            "This items worker is still initializing, but an update request has been queued.",
            QMessageBox::Ok,
            QMessageBox::Ok);
    } else if (isUpdating()) {
        QMessageBox::information(nullptr,
                                 "Acquisition",
                                 "An update is already in progress.",
                                 QMessageBox::Ok,
                                 QMessageBox::Ok);
    } else {
        emit UpdateSignal(type, locations);
    }
}

void ItemsManager::SetAutoUpdate(bool update)
{
    spdlog::trace("ItemsManager::SetAutoUpdate() entered");
    m_settings.setValue("autoupdate", update);
    if (update) {
        spdlog::trace("ItemsManager::SetAutoUpdate() starting automatic updates");
        m_auto_update_timer->start();
    } else {
        spdlog::trace("ItemsManager::SetAutoUpdate() stopping automatic updates");
        m_auto_update_timer->stop();
    }
}

void ItemsManager::SetAutoUpdateInterval(int minutes)
{
    spdlog::trace("ItemsManager::SetAutoUpdateInterval() entered");
    spdlog::trace("ItemsManager::SetAutoUpdateInterval() setting interval to {} minutes", minutes);
    m_settings.setValue("autoupdate_interval", minutes);
    m_auto_update_timer->setInterval(minutes * 60 * 1000);
}

void ItemsManager::OnAutoRefreshTimer()
{
    spdlog::trace("ItemsManager::OnAutoRefreshTimer() entered");
    if (!isUpdating()) {
        Update(TabSelection::Checked);
    } else {
        spdlog::info("Skipping auto update because the previous update is not complete.");
    }
}

void ItemsManager::MigrateBuyouts()
{
    spdlog::trace("ItemsManager::MigrateBuyouts() entered");
    int db_version = m_datastore.GetInt("db_version");
    // Don't migrate twice
    if (db_version == 4) {
        spdlog::trace("ItemsManager::MigrateBuyouts() skipping migration because db_version is 4");
        return;
    }
    spdlog::trace("ItemsManager::MigrateBuyouts() migrating {} items", m_items.size());
    for (auto &item : m_items) {
        m_buyout_manager.MigrateItem(*item);
    }
    spdlog::trace(
        "ItemsManager::MigrateBuyouts() saving buyout manager and setting db_version to 4");
    m_buyout_manager.Save();
    m_datastore.SetInt("db_version", 4);
}
