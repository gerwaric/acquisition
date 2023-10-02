#pragma once

#include <QRectF>
#include <QColor>
#include <set>
#include "rapidjson/document.h"

#include "itemconstants.h"
#include "rapidjson_util.h"
#include "QsLog.h"

enum class ItemLocationType {
	STASH,
	CHARACTER,
    NONE
};

class ItemLocation {
public:
	ItemLocation();
	explicit ItemLocation(const rapidjson::Value &root);
	ItemLocation(int tab_id, std::string tab_unique_id, std::string name, ItemLocationType = ItemLocationType::STASH, int r = 0, int g = 0, int b = 0);
	void ToItemJson(rapidjson::Value *root, rapidjson_allocator &alloc);
	void FromItemJson(const rapidjson::Value &root);
	std::string GetHeader() const;
	QRectF GetRect() const;
	std::string GetForumCode(const std::string &league) const;
	std::string GetUniqueHash() const;
	bool IsValid() const;
    bool IsRemoveOnly() const { return remove_only_; };
	bool operator<(const ItemLocation &other) const;
	bool operator==(const ItemLocation &other) const;
	void set_type(const ItemLocationType type) { type_ = type; }
	ItemLocationType get_type() const { return type_; }
	void set_character(const std::string &character) { character_ = character; }
	void set_tab_id(int tab_id) { tab_id_ = tab_id; }
	void set_tab_label(const std::string &tab_label) { tab_label_ = tab_label; }
	std::string get_tab_label() const { return tab_label_; }
	bool socketed() const { return socketed_; }
	void set_socketed(bool socketed) { socketed_ = socketed; }
	int get_tab_id() const { return tab_id_; }
	int getR() const {return red_;}
	int getG() const {return green_;}
	int getB() const {return blue_;}
	void SetBackgroundColor(int r, int g, int b);
	std::string get_tab_uniq_id() const {return type_ == ItemLocationType::STASH ? tab_unique_id_ : character_;}
	void set_json(rapidjson::Value &value, rapidjson_allocator &alloc);
	std::string get_json() const{return json_;}
private:
    std::string tab_label_{ "" };
    std::string character_{ "" };
    std::string tab_unique_id_{ "" };

    int x_{ -1 }, y_{ -1 };
    int w_{ -1 }, h_{ -1 };
    int red_{ -1 }, green_{ -1 }, blue_{ -1 };
    bool socketed_{ false };
    ItemLocationType type_{ ItemLocationType::NONE };
	int tab_id_{0};
    std::string json_{ "" };
    bool remove_only_{ false };

	std::string inventory_id_{ "" };
};

// Moves to ItemsManagerWorker
//static std::set<std::string> tab_id_index_;
