// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "bucket.h"

#include "util/fatalerror.h"

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
