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
#include "ratelimit.h"

using Util::TabSelection;

class Application;
class DataStore;
class QNetworkReply;
class QSignalMapper;
class QTimer;
class BuyoutManager;
class TabCache;

struct ItemsRequest {
    int id{ -999 };
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
	void UpdateRequest(TabSelection type, const std::vector<ItemLocation> &locations) { updateRequest_ = true; type_ = type, locations_ = locations; }
public slots:
	void ParseItemMods();
	void Update(TabSelection type, const std::vector<ItemLocation> &tab_names = std::vector<ItemLocation>());
public slots:
	void OnMainPageReceived(QNetworkReply* reply);
	void OnCharacterListReceived(QNetworkReply* reply);
	void OnFirstTabReceived(QNetworkReply* reply);
	void OnTabReceived(QNetworkReply* reply, int index, ItemLocation location);
	void FetchItems();
	void PreserveSelectedCharacter();
	void Init();

	void OnStatTranslationsReceived(QNetworkReply* reply);
	void OnItemClassesReceived(QNetworkReply* reply);
	void OnItemBaseTypesReceived(QNetworkReply* reply);
signals:
	void ItemsRefreshed(const Items &items, const std::vector<ItemLocation> &tabs, bool initial_refresh);
	void StatusUpdate(const CurrentStatusUpdate &status);
	void ItemClassesUpdate(const QByteArray &classes);
	void ItemBaseTypesUpdate(const QByteArray &baseTypes);
private:

    QNetworkRequest MakeTabRequest(int tab_index, bool tabs = false);
	QNetworkRequest MakeCharacterRequest(const std::string &name);
	QNetworkRequest MakeCharacterPassivesRequest(const std::string &name);
	void QueueRequest(const QNetworkRequest &request, const ItemLocation &location);
	void ParseItems(rapidjson::Value *value_ptr, ItemLocation base_location, rapidjson_allocator &alloc);
	std::vector<std::pair<std::string, std::string> > CreateTabsSignatureVector(std::string tabs);
	void UpdateModList();

	DataStore &data_;
	QNetworkAccessManager network_manager_;
	std::vector<ItemLocation> tabs_;
	std::queue<ItemsRequest> queue_;
	// tabs_signature_ captures <"n", "id"> from JSON tab list, used as consistency check
	std::vector<std::pair<std::string, std::string> > tabs_signature_;
	bool cancel_update_{false};
	Items items_;
    int total_completed_, total_needed_;
	int requests_completed_, requests_needed_;

    std::set<std::string> tab_id_index_;
	std::string tabs_as_string_;
	std::string league_;
	// set to true if updating right now
	volatile bool updating_;

	volatile bool modsUpdating_;
	bool updateRequest_;
	TabSelection type_;
	std::vector<ItemLocation> locations_;

	int queue_id_;
	std::string selected_character_;

	std::string first_fetch_tab_{""};
    int first_fetch_tab_id_;
	const BuyoutManager &bo_manager_;
	std::string account_name_;
	TabSelection tab_selection_;
	std::set<std::string> selected_tabs_;

    RateLimit::RateLimiter& rate_limiter_;

    void ItemsManagerWorker::CheckForViolation(QNetworkReply* reply);
};
