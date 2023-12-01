#pragma once

#include <QRectF>
#include <QColor>
#include <set>

#include "poe_api/poe_typedefs.h"

class Item;

namespace PoE {
	struct Character;
	struct Item;
	struct StashTab;
}

enum class ItemLocationType {
	INVALID,
	STASH,
	CHARACTER,
};

QDebug& operator<<(QDebug& os, const ItemLocationType& obj);

class ItemLocation {
public:
	ItemLocation();
	explicit ItemLocation(const PoE::Character& character);
	explicit ItemLocation(const PoE::Character& character, const PoE::Item& item);
	explicit ItemLocation(const PoE::StashTab& stash);
	explicit ItemLocation(const PoE::StashTab& stash, const PoE::Item& item);
	explicit ItemLocation(const ItemLocation& location, const PoE::Item& item);
	explicit ItemLocation(int tab_id, std::string tab_unique_id, std::string name);

	//void FromItem(const Item& item);
	//void ToItem(Item& item);
	std::string GetHeader() const;
	std::string GetForumCode(const PoE::LeagueName& league) const;
	QRectF GetRect() const;
	bool IsValid() const { return type_ != ItemLocationType::INVALID; };

	const auto tie() const { return std::tie(type_, stash_index_, name_, id_); };
	bool operator<(const ItemLocation& other) const { return tie() < other.tie(); };
	bool operator==(const ItemLocation& other) const { return tie() == other.tie(); };

	ItemLocationType type() const { return type_; };
	const std::string& id() const { return id_; };
	const std::string& name() const { return name_; };
	const std::string& parent_id() const { return parent_id_; };
	int stash_index() const { return stash_index_; };
	const std::string& stash_type() const { return stash_type_; };
	int red() const { return red_; };
	int green() const { return green_; };
	int blue() const { return blue_; };
	bool removeonly() const { return removeonly_; };
	bool socketed() const { return socketed_; }
	const std::string& inventory_id() const { return inventory_id_; }

private:
	void SocketInto(const PoE::Item& item);

	// Either STASH, CHARACTER, or INVALID.
	ItemLocationType type_;

	// Both stash tabs and characters have a unique id.
	std::string id_;

	// Both stash tabs and characters have a name, but only character names appears to be unique.
	std::string name_;

	// Only stash tabs can have a parent stash.
	std::string parent_id_;

	// Only stash tabs have an index.
	int stash_index_;

	// Only stash tabs have a type, e.g. "CurrencyStash" or "PremiumStash".
	std::string stash_type_;

	// Only stash tabs have color metadata.
	int red_;

	// Only stash tabs have color metadata.
	int green_;

	// Only stash tabs have color metadata.
	int blue_;

	// Only stash tabs can be remove-only.
	bool removeonly_;
	
	int x_, y_, w_, h_;
	bool socketed_;
	std::string inventory_id_;
};

typedef std::vector<ItemLocation> Locations;
