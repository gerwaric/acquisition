// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

class QObject;
class QSettings;

class Application;
class BuyoutManager;
class CurrencyManager;
class DataStore;
class ImageCache;
class ItemsManager;
class ItemsManagerWorker;
class MainWindow;
class NetworkManager;
class RateLimiter;
class Shop;
class UpdateChecker;

MainWindow *CreateMainWindow(QSettings &settings,
                             NetworkManager &network_manager,
                             RateLimiter &rate_limiter,
                             DataStore &datastore,
                             ItemsManager &items_manager,
                             BuyoutManager &buyout_manager,
                             CurrencyManager &currency_manager,
                             Shop &shop,
                             ImageCache &image_cache);

void ConnectMainWindow(Application &application,
                       MainWindow &main_window,
                       ItemsManager &items_manager,
                       ItemsManagerWorker &items_worker,
                       Shop &shop,
                       UpdateChecker &update_checker,
                       ImageCache &image_cache);
void ShowMainWindow(MainWindow &window);
