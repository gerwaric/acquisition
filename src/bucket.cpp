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

#include "bucket.h"

#include <util/fatalerror.h>

Bucket::Bucket(const ItemLocation &location)
    : m_location(location)
{}

void Bucket::AddItem(const std::shared_ptr<Item> &item)
{
    m_items.push_back(item);
}

void Bucket::AddItems(const Items &items)
{
    m_items.reserve(m_items.size() + items.size());
    for (const auto &item : items) {
        m_items.push_back(item);
    }
}

bool Bucket::has_item(int row) const
{
    return (row >= 0) && (row < static_cast<int>(m_items.size()));
}

const std::shared_ptr<Item> &Bucket::item(int row) const
{
    const int item_count = static_cast<int>(m_items.size());
    if ((row < 0) || (row >= item_count)) {
        const QString message
            = QString("Bucket item row out of bounds: %1 item count: %2. Program will abort")
                  .arg(QString::number(row), QString::number(item_count));
        FatalError(message);
    }
    return m_items[row];
}

void Bucket::Sort(const Column &column, Qt::SortOrder order)
{
    std::sort(begin(m_items),
              end(m_items),
              [&](const std::shared_ptr<Item> &lhs, const std::shared_ptr<Item> &rhs) {
                  if (order == Qt::AscendingOrder) {
                      return column.lt(rhs.get(), lhs.get());
                  } else {
                      return column.lt(lhs.get(), rhs.get());
                  }
              });
}
