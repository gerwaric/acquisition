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

#include <deque>
#include <set>
#include <unordered_map>

#include "poe_api/poe_typedefs.h"
#include "poe_api/poe_character.h"
#include "poe_api/poe_item.h"
#include "poe_api/poe_stash.h"

#include "item.h"
#include "mainwindow.h"
#include "ratelimit.h"
#include "repoe.h"
#include "util.h"

class Application;

class ItemsManagerWorker : public QObject {
	Q_OBJECT

public:
	ItemsManagerWorker(Application& app);

public slots:
	void OnStatTranslationsReceived(const RePoE::StatTranslations& translations);
	void Update(TabSelection::Type type, const std::vector<ItemLocation>& locations = {});

signals:
	void GetRequest(const QString& endpoint, const QNetworkRequest& request, RateLimit::Callback callback);
	void ItemsRefreshed(const Items& items, const std::vector<ItemLocation>& tabs, bool initial_refresh);
	void StatusUpdate(const CurrentStatusUpdate& status);
	void RateLimitStatusUpdate(const QString& status);

private:
	void Init();
	void ParseItemMods();
	void ImportItems(const std::vector<PoE::Item>& items, const ItemLocation& location);
	void AddCharacter(const PoE::Character& character);
	void AddStash(const PoE::StashTab& stash);
	void AddItems(const std::vector<PoE::Item>& items, const ItemLocation& location);
	void StartRequests();
	void RequestNextCharacter();
	void RequestNextStash();
	void FinishRequest();
	void FinishUpdate();
	void EmitItemsUpdate();

private slots:
	void OnCharacterListReceived(const std::vector<PoE::Character>& characters);
	void OnCharacterReceived(const PoE::Character& character);
	void OnStashListReceived(const std::vector<PoE::StashTab>& stashes);
	void OnStashReceived(const PoE::StashTab& stash);
	void OnRateLimitStatusUpdate(const QString& string);

private:
	Application& app_;
	std::unordered_map<PoE::CharacterName, PoE::Character> characters_;
	std::unordered_map<PoE::StashId, PoE::StashTab> stashes_;
	std::unordered_map<PoE::ItemId, std::shared_ptr<Item>> items_;
	std::vector<ItemLocation> locations_;

	bool update_requested;
	TabSelection::Type selection_type;
	std::vector<ItemLocation> selected_locations;
	std::set<PoE::StashId> selected_stashes;
	std::set<PoE::CharacterName> selected_characters;

	bool characters_received;
	bool stashes_received;
	std::deque<PoE::StashId> queued_stashes;
	std::deque<PoE::CharacterName> queued_characters;

	size_t requests_needed_;
	size_t requests_completed_;

	std::unordered_map<PoE::StashId, ItemLocation*> parents;

	volatile bool initialized_;
	volatile bool updating_;
	bool cancel_update_;
};
