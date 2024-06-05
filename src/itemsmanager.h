/*
	Copyright 2014 Ilya Zhuravlev

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

#include <vector>

#include <QTimer>

#include "item.h"
#include "itemlocation.h"
#include "itemsmanagerworker.h"
#include "mainwindow.h"
#include "util.h"

class QThread;
class Application;
class BuyoutManager;
class DataStore;
class ItemsManagerWorker;
class Shop;
namespace RateLimit { struct StatusInfo; };

/*
 * ItemsManager manages an ItemsManagerWorker (which lives in a separate thread)
 * and glues it to the rest of Acquisition.
 * (No longer true as of v0.10.0, but we will see if this causes performance issues).
 */
class ItemsManager : public QObject {
	Q_OBJECT
public:
	explicit ItemsManager(Application& app);
	~ItemsManager();
	// Creates and starts the worker
	void Start();
	void Update(TabSelection::Type type, const std::vector<ItemLocation>& tab_names = std::vector<ItemLocation>());
	void SetAutoUpdateInterval(int minutes);
	void SetAutoUpdate(bool update);
	int auto_update_interval() const { return auto_update_interval_; }
	bool auto_update() const { return auto_update_; }
	const Items& items() const { return items_; }
	void ApplyAutoTabBuyouts();
	void ApplyAutoItemBuyouts();
	void PropagateTabBuyouts();
public slots:
	void OnAutoRefreshTimer();
	void OnStatusUpdate(ProgramState state, const QString& status);
	void OnItemsRefreshed(const Items& items, const std::vector<ItemLocation>& tabs, bool initial_refresh);
signals:
	void UpdateSignal(TabSelection::Type type, const std::vector<ItemLocation>& tab_names = std::vector<ItemLocation>());
	void ItemsRefreshed(bool initial_refresh);
	void StatusUpdate(ProgramState state, const QString& status);
	void UpdateModListSignal();
private:
	void MigrateBuyouts();

	// should items be automatically refreshed
	bool auto_update_;
	// items will be automatically updated every X minutes
	int auto_update_interval_;
	std::unique_ptr<QTimer> auto_update_timer_;
	std::unique_ptr<ItemsManagerWorker> worker_;
	Application& app_;
	Items items_;
};
