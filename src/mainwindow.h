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

#include <memory>
#include <QLabel>
#include <QMainWindow>
#include <QMenu>
#include <QPushButton>
#include <QCloseEvent>
#include <QTimer>

#include "QsLogLevel.h"

class QNetworkAccessManager;
class QNetworkReply;
class QSettings;
class QStringListModel;
class QVBoxLayout;

class BuyoutManager;
class Column;
class CurrencyManager;
class DataStore;
class Filter;
class FlowLayout;
class ImageCache;
class Item;
class ItemLocation;
class ItemsManager;
class OAuthManager;
class RateLimiter;
class RateLimitDialog;
class Search;
class Shop;
class UpdateChecker;

struct Buyout;

namespace Ui {
    class MainWindow;
}

enum class ProgramState {
    Unknown,
    Initializing,
    Waiting,
    Ready,
    Busy
};

class MainWindow : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(
        QSettings& settings_,
        QNetworkAccessManager& network_manager,
        RateLimiter& rate_limiter,
        DataStore& datastore,
        ItemsManager& items_mangaer,
        BuyoutManager& buyout_manager,
        Shop& shop);
    ~MainWindow();
    std::vector<Column*> columns;
    void LoadSettings();

    void prepare(
        OAuthManager& oauth_manager,
        CurrencyManager& currency_manager,
        Shop& shop);

signals:
    void UpdateCheckRequested();
    void SetSessionId(const QString& poesessid);
    void SetTheme(const QString& theme);
public slots:
    void OnCurrentItemChanged(const QModelIndex& current, const QModelIndex& previous);
    void OnLayoutChanged();
    void OnSearchFormChange();
    void OnDelayedSearchFormChange();
    void OnTabChange(int index);
    void OnImageFetched(QNetworkReply* reply);
    void OnItemsRefreshed();
    void OnStatusUpdate(ProgramState state, const QString& status);
    void OnBuyoutChange();
    void ResizeTreeColumns();
    void OnExpandAll();
    void OnCollapseAll();
    void OnCheckAll();
    void OnUncheckAll();
    void OnCheckSelected() { CheckSelected(true); };
    void OnUncheckSelected() { CheckSelected(false); };
    void OnRenameTabClicked(int index);
    void OnDeleteTabClicked(int index);
    void OnRefreshSelected();
    void OnUpdateAvailable();
    void OnUploadFinished();

private slots:
    // Tabs menu actions
    void OnRefreshCheckedTabs();
    void OnRefreshAllTabs();
    void OnSetAutomaticTabRefresh();
    void OnSetTabRefreshInterval();

    // Shop menu actions
    void OnSetShopThreads();
    void OnEditShopTemplate();
    void OnCopyShopToClipboard();
    void OnUpdateShops();
    void OnSetAutomaticShopUpdate();
    void OnShowPOESESSID();

    // Theme submenu actions
    void OnSetDarkTheme(bool toggle);
    void OnSetLightTheme(bool toggle);
    void OnSetDefaultTheme(bool toggle);

    // Logging submenu actions
    void OnSetLogging(QsLogging::Level level);

    // Tooltip buttons
    void OnCopyForPOB();
    void OnUploadToImgur();

private:
    void ModelViewRefresh();
    void ClearCurrentItem();
    void UpdateCurrentBucket();
    void UpdateCurrentItem();
    void UpdateCurrentBuyout();
    void NewSearch();
    void InitializeRateLimitDialog();
    void InitializeLogging();
    void InitializeSearchForm();
    void InitializeUi();
    void AddSearchGroup(QLayout* layout, const std::string& name);
    bool eventFilter(QObject* o, QEvent* e);
    void UpdateShopMenu();
    void UpdateBuyoutWidgets(const Buyout& bo);
    void closeEvent(QCloseEvent* event);
    void CheckSelected(bool value);

    QSettings& settings_;
    QNetworkAccessManager& network_manager_;
    RateLimiter& rate_limiter_;
    DataStore& datastore_;
    ItemsManager& items_manager_;
    BuyoutManager& buyout_manager_;
    Shop& shop_;

    Ui::MainWindow* ui;

    std::shared_ptr<Item> current_item_;
    const ItemLocation* current_bucket_location_;
    std::vector<Search*> searches_;
    Search* current_search_;
    QTabBar* tab_bar_;
    std::vector<std::unique_ptr<Filter>> filters_;
    int search_count_;
    ImageCache* image_cache_;
    QLabel* status_bar_label_;
    QVBoxLayout* search_form_layout_;
    QMenu context_menu_;
    QPushButton update_button_;
    QPushButton refresh_button_;
    QTimer delayed_update_current_item_;
    QTimer delayed_search_form_change_;
    RateLimitDialog* rate_limit_dialog_;
    bool quitting_;
};
