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

#include "item.h"

#include <utility>
#include <QString>
#include <sstream>
#include <boost/algorithm/string.hpp>
#include <regex>

#include "QsLog.h"

#include "poe_api/poe_item.h"

#include "modlist.h"
#include "util.h"
#include "porting.h"
#include "itemlocation.h"
#include "itemcategories.h"

static const Item::CategoryReplaceMap replace_map_0_({
	{"Divination", "Divination Cards"},
	{"QuestItems", "Quest Items"} });

static const Item::CategoryReplaceMap replace_map_1_({
	{"BodyArmours", "Body"},
	{"VaalGems", "Vaal"},
	{"AtlasMaps", "2.4"},
	{"act4maps", "2.0"},
	{"OneHandWeapons", "1Hand"},
	{"TwoHandWeapons", "2Hand"} });

static const Item::CategoryReplaceMap replace_map_2_({
	{"OneHandAxes", "Axes"},
	{"OneHandMaces", "Maces"},
	{"OneHandSwords", "Swords"},
	{"TwoHandAxes", "Axes"},
	{"TwoHandMaces", "Maces"},
	{"TwoHandSwords", "Swords"} });

const std::array<Item::CategoryReplaceMap, Item::k_CategoryLevels> Item::replace_map_ = {

	// Category hierarchy 0 replacement map
	replace_map_0_,

	// Category hierarchy 1 replacement map
	replace_map_1_,

	// Category hierarchy 2 replacement map
	replace_map_2_

};

const std::vector<std::string> ITEM_MOD_TYPES = {
	"implicitMods", "enchantMods", "explicitMods", "craftedMods", "fracturedMods"
};

/*
static std::string item_unique_properties(const rapidjson::Value& json, const std::string& name) {
	const char* name_p = name.c_str();
	if (!json.HasMember(name_p))
		return "";
	std::string result;
	for (auto& prop : json[name_p]) {
		result += std::string(prop["name"].GetString()) + "~";
		for (auto& value : prop["values"])
			result += std::string(value[0].GetString()) + "~";
	}
	return result;
}
*/

// Fix up names, remove all <<set:X>> modifiers
static std::string fixup_name(const std::string& name) {
	std::string::size_type right_shift = name.rfind(">>");
	if (right_shift != std::string::npos) {
		return name.substr(right_shift + 2);
	}
	return name;
}

// Construct directly from a new API object.
Item::Item(const PoE::Item& item) :
	item_(item),
	id_(item.id.value_or("")),
	name_(fixup_name(item.name)),
	typeLine_(fixup_name(item.typeLine)),
	baseType_(fixup_name(item.baseType)),
	// set in constructor body: category_(),          
	// set in constructor body: category_vector_(),
	//identified_(item.identified),
	//corrupted_(item.corrupted.value_or(false)),
	//crafted_(item.craftedMods),
	//enchanted_(item.enchantMods),
	// set in constructor body: influenceList_(),
	//w_(item.w),
	//h_(item.h),
	//frameType_(item.frameType ? static_cast<int>(item.frameType.value()) : -1),
	icon_(item.icon),
	// set in constructor body: properties_(),
	// set in constructor body: elemental_damage_(),
	sockets_cnt_(0),
	links_cnt_(0),
	// set in constructor body: sockets_(),
	// set in constructor body: socket_groups_(),
	// set in constructor body: requirements_(),
	//json_(JS::serializeStruct(item)), // WARNING -- not the same as legacy item objects.
	// set in constructor body: count_(),
	//ilvl_(item.ilvl),
	// set in constructor body: text_properties_(),
	// set in constructor body: text_requirements_(),
	// set in constructor body: text_mods_(),
	// set in constructor body: text_sockets_(),
	// note_(item.note.value_or("")),
	// set in constructor body: mod_table_(),
	//uid_(item.id.value_or("NONE")),
	//talisman_tier_(item.talismanTier.value_or(0)),
	// These next fields relate to the item location and used to be hacked in via json.
	//implicit_mods_(item.implicitMods.value_or(std::vector<std::string>())),
	//explicit_mods_(item.explicitMods.value_or(std::vector<std::string>())),
	//crafted_mods_(item.craftedMods.value_or(std::vector<std::string>())),
	//fractured_mods_(item.fracturedMods.value_or(std::vector<std::string>())),
	_x(item.x.value_or(0)),
	_y(item.y.value_or(0)),
	_inventory_id(item.inventoryId.value_or(""))
{
	// Use base type for all hybrid items except vaal gems.
	if (item.hybrid && (!item.hybrid.value().isVaalGem.value_or(false))) {
		typeLine_ = fixup_name(item.hybrid->baseTypeName);
	};

	if (name_.empty()) {
		prettyName_ = typeLine_;
	} else {
		prettyName_ = name_ + " " + typeLine_;
	};

	if (!item.id) {
		QLOG_ERROR() << prettyName_ << "does not have a unique id";
	};
	if (item.itemLevel) {
		if (item.ilvl != *item.itemLevel) {
			QLOG_WARN() << prettyName_ << ": ilvl (" << item.ilvl << ") does not match itemLevel (" << *item_.itemLevel << ")";
		};
	};

	// Other code assumes icon is proper size so force quad=1 to quad=0 here as it's clunky
	// to handle elsewhere
	boost::replace_last(icon_, "quad=1", "quad=0");

	// quad stashes, currency stashes, etc
	boost::replace_last(icon_, "scaleIndex=", "scaleIndex=0&");

	InitInfluences();
	CalculateCategories();
	wearable_ = category_ == "flasks"
		|| category_ == "amulet"
		|| category_ == "ring"
		|| category_ == "belt"
		|| category_.find("armour") != std::string::npos
		|| category_.find("weapons") != std::string::npos
		|| category_.find("jewels") != std::string::npos;

	InitProperties();
	InitRequirements();
	InitSockets();

	count_ = 1;
	if (properties_.find("Stack Size") != properties_.end()) {
		std::string size = properties_["Stack Size"];
		if (size.find("/") != std::string::npos) {
			size = size.substr(0, size.find("/"));
			count_ = std::stoi(size);
		};
	};

	GenerateMods();
	CalculateDPS();
}

// Construct directly from a new API object.
Item::Item(const PoE::Item& item, const ItemLocation& location) :
	Item(item)
{
	location_ = location;
	_type = location.type();
	switch (_type) {
	case ItemLocationType::STASH:
		_tab = location.stash_index();
		_tab_label = location.name();
		_character = "";
		break;
	case ItemLocationType::CHARACTER:
		_tab = 0;
		_tab_label = "";
		_character = location.name();
		break;
	default:
		QLOG_ERROR() << "Item(): location has invalid type:" << _type;
		break;
	};
	_socketed = location.socketed();
	_removeonly = location.removeonly();
	_inventory_id = location.inventory_id();
}

Item::Item() {};

// Create a fake item for test mode.
Item::Item(const std::string& name, const ItemLocation& location) : Item()
{
	id_ = Util::Md5(name + "|" + location.id()); //hash_(Util::Md5(name)) // Unique enough for tests
	name_ = name;
	location_ = location;
}

void Item::InitInfluences() {
	std::vector<INFLUENCE_TYPES> influences;
	influences.reserve(2);
	if (item_.influences) {
		if (item_.influences->shaper.value_or(false))   influences.push_back(SHAPER);
		if (item_.influences->elder.value_or(false))    influences.push_back(ELDER);
		if (item_.influences->crusader.value_or(false)) influences.push_back(CRUSADER);
		if (item_.influences->redeemer.value_or(false)) influences.push_back(REDEEMER);
		if (item_.influences->hunter.value_or(false))   influences.push_back(HUNTER);
		if (item_.influences->warlord.value_or(false))  influences.push_back(WARLORD);
	};
	if (item_.synthesised.value_or(false)) influences.push_back(SYNTHESISED);
	if (item_.fractured.value_or(false))   influences.push_back(FRACTURED);
	if (item_.searing.value_or(false))     influences.push_back(SEARING_EXARCH);
	if (item_.tangled.value_or(false))     influences.push_back(EATER_OF_WORLDS);

	if (influences.size() > 2) { QLOG_ERROR() << "Item has more than two influences." << PrettyName(); } else if (influences.size() > 1) { influence_right_ = influences[1]; } else if (influences.size() > 0) { influence_left_ = influences[0]; };
	influences.clear();
}

void Item::InitProperties() {
	// Import item properties.
	if (!item_.properties) {
		return;
	};
	for (auto& prop : item_.properties.value()) {
		if (prop.name == "Elemental Damage") {
			// Import elemental damage mods
			elemental_damage_.reserve(elemental_damage_.size() + prop.values.size());
			for (auto& value : prop.values) {
				const std::string prop_name = std::get<0>(value);
				const unsigned int prop_value = std::get<1>(value);
				elemental_damage_.push_back({ prop_name, prop_value });
			};
		} else {
			// Look for other properties
			if (prop.values.size() > 0) {
				properties_[prop.name] = std::get<0>(prop.values[0]);
			};
		};
		ItemProperty property;
		property.name = prop.name;
		property.display_mode = prop.displayMode;
		for (auto& value : prop.values) {
			ItemPropertyValue v;
			v.str = std::get<0>(value);
			v.type = std::get<1>(value);
			property.values.push_back(v);
		};
		text_properties_.push_back(property);
	};
}

void Item::InitRequirements() {
	// Import item requirements.
	if (!item_.requirements) {
		return;
	};
	for (auto& req : item_.requirements.value()) {
		if (req.values.size() != 1) {
			QLOG_ERROR() << "Requirement item property values had" << req.values.size() << "elements:" << item_.name;
			continue;
		};
		const auto& value = req.values[0];
		const std::string value_str = std::get<0>(value);
		const unsigned int value_type = std::get<1>(value);
		requirements_[req.name] = std::atoi(value_str.c_str());
		ItemPropertyValue v;
		v.str = value_str;
		v.type = value_type;
		text_requirements_.push_back({ req.name, v });
	};
}

void Item::InitSockets() {
	// Import item sockets.
	if (!item_.sockets) {
		return;
	};
	ItemSocketGroup current_group = { 0, 0, 0, 0 };
	sockets_cnt_ = static_cast<int>(item_.sockets->size());
	int counter = 0;
	int prev_group = -1;
	for (auto& socket : *item_.sockets) {

		// TBD I don't understand what this means, but Acquisition did not check for both
		// attr and sColour.
		// 
		//if (socket.attr && socket.sColour) {
		//	QLOG_WARN() << "Item socket has both attr and sColour:" << item.name;
		//}

		char attr = '\0';
		if (socket.attr) {
			attr = socket.attr.value()[0];
		} else if (socket.sColour) {
			attr = socket.sColour.value()[0];
		};

		if (!attr) {
			continue;
		};

		ItemSocket current_socket = { static_cast<unsigned char>(socket.group), attr };
		text_sockets_.push_back(current_socket);
		if (prev_group != current_socket.group) {
			counter = 0;
			socket_groups_.push_back(current_group);
			current_group = { 0, 0, 0, 0 };
		};
		prev_group = current_socket.group;
		++counter;
		links_cnt_ = std::max(links_cnt_, counter);
		switch (current_socket.attr) {
		case 'S':
			sockets_.r++;
			current_group.r++;
			break;
		case 'D':
			sockets_.g++;
			current_group.g++;
			break;
		case 'I':
			sockets_.b++;
			current_group.b++;
			break;
		case 'G':
			sockets_.w++;
			current_group.w++;
			break;
		};
	};
	socket_groups_.push_back(current_group);
}

void Item::CalculateCategories() {
	auto rslt = itemBaseType_NameToClass.find(baseType_);
	if (rslt != itemBaseType_NameToClass.end()) {
		std::string step1 = rslt->second;
		rslt = itemClassKeyToValue.find(step1);
		if (rslt != itemClassKeyToValue.end()) {
			category_ = rslt->second;
			boost::to_lower(category_);
		};
	};
}

void Item::CalculateDPS() {

	if (!properties_.count("Attacks per Second")) {
		return;
	};

	const double attacks_per_second = std::stod(properties_.at("Attacks per Second"));

	// Physical DPS
	if (properties_.count("Physical Damage")) {
		const std::string& physical_damage = properties_.at("Physical Damage");
		physical_dps_ = attacks_per_second * Util::AverageDamage(physical_damage);
	};

	// Elemental DPS
	if (!elemental_damage_.empty()) {
		double damage = 0.0;
		for (auto& item : elemental_damage_) {
			damage += Util::AverageDamage(item.first);
		};
		elemental_dps_ = attacks_per_second * damage;
	}

	// Chaos DPS
	if (properties_.count("Chaos Damage")) {
		const std::string& chaos_damage = properties_.at("Chaos Damage");
		chaos_dps_ = attacks_per_second * Util::AverageDamage(chaos_damage);
	};

	// Total DPS
	total_dps_ = physical_dps_ + elemental_dps_ + chaos_dps_;
};

void Item::GenerateMods() {

	const auto mod_lists = {
		// TBD: item.utilityMods,
		// TBD: item.enchantMods,
		// TBD: item.scourgeMods,
		item_.implicitMods,
		item_.explicitMods,
		item_.craftedMods,
		item_.fracturedMods
		// TBD: item.crucibleMods,
		// TBD: item.cosmeticMods,
		// TBD: item.veiledMods
	};
	// NOTE: logbookMods and ultimatumMods are excluded because they
	// are different object types from the rest of the mod lists. If 
	// they need to be added, they will need to be added separately.

	for (auto& mod_list : mod_lists) {
		if (mod_list) {
			for (const auto& mod : mod_list.value()) {
				std::string mod_s = mod;
				std::regex rep("([0-9\\.]+)");
				mod_s = std::regex_replace(mod_s, rep, "#");
				auto rslt = mods_map.find(mod_s);
				if (rslt != mods_map.end()) {
					SumModGenerator& gen = *rslt->second;
					gen.Generate(mod, &mod_table_);
				};
			};
		};
	};
}

std::string Item::POBformat() const {
	QLOG_ERROR() << "TODO: reimplement POBformat";
	return "";
	/*
	std::stringstream PoBText;
	PoBText << name();
	PoBText << "\n" << typeLine();

	// Could use uid_ for "Unique ID:", if it'd help PoB avoid duplicate imports later via stash API?

	auto& sockets = text_sockets();
	if (sockets.size() > 0) {
		PoBText << "\nSockets: ";
		ItemSocket prev = { 255, '-' };
		size_t i = 0;
		for (auto& socket : sockets) {
			bool link = socket.group == prev.group;
			if (i > 0) {
				PoBText << (link ? "-" : " ");
			}
			switch (socket.attr) {
			case 'S':
				PoBText << "R";
				break;
			case 'D':
				PoBText << "G";
				break;
			case 'I':
				PoBText << "B";
				break;
			case 'G':
				PoBText << "W";
				break;
			default:
				PoBText << socket.attr;
				break;
			}
			prev = socket;
			++i;
		}
	}

	auto& mods = text_mods();

	auto implicitMods = mods.at("implicitMods");
	auto enchantMods = mods.at("enchantMods");
	PoBText << "\nImplicits: " << (implicitMods.size() + enchantMods.size());
	for (const auto& mod : enchantMods) {
		PoBText << "\n{crafted}" << mod;
	}
	for (const auto& mod : implicitMods) {
		PoBText << "\n" << mod;
	}

	auto explicitMods = mods.at("explicitMods");
	auto craftedMods = mods.at("craftedMods");
	if (!explicitMods.empty() || !craftedMods.empty()) {
		for (const auto& mod : explicitMods) {
			PoBText << "\n" << mod;
		}
		for (const auto& mod : craftedMods) {
			PoBText << "\n{crafted}" << mod;
		}
	}

	return PoBText.str();
	*/
}
