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

#include "QsLog.h"

#include "bucket.h"
#include "porting.h"

class QNetworkReply;
class QVBoxLayout;

class Application;
class Column;
class Filter;
class FlowLayout;
class ImageCache;
class Search;
class QStringListModel;

struct Buyout;

namespace Ui {
	class MainWindow;
}

enum class ProgramState {
	Unknown,
	CharactersReceived,
	ItemsReceive,
	ItemsPaused,
	ItemsCompleted,
	ShopSubmitting,
	ShopCompleted,
	UpdateCancelled,
	ItemsRetrieved
};

struct CurrentStatusUpdate {
	ProgramState state{ ProgramState::Unknown };
	size_t progress{ 0 }, total{ 0 }, cached{ 0 };
};

class MainWindow : public QMainWindow {
	Q_OBJECT
public:
	MainWindow(Application& app);
	~MainWindow();
	std::vector<Column*> columns;
public slots:
	void OnCurrentItemChanged(const QModelIndex& current, const QModelIndex& previous);
	void OnLayoutChanged();
	void OnSearchFormChange();
	void OnDelayedSearchFormChange();
	void OnTabChange(int index);
	void OnImageFetched(QNetworkReply* reply);
	void OnItemsRefreshed();
	void OnStatusUpdate(const CurrentStatusUpdate& status);
	void OnBuyoutChange();
	void ResizeTreeColumns();
	void OnExpandAll();
	void OnCollapseAll();
	void OnCheckAll();
	void OnUncheckAll();
	void OnCheckSelected() { CheckSelected(true); };
	void OnUncheckSelected() { CheckSelected(false); };
	void OnRenameTabClicked();
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
	void OnUpdatePOESESSID();

	// Theme submenu actions
	void OnSetDarkTheme(bool toggle);
	void OnSetLightTheme(bool toggle);
	void OnSetDefaultTheme(bool toggle);

	// Logging submenu actions
	void OnSetLogging(QsLogging::Level level);

	// Currency menu actions
	void OnListCurrency();
	void OnExportCurrency();

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
	void SetCurrentSearch(Search* search);
	void InitializeRateLimitPanel();
	void InitializeLogging();
	void InitializeSearchForm();
	void InitializeUi();
	void AddSearchGroup(QLayout* layout, const std::string& name);
	bool eventFilter(QObject* o, QEvent* e);
	void UpdateShopMenu();
	void UpdateBuyoutWidgets(const Buyout& bo);
	void closeEvent(QCloseEvent* event);
	void CheckSelected(bool value);

	Application& app_;
	Ui::MainWindow* ui;
	std::shared_ptr<Item> current_item_;
	Bucket current_bucket_;
	std::vector<Search*> searches_;
	Search* current_search_;
	Search* previous_search_{ nullptr };
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
	QStringListModel* category_string_model_;
	QStringListModel* rarity_search_model_;

	int rightClickedTabIndex = -1;
};
