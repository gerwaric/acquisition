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

#include "mainwindow.h"
#include "ui_mainwindow.h"

#include <QApplication>
#include <QBuffer>
#include <QClipboard>
#include <QEvent>
#include <QFile>
#include <QFontDatabase>
#include <QImageReader>
#include <QInputDialog>
#include <QLayout>
#include <QMessageBox>
#include <QMouseEvent>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QPainter>
#include <QPushButton>
#include <QScrollArea>
#include <QString>
#include <QStringList>
#include <QTabBar>
#include <QVersionNumber>
#include <QSettings>
#include <QStringListModel>
#include "QsLog.h"

#include <vector>

#include "buyoutmanager.h"
#include "currencymanager.h"
#include "datastore.h"
#include "filesystem.h"
#include "filters.h"
#include "flowlayout.h"
#include "imagecache.h"
#include "item.h"
#include "itemcategories.h"
#include "itemconstants.h"
#include "itemlocation.h"
#include "itemtooltip.h"
#include "itemsmanager.h"
#include "items_model.h"
#include "logpanel.h"
#include "modsfilter.h"
#include "network_info.h"
#include "oauthmanager.h"
#include "ratelimit.h"
#include "ratelimitdialog.h"
#include "ratelimiter.h"
#include "replytimeout.h"
#include "search.h"
#include "shop.h"
#include "updatechecker.h"
#include "util.h"
#include "version_defines.h"
#include "verticalscrollarea.h"

constexpr const char* POE_WEBCDN = "http://webcdn.pathofexile.com"; // Should be updated to https://web.poecdn.com ?

constexpr int CURRENT_ITEM_UPDATE_DELAY_MS = 100;
constexpr int SEARCH_UPDATE_DELAY_MS = 350;

MainWindow::MainWindow(
    QSettings& settings,
    QNetworkAccessManager& network_manager,
    RateLimiter& rate_limiter,
    DataStore& datastore,
    ItemsManager& items_manager,
    BuyoutManager& buyout_manager,
    Shop& shop)
    : settings_(settings)
    , network_manager_(network_manager)
    , rate_limiter_(rate_limiter)
    , datastore_(datastore)
    , items_manager_(items_manager)
    , buyout_manager_(buyout_manager)
    , shop_(shop)
    , ui(new Ui::MainWindow)
    , current_bucket_location_(nullptr)
    , current_search_(nullptr)
    , search_count_(0)
    , rate_limit_dialog_(nullptr)
    , quitting_(false)
{
    image_cache_ = new ImageCache(Filesystem::UserDir() + "/cache");

    connect(qApp, &QCoreApplication::aboutToQuit, this, [&]() { quitting_ = true; });

    InitializeUi();
    InitializeRateLimitDialog();
    InitializeLogging();
    InitializeSearchForm();

    const QString title = QString("Acquisition [%1] - %2 League [%3]").arg(
        QString(APP_VERSION_STRING),
        settings_.value("league").toString(),
        settings_.value("account").toString());
    setWindowTitle(title);
    setWindowIcon(QIcon(":/icons/assets/icon.svg"));

    delayed_update_current_item_.setInterval(CURRENT_ITEM_UPDATE_DELAY_MS);
    delayed_update_current_item_.setSingleShot(true);
    connect(&delayed_update_current_item_, &QTimer::timeout, this, &MainWindow::UpdateCurrentItem);

    delayed_search_form_change_.setInterval(SEARCH_UPDATE_DELAY_MS);
    delayed_search_form_change_.setSingleShot(true);
    connect(&delayed_search_form_change_, &QTimer::timeout, this, &MainWindow::OnSearchFormChange);

    LoadSettings();
    NewSearch();
}

MainWindow::~MainWindow() {
    delete ui;
    for (auto& search : searches_) {
        delete(search);
    };
    rate_limit_dialog_->close();
    rate_limit_dialog_->deleteLater();
}

void MainWindow::prepare(
    OAuthManager& oauth_manager,
    CurrencyManager& currency_manager,
    Shop& shop)
{
    connect(ui->actionShowOAuthToken, &QAction::triggered, &oauth_manager, &OAuthManager::showStatus);
    connect(ui->actionRefreshOAuthToken, &QAction::triggered, &oauth_manager, &OAuthManager::requestRefresh);

    connect(ui->actionListCurrency, &QAction::triggered, &currency_manager, &CurrencyManager::DisplayCurrency);
    connect(ui->actionExportCurrency, &QAction::triggered, &currency_manager, &CurrencyManager::ExportCurrency);
}

void MainWindow::InitializeRateLimitDialog() {
    rate_limit_dialog_ = new RateLimitDialog(this, &rate_limiter_);
    auto* const button = new QPushButton(this);
    button->setFlat(false);
    button->setText("Rate Limit Status");
    connect(button, &QPushButton::clicked, rate_limit_dialog_, &RateLimitDialog::show);
    connect(&rate_limiter_, &RateLimiter::Paused, this,
        [=](int pause) {
            if (pause > 0) {
                button->setText("Rate limited for " + QString::number(pause) + " seconds");
                button->setStyleSheet("font-weight: bold; color: red");
            } else if (pause == 0) {
                button->setText("Rate limiting is OFF");
                button->setStyleSheet("");
            } else {
                button->setText("ERROR: pause is " + QString::number(pause));
                button->setStyleSheet("");
            };
        });
    statusBar()->addPermanentWidget(button);
}

void MainWindow::InitializeLogging() {
    LogPanel* log_panel = new LogPanel(this, ui);
    QsLogging::DestinationPtr log_panel_ptr(log_panel);
    QsLogging::Logger::instance().addDestination(log_panel_ptr);

    // display warnings here so it's more visible
#if defined(_DEBUG)
    QLOG_WARN() << "Maintainer: This is a debug build";
#endif
}

void MainWindow::InitializeUi() {
    ui->setupUi(this);

    status_bar_label_ = new QLabel("Ready");
    statusBar()->addWidget(status_bar_label_);
    ui->itemLayout->setAlignment(Qt::AlignTop);
    ui->itemLayout->setAlignment(ui->minimapLabel, Qt::AlignHCenter);
    ui->itemLayout->setAlignment(ui->nameLabel, Qt::AlignHCenter);
    ui->itemLayout->setAlignment(ui->imageLabel, Qt::AlignHCenter);
    ui->itemLayout->setAlignment(ui->locationLabel, Qt::AlignHCenter);

    tab_bar_ = new QTabBar;
    tab_bar_->installEventFilter(this);
    tab_bar_->setExpanding(false);
    tab_bar_->addTab("+");
    tab_bar_->setSelectionBehaviorOnRemove(QTabBar::SelectLeftTab);
    connect(tab_bar_, &QTabBar::currentChanged, this, &MainWindow::OnTabChange);
    ui->mainLayout->insertWidget(0, tab_bar_);

    Util::PopulateBuyoutTypeComboBox(ui->buyoutTypeComboBox);
    Util::PopulateBuyoutCurrencyComboBox(ui->buyoutCurrencyComboBox);

    connect(ui->buyoutCurrencyComboBox, &QComboBox::activated, this, &MainWindow::OnBuyoutChange);
    connect(ui->buyoutTypeComboBox, &QComboBox::activated, this, &MainWindow::OnBuyoutChange);
    connect(ui->buyoutValueLineEdit, &QLineEdit::textEdited, this, &MainWindow::OnBuyoutChange);

    ui->viewComboBox->addItems({ "By Tab", "By Item" });
    connect(ui->viewComboBox, &QComboBox::activated, this,
        [&](int n) {
            const auto mode = static_cast<Search::ViewMode>(n);
            current_search_->SetViewMode(mode);
            if (mode == Search::ViewMode::ByItem) {
                OnExpandAll();
            } else {
                ResizeTreeColumns();
            };
        });

    ui->buyoutTypeComboBox->setEnabled(false);
    ui->buyoutValueLineEdit->setEnabled(false);
    ui->buyoutCurrencyComboBox->setEnabled(false);

    search_form_layout_ = new QVBoxLayout;
    search_form_layout_->setAlignment(Qt::AlignTop);
    search_form_layout_->setContentsMargins(0, 0, 0, 0);

    auto search_form_container = new QWidget;
    search_form_container->setLayout(search_form_layout_);

    auto scroll_area = new VerticalScrollArea;
    scroll_area->setFrameShape(QFrame::NoFrame);
    scroll_area->setWidgetResizable(true);
    scroll_area->setWidget(search_form_container);
    scroll_area->setMinimumWidth(150); // TODO(xyz): remove magic numbers
    scroll_area->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);

    ui->scrollArea->setFrameShape(QFrame::NoFrame);
    ui->scrollArea->setWidgetResizable(true);

    ui->horizontalLayout_2->insertWidget(0, scroll_area);
    search_form_container->show();

    ui->horizontalLayout_2->setStretchFactor(0, 2);
    ui->horizontalLayout_2->setStretchFactor(1, 5);
    ui->horizontalLayout_2->setStretchFactor(2, 0);

    ui->treeView->setContextMenuPolicy(Qt::CustomContextMenu);
    ui->treeView->setSelectionMode(QAbstractItemView::ExtendedSelection);
    ui->treeView->setSortingEnabled(true);

    context_menu_.addAction("Refresh Selected", this, &MainWindow::OnRefreshSelected);
    context_menu_.addAction("Check Selected", this, &MainWindow::OnCheckSelected);
    context_menu_.addAction("Uncheck Selected", this, &MainWindow::OnUncheckSelected);
    context_menu_.addSeparator();
    context_menu_.addAction("Check All", this, &MainWindow::OnCheckAll);
    context_menu_.addAction("Uncheck All", this, &MainWindow::OnUncheckAll);
    context_menu_.addSeparator();
    context_menu_.addAction("Expand All", this, &MainWindow::OnExpandAll);
    context_menu_.addAction("Collapse All", this, &MainWindow::OnCollapseAll);

    connect(ui->treeView, &QTreeView::customContextMenuRequested, this,
        [&](const QPoint& pos) {
            context_menu_.popup(ui->treeView->viewport()->mapToGlobal(pos));
        });

    refresh_button_.setStyleSheet("color: blue; font-weight: bold;");
    refresh_button_.setFlat(true);
    refresh_button_.hide();
    statusBar()->addPermanentWidget(&refresh_button_);
    connect(&refresh_button_, &QPushButton::clicked, this, &MainWindow::OnRefreshAllTabs);

    update_button_.setText("Update available");
    update_button_.setStyleSheet("color: blue; font-weight: bold;");
    update_button_.setFlat(true);
    update_button_.hide();
    statusBar()->addPermanentWidget(&update_button_);
    connect(&update_button_, &QPushButton::clicked, this, [=]() { emit UpdateCheckRequested(); });

    // resize columns when a tab is expanded/collapsed
    connect(ui->treeView, &QTreeView::collapsed, this, &MainWindow::ResizeTreeColumns);
    connect(ui->treeView, &QTreeView::expanded, this, &MainWindow::ResizeTreeColumns);

    ui->propertiesLabel->setStyleSheet("QLabel { background-color: black; color: #7f7f7f; padding: 10px; font-size: 17px; }");
    ui->propertiesLabel->setFont(QFont("Fontin SmallCaps"));
    ui->itemNameFirstLine->setFont(QFont("Fontin SmallCaps"));
    ui->itemNameSecondLine->setFont(QFont("Fontin SmallCaps"));
    ui->itemNameFirstLine->setAlignment(Qt::AlignCenter);
    ui->itemNameSecondLine->setAlignment(Qt::AlignCenter);

    ui->itemTextTooltip->setStyleSheet("QLabel { background-color: black; color: #7f7f7f; padding: 3px; }");

    ui->itemTooltipWidget->hide();
    ui->itemButtonsWidget->hide();

    // Make sure the right logging level menu item is checked.
    OnSetLogging(QsLogging::Logger::instance().loggingLevel());

    connect(ui->itemInfoTypeTabs, &QTabWidget::currentChanged, this,
        [=](int idx) {
            auto tabs = ui->itemInfoTypeTabs;
            for (int i = 0; i < tabs->count(); i++) {
                if (i != idx) {
                    tabs->widget(i)->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Ignored);
                };
            };
            tabs->widget(idx)->setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Preferred);
            tabs->widget(idx)->resize(tabs->widget(idx)->minimumSizeHint());
            tabs->widget(idx)->adjustSize();
            settings_.setValue("tooltip_tab", idx);
        });

    // Connect the Tabs menu
    connect(ui->actionRefreshCheckedTabs, &QAction::triggered, this, &MainWindow::OnRefreshCheckedTabs);
    connect(ui->actionRefreshAllTabs, &QAction::triggered, this, &MainWindow::OnRefreshAllTabs);
    connect(ui->actionSetAutomaticTabRefresh, &QAction::triggered, this, &MainWindow::OnSetAutomaticTabRefresh);
    connect(ui->actionSetTabRefreshInterval, &QAction::triggered, this, &MainWindow::OnSetTabRefreshInterval);

    // Connect the Shop menu
    connect(ui->actionSetShopThreads, &QAction::triggered, this, &MainWindow::OnSetShopThreads);
    connect(ui->actionEditShopTemplate, &QAction::triggered, this, &MainWindow::OnEditShopTemplate);
    connect(ui->actionCopyShopToClipboard, &QAction::triggered, this, &MainWindow::OnCopyShopToClipboard);
    connect(ui->actionUpdateShops, &QAction::triggered, this, &MainWindow::OnUpdateShops);
    connect(ui->actionSetAutomaticallyShopUpdate, &QAction::triggered, this, &MainWindow::OnSetAutomaticShopUpdate);

    // Connect the Theme submenu
    connect(ui->actionSetDarkTheme, &QAction::triggered, this, &MainWindow::OnSetDarkTheme);
    connect(ui->actionSetLightTheme, &QAction::triggered, this, &MainWindow::OnSetLightTheme);
    connect(ui->actionSetDefaultTheme, &QAction::triggered, this, &MainWindow::OnSetDefaultTheme);

    // Connect the Logging submenu
    connect(ui->actionLoggingOFF, &QAction::triggered, this, [=]() { OnSetLogging(QsLogging::OffLevel); });
    connect(ui->actionLoggingFATAL, &QAction::triggered, this, [=]() { OnSetLogging(QsLogging::FatalLevel); });
    connect(ui->actionLoggingERROR, &QAction::triggered, this, [=]() { OnSetLogging(QsLogging::ErrorLevel); });
    connect(ui->actionLoggingWARN, &QAction::triggered, this, [=]() { OnSetLogging(QsLogging::WarnLevel); });
    connect(ui->actionLoggingINFO, &QAction::triggered, this, [=]() { OnSetLogging(QsLogging::InfoLevel); });
    connect(ui->actionLoggingDEBUG, &QAction::triggered, this, [=]() { OnSetLogging(QsLogging::DebugLevel); });
    connect(ui->actionLoggingTRACE, &QAction::triggered, this, [=]() { OnSetLogging(QsLogging::TraceLevel); });

    // Connect the POESESSID submenu
    connect(ui->actionShowPOESESSID, &QAction::triggered, this, &MainWindow::OnShowPOESESSID);

    // Connect the Tooltip tab buttons
    connect(ui->uploadTooltipButton, &QPushButton::clicked, this, &MainWindow::OnUploadToImgur);
    connect(ui->pobTooltipButton, &QPushButton::clicked, this, &MainWindow::OnCopyForPOB);
}

void MainWindow::LoadSettings() {

    // Make sure the theme button is checked.
    const QString theme = settings_.value("theme", "default").toString().toLower();
    ui->actionSetDarkTheme->setChecked(theme == "dark");
    ui->actionSetLightTheme->setChecked(theme == "light");
    ui->actionSetDefaultTheme->setChecked(theme == "default");

    ui->actionSetAutomaticTabRefresh->setChecked(settings_.value("autoupdate").toBool());
    UpdateShopMenu();

    ui->itemInfoTypeTabs->setCurrentIndex(settings_.value("tooltip_tab").toInt());
}

void MainWindow::OnExpandAll() {
    QLOG_TRACE() << "MainWindow::OnExpandAll() entered";
    // Only need to expand the top level, which corresponds to buckets,
    // aka stash tabs and characters. Signals are blocked during this
    // operation, otherwise the column resize function connected to
    // the expanded() signal would be called repeatedly.
    setCursor(Qt::WaitCursor);
    ui->treeView->blockSignals(true);
    ui->treeView->expandToDepth(0);
    ui->treeView->blockSignals(false);
    ResizeTreeColumns();
    unsetCursor();
}

void MainWindow::OnCollapseAll() {
    QLOG_TRACE() << "MainWindow::OnCollapseAll() entered";
    // There is no depth-based collapse method, so manuall looping
    // over rows can be much faster than collapseAll() under some
    // conditions, possibly beecause those funcitons check every
    // element in the tree, which in our case will include all items.
    // 
    // Signals are blocked for the same reason as the expand all case.
    setCursor(Qt::WaitCursor);
    ui->treeView->blockSignals(true);
    const auto& model = *ui->treeView->model();
    const int rowCount = model.rowCount();
    for (int row = 0; row < rowCount; ++row) {
        const QModelIndex idx = model.index(row, 0, QModelIndex());
        ui->treeView->collapse(idx);
    };
    ui->treeView->blockSignals(false);
    ResizeTreeColumns();
    unsetCursor();
}

void MainWindow::OnCheckAll() {
    QLOG_TRACE() << "MainWindow::OnCheckAll() entered";
    for (auto const& bucket : current_search_->buckets()) {
        buyout_manager_.SetRefreshChecked(bucket.location(), true);
    };
    emit ui->treeView->model()->layoutChanged();
}

void MainWindow::OnUncheckAll() {
    QLOG_TRACE() << "MainWindow::OnUncheckAll() entered";
    for (auto const& bucket : current_search_->buckets()) {
        buyout_manager_.SetRefreshChecked(bucket.location(), false);
    };
    emit ui->treeView->model()->layoutChanged();
}

void MainWindow::OnRefreshSelected() {
    QLOG_TRACE() << "MainWindow::OnRefreshSelected()";
    // Get names of tabs to refresh
    std::vector<ItemLocation> locations;
    for (auto const& index : ui->treeView->selectionModel()->selectedRows()) {
        // Fetch tab names per index
        locations.push_back(current_search_->GetTabLocation(index));
    };
    items_manager_.Update(TabSelection::Selected, locations);
}

void MainWindow::CheckSelected(bool value) {
    QLOG_TRACE() << "MainWindow::CheckSelected() entered";
    for (auto const& index : ui->treeView->selectionModel()->selectedRows()) {
        buyout_manager_.SetRefreshChecked(current_search_->GetTabLocation(index), value);
    };
}

void MainWindow::ResizeTreeColumns() {
    QLOG_TRACE() << "MainWindow::ResizeTreeColumns() entered";
    for (int i = 0; i < ui->treeView->header()->count(); ++i) {
        ui->treeView->resizeColumnToContents(i);
    };
}

void MainWindow::OnBuyoutChange() {
    QLOG_TRACE() << "MainWindow::OnBuyoutChange() entered";
    shop_.ExpireShopData();

    Buyout bo;
    bo.type = Buyout::IndexAsBuyoutType(ui->buyoutTypeComboBox->currentIndex());
    bo.currency = Currency::FromIndex(ui->buyoutCurrencyComboBox->currentIndex());
    bo.value = ui->buyoutValueLineEdit->text().replace(',', ".").toDouble();
    bo.last_update = QDateTime::currentDateTime();

    if (bo.IsPriced()) {
        ui->buyoutCurrencyComboBox->setEnabled(true);
        ui->buyoutValueLineEdit->setEnabled(true);
    } else {
        ui->buyoutCurrencyComboBox->setEnabled(false);
        ui->buyoutValueLineEdit->setEnabled(false);
    };

    if (!bo.IsValid()) {
        QLOG_TRACE() << "MainWindow::OnBuyoutChange() buyout is invalid";
        return;
    };

    // Don't assign a zero buyout if nothing is entered in the value textbox
    if (ui->buyoutValueLineEdit->text().isEmpty() && bo.IsPriced()) {
        QLOG_TRACE() << "MainWindow::OnBuyoutChange() buyout iempty";
        return;
    };

    for (auto const& index : ui->treeView->selectionModel()->selectedRows()) {
        auto const& tab = current_search_->GetTabLocation(index).GetUniqueHash();

        // Don't allow users to manually update locked tabs (game priced)
        if (buyout_manager_.GetTab(tab).IsGameSet()) {
            QLOG_TRACE() << "MainWindow::OnBuyoutChange() refusing to update locked tab:" << tab;
            continue;
        };
        if (!index.parent().isValid()) {
            buyout_manager_.SetTab(tab, bo);
        } else {
            const int bucket_row = index.parent().row();
            if (current_search_->has_bucket(bucket_row)) {
                const Bucket& bucket = current_search_->bucket(bucket_row);
                const int item_row = index.row();
                if (bucket.has_item(item_row)) {
                    const Item& item = *bucket.item(item_row);
                    // Don't allow users to manually update locked items (game priced per item in note section)
                    if (buyout_manager_.Get(item).IsGameSet()) {
                        QLOG_TRACE() << "MainWindow::OnBuyoutChange() refusing to update locked item:" << item.name();
                        continue;
                    };
                    buyout_manager_.Set(item, bo);
                } else {
                    QLOG_ERROR() << "OnBuyoutChange(): bucket" << bucket_row << "does not have" << item_row << "items";
                };
            } else {
                QLOG_ERROR() << "OnBuyoutChange(): bucket" << bucket_row << "does not exist";
            };
        };
    };
    items_manager_.PropagateTabBuyouts();
    ResizeTreeColumns();
}

void MainWindow::OnStatusUpdate(ProgramState state, const QString& message) {
    QString status;
    switch (state) {
    case ProgramState::Initializing: status = "Initializing"; break;
    case ProgramState::Ready: status = "Ready"; break;
    case ProgramState::Busy: status = "Busy"; break;
    case ProgramState::Waiting: status = "Waiting"; break;
    case ProgramState::Unknown: status = "Unknown State"; break;
    };
    if (!message.isEmpty()) {
        status += ": " + message;
    };
    status_bar_label_->setText(status);
    status_bar_label_->update();
}

bool MainWindow::eventFilter(QObject* o, QEvent* e) {
    if ((o == tab_bar_) && (e->type() == QEvent::MouseButtonPress)) {
        QMouseEvent* mouse_event = static_cast<QMouseEvent*>(e);
        int index = tab_bar_->tabAt(mouse_event->pos());
        if ((index >= 0) && (index < tab_bar_->count() - 1)) {
            if (mouse_event->button() == Qt::MiddleButton) {
                OnDeleteTabClicked(index);
                return true;
            } else if (mouse_event->button() == Qt::RightButton) {
                QMenu menu;
                menu.addAction("Rename Tab", this, [=]() { OnRenameTabClicked(index); });
                menu.addAction("Delete Tab", this, [=]() { OnDeleteTabClicked(index); });
                menu.exec(QCursor::pos());
            };
        };
    };
    return QMainWindow::eventFilter(o, e);
}

void MainWindow::OnRenameTabClicked(int index) {
    bool ok;
    QString name = QInputDialog::getText(this, "Rename Tab",
        "Rename Tab here",
        QLineEdit::Normal, "", &ok);

    if (ok && !name.isEmpty()) {
        searches_[index]->RenameCaption(name.toStdString());
        tab_bar_->setTabText(index, searches_[index]->GetCaption());
    };
}

void MainWindow::OnDeleteTabClicked(int index) {
    // If the user is deleting the last search, create a new
    // one to replace it, because the UI breaks without at
    // least one search.
    if (searches_.size() == 1) {
        NewSearch();
    };

    // Delete the search.
    Search* search = searches_[index];
    if (current_search_ == search) {
        current_search_ = nullptr;
    };
    delete search;
    searches_.erase(searches_.begin() + index);

    // Remove the tab from the UI
    tab_bar_->removeTab(index);
}

void MainWindow::OnImageFetched(QNetworkReply* reply) {
    std::string url = reply->url().toString().toStdString();
    if (reply->error()) {
        QLOG_WARN() << "Failed to download item image," << url.c_str();
        return;
    };
    QImageReader image_reader(reply);
    QImage image = image_reader.read();

    image_cache_->Set(url, image);

    if (current_item_ && (url == current_item_->icon() || url == POE_WEBCDN + current_item_->icon())) {
        ui->imageLabel->setPixmap(GenerateItemIcon(*current_item_, image));
    };
}

void MainWindow::OnSearchFormChange() {
    QLOG_TRACE() << "MainWindow::OnSearchFormChange() entered";
    current_search_->SaveViewProperties();
    current_search_->SetRefreshReason(RefreshReason::SearchFormChanged);
    ModelViewRefresh();
}

void MainWindow::ModelViewRefresh() {
    QLOG_TRACE() << "MainWindow::ModelViewRefresh() entered";
    buyout_manager_.Save();

    QLOG_TRACE() << "MainWindow::ModelViewRefresh() activing current search";
    current_search_->Activate(items_manager_.items());
    ResizeTreeColumns();

    // This updates the item information when current item changes.
    connect(ui->treeView->selectionModel(), &QItemSelectionModel::currentChanged, this, &MainWindow::OnCurrentItemChanged);

    // This updates the item information when a search or sort order changes.
    connect(ui->treeView->model(), &QAbstractItemModel::layoutChanged, this, &MainWindow::OnLayoutChanged);

    ui->viewComboBox->setCurrentIndex(static_cast<int>(current_search_->GetViewMode()));

    tab_bar_->setTabText(tab_bar_->currentIndex(), current_search_->GetCaption());
}

void MainWindow::OnCurrentItemChanged(const QModelIndex& current, const QModelIndex& previous) {
    Q_UNUSED(previous);
    QLOG_TRACE() << "MainWindow::OnCurrentItemChange() entered";
    buyout_manager_.Save();

    if (!current.isValid()) {
        return;
    };

    if (current.parent().isValid()) {
        // Clicked on an item
        const int bucket_row = current.parent().row();
        if (current_search_->has_bucket(bucket_row)) {
            const Bucket& bucket = current_search_->bucket(bucket_row);
            const int item_row = current.row();
            if (bucket.has_item(item_row)) {
                current_item_ = bucket.item(item_row);
                delayed_update_current_item_.start();
            } else {
                QLOG_WARN() << "OnCurrentItemChanged(): parent bucket" << bucket_row << "does not have" << item_row << "rows";
            };
        } else {
            QLOG_WARN() << "OnCurrentItemChanged(): parent bucket" << bucket_row << "does not exist";
        };
    } else {
        // Clicked on a bucket
        current_item_ = nullptr;
        const int bucket_row = current.row();
        if (current_search_->has_bucket(bucket_row)) {
            current_bucket_location_ = &current_search_->bucket(bucket_row).location();
            UpdateCurrentBucket();
        } else {
            QLOG_WARN() << "OnCurrentItemChanged(): bucket" << bucket_row << "does not exist";
        };
    };
    UpdateCurrentBuyout();
}

void MainWindow::OnLayoutChanged() {
    QLOG_TRACE() << "MainWindow::OnLayoutChanged() entered";

    // Do nothing is nothing is selected.
    if (current_item_ == nullptr) {
        QLOG_TRACE() << "MainWindow::OnLayoutChange() nothing was selected";
        return;
    };

    // Reset the selection model, because using clear can cause exceptions
    // when after search updates for some reason that's not clear yet.
    ui->treeView->selectionModel()->reset();

    // Look for the new index of the currently selected item.
    const QModelIndex index = current_search_->index(current_item_);

    if (!index.isValid()) {
        // The previously selected item is no longer in search results.
        QLOG_TRACE() << "MainWindow::OnLayoutChange() the previously selected item is gone";
        current_item_ = nullptr;
        ClearCurrentItem();
    } else {
        // Reselect the item in the updated layout.
        QLOG_TRACE() << "MainWindow::OnLayouotChange() reselecting the previous item";
        ui->treeView->selectionModel()->select(index,
            QItemSelectionModel::Current |
            QItemSelectionModel::Select |
            QItemSelectionModel::Rows);
    };
}

void MainWindow::OnDelayedSearchFormChange() {
    delayed_search_form_change_.start();
}

void MainWindow::OnTabChange(int index) {
    if (static_cast<size_t>(index) == searches_.size()) {
        // "+" clicked
        NewSearch();
    } else {
        current_search_ = searches_[index];
        current_search_->SetRefreshReason(RefreshReason::TabChanged);
        current_search_->ToForm();
        ModelViewRefresh();
    };
}

void MainWindow::AddSearchGroup(QLayout* layout, const std::string& name = "") {
    if (!name.empty()) {
        auto label = new QLabel(("<h3>" + name + "</h3>").c_str());
        search_form_layout_->addWidget(label);
    };
    layout->setContentsMargins(0, 0, 0, 0);
    auto layout_container = new QWidget;
    layout_container->setLayout(layout);
    search_form_layout_->addWidget(layout_container);
}

void MainWindow::InitializeSearchForm() {

    // Initialize category list once.
    auto* category_model = new QStringListModel(GetItemCategories(), this);

    // Initialize rarity list once.
    auto* rarity_model = new QStringListModel(RaritySearchFilter::RARITY_LIST, this);

    auto name_search = std::make_unique<NameSearchFilter>(search_form_layout_);
    auto category_search = std::make_unique<CategorySearchFilter>(search_form_layout_, category_model);
    auto rarity_search = std::make_unique<RaritySearchFilter>(search_form_layout_, rarity_model);
    auto offense_layout = new FlowLayout;
    auto defense_layout = new FlowLayout;
    auto sockets_layout = new FlowLayout;
    auto requirements_layout = new FlowLayout;
    auto misc_layout = new FlowLayout;
    auto misc_flags_layout = new FlowLayout;
    auto misc_flags2_layout = new FlowLayout;
    auto mods_layout = new QVBoxLayout;

    AddSearchGroup(offense_layout, "Offense");
    AddSearchGroup(defense_layout, "Defense");
    AddSearchGroup(sockets_layout, "Sockets");
    AddSearchGroup(requirements_layout, "Requirements");
    AddSearchGroup(misc_layout, "Misc");
    AddSearchGroup(misc_flags_layout);
    AddSearchGroup(misc_flags2_layout);
    AddSearchGroup(mods_layout, "Mods");

    using move_only = std::unique_ptr<Filter>;
    move_only init[] = {
        std::move(name_search),
        std::move(category_search),
        std::move(rarity_search),
        // Offense
        // new DamageFilter(offense_layout, "Damage"),
        std::make_unique<SimplePropertyFilter>(offense_layout, "Critical Strike Chance", "Crit."),
        std::make_unique<ItemMethodFilter>(offense_layout, [](Item* item) { return item->DPS(); }, "DPS"),
        std::make_unique<ItemMethodFilter>(offense_layout, [](Item* item) { return item->pDPS(); }, "pDPS"),
        std::make_unique<ItemMethodFilter>(offense_layout, [](Item* item) { return item->eDPS(); }, "eDPS"),
        std::make_unique<ItemMethodFilter>(offense_layout, [](Item* item) { return item->cDPS(); }, "cDPS"),
        std::make_unique<SimplePropertyFilter>(offense_layout, "Attacks per Second", "APS"),
        // Defense
        std::make_unique<SimplePropertyFilter>(defense_layout, "Armour"),
        std::make_unique<SimplePropertyFilter>(defense_layout, "Evasion Rating", "Evasion"),
        std::make_unique<SimplePropertyFilter>(defense_layout, "Energy Shield", "Shield"),
        std::make_unique<SimplePropertyFilter>(defense_layout, "Chance to Block", "Block"),
        // Sockets
        std::make_unique<SocketsFilter>(sockets_layout, "Sockets"),
        std::make_unique<LinksFilter>(sockets_layout, "Links"),
        std::make_unique<SocketsColorsFilter>(sockets_layout),
        std::make_unique<LinksColorsFilter>(sockets_layout),
        // Requirements
        std::make_unique<RequiredStatFilter>(requirements_layout, "Level", "R. Level"),
        std::make_unique<RequiredStatFilter>(requirements_layout, "Str", "R. Str"),
        std::make_unique<RequiredStatFilter>(requirements_layout, "Dex", "R. Dex"),
        std::make_unique<RequiredStatFilter>(requirements_layout, "Int", "R. Int"),
        // Misc
        std::make_unique<DefaultPropertyFilter>(misc_layout, "Quality", 0),
        std::make_unique<SimplePropertyFilter>(misc_layout, "Level"),
        std::make_unique<SimplePropertyFilter>(misc_layout, "Map Tier"),
        std::make_unique<ItemlevelFilter>(misc_layout, "ilvl"),
        std::make_unique<AltartFilter>(misc_flags_layout, "", "Alt. art"),
        std::make_unique<PricedFilter>(misc_flags_layout, "", "Priced", buyout_manager_),
        std::make_unique<UnidentifiedFilter>(misc_flags2_layout, "", "Unidentified"),
        std::make_unique<InfluencedFilter>(misc_flags2_layout, "", "Influenced"),
        std::make_unique<CraftedFilter>(misc_flags2_layout, "", "Master-crafted"),
        std::make_unique<EnchantedFilter>(misc_flags2_layout, "", "Enchanted"),
        std::make_unique<CorruptedFilter>(misc_flags2_layout, "", "Corrupted"),
        std::make_unique<ModsFilter>(mods_layout)
    };
    filters_ = std::vector<move_only>(std::make_move_iterator(std::begin(init)), std::make_move_iterator(std::end(init)));
}

void MainWindow::NewSearch() {
    QLOG_TRACE() << "MainWindow::NewSearch() entered";

    ++search_count_;

    QString caption = QString("Search %1").arg(search_count_);

    QLOG_TRACE() << "MainWindow::NewSearch() adding tab";
    tab_bar_->setTabText(tab_bar_->count() - 1, caption);
    tab_bar_->addTab("+");

    QLOG_TRACE() << "MainWindow::NewSearch() setting current search:" << caption;
    current_search_ = new Search(
        buyout_manager_,
        caption.toStdString(),
        filters_,
        ui->treeView);
    current_search_->SetRefreshReason(RefreshReason::TabCreated);

    // this can't be done in ctor because it'll call OnSearchFormChange slot
    // and remove all previous search data
    QLOG_TRACE() << "MainWindow::NewSearch() reseting search form and adding the search";
    current_search_->ResetForm();
    searches_.push_back(current_search_);

    QLOG_TRACE() << "MainWindow::NewSearch() triggering model view refresh";
    ModelViewRefresh();
}

void MainWindow::ClearCurrentItem() {
    QLOG_TRACE() << "MainWindow::ClearCurrentItem() entered";
    ui->imageLabel->hide();
    ui->minimapLabel->hide();
    ui->locationLabel->hide();
    ui->itemTooltipWidget->hide();
    ui->itemButtonsWidget->hide();

    ui->nameLabel->setText("Select an item");
    ui->nameLabel->show();

    ui->pobTooltipButton->setEnabled(false);
}

void MainWindow::UpdateCurrentBucket() {
    QLOG_TRACE() << "MainWindow::UpdateCurrentBucket() entered";
    ui->imageLabel->hide();
    ui->minimapLabel->hide();
    ui->locationLabel->hide();
    ui->itemTooltipWidget->hide();
    ui->itemButtonsWidget->hide();

    ui->nameLabel->setText(current_bucket_location_->GetHeader().c_str());
    ui->nameLabel->show();

    ui->pobTooltipButton->setEnabled(false);
}

void MainWindow::UpdateCurrentItem() {
    QLOG_TRACE() << "MainWindow::UpdateCurrentItem() entered";
    if (current_item_ == nullptr) {
        ClearCurrentItem();
        return;
    };

    ui->imageLabel->show();
    ui->minimapLabel->show();
    ui->locationLabel->show();
    ui->itemTooltipWidget->show();
    ui->itemButtonsWidget->show();
    ui->nameLabel->hide();

    ui->imageLabel->setText("Loading...");
    ui->imageLabel->setStyleSheet("QLabel { background-color : rgb(12, 12, 43); color: white }");
    ui->imageLabel->setFixedSize(QSize(current_item_->w(), current_item_->h()) * PIXELS_PER_SLOT);

    // Everything except item image now lives in itemtooltip.cpp
    // in future should move everything tooltip-related there
    UpdateItemTooltip(*current_item_, ui);

    ui->pobTooltipButton->setEnabled(current_item_->Wearable());

    std::string icon = current_item_->icon();
    if (icon.size() && icon[0] == '/') {
        icon = POE_WEBCDN + icon;
    };
    if (!image_cache_->Exists(icon)) {
        QNetworkRequest request = QNetworkRequest(QUrl(icon.c_str()));
        request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
        QNetworkReply* reply = network_manager_.get(request);
        connect(reply, &QNetworkReply::finished, this, [=]() { OnImageFetched(reply); });
    } else {
        ui->imageLabel->setPixmap(GenerateItemIcon(*current_item_, image_cache_->Get(icon)));
    };
    ui->locationLabel->setText(current_item_->location().GetHeader().c_str());
}

void MainWindow::UpdateBuyoutWidgets(const Buyout& bo) {
    QLOG_TRACE() << "MainWindow::UpdateBuyoutWidgets() entered";
    ui->buyoutTypeComboBox->setCurrentIndex(bo.type);
    ui->buyoutTypeComboBox->setEnabled(!bo.IsGameSet());
    ui->buyoutCurrencyComboBox->setEnabled(false);
    ui->buyoutValueLineEdit->setEnabled(false);

    if (bo.IsPriced()) {
        ui->buyoutCurrencyComboBox->setCurrentIndex(bo.currency.type);
        ui->buyoutValueLineEdit->setText(QString::number(bo.value));
        if (!bo.IsGameSet()) {
            ui->buyoutCurrencyComboBox->setEnabled(true);
            ui->buyoutValueLineEdit->setEnabled(true);
        };
    } else {
        ui->buyoutValueLineEdit->setText("");
    };
}

void MainWindow::UpdateCurrentBuyout() {
    QLOG_TRACE() << "MainWindow::UpdateCurrentBuyout() entered";
    if (current_item_) {
        UpdateBuyoutWidgets(buyout_manager_.Get(*current_item_));
    } else {
        std::string tab = current_bucket_location_->GetUniqueHash();
        UpdateBuyoutWidgets(buyout_manager_.GetTab(tab));
    };
}

void MainWindow::OnItemsRefreshed() {
    QLOG_TRACE() << "MainWindow::OnItemsRefreshed() entered";
    int tab = 0;
    for (auto search : searches_) {
        search->SetRefreshReason(RefreshReason::ItemsChanged);
        // Don't update current search - it will be updated in OnSearchFormChange
        if (search != current_search_) {
            search->FilterItems(items_manager_.items());
            tab_bar_->setTabText(tab, search->GetCaption());
        };
        tab++;
    };
    ModelViewRefresh();
}

void MainWindow::OnSetShopThreads() {
    bool ok;
    QString thread = QInputDialog::getText(this, "Shop thread",
        "Enter thread number. You can enter multiple shops by separating them with a comma. More than one shop may be needed if you have a lot of items.",
        QLineEdit::Normal, Util::StringJoin(shop_.threads(), ",").c_str(), &ok);
    if (ok && !thread.isEmpty()) {
        static const auto spaces = QRegularExpression("\\s+");
        shop_.SetThread(Util::StringSplit(thread.remove(spaces).toStdString(), ','));
    };
    UpdateShopMenu();
}

void MainWindow::OnShowPOESESSID() {

    static QInputDialog* dialog = nullptr;

    // Create and configure the input dialog.
    if (!dialog) {
        dialog = new QInputDialog(this);
        dialog->setWindowTitle("Path of Exile - Session ID");
        dialog->setLabelText("POESESSID:");
        dialog->setInputMode(QInputDialog::TextInput);
        auto lineEdit = dialog->findChild<QLineEdit*>();
        if (lineEdit) {
            // Use a fixed width font for the input, and set it to be exactly
            // as wide as a POESESSID cookie.
            const QFont font = QFontDatabase::systemFont(QFontDatabase::FixedFont);
            const QFontMetrics metric(font);
            const int w = metric.horizontalAdvance("00000000000000000000000000000000");
            lineEdit->setFont(font);
            lineEdit->setMinimumWidth(w);
        };
    };

    // Load the session_id if it exists.
    dialog->setTextValue(settings_.value("session_id").toString());

    // Get the user input and set the session cookie.
    int code = dialog->exec();
    if (code == QDialog::DialogCode::Accepted) {
        const QString poesessid = dialog->textValue();
        emit SetSessionId(poesessid);
    };
}

void MainWindow::UpdateShopMenu() {
    std::string title = "Forum shop thread...";
    if (!shop_.threads().empty()) {
        title += " [" + Util::StringJoin(shop_.threads(), ",") + "]";
    };
    ui->actionSetShopThreads->setText(title.c_str());
    ui->actionSetAutomaticallyShopUpdate->setChecked(shop_.auto_update());
}

void MainWindow::OnUpdateAvailable() {
    update_button_.show();
}

void MainWindow::OnCopyShopToClipboard() {
    shop_.CopyToClipboard();
}

void MainWindow::OnSetTabRefreshInterval() {
    int interval = QInputDialog::getInt(this, "Auto refresh items", "Refresh items every X minutes",
        QLineEdit::Normal, settings_.value("autoupdate_interval").toInt());
    if (interval > 0) {
        items_manager_.SetAutoUpdateInterval(interval);
    };
}

void MainWindow::OnRefreshAllTabs() {
    items_manager_.Update(TabSelection::All);
}

void MainWindow::OnRefreshCheckedTabs() {
    items_manager_.Update(TabSelection::Checked);
}

void MainWindow::OnSetAutomaticTabRefresh() {
    items_manager_.SetAutoUpdate(ui->actionSetAutomaticTabRefresh->isChecked());
}

void MainWindow::OnUpdateShops() {
    shop_.SubmitShopToForum(true);
}

void MainWindow::OnEditShopTemplate() {
    bool ok;
    QString text = QInputDialog::getMultiLineText(this, "Shop template", "Enter shop template. [items] will be replaced with the list of items you marked for sale.",
        shop_.shop_template().c_str(), &ok);
    if (ok && !text.isEmpty()) {
        shop_.SetShopTemplate(text.toStdString());
    };
}

void MainWindow::OnSetAutomaticShopUpdate() {
    shop_.SetAutoUpdate(ui->actionSetAutomaticallyShopUpdate->isChecked());
}

void MainWindow::OnSetDarkTheme(bool toggle) {
    if (toggle) {
        emit SetTheme("dark");
        ui->actionSetLightTheme->setChecked(false);
        ui->actionSetDefaultTheme->setChecked(false);
        settings_.setValue("theme", "dark");
    };
    ui->actionSetDarkTheme->setChecked(toggle);
}

void MainWindow::OnSetLightTheme(bool toggle) {
    if (toggle) {
        emit SetTheme("light");
        ui->actionSetDarkTheme->setChecked(false);
        ui->actionSetDefaultTheme->setChecked(false);
        settings_.setValue("theme", "light");
    };
    ui->actionSetLightTheme->setChecked(toggle);
}

void MainWindow::OnSetDefaultTheme(bool toggle) {
    if (toggle) {
        emit SetTheme("default");
        ui->actionSetDarkTheme->setChecked(false);
        ui->actionSetLightTheme->setChecked(false);
        settings_.setValue("theme", "default");
    };
    ui->actionSetDefaultTheme->setChecked(toggle);
}

void MainWindow::OnSetLogging(QsLogging::Level level) {
    if ((level < QsLogging::TraceLevel) || (level > QsLogging::OffLevel)) {
        QLOG_ERROR() << "Cannot set invalid log level value:" << static_cast<int>(level);
        return;
    };
    QsLogging::Logger::instance().setLoggingLevel(level);
    ui->actionLoggingOFF->setChecked(level == QsLogging::OffLevel);
    ui->actionLoggingFATAL->setChecked(level == QsLogging::FatalLevel);
    ui->actionLoggingERROR->setChecked(level == QsLogging::ErrorLevel);
    ui->actionLoggingWARN->setChecked(level == QsLogging::WarnLevel);
    ui->actionLoggingINFO->setChecked(level == QsLogging::InfoLevel);
    ui->actionLoggingDEBUG->setChecked(level == QsLogging::DebugLevel);
    ui->actionLoggingTRACE->setChecked(level == QsLogging::TraceLevel);
    const QString level_name = Util::LogLevelToText(level);
    QLOG_INFO() << "Logging level set to" << level_name;
    settings_.setValue("log_level", level_name);
}

void MainWindow::closeEvent(QCloseEvent* event) {

    if (quitting_) {
        event->accept();
        return;
    };

    QMessageBox msgbox(this);
    msgbox.setWindowTitle("Acquisition");
    msgbox.setText(tr("Are you sure you want to quit?"));
    msgbox.setStandardButtons(QMessageBox::No | QMessageBox::Yes);
    msgbox.setDefaultButton(QMessageBox::Yes);

    const auto button = msgbox.exec();
    if (button == QMessageBox::Yes) {
        event->accept();
    } else {
        event->ignore();
    };
}

void MainWindow::OnUploadToImgur() {
    ui->uploadTooltipButton->setDisabled(true);
    ui->uploadTooltipButton->setText("Uploading...");

    QPixmap pixmap(ui->itemTooltipWidget->size());
    ui->itemTooltipWidget->render(&pixmap);

    QByteArray bytes;
    QBuffer buffer(&bytes);
    buffer.open(QIODevice::WriteOnly);
    pixmap.save(&buffer, "PNG"); // writes pixmap into bytes in PNG format

    QNetworkRequest request(QUrl("https://api.imgur.com/3/upload/"));
    request.setRawHeader("Authorization", "Client-ID d6d2d8a0437a90f");
    request.setHeader(QNetworkRequest::ContentTypeHeader, "application/x-www-form-urlencoded");
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    request.setTransferTimeout(kImgurUploadTimeout);
    QByteArray image_data = "image=" + QUrl::toPercentEncoding(bytes.toBase64());
    QNetworkReply* reply = network_manager_.post(request, image_data);
    connect(reply, &QNetworkReply::finished, this, &MainWindow::OnUploadFinished);
}

void MainWindow::OnCopyForPOB() {
    if (current_item_ == nullptr) {
        return;
    };
    // if category isn't wearable, including flasks, don't do anything
    if (!current_item_->Wearable()) {
        QLOG_WARN() << current_item_->PrettyName().c_str() << ", category:" << current_item_->category().c_str() << ", should not have been exportable.";
        return;
    };

    QApplication::clipboard()->setText(QString::fromStdString(current_item_->POBformat()));
    QLOG_INFO() << current_item_->PrettyName().c_str() << "was copied to your clipboard in Path of Building's \"Create custom\" format.";
}

void MainWindow::OnUploadFinished() {
    ui->uploadTooltipButton->setDisabled(false);
    ui->uploadTooltipButton->setText("Upload to imgur");

    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    QByteArray bytes = reply->readAll();
    reply->deleteLater();

    rapidjson::Document doc;
    doc.Parse(bytes.constData());

    if (doc.HasParseError() || !doc.IsObject() || !doc.HasMember("status") || !doc["status"].IsNumber()) {
        QLOG_ERROR() << "Imgur API returned invalid data (or timed out): " << bytes;
        return;
    };
    if (doc["status"].GetInt() != 200) {
        QLOG_ERROR() << "Imgur API returned status!=200: " << bytes;
        return;
    };
    if (!doc.HasMember("data") || !doc["data"].HasMember("link") || !doc["data"]["link"].IsString()) {
        QLOG_ERROR() << "Imgur API returned malformed reply: " << bytes;
        return;
    };
    std::string url = doc["data"]["link"].GetString();
    QApplication::clipboard()->setText(url.c_str());
    QLOG_INFO() << "Image successfully uploaded, the URL is" << url.c_str() << "It also was copied to your clipboard.";
}
