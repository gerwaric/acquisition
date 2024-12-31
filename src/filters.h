/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include <functional>
#include <memory>

#include "item.h"

class QLineEdit;
class QCheckBox;
class QComboBox;
class QCompleter;
class QAbstractListModel;
class QLayout;

class BuyoutManager;
class FilterData;
class SearchComboBox;

/*
 * Objects of subclasses of this class do the following:
 * 1) FromForm: provided with a FilterData fill it with data from form
 * 2) ToForm: provided with a FilterData fill form with data from it
 * 3) Matches: check if an item matches the filter provided with FilterData
 */
class Filter {
public:

    Filter() = default;
    virtual ~Filter() {};

    // Non-copyable
    Filter(const Filter&) = delete;
    Filter& operator= (const Filter&) = delete;

    // Moveable
    Filter(Filter&&) = default;
    Filter& operator= (Filter&&) = default;

    virtual void FromForm(FilterData* data) = 0;
    virtual void ToForm(FilterData* data) = 0;
    virtual void ResetForm() = 0;
    virtual bool Matches(const std::shared_ptr<Item>& item, FilterData* data) = 0;

    std::unique_ptr<FilterData> CreateData();
    bool IsActive() const { return m_active; };

protected:
    bool m_active{ false };
};

struct ModFilterData {
    ModFilterData(const std::string& m_mod, double m_min, double m_max, bool m_min_filled, bool m_max_filled) :
        mod(m_mod),
        min(m_min),
        max(m_max),
        min_filled(m_min_filled),
        max_filled(m_max_filled)
    {}

    std::string mod;
    double min, max;
    bool min_filled, max_filled;
};

/*
 * This is used to store filter data in Search,
 * i.e. min-max values that the user has specified.
 */
class FilterData {
public:
    FilterData(Filter* filter);
    Filter* filter() { return m_filter; }
    bool Matches(const std::shared_ptr<Item> item);
    void FromForm();
    void ToForm();
    // Various types of data for various filters
    // It's probably not a very elegant solution but it works.
    std::string text_query;
    double min, max;
    bool min_filled, max_filled;
    int r, g, b;
    bool r_filled, g_filled, b_filled;
    bool checked;
    std::vector<ModFilterData> mod_data;
private:
    Filter* m_filter;
};

class NameSearchFilter : public Filter {
public:
    explicit NameSearchFilter(QLayout* parent);
    void FromForm(FilterData* data);
    void ToForm(FilterData* data);
    void ResetForm();
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
    void Initialize(QLayout* parent);
private:
    QLineEdit* m_textbox;
};

class CategorySearchFilter : public Filter {
public:
    explicit CategorySearchFilter(QLayout* parent, QAbstractListModel* model);
    void FromForm(FilterData* data);
    void ToForm(FilterData* data);
    void ResetForm();
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
    void Initialize(QLayout* parent);
    static const std::string k_Default;
private:
    SearchComboBox* m_combobox;
    QAbstractListModel* m_model;
};

class RaritySearchFilter : public Filter {
public:
    explicit RaritySearchFilter(QLayout* parent, QAbstractListModel* model);
    void FromForm(FilterData* data);
    void ToForm(FilterData* data);
    void ResetForm();
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
    void Initialize(QLayout* parent);
    static const std::string k_Default;
    static const QStringList RARITY_LIST;
private:
    QComboBox* m_combobox;
    QAbstractListModel* m_model;
};

class MinMaxFilter : public Filter {
public:
    MinMaxFilter(QLayout* parent, std::string property);
    MinMaxFilter(QLayout* parent, std::string property, std::string caption);
    void FromForm(FilterData* data);
    void ToForm(FilterData* data);
    void ResetForm();
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
    void Initialize(QLayout* parent);
protected:
    virtual double GetValue(const std::shared_ptr<Item>& item) = 0;
    virtual bool IsValuePresent(const std::shared_ptr<Item>& item) = 0;

    std::string m_property, m_caption;
private:
    QLineEdit* m_textbox_min, * m_textbox_max;
};

class SimplePropertyFilter : public MinMaxFilter {
public:
    SimplePropertyFilter(QLayout* parent, std::string property) :
        MinMaxFilter(parent, property) {}
    SimplePropertyFilter(QLayout* parent, std::string property, std::string caption) :
        MinMaxFilter(parent, property, caption) {}
protected:
    bool IsValuePresent(const std::shared_ptr<Item>& item);
    double GetValue(const std::shared_ptr<Item>& item);
};

// Just like SimplePropertyFilter but assumes given default value instead of excluding items
class DefaultPropertyFilter : public SimplePropertyFilter {
public:
    DefaultPropertyFilter(QLayout* parent, std::string property, double default_value) :
        SimplePropertyFilter(parent, property),
        m_default_value(default_value)
    {}
    DefaultPropertyFilter(QLayout* parent, std::string property, std::string caption, double default_value) :
        SimplePropertyFilter(parent, property, caption),
        m_default_value(default_value)
    {}
protected:
    bool IsValuePresent(const std::shared_ptr<Item>& /* item */) { return true; }
    double GetValue(const std::shared_ptr<Item>& item);
private:
    double m_default_value;
};

class RequiredStatFilter : public MinMaxFilter {
public:
    RequiredStatFilter(QLayout* parent, std::string property) :
        MinMaxFilter(parent, property) {}
    RequiredStatFilter(QLayout* parent, std::string property, std::string caption) :
        MinMaxFilter(parent, property, caption) {}
private:
    bool IsValuePresent(const std::shared_ptr<Item>& /* item */) { return true; }
    double GetValue(const std::shared_ptr<Item>& item);
};

class ItemMethodFilter : public MinMaxFilter {
public:
    ItemMethodFilter(QLayout* parent, std::function<double(Item*)> func, std::string caption);
private:
    bool IsValuePresent(const std::shared_ptr<Item>& /* item */) { return true; }
    double GetValue(const std::shared_ptr<Item>& item);
    std::function<double(Item*)> m_func;
};

class SocketsFilter : public MinMaxFilter {
public:
    SocketsFilter(QLayout* parent, std::string property) :
        MinMaxFilter(parent, property) {}
    SocketsFilter(QLayout* parent, std::string property, std::string caption) :
        MinMaxFilter(parent, property, caption) {}
    bool IsValuePresent(const std::shared_ptr<Item>& /* item */) { return true; }
    double GetValue(const std::shared_ptr<Item>& item);
};

class LinksFilter : public MinMaxFilter {
public:
    LinksFilter(QLayout* parent, std::string property) :
        MinMaxFilter(parent, property) {}
    LinksFilter(QLayout* parent, std::string property, std::string caption) :
        MinMaxFilter(parent, property, caption) {}
    bool IsValuePresent(const std::shared_ptr<Item>& /* item */) { return true; }
    double GetValue(const std::shared_ptr<Item>& item);
};

class SocketsColorsFilter : public Filter {
public:
    SocketsColorsFilter() {};
    explicit SocketsColorsFilter(QLayout* parent);
    void FromForm(FilterData* data);
    void ToForm(FilterData* data);
    void ResetForm();
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
    void Initialize(QLayout* parent, const char* caption);
protected:
    bool Check(int need_r, int need_g, int need_b, int got_r, int got_g, int got_b, int got_w);
    QLineEdit* m_textbox_r{ nullptr }, * m_textbox_g{ nullptr }, * m_textbox_b{ nullptr };
};

class LinksColorsFilter : public SocketsColorsFilter {
public:
    explicit LinksColorsFilter(QLayout* parent);
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
};

class BooleanFilter : public Filter {
public:
    BooleanFilter(QLayout* parent, std::string property, std::string caption);
    void FromForm(FilterData* data);
    void ToForm(FilterData* data);
    void ResetForm();
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
    void Initialize(QLayout* parent);
private:
    QCheckBox* m_checkbox;
protected:
    std::string m_property, m_caption;
};

class AltartFilter : public BooleanFilter {
public:
    AltartFilter(QLayout* parent, std::string property, std::string caption) :
        BooleanFilter(parent, property, caption) {}
    using BooleanFilter::BooleanFilter;
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
};

class PricedFilter : public BooleanFilter {
public:
    PricedFilter(QLayout* parent, std::string property, std::string caption, const BuyoutManager& bm) :
        BooleanFilter(parent, property, caption),
        m_bm(bm)
    {}
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
private:
    const BuyoutManager& m_bm;
};

class UnidentifiedFilter : public BooleanFilter {
public:
    UnidentifiedFilter(QLayout* parent, std::string property, std::string caption) :
        BooleanFilter(parent, property, caption) {}
    using BooleanFilter::BooleanFilter;
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
};

class CraftedFilter : public BooleanFilter {
public:
    CraftedFilter(QLayout* parent, std::string property, std::string caption) :
        BooleanFilter(parent, property, caption) {}
    using BooleanFilter::BooleanFilter;
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
};

class EnchantedFilter : public BooleanFilter {
public:
    EnchantedFilter(QLayout* parent, std::string property, std::string caption) :
        BooleanFilter(parent, property, caption) {}
    using BooleanFilter::BooleanFilter;
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
};

class InfluencedFilter : public BooleanFilter {
public:
    InfluencedFilter(QLayout* parent, std::string property, std::string caption) :
        BooleanFilter(parent, property, caption) {}
    using BooleanFilter::BooleanFilter;
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
};

class CorruptedFilter : public BooleanFilter {
public:
    CorruptedFilter(QLayout* parent, std::string property, std::string caption) :
        BooleanFilter(parent, property, caption) {}
    using BooleanFilter::BooleanFilter;
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
};

class ItemlevelFilter : public MinMaxFilter {
public:
    ItemlevelFilter(QLayout* parent, std::string property) :
        MinMaxFilter(parent, property) {}
    ItemlevelFilter(QLayout* parent, std::string property, std::string caption) :
        MinMaxFilter(parent, property, caption) {}
    bool IsValuePresent(const std::shared_ptr<Item>& /* item */) { return true; }
    double GetValue(const std::shared_ptr<Item>& item);
};
