/*
    Copyright 2014 Ilya Zhuravlev

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

#include <QMessageBox>

#include "QsLog.h"

Bucket::Bucket(const ItemLocation& location) :
    location_(location)
{}

void Bucket::AddItem(const std::shared_ptr<Item>& item) {
    items_.push_back(item);
}

const std::shared_ptr<Item>& Bucket::item(int row) const
{
    if ((row < 0) || (row >= items_.size())) {
        const QString message = QString("Item row out of bounds: %1 item count: %2. Program will abort").arg(
            QString::number(row),
            QString::number(items_.size()));
        QLOG_FATAL() << message;
        QMessageBox::critical(nullptr, "Fatal Error", message);
        abort();
    };
    return items_[row];
}

void Bucket::Sort(const Column& column, Qt::SortOrder order)
{
    std::sort(begin(items_), end(items_),
        [&](const std::shared_ptr<Item>& lhs, const std::shared_ptr<Item>& rhs)
        {
            if (order == Qt::AscendingOrder) {
                return column.lt(rhs.get(), lhs.get());
            } else {
                return column.lt(lhs.get(), rhs.get());
            };
        }
    );
}
