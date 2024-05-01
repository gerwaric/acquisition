/*
	Copyright 2015 Ilya Zhuravlev

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

#include "memorydatastore.h"

#include "currencymanager.h"

std::string MemoryDataStore::Get(const std::string& key, const std::string& default_value) {
	auto i = data_.find(key);
	if (i == data_.end())
		return default_value;
	return i->second;
}

Locations MemoryDataStore::GetTabs(const ItemLocationType& type) {
	auto i = tabs_.find(type);
	if (i == tabs_.end())
		return {};
	return i->second;
}

Items MemoryDataStore::GetItems(const ItemLocation& loc) {
	auto i = items_.find(loc.get_tab_uniq_id());
	if (i == items_.end())
		return {};
	return i->second;
}

void MemoryDataStore::Set(const std::string& key, const std::string& value) {
	data_[key] = value;
}

void MemoryDataStore::SetTabs(const ItemLocationType& type, const Locations& tabs) {
	tabs_[type] = tabs;
}

void MemoryDataStore::SetItems(const ItemLocation& loc, const Items& items) {
	items_[loc.get_tab_uniq_id()] = items;
}

void MemoryDataStore::InsertCurrencyUpdate(const CurrencyUpdate& update) {
	currency_updates_.push_back(update);
}

std::vector<CurrencyUpdate> MemoryDataStore::GetAllCurrency() {
	return currency_updates_;
}
