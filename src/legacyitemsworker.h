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

#include <QNetworkRequest>
#include <QObject>

#include <queue>
#include <set>
#include <utility>

#include "item.h"
#include "itemsmanagerworker.h"
#include "mainwindow.h"
#include "util.h"

class Application;
class QNetworkReply;

class LegacyItemsWorker : public ItemsManagerWorker {
	Q_OBJECT
public:
	LegacyItemsWorker(Application& app);
	virtual ~LegacyItemsWorker();
public slots:
	virtual void DoUpdate();
public slots:
	void OnMainPageReceived(QNetworkReply* reply);
	void OnCharacterListReceived(QNetworkReply* reply);
	void OnFirstTabReceived(QNetworkReply* reply);
	void OnTabReceived(QNetworkReply* reply, ItemLocation location);
	void FetchItems();
	void PreserveSelectedCharacter();
private:
	QNetworkRequest MakeTabRequest(int tab_index, bool tabs = false);
	QNetworkRequest MakeCharacterRequest(const std::string& name);
	QNetworkRequest MakeCharacterPassivesRequest(const std::string& name);
	void QueueRequest(const QNetworkRequest& request, const ItemLocation& location);
	void ParseItems(rapidjson::Value* value_ptr, ItemLocation base_location, rapidjson_allocator& alloc);
	std::vector<std::pair<std::string, std::string> > CreateTabsSignatureVector(std::string tabs);
	bool TabsChanged(rapidjson::Document& doc, QNetworkReply* network_reply, ItemLocation& location);

	std::queue<ItemsRequest> queue_;

	size_t total_completed_, total_needed_;
	size_t requests_completed_, requests_needed_;

	std::string tabs_as_string_;

	int queue_id_;
	std::string selected_character_;
};
