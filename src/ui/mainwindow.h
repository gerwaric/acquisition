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

#include <memory>
#include <QLabel>
#include <QMainWindow>
#include <QMenu>
#include <QPushButton>
#include <QCloseEvent>
#include <QTimer>

#include <util/spdlog_qt.h>

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
class LogPanel;
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
        QSettings& m_settings,
        QNetworkAccessManager& network_manager,
        RateLimiter& rate_limiter,
        DataStore& datastore,
        ItemsManager& items_mangaer,
        BuyoutManager& buyout_manager,
        Shop& shop,
        ImageCache& image_cache);
    ~MainWindow();
    std::vector<Column*> columns;
    void LoadSettings();

    void prepare(
        OAuthManager& oauth_manager,
        CurrencyManager& currency_manager);

signals:
    void UpdateCheckRequested();
    void SetSessionId(const QString& poesessid);
    void SetTheme(const QString& theme);
    void GetImage(const QString& url);
public slots:
    void OnCurrentItemChanged(const QModelIndex& current, const QModelIndex& previous);
    void OnLayoutChanged();
    void OnSearchFormChange();
    void OnDelayedSearchFormChange();
    void OnTabChange(int index);
    void OnImageFetched(const QString& url);
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
    void OnUpdateStashIndex();
    void OnUpdateShops();
    void OnSetAutomaticShopUpdate();
    void OnShowPOESESSID();

    // Theme submenu actions
    void OnSetDarkTheme(bool toggle);
    void OnSetLightTheme(bool toggle);
    void OnSetDefaultTheme(bool toggle);

    // Logging submenu actions
    void OnSetLogging(spdlog::level::level_enum level);

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
    void AddSearchGroup(QLayout* layout, const QString& name);
    bool eventFilter(QObject* o, QEvent* e);
    void UpdateShopMenu();
    void UpdateBuyoutWidgets(const Buyout& bo);
    void closeEvent(QCloseEvent* event);
    void CheckSelected(bool value);

    QSettings& m_settings;
    QNetworkAccessManager& m_network_manager;
    RateLimiter& m_rate_limiter;
    DataStore& m_datastore;
    ItemsManager& m_items_manager;
    BuyoutManager& m_buyout_manager;
    Shop& m_shop;
    ImageCache& m_image_cache;

    Ui::MainWindow* ui;

    std::shared_ptr<Item> m_current_item;
    const ItemLocation* m_current_bucket_location;
    std::vector<Search*> m_searches;
    Search* m_current_search;
    QTabBar* m_tab_bar;
    LogPanel* m_log_panel;
    std::vector<std::unique_ptr<Filter>> m_filters;
    int m_search_count;

    QLabel* m_status_bar_label;
    QVBoxLayout* m_search_form_layout;
    QMenu m_context_menu;
    QPushButton m_update_button;
    QPushButton m_refresh_button;
    QTimer m_delayed_update_current_item;
    QTimer m_delayed_search_form_change;
    RateLimitDialog* m_rate_limit_dialog;
    bool m_quitting;
};
