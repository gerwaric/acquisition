#pragma once

#include <QRectF>
#include <QColor>
#include "rapidjson/document.h"
#include "rapidjson_util.h"

enum class ItemLocationType {
	STASH,
	CHARACTER
};
QDebug& operator<<(QDebug& os, const ItemLocationType& obj);


class ItemLocation {
public:
	ItemLocation();
	explicit ItemLocation(const rapidjson::Value& root);
	explicit ItemLocation(int tab_id, std::string tab_unique_id, std::string name);
	explicit ItemLocation(int tab_id, std::string tab_unique_id, std::string name,
		ItemLocationType type, int r, int g, int b,
		rapidjson::Value& value, rapidjson_allocator& alloc);

	void ToItemJson(rapidjson::Value* root, rapidjson_allocator& alloc);
	void FromItemJson(const rapidjson::Value& root);
	std::string GetHeader() const;
	QRectF GetRect() const;
	std::string GetForumCode(const std::string& league) const;
	std::string GetUniqueHash() const;
	bool IsValid() const;
	bool operator<(const ItemLocation& other) const;
	bool operator==(const ItemLocation& other) const;
	ItemLocationType get_type() const { return type_; }
	std::string get_tab_label() const { return tab_label_; }
	std::string get_character() const { return character_; }
	bool socketed() const { return socketed_; }
	bool removeonly() const { return removeonly_; }
	void set_socketed(bool socketed) { socketed_ = socketed; }
	int get_tab_id() const { return tab_id_; }
	int getR() const { return red_; }
	int getG() const { return green_; }
	int getB() const { return blue_; }
	std::string get_tab_uniq_id() const { return type_ == ItemLocationType::STASH ? tab_unique_id_ : character_; }
	std::string get_json() const { return json_; }
private:
	void FixUid();

	int x_, y_, w_, h_;
	int red_, green_, blue_;
	bool socketed_;
	bool removeonly_;
	ItemLocationType type_;
	int tab_id_;
	std::string json_;

	//this would be the value "tabs -> id", which seems to be a hashed value generated on their end
	std::string tab_unique_id_;

	std::string tab_label_;
	std::string character_;
	std::string inventory_id_;
};

typedef std::vector<ItemLocation> Locations;
