#include "itemlocation.h"

#include <QString>

#include <boost/algorithm/string/predicate.hpp>

#include "rapidjson_util.h"
#include "util.h"

ItemLocation::ItemLocation()
{}

ItemLocation::ItemLocation(const rapidjson::Value &root):
	ItemLocation()
{
	FromItemJson(root);
}

ItemLocation::ItemLocation(int tab_id, std::string tab_unique_id, std::string name, ItemLocationType type, int r, int g, int b) :
	x_(0), y_(0), w_(0), h_(0), red_(r), green_(g), blue_(b),
	socketed_(false)
{
	type_ = type;
	tab_id_ = tab_id;
    if (type_ == ItemLocationType::STASH) {
        tab_label_ = name;
        remove_only_ = boost::algorithm::ends_with(tab_label_, "(Remove-only)");
    }
    else {
        character_ = name;
    };
	//r_ = r;
	//g_ = g;
	//b_ = b;

	tab_unique_id_ = tab_unique_id;
}

void ItemLocation::SetBackgroundColor(int r, int g, int b) {
	red_ = r;
	green_ = g;
	blue_ = b;
}

void ItemLocation::FromItemJson(const rapidjson::Value &root) {
	if (root.HasMember("_type")) {
		type_ = static_cast<ItemLocationType>(root["_type"].GetInt());
		if (type_ == ItemLocationType::STASH) {
			tab_label_ = root["_tab_label"].GetString();
			tab_id_ = root["_tab"].GetInt();
		} else {
			character_ = root["_character"].GetString();
		}
		socketed_ = false;
		if (root.HasMember("_socketed"))
			socketed_ = root["_socketed"].GetBool();
		// socketed items have x/y pointing to parent
		if (socketed_) {
			x_ = root["_x"].GetInt();
			y_ = root["_y"].GetInt();
		}
	}
	if (root.HasMember("x") && root.HasMember("y") && root["x"].IsInt() && root["y"].IsInt()) {
		x_ = root["x"].GetInt();
		y_ = root["y"].GetInt();
	}
	if (root.HasMember("w") && root.HasMember("h") && root["w"].IsInt() && root["h"].IsInt()) {
		w_ = root["w"].GetInt();
		h_ = root["h"].GetInt();
	}
	if (root.HasMember("inventoryId") && root["inventoryId"].IsString())
		inventory_id_ = root["inventoryId"].GetString();
}

void ItemLocation::ToItemJson(rapidjson::Value *root_ptr, rapidjson_allocator &alloc) {
	auto &root = *root_ptr;
	rapidjson::Value string_val(rapidjson::kStringType);
	root.AddMember("_type", static_cast<int>(type_), alloc);
	if (type_ == ItemLocationType::STASH) {
		root.AddMember("_tab", tab_id_, alloc);
		string_val.SetString(tab_label_.c_str(), alloc);
		root.AddMember("_tab_label", string_val, alloc);
	} else {
		string_val.SetString(character_.c_str(), alloc);
		root.AddMember("_character", string_val, alloc);
	}
	if (socketed_) {
		root.AddMember("_x", x_, alloc);
		root.AddMember("_y", y_, alloc);
	}
	root.AddMember("_socketed", socketed_, alloc);
}

std::string ItemLocation::GetHeader() const {
	if (type_ == ItemLocationType::STASH) {
		QString format("#%1, \"%2\"");
		return format.arg(tab_id_ + 1).arg(tab_label_.c_str()).toStdString();
	} else {
		return character_;
	}
}

QRectF ItemLocation::GetRect() const {
	QRectF result;
	position itemPos;
	itemPos.x = x_;
	itemPos.y = y_;

	if ((!inventory_id_.empty()) && (type_ == ItemLocationType::CHARACTER)) {
		if (inventory_id_ == "MainInventory") {
			itemPos.y += POS_MAP.at(inventory_id_).y;
		} else if (inventory_id_ == "Flask") {
			itemPos.x += POS_MAP.at(inventory_id_).x;
			itemPos.y = POS_MAP.at(inventory_id_).y;
		} else if (POS_MAP.count(inventory_id_)) {
			itemPos = POS_MAP.at(inventory_id_);
		}
	}

	result.setX(MINIMAP_SIZE * itemPos.x / INVENTORY_SLOTS);
	result.setY(MINIMAP_SIZE * itemPos.y / INVENTORY_SLOTS);
	result.setWidth(MINIMAP_SIZE * w_ / INVENTORY_SLOTS);
	result.setHeight(MINIMAP_SIZE * h_ / INVENTORY_SLOTS);
	return result;
}

std::string ItemLocation::GetForumCode(const std::string &league) const {
	if (type_ == ItemLocationType::STASH) {
		QString format("[linkItem location=\"Stash%1\" league=\"%2\" x=\"%3\" y=\"%4\"]");
		return format.arg(QString::number(tab_id_ + 1), league.c_str(), QString::number(x_), QString::number(y_)).toStdString();
	} else {
		QString format("[linkItem location=\"%1\" character=\"%2\" x=\"%3\" y=\"%4\"]");
		return format.arg(inventory_id_.c_str(), character_.c_str(), QString::number(x_), QString::number(y_)).toStdString();
	}
}

bool ItemLocation::IsValid() const {
    // Stash tabs can have empty labels, but character tabs must have a name.
	return (type_ == ItemLocationType::STASH) || ((type_ == ItemLocationType::CHARACTER) && !character_.empty());
}

std::string ItemLocation::GetUniqueHash() const {
    // Use the unique tab id because it never be empty.
    return IsValid() ? tab_unique_id_ : "";
    //if (!IsValid())
    //	return "";
    //if (type_ == ItemLocationType::STASH)
	//	return "stash:" + tab_label_;
	//else
	//	return "character:" + character_;
}

void ItemLocation::set_json(rapidjson::Value &value, rapidjson_allocator &alloc) {
	if(type_ == ItemLocationType::CHARACTER){
		value.AddMember("i", tab_id_, alloc);
	}

	json_ = Util::RapidjsonSerialize(value);
}

bool ItemLocation::operator<(const ItemLocation &rhs) const {
	if (type_ == ItemLocationType::STASH)
		return std::tie(type_,tab_id_) < std::tie(rhs.type_,rhs.tab_id_);
	else
		return std::tie(type_,character_) < std::tie(rhs.type_,rhs.character_);
}

bool ItemLocation::operator==(const ItemLocation &other) const {
	return get_tab_uniq_id() == other.get_tab_uniq_id();
}
