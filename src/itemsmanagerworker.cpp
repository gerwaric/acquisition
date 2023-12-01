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

#include "itemsmanagerworker.h"

#include <QNetworkAccessManager>
#include <QNetworkCookie>
#include <QNetworkCookieJar>
#include <QNetworkReply>
#include <QSignalMapper>
#include "QsLog.h"
#include <QTimer>
#include <QUrlQuery>
#include <algorithm>
#include <boost/algorithm/string.hpp>

#include "json_struct/json_struct.h"
#include "poe_api/poe_character.h"
#include "poe_api/poe_stash.h"

#include "application.h"
#include "datastore.h"
#include "util.h"
#include "mainwindow.h"
#include "buyoutmanager.h"
#include "modlist.h"
#include "ratelimit.h"
#include "repoe.h"
#include "oauth.h"

using RateLimit::RateLimiter;

const char* kPOE_trade_stats = "https://www.pathofexile.com/api/trade/data/stats";

ItemsManagerWorker::ItemsManagerWorker(Application& app) :
	app_(app),
	requests_needed_(0),
	requests_completed_(0),
	initialized_(false),
	updating_(false),
	cancel_update_(false),
	update_requested(false),
	selection_type(TabSelection::Type::Checked),
	characters_received(false),
	stashes_received(false)
{
	// This is how we make rate-limited requests.
	connect(this, &ItemsManagerWorker::GetRequest, &app_.rate_limiter(), &RateLimit::RateLimiter::Submit);

	// This is how we get the status of rate limiting.
	connect(&app_.rate_limiter(), &RateLimiter::StatusUpdate, this, &ItemsManagerWorker::OnRateLimitStatusUpdate);
}

void ItemsManagerWorker::OnStatTranslationsReceived(const RePoE::StatTranslations& translations) {

	std::set<std::string> stat_strings;
	for (const auto& translation : translations) {
		for (const auto& stat : translation.English) {
			const std::vector<std::string>& formats = stat.format;
			std::string stat_string = stat.string;
			for (int i = 0; i < formats.size(); i++) {
				if (formats[i] == "#") {
					const std::string searchString = "{" + std::to_string(i) + "}";
					boost::replace_all(stat_string, searchString, formats[i]);
				};
			};
			stat_strings.insert(stat_string);
		};
	};

	mods.clear();
	for (std::string stat_string : stat_strings) {
		mods.push_back({ stat_string });
	};

	Init();
}

void ItemsManagerWorker::Init() {

	if (updating_) {
		QLOG_WARN() << "ItemsManagerWorker::Init() called while updating, skipping Mod List Update";
		return;
	};
	updating_ = true;

	// Create a separate thread to load the items, which allows the UI to
	// update the status bar while items are being parsed. This operation
	// can take tens of seconds or longer depending on the nubmer of tabs
	// and items.
	QThread* parser = QThread::create([=]() { ParseItemMods(); });
	parser->start();
}

void ItemsManagerWorker::ParseItemMods() {
	InitModlist();

	stashes_ = app_.data().GetStashes();
	characters_ = app_.data().GetCharacters();

	locations_.clear();
	locations_.reserve(stashes_.size() + characters_.size());

	items_.clear();

	for (auto& it : stashes_) {
		const auto& stash = it.second;
		if (stash.items) {
			const auto& here = stash.parent ? stashes_[PoE::StashId(*stash.parent)] : stash;
			const ItemLocation location(here);
			locations_.push_back(location);
			ImportItems(*stash.items, location);
		};
	};

	for (auto& it : characters_) {
		const auto& character = it.second;
		const auto& inventories = { character.equipment, character.inventory, character.jewels };
		for (const auto& items : inventories) {
			if (items) {
				ItemLocation location(character);
				locations_.push_back(location);
				ImportItems(*items, location);
			};
		};
	};

	initialized_ = true;
	updating_ = false;

	EmitItemsUpdate();

	if (update_requested) {
		update_requested = false;
		Update(selection_type, selected_locations);
	};
}

void ItemsManagerWorker::ImportItems(const std::vector<PoE::Item>& items, const ItemLocation& location) {
	for (const auto& item : items) {
		if (!item.id) {
			QLOG_ERROR() << item.name << ":" << item.typeLine << "does not have an id.";
			continue;
		};
		items_.emplace(PoE::ItemId(*item.id), std::make_shared<Item>(item, location));
		if (item.socketedItems) {
			ImportItems(*item.socketedItems, ItemLocation(location, item));
		};
	};
}

void ItemsManagerWorker::EmitItemsUpdate() {

	// It's possible that we receive character vs stash tabs out of order, or users
	// move items around in a tab and we get them in a different order. For
	// consistency we want to present the tab data in a deterministic way to the rest
	// of the application.  Especially so we don't try to update shop when nothing actually
	// changed.  So sort items_ here before emitting and then generate
	// item list as strings.

	// Create a sorted vector of items.
	Items items;
	items.reserve(items_.size());
	for (auto& it : items_) { items.push_back(it.second); };
	std::sort(items.begin(), items.end(), ItemsComparator);

	// Sort the locations vector.
	std::sort(locations_.begin(), locations_.end());

	// Notify anyone subscribed to the ItemsRefreshed signal.
	emit ItemsRefreshed(items, locations_, true);
}

void ItemsManagerWorker::AddCharacter(const PoE::Character& character) {
	const PoE::CharacterName& character_name = PoE::CharacterName(character.name);
	characters_[character_name] = character;
	PoE::Character& saved_character = characters_[character_name];
	locations_.emplace_back(character);
	ItemLocation& location = locations_.back();
	for (auto& item_store : { saved_character.equipment, saved_character.inventory, saved_character.jewels }) {
		if (item_store) {
			AddItems(*item_store, location);
		};
	};
}

void ItemsManagerWorker::AddStash(const PoE::StashTab& stash) {

	// We need to be able to recursively add stashes because some stashes
	// have children, and those children might have children.
	const PoE::StashId stash_id = PoE::StashId(stash.id);
	const PoE::StashId parent_id = PoE::StashId(stash.parent.value_or(""));
	stashes_[stash_id] = stash;

	// Only create a new location for "root" stashes.
	if (!stash.parent) {
		locations_.emplace_back(stash);
	} else {
		if (parents.count(parent_id) == 0) {
			QLOG_ERROR() << "Parent stash tab not found";
			return;
		};
	};

	// If this tab has a parent, use the parent's location, otherwise
	// use the location that was just added.
	ItemLocation* location = (stash.parent) ? parents[parent_id] : &locations_.back();
	if (location == nullptr) {
		QLOG_ERROR() << "parent is missing.";
		return;
	};

	PoE::StashTab& saved_stash = stashes_[stash_id];

	// Add items from this stash.
	if (saved_stash.items) {
		AddItems(*saved_stash.items, *location);
	};

	// Add children as well.
	if (saved_stash.children) {
		parents[stash_id] = location;
		for (auto& child : *saved_stash.children) {
			AddStash(child);
		};
	};
};

void ItemsManagerWorker::AddItems(const std::vector<PoE::Item>& items, const ItemLocation& location) {
	ItemLocation base_location = location;
	for (auto& api_item : items) {
		auto acquisition_item = std::make_shared<Item>(api_item, location);
		if (api_item.socketedItems) {
			AddItems(*api_item.socketedItems, base_location);
		};
	};
}

void ItemsManagerWorker::Update(TabSelection::Type type, const std::vector<ItemLocation>& locations) {

	if (updating_) {
		QLOG_WARN() << "ItemsManagerWorker::Update called while updating";
		update_requested = true;
		selection_type = type;
		selected_locations = locations;
		return;
	};

	selection_type = type;
	selected_stashes.clear();
	selected_characters.clear();

	switch (selection_type) {
	case TabSelection::All:
	case TabSelection::Checked:
		if (!locations.empty()) {
			QLOG_ERROR() << "Inconsistent items update request";
		};
		break;
	case TabSelection::Selected:
		if (locations.empty()) {
			QLOG_ERROR() << "Nothing selected for update.";
		};
		for (auto& location : locations) {
			switch (location.type()) {
			case ItemLocationType::STASH:
				selected_stashes.insert(PoE::StashId(location.id()));
				break;
			case ItemLocationType::CHARACTER:
				selected_characters.insert(PoE::CharacterName(location.name()));
				break;
			};
		};
		break;
	default:
		QLOG_ERROR() << "Invalid update tab selection type:" << selection_type;
		break;
	};

	stashes_received = false;
	characters_received = false;

	PoE::ListCharacters(this,
		[=](const auto& characters) {
			OnCharacterListReceived(characters);
		});

	PoE::ListStashes(this,
		[=](const auto& stashes) {
			OnStashListReceived(stashes);
		},
		PoE::LeagueName(app_.league()));
}

void ItemsManagerWorker::OnCharacterListReceived(const std::vector<PoE::Character>& characters)
{
	QLOG_DEBUG() << "Character list received with" << characters.size() << "characters (all leagues).";

	if (!queued_characters.empty()) {
		QLOG_ERROR() << "Stash request queue has" << queued_characters.size() << "items that are being discarded";
		queued_characters.clear();
	};

	switch (selection_type) {
	case TabSelection::All:
		for (auto& character : characters) {
			if (!character.league) {
				QLOG_WARN() << "Skipping character without a league:" << std::string(character.name);
			} else if (PoE::LeagueName(*character.league) == app_.league()) {
				queued_characters.push_back(PoE::CharacterName(character.name));
			};
		};
		break;
	case TabSelection::Checked:
		for (auto& it : characters_) {
			ItemLocation tab(it.second);
			if ((tab.IsValid()) && (app_.buyout_manager().GetRefreshChecked(tab) == true)) {
				queued_characters.push_back(it.first);
			};
		};
		break;
	case TabSelection::Selected:
		std::set<PoE::CharacterName> current_characters;
		for (auto& it : characters_) {
			current_characters.insert(it.first);
		};
		for (auto& character : characters) {
			bool is_selected = selected_characters.count(character.name) > 0;
			bool is_new = stashes_.count(character.name) == 0;
			if (is_selected || is_new) {
				queued_stashes.push_back(character.name);
			};
		};
		break;
	};
	characters_received = true;
	StartRequests();
}

void ItemsManagerWorker::OnStashListReceived(const std::vector<PoE::StashTab>& stashes)
{
	QLOG_TRACE() << "Stash list received with" << stashes.size() << "stash tabs.";

	if (!queued_stashes.empty()) {
		QLOG_ERROR() << "Stash request queue has" << queued_stashes.size() << "items that are being discarded";
		queued_stashes.clear();
	};

	switch (selection_type) {
	case TabSelection::All:
		for (auto& stash : stashes) {
			queued_stashes.push_back(stash.id);
		};
		break;
	case TabSelection::Checked:
		for (auto& it : stashes_) {
			ItemLocation tab(it.second);
			if ((tab.IsValid()) && (app_.buyout_manager().GetRefreshChecked(tab) == true)) {
				queued_stashes.push_back(it.first);
			};
		};
		break;
	case TabSelection::Selected:
		std::set<PoE::StashId> current_stashes;
		for (auto& it : stashes_) {
			current_stashes.insert(it.first);
		};
		for (auto& stash : stashes) {
			bool is_selected = selected_stashes.count(stash.id) > 0;
			bool is_new = stashes_.count(stash.id) == 0;
			if (is_selected || is_new) {
				queued_stashes.push_back(stash.id);
			};
		};
		break;
	};
	stashes_received = true;
	StartRequests();
}

void ItemsManagerWorker::StartRequests() {

	if (!stashes_received) return;
	if (!characters_received) return;

	QLOG_DEBUG() << "Fetching items from"
		<< queued_stashes.size() << "stashes and"
		<< queued_characters.size() << "characters.";

	requests_needed_ = queued_stashes.size() + queued_characters.size();
	requests_completed_ = 0;

	if (requests_needed_ == 0) {
		QLOG_ERROR() << "Nothing to fetch.";
		updating_ = false;
		return;
	};

	items_.clear();
	locations_.clear();
	parents.clear();

	RequestNextCharacter();
	RequestNextStash();
}

void ItemsManagerWorker::RequestNextCharacter() {

	if (queued_characters.empty()) {
		FinishUpdate();
	} else {
		const PoE::CharacterName character_name = queued_characters.front();
		queued_characters.pop_front();
		PoE::GetCharacter(this,
			[=](const auto& character) { OnCharacterReceived(character);  },
			character_name);
	};
}

void ItemsManagerWorker::RequestNextStash() {

	if (queued_stashes.empty()) {
		FinishUpdate();
	} else {
		const PoE::LeagueName league = PoE::LeagueName(app_.league());
		const PoE::StashId stash_id = queued_stashes.front();
		queued_stashes.pop_front();
		PoE::GetStash(this,
			[=](const auto& stash) { OnStashReceived(stash); },
			league, stash_id);
	};
}

void ItemsManagerWorker::OnCharacterReceived(const PoE::Character& character) {
	ItemLocation location(character);
	locations_.push_back(location);
	for (auto& items : { character.equipment, character.inventory, character.jewels }) {
		if (items) {
			AddItems(*items, location);
		};
	};
	FinishRequest();
	app_.data().SetCharacter(character);
	RequestNextCharacter();
}

void ItemsManagerWorker::OnStashReceived(const PoE::StashTab& stash) {

	// Only create new locations for "root" stash tabs.
	if (!stash.parent) {
		locations_.emplace_back(stash);
	};

	const PoE::StashId stash_id(stash.id);
	const PoE::StashId parent_id(stash.parent.value_or(""));

	// The location will either be the one we just added, or the location associated
	// with this stash tab's parent.
	ItemLocation* location = (stash.parent) ? parents[parent_id] : &locations_.back();

	if (location == nullptr) {
		QLOG_ERROR() << "Item location is null.";
		return;
	}

	// Add the items contained directly within this stash tab.
	if (stash.items) {
		AddItems(*stash.items, *location);
	};

	// Create separate requests for and child stash tabs.
	if (stash.children) {
		QLOG_WARN() << "Requests" << stash.children->size() << "children of stash" << stash.id << "(" << stash.type << "/" << stash.name << ")";
		requests_needed_ += stash.children->size();
		for (auto& child : *stash.children) {
			queued_stashes.push_front(PoE::StashId(stash.id + "/" + child.id));
		};
		// Save the parent tab's location for the children to use.
		parents[stash_id] = location;
	};
	FinishRequest();
	app_.data().SetStash(stash);
	RequestNextStash();
}



void ItemsManagerWorker::FinishRequest() {

	// Mark one more request as complete.
	++requests_completed_;

	// Update the program status.
	CurrentStatusUpdate status = CurrentStatusUpdate();
	status.state = ProgramState::ItemsReceive;
	status.progress = requests_completed_;
	status.total = requests_needed_;
	if (requests_completed_ == requests_needed_) {
		status.state = ProgramState::ItemsCompleted;
	};
	if (cancel_update_) {
		status.state = ProgramState::UpdateCancelled;
	};
	emit StatusUpdate(status);
}

void ItemsManagerWorker::FinishUpdate() {

	if (!queued_stashes.empty()) return;
	if (!queued_characters.empty()) return;

	EmitItemsUpdate();

	updating_ = false;
	QLOG_DEBUG() << "Update finished.";
}


void ItemsManagerWorker::OnRateLimitStatusUpdate(const QString& status) {
	emit RateLimitStatusUpdate(status);
}
