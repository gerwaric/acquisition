// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include "column.h"
#include "item.h"
#include "itemlocation.h"

// A bucket holds set of filtered items.
// Items are "bucketed" by their location: stash tab / character.
class Bucket
{
public:
    Bucket() = default;
    explicit Bucket(const ItemLocation &location);

    // Non-copyable
    Bucket(const Bucket &) = delete;
    Bucket &operator=(const Bucket &) = delete;

    // Moveable
    Bucket(Bucket &&) = default;
    Bucket &operator=(Bucket &&) = default;

    void AddItem(const std::shared_ptr<Item> &item);
    void AddItems(const Items &items);
    const Items &items() const { return m_items; }
    bool has_item(int row) const;
    const std::shared_ptr<Item> &item(int row) const;
    const ItemLocation &location() const { return m_location; }
    void Sort(const Column &column, Qt::SortOrder order);

private:
    Items m_items;
    ItemLocation m_location;
};
