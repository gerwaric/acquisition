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

#include <QString>
#include <QTimer>

#include <vector>

#include <ui/mainwindow.h>
#include <util/util.h>

#include "item.h"
#include "itemlocation.h"
#include "itemsmanagerworker.h"
#include "network_info.h"

class QSettings;

class BuyoutManager;
class DataStore;
class ItemsManagerWorker;
class RePoE;
class Shop;

/*
 * ItemsManager manages an ItemsManagerWorker (which lives in a separate thread)
 * and glues it to the rest of Acquisition.
 * (No longer true as of v0.10.0, but we will see if this causes performance issues).
 */
class ItemsManager : public QObject
{
    Q_OBJECT
public:
    explicit ItemsManager(QSettings &settings,
                          QNetworkAccessManager &network_manager,
                          RePoE &repoe,
                          BuyoutManager &buyout_manager,
                          DataStore &datastore,
                          RateLimiter &rate_limiter);
    ~ItemsManager();
    bool isInitialized() const { return m_worker ? m_worker->isInitialized() : false; }
    bool isUpdating() const { return m_worker ? m_worker->isUpdating() : false; }
    // Creates and starts the worker
    void Start(POE_API mode);
    void Update(TabSelection type,
                const std::vector<ItemLocation> &tab_names = std::vector<ItemLocation>());
    void SetAutoUpdateInterval(int minutes);
    void SetAutoUpdate(bool update);
    const Items &items() const { return m_items; }
    void ApplyAutoTabBuyouts();
    void ApplyAutoItemBuyouts();
    void PropagateTabBuyouts();
public slots:
    void OnAutoRefreshTimer();
    void OnStatusUpdate(ProgramState state, const QString &status);
    void OnItemsRefreshed(const Items &items,
                          const std::vector<ItemLocation> &tabs,
                          bool initial_refresh);
signals:
    void UpdateSignal(TabSelection type,
                      const std::vector<ItemLocation> &tab_names = std::vector<ItemLocation>());
    void ItemsRefreshed(bool initial_refresh);
    void StatusUpdate(ProgramState state, const QString &status);
    void UpdateModListSignal();

private:
    void MigrateBuyouts();

    QSettings &m_settings;
    QNetworkAccessManager &m_network_manager;
    RePoE &m_repoe;
    BuyoutManager &m_buyout_manager;
    DataStore &m_datastore;
    RateLimiter &m_rate_limiter;

    std::unique_ptr<QTimer> m_auto_update_timer;
    std::unique_ptr<ItemsManagerWorker> m_worker;
    Items m_items;
};
