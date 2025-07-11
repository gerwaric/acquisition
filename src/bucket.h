/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include "item.h"
#include "itemlocation.h"
#include "column.h"

// A bucket holds set of filtered items.
// Items are "bucketed" by their location: stash tab / character.
class Bucket {
public:
    Bucket() = default;
    explicit Bucket(const ItemLocation& location);

    // Non-copyable
    Bucket(const Bucket&) = delete;
    Bucket& operator= (const Bucket&) = delete;

    // Moveable
    Bucket(Bucket&&) = default;
    Bucket& operator = (Bucket&&) = default;

    void AddItem(const std::shared_ptr<Item>& item);
    void AddItems(const Items& items);
    const Items& items() const { return m_items; }
    bool has_item(int row) const;
    const std::shared_ptr<Item>& item(int row) const;
    const ItemLocation& location() const { return m_location; }
    void Sort(const Column& column, Qt::SortOrder order);

private:
    Items m_items;
    ItemLocation m_location;
};
