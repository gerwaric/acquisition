// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QString>
#include <QTimer>

#include <vector>

#include "item.h"
#include "itemlocation.h"
#include "itemsmanagerworker.h"
#include "ui/mainwindow.h"
#include "util/util.h"

class BuyoutManager;
class SessionStore;
class Shop;
namespace app {
    class UserSettings;
}

/*
 * ItemsManager manages an ItemsManagerWorker (which lives in a separate thread)
 * and glues it to the rest of Acquisition.
 * (No longer true as of v0.10.0, but we will see if this causes performance issues).
 */
class ItemsManager : public QObject
{
    Q_OBJECT
public:
    explicit ItemsManager(app::UserSettings &settings,
                          BuyoutManager &buyout_manager,
                          SessionStore &data);
    ~ItemsManager();

    //bool isInitialized() const { return m_worker ? m_worker->isInitialized() : false; }
    //bool isUpdating() const { return m_worker ? m_worker->isUpdating() : false; }
    // Creates and starts the worker
    void Start();
    void Update(TabSelection type, const std::vector<ItemLocation> &tab_names = {});
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
    void UpdateSignal(Util::TabSelection type, const std::vector<ItemLocation> &tab_names = {});
    void ItemsRefreshed(bool initial_refresh);
    void StatusUpdate(ProgramState state, const QString &status);
    void UpdateModListSignal();

private:
    app::UserSettings &m_settings;
    SessionStore &m_data;
    BuyoutManager &m_buyout_manager;

    std::unique_ptr<QTimer> m_auto_update_timer;
    Items m_items;
};
