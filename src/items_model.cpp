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

#include "items_model.h"

#include <QsLog/QsLog.h>

#include "util/util.h"

#include "application.h"
#include "bucket.h"
#include "buyoutmanager.h"
#include "itemlocation.h"
#include "search.h"

ItemsModel::ItemsModel(BuyoutManager& bo_manager, Search& search)
    : m_bo_manager(bo_manager)
    , m_search(search)
    , m_sort_order(Qt::DescendingOrder)
    , m_sort_column(0)
    , m_sorted(false)
{
}

/*
    Tree structure:

    + stash tab title (called "bucket" elsewhere)
    |- item
    |- item
      ...
    + another stash tab or character
    |- item
    |- item

    and so on
*/

int ItemsModel::rowCount(const QModelIndex& parent) const {
    // Root element, contains buckets
    if (!parent.isValid()) {
        return static_cast<int>(m_search.buckets().size());
    };
    // Bucket, contains elements
    if (parent.isValid() && !parent.parent().isValid()) {
        const int bucket_row = parent.row();
        if (m_search.has_bucket(bucket_row)) {
            return static_cast<int>(m_search.bucket(bucket_row).items().size());
        } else {
            return 0;
        };
    };
    // Element, contains nothing
    return 0;
}

int ItemsModel::columnCount(const QModelIndex& parent) const {
    // Root element, contains buckets
    if (!parent.isValid()) {
        return static_cast<int>(m_search.columns().size());
    };
    // Bucket, contains elements
    if (parent.isValid() && !parent.parent().isValid()) {
        return static_cast<int>(m_search.columns().size());
    };
    // Element, contains nothing
    return 0;
}

QVariant ItemsModel::headerData(int section, Qt::Orientation /* orientation */, int role) const {
    if (role == Qt::DisplayRole) {
        return QString(m_search.columns()[section]->name());
    };
    return QVariant();
}

QVariant ItemsModel::data(const QModelIndex& index, int role) const {
    // Bucket title
    if (!index.isValid()) {
        return QVariant();
    };

    if (index.internalId() == 0) {
        if (index.column() > 0) {
            return QVariant();
        };

        const ItemLocation& location = m_search.GetTabLocation(index);
        if (role == Qt::CheckStateRole) {
            if (!location.IsValid()) {
                return QVariant();
            };
            if (m_bo_manager.GetRefreshLocked(location)) {
                return Qt::PartiallyChecked;
            };
            return (m_bo_manager.GetRefreshChecked(location) ? Qt::Checked : Qt::Unchecked);
        };
        if (role == Qt::DisplayRole) {
            if (!location.IsValid()) {
                return "All Items";
            };
            QString title(location.GetHeader());
            auto const& bo = m_bo_manager.GetTab(location.GetUniqueHash());
            if (bo.IsActive()) {
                title += QString(" [%1]").arg(bo.AsText());
            };
            return title;
        };
        if (location.IsValid() && location.get_type() == ItemLocationType::STASH) {
            if (role == Qt::BackgroundRole) {
                QColor backgroundColor(location.getR(), location.getG(), location.getB());
                if (backgroundColor.isValid()) {
                    return backgroundColor;
                };
            };
            if (role == Qt::ForegroundRole) {
                QColor backgroundColor(location.getR(), location.getG(), location.getB());
                return Util::recommendedForegroundTextColor(backgroundColor);
            };
        };
        return QVariant();
    };
    auto& column = m_search.columns()[index.column()];
    const int bucket_row = index.parent().row();
    if (m_search.has_bucket(bucket_row)) {
        const Bucket& bucket = m_search.bucket(bucket_row);
        const int item_row = index.row();
        if (bucket.has_item(item_row)) {
            const Item& item = *bucket.item(item_row);
            if (role == Qt::DisplayRole) {
                return column->value(item);
            } else if (role == Qt::ForegroundRole) {
                return column->color(item);
            } else if (role == Qt::DecorationRole) {
                return column->icon(item);
            };
        } else {
            QLOG_ERROR() << "items model cannot get data: bucket" << bucket_row << "does not have" << item_row << "items";
        };
    } else {
        QLOG_ERROR() << "items model cannot get data: bucket" << bucket_row << "does not exist";
    };
    return QVariant();
}

Qt::ItemFlags ItemsModel::flags(const QModelIndex& index) const
{
    if (!index.isValid()) {
        return Qt::ItemFlags();
    };
    if (index.column() == 0 && index.internalId() == 0) {
        const ItemLocation& location = m_search.GetTabLocation(index);
        if (location.IsValid() && !m_bo_manager.GetRefreshLocked(location)) {
            return Qt::ItemIsSelectable | Qt::ItemIsEnabled | Qt::ItemIsUserCheckable;
        };
    };
    return Qt::ItemIsSelectable | Qt::ItemIsEnabled;
}

bool ItemsModel::setData(const QModelIndex& index, const QVariant& value, int role) {
    if (!index.isValid()) {
        return false;
    };

    if (role == Qt::CheckStateRole) {
        const ItemLocation& location = m_search.GetTabLocation(index);
        m_bo_manager.SetRefreshChecked(location, value.toBool());

        // It's possible that our tabs can have the same name.  Right now we don't have a
        // way to differentiate these tabs so indicate dataChanged event for each tab with
        // the same name as the current checked tab so the 'check' is properly updated in
        // the layout
        QString target_hash = location.GetUniqueHash();
        auto row_count = rowCount();
        for (int i = 0; i < row_count; ++i) {
            auto match_index = this->index(i);
            if (m_search.GetTabLocation(match_index).GetUniqueHash() == target_hash) {
                emit dataChanged(match_index, match_index);
            };
        };
        return true;
    };
    return false;
}

void ItemsModel::sort(int column, Qt::SortOrder order)
{
    // Ignore sort requests if we're already sorted
    if (m_sorted && (m_sort_column == column) && (m_sort_order == order))
        return;

    QLOG_DEBUG() << "Sorting items model by column" << column;
    m_sort_order = order;
    m_sort_column = column;

    m_search.Sort(column, order);
    emit layoutChanged();
    SetSorted(true);
}

void ItemsModel::sort()
{
    sort(m_sort_column, m_sort_order);
}

QModelIndex ItemsModel::parent(const QModelIndex& index) const {
    // bucket
    if (!index.isValid() || index.internalId() == 0) {
        return QModelIndex();
    }
    // item
    return createIndex(index.internalId() - 1, 0, static_cast<quintptr>(0));
}

QModelIndex ItemsModel::index(int row, int column, const QModelIndex& parent) const {
    int bucket_count = static_cast<int>(m_search.buckets().size());
    if (parent.isValid()) {
        if (parent.row() >= bucket_count) {
            QLOG_WARN() << "ItemsModel: index parent row is invalid";
            return QModelIndex();
        };
        // item, we pass parent's (bucket's) row through ID parameter
        return createIndex(row, column, static_cast<quintptr>(parent.row()) + 1);
    } else {
        if (row >= bucket_count) {
            QLOG_WARN() << "ItemsModel: index row is invalid:" + QString::number(row);
            return QModelIndex();
        };
        return createIndex(row, column, static_cast<quintptr>(0));
    };
}
