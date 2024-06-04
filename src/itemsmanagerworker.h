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

#include <QNetworkCookie>
#include <QNetworkRequest>
#include <QObject>

#include <queue>
#include <set>

#include "item.h"
#include "mainwindow.h"
#include "util.h"

class Application;
class DataStore;
class QNetworkReply;
class QSignalMapper;
class QTimer;
class BuyoutManager;
namespace RateLimit {
	struct StatusInfo;
	class RateLimiter;
};

struct ItemsRequest {
	int id{ -1 };
	QString endpoint;
	QNetworkRequest network_request;
	ItemLocation location;
};

struct ItemsReply {
	QNetworkReply* network_reply;
	ItemsRequest request;
};

class ItemsManagerWorker : public QObject {
	Q_OBJECT
public:
	ItemsManagerWorker(Application& app);
	bool isInitialized() const { return initialized_; }
	void UpdateRequest(TabSelection::Type type, const std::vector<ItemLocation>& locations);
public slots:
	void ParseItemMods();
	void Update(TabSelection::Type type, const std::vector<ItemLocation>& tab_names = std::vector<ItemLocation>());
public slots:
	void OnMainPageReceived();
	void OnCharacterListReceived(QNetworkReply* reply);
	void OnFirstTabReceived(QNetworkReply* reply);
	void OnTabReceived(QNetworkReply* reply, ItemLocation location);
	void FetchItems();
	void PreserveSelectedCharacter();
	void Init();
	void OnStatTranslationsReceived();
	void OnItemClassesReceived();
	void OnItemBaseTypesReceived();
signals:
	void ItemsRefreshed(const Items& items, const std::vector<ItemLocation>& tabs, bool initial_refresh);
	void StatusUpdate(ProgramState state, const QString& status);
	void ItemClassesUpdate(const QByteArray& classes);
	void ItemBaseTypesUpdate(const QByteArray& baseTypes);
private:
	void RemoveUpdatingTabs(const std::set<std::string>& tab_ids);
	void RemoveUpdatingItems(const std::set<std::string>& tab_ids);
	QNetworkRequest MakeTabRequest(int tab_index, bool tabs = false);
	QNetworkRequest MakeCharacterRequest(const std::string& name);
	QNetworkRequest MakeCharacterPassivesRequest(const std::string& name);
	void QueueRequest(const QString& endpoint, const QNetworkRequest& request, const ItemLocation& location);
	void ParseItems(rapidjson::Value* value_ptr, ItemLocation base_location, rapidjson_allocator& alloc);
	std::vector<std::pair<std::string, std::string> > CreateTabsSignatureVector(std::string tabs);
	void UpdateModList();
	bool TabsChanged(rapidjson::Document& doc, QNetworkReply* network_reply, ItemLocation& location);
	void FinishUpdate();

	Application& app_;
	RateLimit::RateLimiter& rate_limiter_;

	bool test_mode_;
	//std::unique_ptr<RateLimit::RateLimiter> rate_limiter_;
	std::vector<ItemLocation> tabs_;
	std::queue<ItemsRequest> queue_;

	// tabs_signature_ captures <"n", "id"> from JSON tab list, used as consistency check
	std::vector<std::pair<std::string, std::string> > tabs_signature_;
	Items items_;
	size_t total_completed_, total_needed_;
	size_t requests_completed_, requests_needed_;

	std::set<std::string> tab_id_index_;
	std::string tabs_as_string_;

	volatile bool initialized_;
	volatile bool updating_;

	bool cancel_update_;
	bool updateRequest_;
	TabSelection::Type type_;
	std::vector<ItemLocation> locations_;

	int queue_id_;
	std::string selected_character_;

	int first_stash_request_index_;
	std::string first_character_request_name_;

	bool need_stash_list_;
	bool need_character_list_;

	bool has_stash_list_;
	bool has_character_list_;

	std::queue<std::string> stat_translation_queue_;
};
