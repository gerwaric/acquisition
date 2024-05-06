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

//#include <QNetworkCookie>
#include <QNetworkRequest>
#include <QObject>

#include <set>
#include <string>
#include <vector>

#include "item.h"
#include "mainwindow.h"
#include "util.h"

class Application;
class QNetworkReply;

namespace RateLimit { class RateLimiter; };

struct ItemsRequest {
	int id{ -1 };
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
	virtual ~ItemsManagerWorker() = 0;
	bool isInitialized() const { return initialized_; }
	void UpdateRequest(TabSelection::Type type, const std::vector<ItemLocation>& locations);
public slots:
	void ParseItemMods();
	void Update(TabSelection::Type type, const std::vector<ItemLocation>& tab_names = std::vector<ItemLocation>());
	virtual void DoUpdate() = 0;
	void Init();
	void OnStatTranslationsReceived(QNetworkReply* reply);
	void OnItemClassesReceived(QNetworkReply* reply);
	void OnItemBaseTypesReceived(QNetworkReply* reply);
signals:
	void ItemsRefreshed(const Items& items, const std::vector<ItemLocation>& tabs, bool initial_refresh);
	void StatusUpdate(const CurrentStatusUpdate& status);
	void ItemClassesUpdate(const QByteArray& classes);
	void ItemBaseTypesUpdate(const QByteArray& baseTypes);
	void StatTranslationsUpdate(const QByteArray& statTranslations);
private:
	void RemoveUpdatingTabs(const std::set<std::string>& tab_ids);
	void RemoveUpdatingItems(const std::set<std::string>& tab_ids);
	void ParseItems(rapidjson::Value* value_ptr, ItemLocation base_location, rapidjson_allocator& alloc);
	std::vector<std::pair<std::string, std::string> > CreateTabsSignatureVector(std::string tabs);
	void UpdateModList(QStringList StatTranslationUrls);
protected:
	void FinishUpdate();

protected:

	Application& app_;
	RateLimit::RateLimiter& rate_limiter_;

	volatile bool updating_;

	bool cancel_update_;

	std::vector<ItemLocation> tabs_;
	std::set<std::string> tab_id_index_;
	std::vector<std::pair<std::string, std::string> > tabs_signature_;

	Items items_;

	std::string first_fetch_tab_;
	int first_fetch_tab_id_;

private:

	volatile bool initialized_;

	bool updateRequest_;
	TabSelection::Type type_;
	std::vector<ItemLocation> locations_;

};
