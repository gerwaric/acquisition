// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <functional>
#include <memory>
#include <vector>

#include "item.h"

class QLineEdit;
class QLayout;
class QObject;

class FilterData;

struct FilterCallbacks
{
    QObject *receiver = nullptr;
    std::function<void()> onChanged;
    std::function<void()> onChangedDelayed;
};

/*
 * Objects of subclasses of this class do the following:
 * 1) FromForm: provided with a FilterData fill it with data from form
 * 2) ToForm: provided with a FilterData fill form with data from it
 * 3) Matches: check if an item matches the filter provided with FilterData
 */
class Filter
{
public:
    explicit Filter(const FilterCallbacks &callbacks = {});
    virtual ~Filter() {};

    // Non-copyable
    Filter(const Filter &) = delete;
    Filter &operator=(const Filter &) = delete;

    // Moveable
    Filter(Filter &&) = default;
    Filter &operator=(Filter &&) = default;

    virtual void FromForm(FilterData *data) = 0;
    virtual void ToForm(FilterData *data) = 0;
    virtual void ResetForm() = 0;
    virtual bool Matches(const std::shared_ptr<Item> &item, FilterData *data) = 0;

    std::unique_ptr<FilterData> CreateData();
    bool IsActive() const { return m_active; }

protected:
    bool m_active{false};
    FilterCallbacks m_callbacks;
};

struct ModFilterData
{
    ModFilterData(
        const QString &m_mod, double m_min, double m_max, bool m_min_filled, bool m_max_filled)
        : mod(m_mod)
        , min(m_min)
        , max(m_max)
        , min_filled(m_min_filled)
        , max_filled(m_max_filled)
    {}

    QString mod;
    double min, max;
    bool min_filled, max_filled;
};

/*
 * This is used to store legacy filter data in Search.
 */
class FilterData
{
public:
    FilterData(Filter *filter);
    Filter *filter() { return m_filter; }
    bool Matches(const std::shared_ptr<Item> &item);
    void FromForm();
    void ToForm();
    int r, g, b;
    bool r_filled, g_filled, b_filled;
    std::vector<ModFilterData> mod_data;

private:
    Filter *m_filter;
};

class SocketsColorsFilter : public Filter
{
public:
    SocketsColorsFilter() {};
    SocketsColorsFilter(QLayout *parent, const FilterCallbacks &callbacks);
    void FromForm(FilterData *data);
    void ToForm(FilterData *data);
    void ResetForm();
    bool Matches(const std::shared_ptr<Item> &item, FilterData *data);
    void Initialize(QLayout *parent, const char *caption, const FilterCallbacks &callbacks);

protected:
    bool Check(int need_r, int need_g, int need_b, int got_r, int got_g, int got_b, int got_w);
    QLineEdit *m_textbox_r{nullptr}, *m_textbox_g{nullptr}, *m_textbox_b{nullptr};
};

class LinksColorsFilter : public SocketsColorsFilter
{
public:
    LinksColorsFilter(QLayout *parent, const FilterCallbacks &callbacks);
    bool Matches(const std::shared_ptr<Item> &item, FilterData *data);
};
