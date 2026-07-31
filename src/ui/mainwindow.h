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
#include <optional>
#include <vector>

#include <spdlog/spdlog.h>

#include "fetchsourcekey.h"
#include "filters/filterspec.h"
#include "item.h"
#include "itemlocation.h"
#include "refreshoutcome.h"
#include "util/programstate.h"

class QNetworkReply;
class QSettings;
class QVBoxLayout;

class BuyoutManager;
class CurrencyDialog;
class CurrencyManager;
class DataStore;
class ImageCache;
class Item;
class ItemLocation;
class ItemsManager;
class LogPanel;
class NetworkManager;
class RateLimiter;
class RateLimitDialog;
class Search;
class SearchForm;
class Shop;
class UpdateChecker;

struct Buyout;
struct BuyoutChangeSet;

namespace Ui {
    class MainWindow;
}

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
    void LoadSettings();

    // Injectable throttle period S for the D9 streamed-delta refilter
    // (items-pipeline M2; production value 60 s). The suite drives it at
    // milliseconds so throttle pins never wait wall-clock S.
    void SetDeltaThrottleInterval(int ms);

signals:
    void UpdateCheckRequested();
    void SetSessionId(const QString &poesessid);
    void SetTheme(const QString &theme);
    void GetImage(const QString &url);
public slots:
    // Streamed-delta consumers (items-pipeline M3, S4/D3): an active
    // By-Tab search applies each delta immediately as bucket-scoped row
    // operations and stays clean (R1-7); background searches keep M2 D9
    // rule 1 verbatim. A By-Item active search keeps the D9 throttled
    // fallback (S4 seam, deleted in S5). Aggregate reconciliations apply
    // as row removals scoped to the parent's bucket.
    void OnTabRefreshed(const ItemLocation &location, const Items &items);
    void OnChildrenReconciled(const ItemLocation &parent,
                              const std::vector<FetchSourceKey> &expected);
    // The typed terminal event (M2 D4): closes the selection-intent
    // window on every outcome, terminal failure included (R1-3/R2-1).
    void OnRefreshFinished(const RefreshOutcome &outcome);

    void OnCurrentItemChanged(const QModelIndex &current, const QModelIndex &previous);
    void OnSearchFormChange();
    void OnDelayedSearchFormChange();
    void OnTabChange(int index);
    void OnImageFetched(const QString &url);
    void OnItemsRefreshed();
    void OnStatusUpdate(ProgramState state, const QString &status);
    void OnNotifyUser(const QString &message);
    void OnShopWarning(const QString &message);
    void OnBuyoutChange();
    // The M3 S2 buyout batch response: one per outer batch boundary
    // (BuyoutManager::BuyoutsChanged).
    void OnBuyoutsChanged(const BuyoutChangeSet &changes);
    void ResizeTreeColumns();
    void ScheduleResizeTreeColumns();
    void OnExpandAll();
    void OnCollapseAll();
    void OnCheckAll();
    void OnUncheckAll();
    void OnCheckSelected() { CheckSelected(true); }
    void OnUncheckSelected() { CheckSelected(false); }
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

    // Currency menu actions
    void OnListCurrency();
    void OnExportCurrency();

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
    // D9 throttle internals, kept as the By-Item fallback seam (M3 S4,
    // deleted in S5): a non-resetting trailing throttle with period S
    // owned by the current search.
    bool DeltaIntersectsCurrentSearch(const ItemLocation &location, const Items &items) const;
    void ScheduleThrottledRefilter();
    void OnDeltaThrottleTimeout();

    // S4 delta-application tail: dirty-flag adjudication (R1-7), view-side
    // default-expansion of an inserted bucket, caption refresh, and the
    // selection-intent pass.
    void FinishDeltaApplication(bool processed, bool rows_changed, int inserted_bucket_row);
    // R1-3: reconciles the view's selection with the stable-id intent
    // after row operations — a removed selected row lapses visually but
    // keeps the intent; a visible item with the intent's id re-adopts the
    // selection through the global identity index.
    void ReconcileSelectionIntent();

    void ModelViewRefresh();
    void ReselectCurrentItem();
    void ReselectCurrentBucket();
    void FlushPendingSearchFormChange();
    void SaveViewExpansion(Search &search);
    void RestoreViewExpansion(Search &search);
    // R6-3 scroll preservation: a top-row anchor (bucket stable key +
    // item stable id) with the raw scrollbar value as the fallback when
    // the anchored row no longer exists after the reset.
    void SaveViewScroll(Search &search);
    void RestoreViewScroll(Search &search);
    void ClearCurrentItem();
    void UpdateCurrentBucket();
    void UpdateCurrentItem();
    void UpdateCurrentBuyout();
    void ResetBuyoutWidgets();
    void NewSearch();
    void InitializeRateLimitDialog();
    void InitializeLogging();
    void InitializeSearchForm();
    void InitializeUi();
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

    // Application owns BuyoutManager and outlives MainWindow. Keep m_searches
    // declared after m_filter_catalog so reverse destruction destroys searches
    // before the catalog they reference.
    FilterCatalog m_filter_catalog;

    Ui::MainWindow *ui;
    CurrencyDialog *m_currency_dialog;

    std::shared_ptr<Item> m_current_item;
    std::optional<ItemLocation> m_current_bucket_location;
    // The selection intent (M3 R1-3): the selected item's stable id, held
    // independently of row existence. During an active refresh (first
    // delta to the terminal outcome) a removal keeps it alive and a later
    // insertion re-adopts it; every RefreshFinished outcome closes the
    // window (R2-1). A user selection overwrites it at any time.
    QString m_selection_intent_id;
    bool m_refresh_active{false};
    // R2-1 closure deferred past the fallback seam (S4 review round 1): a
    // terminal outcome arriving while the current search is items-dirty
    // (the By-Item fallback skipped application) cannot adjudicate the
    // intent against that search's stale indexes; the search's next
    // refilter is the first honest state, and the closure runs there.
    bool m_intent_close_pending{false};
    // Row operations make the view shuffle its current index (Qt moves
    // current off a removed row); while set, OnCurrentItemChanged ignores
    // those shifts so they cannot overwrite the intent.
    bool m_applying_delta{false};
    // While set, ModelViewRefresh skips its capture pass (S4 review
    // round 2): the dirty mode-switch handler captures the PRE-SWITCH
    // view state itself, and a capture inside the refresh would observe
    // the post-flip transient instead and overwrite it.
    bool m_suppress_view_capture{false};
    std::vector<std::unique_ptr<Search>> m_searches;
    Search *m_current_search;
    QTabBar *m_tab_bar;
    LogPanel *m_log_panel;
    std::unique_ptr<SearchForm> m_search_form;
    int m_search_count;

    QLabel *m_status_bar_label;
    QVBoxLayout *m_search_form_layout;
    QMenu m_context_menu;
    QPushButton m_update_button;
    QPushButton m_refresh_button;
    QTimer m_delayed_update_current_item;
    QTimer m_delayed_search_form_change;
    QTimer m_delayed_resize_columns;
    // The current search's pending D9 tick (single-shot, period S). Started
    // by the first intersecting delta, never re-armed by later arrivals;
    // canceled by a successful refilter of the current search, a tab switch
    // or deletion of the current search, and the final snapshot.
    QTimer m_delta_throttle;
    QMetaObject::Connection m_current_item_conn;
    RateLimitDialog *m_rate_limit_dialog;
    bool m_quitting;
};
