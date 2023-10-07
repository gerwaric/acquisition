#include "itemlocation.h"

#include <QString>

#include <boost/algorithm/string/predicate.hpp>

#include "rapidjson_util.h"
#include "util.h"

using boost::algorithm::ends_with;

QDebug& operator<<(QDebug& os, const ItemLocationType& obj) {
	switch (obj) {
	case ItemLocationType::STASH: return os << "STASH";
	case ItemLocationType::CHARACTER: return os << "CHARACTER";
	};
}

ItemLocation::ItemLocation():
	x_(0), y_(0), w_(0), h_(0), red_(0), green_(0), blue_(0),
	tab_id_(0),
	socketed_(false),
	removeonly_(false),
	type_(ItemLocationType::STASH)
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
	switch (type_) {
	case ItemLocationType::STASH:
		tab_label_ = name;
		character_ = "";
		removeonly_ = ends_with(name, "(Remove-only)");
		break;
	case ItemLocationType::CHARACTER:
		tab_label_ = "";
		character_ = name;
		removeonly_ = false;
		break;
	};
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
		switch (type_) {
		case ItemLocationType::STASH:
			tab_label_ = root["_tab_label"].GetString();
			tab_id_ = root["_tab"].GetInt();
			break;
		case ItemLocationType::CHARACTER:
			character_ = root["_character"].GetString();
			break;
		};
		socketed_ = false;
		if (root.HasMember("_socketed")) {
			socketed_ = root["_socketed"].GetBool();
		};
		if (root.HasMember("_removeonly")) {
			removeonly_ = root["_removeonly"].GetBool();
		};
		// socketed items have x/y pointing to parent
		if (socketed_) {
			x_ = root["_x"].GetInt();
			y_ = root["_y"].GetInt();
		};
	};
	if (root.HasMember("x") && root.HasMember("y") && root["x"].IsInt() && root["y"].IsInt()) {
		x_ = root["x"].GetInt();
		y_ = root["y"].GetInt();
	};
	if (root.HasMember("w") && root.HasMember("h") && root["w"].IsInt() && root["h"].IsInt()) {
		w_ = root["w"].GetInt();
		h_ = root["h"].GetInt();
	};
	if (root.HasMember("inventoryId") && root["inventoryId"].IsString())
		inventory_id_ = root["inventoryId"].GetString();
}

void ItemLocation::ToItemJson(rapidjson::Value *root_ptr, rapidjson_allocator &alloc) {
	auto &root = *root_ptr;
	rapidjson::Value string_val(rapidjson::kStringType);
	root.AddMember("_type", static_cast<int>(type_), alloc);
	switch (type_) {
	case ItemLocationType::STASH:
		root.AddMember("_tab", tab_id_, alloc);
		string_val.SetString(tab_label_.c_str(), alloc);
		root.AddMember("_tab_label", string_val, alloc);
		break;
	case ItemLocationType::CHARACTER:
		string_val.SetString(character_.c_str(), alloc);
		root.AddMember("_character", string_val, alloc);
		break;
	};
	if (socketed_) {
		root.AddMember("_x", x_, alloc);
		root.AddMember("_y", y_, alloc);
	};
	root.AddMember("_socketed", socketed_, alloc);
	root.AddMember("_removeonly", removeonly_, alloc);
}

std::string ItemLocation::GetHeader() const {
	switch (type_) {
	case ItemLocationType::STASH: return QString("#%1, \"%2\"").arg(tab_id_ + 1).arg(tab_label_.c_str()).toStdString();
	case ItemLocationType::CHARACTER: return character_;
	};
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
		};
	};

	result.setX(MINIMAP_SIZE * itemPos.x / INVENTORY_SLOTS);
	result.setY(MINIMAP_SIZE * itemPos.y / INVENTORY_SLOTS);
	result.setWidth(MINIMAP_SIZE * w_ / INVENTORY_SLOTS);
	result.setHeight(MINIMAP_SIZE * h_ / INVENTORY_SLOTS);
	return result;
}

std::string ItemLocation::GetForumCode(const std::string &league) const {
	switch (type_) {
	case ItemLocationType::STASH:
		return QString("[linkItem location=\"Stash%1\" league=\"%2\" x=\"%3\" y=\"%4\"]")
			.arg(QString::number(tab_id_ + 1), league.c_str(), QString::number(x_), QString::number(y_))
			.toStdString();
	case ItemLocationType::CHARACTER:
		return QString("[linkItem location=\"%1\" character=\"%2\" x=\"%3\" y=\"%4\"]")
			.arg(inventory_id_.c_str(), character_.c_str(), QString::number(x_), QString::number(y_))
			.toStdString();
	};
}

bool ItemLocation::IsValid() const {
	switch (type_) {
	case ItemLocationType::STASH: return !tab_unique_id_.empty();
	case ItemLocationType::CHARACTER: return !character_.empty();
	};
}

std::string ItemLocation::GetUniqueHash() const {
	if (!IsValid()) {
		QLOG_ERROR() << "ItemLocation is invalid:" << json_.c_str();;
	};
	switch (type_) {
	case ItemLocationType::STASH: return "stash:" + tab_unique_id_; // use unique id because tab label is not unique
	case ItemLocationType::CHARACTER: return "character:" + character_;
	};
}

void ItemLocation::set_json(rapidjson::Value &value, rapidjson_allocator &alloc) {
	if (type_ == ItemLocationType::CHARACTER) {
		value.AddMember("i", tab_id_, alloc);
	};
	json_ = Util::RapidjsonSerialize(value);
}

bool ItemLocation::operator<(const ItemLocation &rhs) const {
	switch (type_) {
	case ItemLocationType::STASH: return std::tie(type_, tab_id_) < std::tie(rhs.type_, rhs.tab_id_);
	case ItemLocationType::CHARACTER: return std::tie(type_, character_) < std::tie(rhs.type_, rhs.character_);
	};
}

bool ItemLocation::operator==(const ItemLocation &other) const {
	return get_tab_uniq_id() == other.get_tab_uniq_id();
}
