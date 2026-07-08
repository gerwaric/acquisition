// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "ui/mainwindow_bridge.h"

#include "application.h"
#include "imagecache.h"
#include "itemsmanager.h"
#include "itemsmanagerworker.h"
#include "shop.h"
#include "ui/mainwindow.h"
#include "util/updatechecker.h"

void MainWindowDeleter::operator()(MainWindow *window) const
{
    delete window;
}

MainWindow *CreateMainWindow(QSettings &settings,
                             NetworkManager &network_manager,
                             RateLimiter &rate_limiter,
                             DataStore &datastore,
                             ItemsManager &items_manager,
                             BuyoutManager &buyout_manager,
                             CurrencyManager &currency_manager,
                             Shop &shop,
                             ImageCache &image_cache)
{
    return new MainWindow(settings,
                          network_manager,
                          rate_limiter,
                          datastore,
                          items_manager,
                          buyout_manager,
                          currency_manager,
                          shop,
                          image_cache);
}

void ConnectMainWindow(Application &application,
                       MainWindow &main_window,
                       ItemsManager &items_manager,
                       ItemsManagerWorker &items_worker,
                       Shop &shop,
                       UpdateChecker &update_checker,
                       ImageCache &image_cache)
{
    QObject::connect(&main_window,
                     &MainWindow::SetSessionId,
                     &application,
                     &Application::SetSessionId);
    QObject::connect(&main_window, &MainWindow::SetTheme, &application, &Application::SetTheme);
    QObject::connect(&main_window,
                     &MainWindow::UpdateCheckRequested,
                     &update_checker,
                     &UpdateChecker::CheckForUpdates);

    QObject::connect(&items_manager,
                     &ItemsManager::ItemsRefreshed,
                     &main_window,
                     &MainWindow::OnItemsRefreshed);
    QObject::connect(&items_manager,
                     &ItemsManager::StatusUpdate,
                     &main_window,
                     &MainWindow::OnStatusUpdate);
    QObject::connect(&items_worker,
                     &ItemsManagerWorker::NotifyUser,
                     &main_window,
                     &MainWindow::OnNotifyUser);

    QObject::connect(&main_window, &MainWindow::GetImage, &image_cache, &ImageCache::fetch);
    QObject::connect(&image_cache,
                     &ImageCache::imageReady,
                     &main_window,
                     &MainWindow::OnImageFetched);

    QObject::connect(&shop, &Shop::StatusUpdate, &main_window, &MainWindow::OnStatusUpdate);
    QObject::connect(&update_checker,
                     &UpdateChecker::UpdateAvailable,
                     &main_window,
                     &MainWindow::OnUpdateAvailable);
}

void ShowMainWindow(MainWindow &window)
{
    window.show();
}
