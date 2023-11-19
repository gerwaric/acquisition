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
#include <QNetworkCookie>
#include <QNetworkRequest>
#include <QObject>

#include "util.h"
#include "item.h"
#include "mainwindow.h"
#include "ratelimit.h"

class Application;
class DataStore;
class QNetworkReply;
class QSignalMapper;
class QTimer;
class BuyoutManager;

namespace PoE {
	struct ListCharactersResult;
	struct ListStashesResult;
	struct GetCharacterResult;
	struct GetStashResult;
	struct Item;
}

struct ItemsRequest {
	int id;
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
	void Init();
	void Update(TabSelection::Type type, const std::vector<ItemLocation>& tab_names = std::vector<ItemLocation>());
	void OnItemClassesReceived(QNetworkReply* reply);
	void OnItemBaseTypesReceived(QNetworkReply* reply);
	void OnStatTranslationsReceived(QNetworkReply* reply);
	void OnStashListReceived(const PoE::ListStashesResult& result);
	void OnStashReceived(const PoE::GetStashResult& result);
	void OnCharacterListReceived(const PoE::ListCharactersResult& result);
	void OnCharacterReceived(const PoE::GetCharacterResult& result);
	void FetchItems();
	void OnRateLimitStatusUpdate(const QString& string);

signals:
	void GetRequest(const QString& endpoint, const QNetworkRequest& request, RateLimit::Callback callback);
	void ItemsRefreshed(const Items& items, const std::vector<ItemLocation>& tabs, bool initial_refresh);
	void StatusUpdate(const CurrentStatusUpdate& status);
	void RateLimitStatusUpdate(const QString& status);
	void ItemClassesUpdate(const QByteArray& classes);
	void ItemBaseTypesUpdate(const QByteArray& baseTypes);
private:
	void ParseItemMods();
	void RemoveUpdatingTabs(const std::set<std::string>& tab_ids);
	void RemoveUpdatingItems(const std::set<std::string>& tab_ids);
	void AddItems(const std::vector<PoE::Item>& items, const ItemLocation& location);
	void FinishRequest();
	void FinishUpdate();

	Application& app_;

	RateLimit::RateLimiter& rate_limiter_;
	std::vector<ItemLocation> tabs_;
	Items items_;
	std::set<std::string> tab_id_index_;

	std::set<std::string> stashes_to_request_;
	std::set<std::string> characters_to_request_;
	size_t requests_needed_, requests_completed_;

	volatile bool initialized_;
	volatile bool updating_;

	bool cancel_update_;
	bool updateRequest_;
	TabSelection::Type type_;
	std::vector<ItemLocation> locations_;
};
