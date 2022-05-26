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

#include <queue>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QObject>

#include "util.h"
#include "item.h"
#include "mainwindow.h"

class Application;
class DataStore;
class QNetworkReply;
class QSignalMapper;
class QTimer;
class BuyoutManager;
class TabCache;

const int kThrottleRequests = 45;
const int kThrottleSleep = 60;
const int kMaxCacheSize = (1000*1024*1024); // 1GB

struct ItemsRequest {
	int id;
	QNetworkRequest network_request;
	ItemLocation location;
};

struct ItemsReply {
	QNetworkReply *network_reply;
	ItemsRequest request;
};

class ItemsManagerWorker : public QObject {
	Q_OBJECT
public:
	ItemsManagerWorker(Application &app, QThread *thread);
	~ItemsManagerWorker();
	bool isModsUpdating(){ return modsUpdating_; }
	volatile bool initialModUpdateCompleted_;
	void UpdateRequest(TabSelection::Type type, const std::vector<ItemLocation> &locations) { updateRequest_ = true; type_ = type, locations_ = locations; }
public slots:
	void ParseItemMods();
	void Update(TabSelection::Type type, const std::vector<ItemLocation> &tab_names = std::vector<ItemLocation>());
public slots:
	void OnMainPageReceived();
	void OnCharacterListReceived();
	void OnFirstTabReceived();
	void OnTabReceived(int index);
	/*
	* Makes 45 requests at once, should be called every minute.
	* These values are approximated (GGG throttles requests)
	* based on some quick testing.
	*/
	void FetchItems(int limit = kThrottleRequests);
	void PreserveSelectedCharacter();
	void Init();

	void OnStatsReceived();
	void OnStatTranslationsReceived();
	void OnItemClassesReceived();
	void OnItemBaseTypesReceived();
signals:
	void ItemsRefreshed(const Items &items, const std::vector<ItemLocation> &tabs, bool initial_refresh);
	void StatusUpdate(const CurrentStatusUpdate &status);
	void ItemClassesUpdate(const QByteArray &classes);
	void ItemBaseTypesUpdate(const QByteArray &baseTypes);
private:

	QNetworkRequest MakeTabRequest(int tab_index, const ItemLocation &location, bool tabs = false, bool refresh = false);
	QNetworkRequest MakeCharacterRequest(const std::string &name, const ItemLocation &location);
	QNetworkRequest MakeCharacterPassivesRequest(const std::string &name, const ItemLocation &location);
	void QueueRequest(const QNetworkRequest &request, const ItemLocation &location);
	void ParseItems(rapidjson::Value *value_ptr, ItemLocation base_location, rapidjson_allocator &alloc);
	std::vector<std::pair<std::string, std::string> > CreateTabsSignatureVector(std::string tabs);
	void UpdateModList();

	QNetworkRequest Request(QUrl url, const ItemLocation &location, TabCache::Flags flags = TabCache::None);
	DataStore &data_;
	QNetworkAccessManager network_manager_;
	QSignalMapper *signal_mapper_;
	std::vector<ItemLocation> tabs_;
	std::queue<ItemsRequest> queue_;
	std::map<int, ItemsReply> replies_;
	// tabs_signature_ captures <"n", "id"> from JSON tab list, used as consistency check
	std::vector<std::pair<std::string, std::string> > tabs_signature_;
	bool cancel_update_{false};
	Items items_;
	int total_completed_, total_needed_, total_cached_;
	int requests_completed_, requests_needed_;
	int cached_requests_completed_{0};

	std::string tabs_as_string_;
	std::string league_;
	// set to true if updating right now
	volatile bool updating_;

	volatile bool modsUpdating_;
	bool updateRequest_;
	TabSelection::Type type_;
	std::vector<ItemLocation> locations_;

	int queue_id_;
	std::string selected_character_;

	std::string first_fetch_tab_{""};
	TabCache *tab_cache_{new TabCache()};
	const BuyoutManager &bo_manager_;
	std::string account_name_;
	TabSelection::Type tab_selection_;
	std::set<std::string> selected_tabs_;
};
