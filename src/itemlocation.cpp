#include "itemlocation.h"

#include <QString>

#include <boost/algorithm/string/predicate.hpp>

#include "poe_api/poe_character.h"
#include "poe_api/poe_item.h"
#include "poe_api/poe_stash.h"

#include "QsLog.h"
#include "item.h"
#include "itemconstants.h"
#include "util.h"

using boost::algorithm::ends_with;

QDebug& operator<<(QDebug& os, const ItemLocationType& obj) {
	switch (obj) {
	case ItemLocationType::STASH: return os << "STASH";
	case ItemLocationType::CHARACTER: return os << "CHARACTER";
	case ItemLocationType::INVALID: return os << "INVALID";
	default: return os << "<Invalid ItemLocationType(" << static_cast<long long int>(obj) << ")>";
	};
}

ItemLocation::ItemLocation() :
	type_(ItemLocationType::INVALID),
	stash_index_(0),
	red_(0), green_(0), blue_(0),
	removeonly_(false),
	x_(0), y_(0), w_(0), h_(0),
	socketed_(false)
{}

ItemLocation::ItemLocation(const PoE::Character& character) :
	ItemLocation()
{
	type_ = ItemLocationType::CHARACTER;
	id_ = character.id;
	name_ = character.name;
}

ItemLocation::ItemLocation(const PoE::Character& character, const PoE::Item& item) :
	ItemLocation(character)
{
	SocketInto(item);
}

ItemLocation::ItemLocation(const PoE::StashTab& stash) :
	ItemLocation()
{
	type_ = ItemLocationType::STASH;
	id_ = stash.id;
	name_ = stash.name;
	if (stash.index) { stash_index_ = *stash.index; };
	if (stash.metadata.colour) {
		red_ = std::stoul(stash.metadata.colour->substr(0, 2));
		green_ = std::stoul(stash.metadata.colour->substr(2, 2));
		blue_ = std::stoul(stash.metadata.colour->substr(4, 2));
	};
	removeonly_ = ends_with(stash.name, "(Remove-only)");
}

ItemLocation::ItemLocation(const PoE::StashTab& stash, const PoE::Item& item) :
	ItemLocation(stash)
{
	SocketInto(item);
}

ItemLocation::ItemLocation(const ItemLocation& location, const PoE::Item& item) :
	ItemLocation(location)
{
	SocketInto(item);
}

ItemLocation::ItemLocation(int tab_id, std::string tab_unique_id, std::string name) :
	ItemLocation()
{
	type_ = ItemLocationType::STASH;
	id_ = tab_unique_id;
	name_ = name;
	stash_index_ = tab_id;
}

void ItemLocation::SocketInto(const PoE::Item& item) {
	w_ = item.w;
	h_ = item.h;
	if (item.x) { x_ = *item.x; };
	if (item.y) { y_ = *item.y; };
	if (item.inventoryId) { inventory_id_ = *item.inventoryId; };
	socketed_ = true;
}

std::string ItemLocation::GetHeader() const {
	switch (type_) {
	case ItemLocationType::STASH: return QString("#%1, \"%2\"").arg(
		QString::number(stash_index_ + 1),
		QString::fromStdString(name_)).toStdString();
	case ItemLocationType::CHARACTER: return name_;
	default: return "";
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

std::string ItemLocation::GetForumCode(const PoE::LeagueName& league) const {
	switch (type_) {
	case ItemLocationType::STASH:
		return QString("[linkItem location=\"Stash%1\" league=\"%2\" x=\"%3\" y=\"%4\"]").arg(
			QString::number(stash_index_ + 1),
			QString(league),
			QString::number(x_),
			QString::number(y_)).toStdString();
	case ItemLocationType::CHARACTER:
		return QString("[linkItem location=\"%1\" character=\"%2\" x=\"%3\" y=\"%4\"]").arg(
			QString::fromStdString(inventory_id_),
			QString::fromStdString(name_),
			QString::number(x_),
			QString::number(y_)).toStdString();
	default:
		return "";
	};
}

