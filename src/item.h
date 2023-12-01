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

#include <memory>
#include <map>
#include <string>
#include <unordered_map>
#include <array>
#include <vector>

#include "poe_api/poe_item.h"

#include "itemconstants.h"
#include "itemlocation.h"

extern const std::vector<std::string> ITEM_MOD_TYPES;

struct ItemSocketGroup {
	int r, g, b, w;
};

struct ItemPropertyValue {
	std::string str;
	int type;
};

struct ItemProperty {
	std::string name;
	std::vector<ItemPropertyValue> values;
	int display_mode;
};

struct ItemRequirement {
	std::string name;
	ItemPropertyValue value;
};

struct ItemSocket {
	unsigned char group;
	char attr;
};

typedef std::vector<std::string> ItemMods;
typedef std::unordered_map<std::string, double> ModTable;

class Item {
public:
	friend class ItemLocation;
	typedef const std::unordered_map<std::string, std::string> CategoryReplaceMap;

	enum INFLUENCE_TYPES {
		NONE,
		SHAPER,
		ELDER,
		CRUSADER,
		REDEEMER,
		HUNTER,
		WARLORD,
		SYNTHESISED,
		FRACTURED,
		SEARING_EXARCH,
		EATER_OF_WORLDS
	};

	explicit Item(const PoE::Item& item, const ItemLocation& location);
	explicit Item(const std::string& name, const ItemLocation& location);
	
	const PoE::Item& item() const { return item_; };
	const std::string id() const { return id_; };
	const std::string name() const { return item_.name; };
	const std::string typeLine() const { return typeLine_; };
	int x() const { return item_.x.value_or(0); };
	int y() const { return item_.y.value_or(0); };
	int w() const { return item_.w; };
	int h() const { return item_.h; };
	int frameType() const { return item_.frameType ? static_cast<int>(*item_.frameType) : 0; };
	const std::string icon() const { return item_.icon; };
	
	bool identified() const { return item_.identified; };
	bool corrupted() const { return item_.corrupted.value_or(false); };
	bool crafted() const { return item_.craftedMods.has_value(); };
	bool enchanted() const { return item_.enchantMods.has_value(); };

	bool hasInfluence() const { return (influence_left_ != INFLUENCE_TYPES::NONE) && (influence_right_ != INFLUENCE_TYPES::NONE); };
	bool hasInfluence(INFLUENCE_TYPES influence) const { return (influence_left_ == influence) || (influence_right_ == influence); };
	INFLUENCE_TYPES influenceLeft() const { return influence_left_; }
	INFLUENCE_TYPES influenceRight() const { return influence_right_; }

	int itemLevel() const { return item_.itemLevel.value_or(item_.ilvl); };

	const std::vector<std::optional<std::vector<std::string>>> mods() const {
		return { item_.implicitMods, item_.enchantMods, item_.explicitMods, item_.craftedMods, item_.fracturedMods };
	};

	//std::string name() const { return name_; }
	//std::string id() const { return uid_; }
	//std::string typeLine() const { return typeLine_; }
	std::string PrettyName() const { return prettyName_; };
	//bool identified() const { return identified_; }
	//bool corrupted() const { return corrupted_; }
	//bool crafted() const { return crafted_; }
	//bool enchanted() const { return enchanted_; }
	//bool hasInfluence(INFLUENCE_TYPES type) const { return std::find(influenceList_.begin(), influenceList_.end(), type) != influenceList_.end(); }
	//bool hasInfluence() const { return !influenceList_.empty(); }
	//int w() const { return w_; }
	//int h() const { return h_; }
	//int frameType() const { return frameType_; }
	//const std::string& icon() const { return icon_; }
	const std::map<std::string, std::string>& properties() const { return properties_; }
	const std::vector<ItemProperty>& text_properties() const { return text_properties_; }
	const std::vector<ItemRequirement>& text_requirements() const { return text_requirements_; }
	//const std::map<std::string, ItemMods>& text_mods() const { return text_mods_; }
	const std::vector<ItemSocket>& text_sockets() const { return text_sockets_; }
	const std::vector<std::pair<std::string, int>>& elemental_damage() const { return elemental_damage_; }
	const std::map<std::string, int>& requirements() const { return requirements_; }
	double DPS() const { return total_dps_; };
	double pDPS() const { return physical_dps_; };
	double eDPS() const { return elemental_dps_; };
	double cDPS() const { return chaos_dps_; };
	int sockets_cnt() const { return sockets_cnt_; }
	int links_cnt() const { return links_cnt_; }
	const ItemSocketGroup& sockets() const { return sockets_; }
	const std::vector<ItemSocketGroup>& socket_groups() const { return socket_groups_; }
	const ItemLocation& location() const { return location_; }
	//const std::string& json() const { return json_; }
	const std::string note() const { return (item_.note.has_value()) ? *item_.note : ""; }
	const std::string& category() const { return category_; }
	const std::vector<std::string>& category_vector() const { return category_vector_; }
	uint talisman_tier() const { return item_.talismanTier.value_or(0); }
	int count() const { return count_; }
	const ModTable& mod_table() const { return mod_table_; }
	//int itemLevel() const { return ilvl_; }
	bool operator<(const Item& other) const { return id_ < other.id_; };
	bool Wearable() const { return wearable_; };
	std::string POBformat() const;
	static const size_t k_CategoryLevels = 3;
	static const std::array<CategoryReplaceMap, k_CategoryLevels> replace_map_;

private:
	Item();
	Item(const PoE::Item& item);

	void InitInfluences();
	void CalculateCategories();
	void InitProperties();
	void InitRequirements();
	void InitSockets();

	// The point of GenerateMods is to create combined (e.g. implicit+explicit) poe.trade-like mod map to be searched by mod filter.
	// For now it only does that for a small chosen subset of mods (think "popular" + "pseudo" sections at poe.trade)
	void GenerateMods();
	void CalculateDPS();

	const PoE::Item item_;

	std::string id_;
	std::string name_;
	std::string icon_;
	std::string typeLine_;
	std::string baseType_;
	std::string prettyName_;

	INFLUENCE_TYPES influence_left_{ INFLUENCE_TYPES::NONE };
	INFLUENCE_TYPES influence_right_{ INFLUENCE_TYPES::NONE };

	double elemental_dps_{ 0.0 };
	double physical_dps_{ 0.0 };
	double chaos_dps_{ 0.0 };
	double total_dps_{ 0.0 };

	bool wearable_{ false };

	//std::string name_;
	ItemLocation location_;
	//std::string typeLine_;
	//std::string baseType_;
	std::string category_;
	std::vector<std::string> category_vector_;
	//bool identified_;
	//bool corrupted_;
	//bool crafted_;
	//bool enchanted_;
	//std::vector<INFLUENCE_TYPES> influenceList_;
	//int w_, h_;
	//int frameType_{ 0 };
	//std::string icon_;
	std::map<std::string, std::string> properties_;
	// vector of pairs [damage, type]
	std::vector<std::pair<std::string, int>> elemental_damage_;
	int sockets_cnt_;
	int links_cnt_;
	ItemSocketGroup sockets_;
	std::vector<ItemSocketGroup> socket_groups_;
	std::map<std::string, int> requirements_;
	//std::string json_;
	int count_;
	//int ilvl_;
	std::vector<ItemProperty> text_properties_;
	std::vector<ItemRequirement> text_requirements_;
	//std::map<std::string, ItemMods> text_mods_;
	std::vector<ItemSocket> text_sockets_;
	//std::string note_;
	ModTable mod_table_;
	//std::string uid_;
	//uint talisman_tier_{ 0 };

	//std::vector<std::string> implicit_mods_, enchant_mods_, explicit_mods_, crafted_mods_, fractured_mods_;

	// Location-related fields (used to be hacked into the json)
	ItemLocationType _type;
	int _tab;
	std::string _tab_label;
	std::string _character;
	int _x;
	int _y;
	bool _socketed;
	bool _removeonly;
	std::string _inventory_id;
};

typedef std::vector<std::shared_ptr<Item>> Items;

inline bool ItemsComparator(const std::shared_ptr<Item>& a, const std::shared_ptr<Item>& b) {
	return *a < *b;
};
