// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QCloseEvent>
#include <QLabel>
#include <QMainWindow>
#include <QMenu>
#include <QPushButton>
#include <QTimer>

#include <memory>

#include <spdlog/spdlog.h>

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
class NetworkManager;
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

enum class ProgramState { Unknown, Initializing, Waiting, Ready, Busy };

class MainWindow : public QMainWindow
{
    Q_OBJECT
public:
    explicit MainWindow(QSettings &m_settings,
                        NetworkManager &network_manager,
                        RateLimiter &rate_limiter,
                        DataStore &datastore,
                        ItemsManager &items_mangaer,
                        BuyoutManager &buyout_manager,
                        CurrencyManager &currency_manager,
                        Shop &shop,
                        ImageCache &image_cache);
    ~MainWindow();
    std::vector<Column *> columns;
    void LoadSettings();

signals:
    void UpdateCheckRequested();
    void SetSessionId(const QString &poesessid);
    void SetTheme(const QString &theme);
    void GetImage(const QString &url);
public slots:
    void OnCurrentItemChanged(const QModelIndex &current, const QModelIndex &previous);
    void OnLayoutChanged();
    void OnSearchFormChange();
    void OnDelayedSearchFormChange();
    void OnTabChange(int index);
    void OnImageFetched(const QString &url);
    void OnItemsRefreshed();
    void OnStatusUpdate(ProgramState state, const QString &status);
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
    void OnFetchTabsList();
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
    void OnSetLogging(spdlog::level::level_enum level);

    // Buyouts menu
    void OnImportBuyouts();

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
    void AddSearchGroup(QLayout *layout, const QString &name);
    bool eventFilter(QObject *o, QEvent *e);
    void UpdateShopMenu();
    void UpdateBuyoutWidgets(const Buyout &bo);
    void closeEvent(QCloseEvent *event);
    void CheckSelected(bool value);

    QSettings &m_settings;
    NetworkManager &m_network_manager;
    RateLimiter &m_rate_limiter;
    DataStore &m_datastore;
    ItemsManager &m_items_manager;
    BuyoutManager &m_buyout_manager;
    CurrencyManager &m_currency_manager;
    Shop &m_shop;
    ImageCache &m_image_cache;

    Ui::MainWindow *ui;

    std::shared_ptr<Item> m_current_item;
    const ItemLocation *m_current_bucket_location;
    std::vector<Search *> m_searches;
    Search *m_current_search;
    QTabBar *m_tab_bar;
    LogPanel *m_log_panel;
    std::vector<std::unique_ptr<Filter>> m_filters;
    int m_search_count;

    QLabel *m_status_bar_label;
    QVBoxLayout *m_search_form_layout;
    QMenu m_context_menu;
    QPushButton m_update_button;
    QPushButton m_refresh_button;
    QTimer m_delayed_update_current_item;
    QTimer m_delayed_search_form_change;
    RateLimitDialog *m_rate_limit_dialog;
    bool m_quitting;
};
