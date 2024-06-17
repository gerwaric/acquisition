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

#include "item.h"
#include <QDateTime>
#include <set>

#include "currency.h"

class ItemLocation;

enum BuyoutType {
	BUYOUT_TYPE_IGNORE,
	BUYOUT_TYPE_BUYOUT,
	BUYOUT_TYPE_FIXED,
	BUYOUT_TYPE_CURRENT_OFFER,
	BUYOUT_TYPE_NO_PRICE,
	BUYOUT_TYPE_INHERIT,
};

enum BuyoutSource {
	BUYOUT_SOURCE_NONE,
	BUYOUT_SOURCE_MANUAL,
	BUYOUT_SOURCE_GAME,
	BUYOUT_SOURCE_AUTO
};

struct Buyout {
	typedef std::map<BuyoutType, std::string> BuyoutTypeMap;
	typedef std::map<BuyoutSource, std::string> BuyoutSourceMap;

	double value;
	BuyoutType type;
	BuyoutSource source{ BUYOUT_SOURCE_MANUAL };
	Currency currency;
	QDateTime last_update;
	bool inherited = false;
	bool operator==(const Buyout& o) const;
	bool operator!=(const Buyout& o) const;
	bool IsValid() const;
	bool IsActive() const;
	bool IsInherited() const { return inherited || type == BUYOUT_TYPE_INHERIT; };
	bool IsSavable() const { return IsValid() && !(type == BUYOUT_TYPE_INHERIT); };
	bool IsPostable() const;
	bool IsPriced() const;
	bool IsGameSet() const;
	bool RequiresRefresh() const;

	static BuyoutType TagAsBuyoutType(std::string tag);
	static BuyoutType IndexAsBuyoutType(int index);
	static BuyoutSource TagAsBuyoutSource(std::string tag);

	std::string AsText() const;
	const std::string& BuyoutTypeAsTag() const;
	const std::string& BuyoutTypeAsPrefix() const;
	const std::string& BuyoutSourceAsTag() const;
	const std::string& CurrencyAsTag() const;

	Buyout() :
		value(0),
		type(BUYOUT_TYPE_INHERIT),
		currency(CURRENCY_NONE)
	{}
	Buyout(double value_, BuyoutType type_, Currency currency_, QDateTime last_update_) :
		value(value_),
		type(type_),
		currency(currency_),
		last_update(last_update_)
	{}
private:
	static const std::string buyout_type_error_;
	static const BuyoutTypeMap buyout_type_as_tag_;
	static const BuyoutTypeMap buyout_type_as_prefix_;
	static const BuyoutSourceMap buyout_source_as_tag_;
};

class DataStore;

class BuyoutManager {
public:
	explicit BuyoutManager(DataStore& data);
	void Set(const Item& item, const Buyout& buyout);
	Buyout Get(const Item& item) const;

	void SetTab(const std::string& tab, const Buyout& buyout);
	Buyout GetTab(const std::string& tab) const;
	void CompressTabBuyouts();
	void CompressItemBuyouts(const Items& items);

	void SetRefreshChecked(const ItemLocation& tab, bool value);
	bool GetRefreshChecked(const ItemLocation& tab) const;

	bool GetRefreshLocked(const ItemLocation& tab) const;
	void SetRefreshLocked(const ItemLocation& tab);
	void ClearRefreshLocks();

	void SetStashTabLocations(const std::vector<ItemLocation>& tabs);
	const std::vector<ItemLocation> GetStashTabLocations() const;
	void Clear();

	Buyout StringToBuyout(std::string);

	void Save();
	void Load();

	void MigrateItem(const Item& item);
private:
	BuyoutType StringToBuyoutType(std::string bo_str) const;

	std::string Serialize(const std::map<std::string, Buyout>& buyouts);
	void Deserialize(const std::string& data, std::map<std::string, Buyout>* buyouts);

	std::string Serialize(const std::map<std::string, bool>& obj);
	void Deserialize(const std::string& data, std::map<std::string, bool>& obj);

	DataStore& data_;
	std::map<std::string, Buyout> buyouts_;
	std::map<std::string, Buyout> tab_buyouts_;
	std::map<std::string, bool> refresh_checked_;
	std::set<std::string> refresh_locked_;
	bool save_needed_;
	std::vector<ItemLocation> tabs_;
	static const std::map<std::string, BuyoutType> string_to_buyout_type_;
};

