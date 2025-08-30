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

#include <QObject>
#include <QString>

#include <memory>
#include <set>
#include <vector>

#include <util/util.h>

#include "bucket.h"
#include "column.h"
#include "item.h"
#include "items_model.h"

class BuyoutManager;
class Filter;
class FilterData;
class ItemsModel;
class QTreeView;
class QModelIndex;

class Search
{
    Q_GADGET
public:
    enum class ViewMode : int { ByTab = 0, ByItem = 1 };
    Q_ENUM(ViewMode)

    Search(BuyoutManager &bo,
           const QString &caption,
           const std::vector<std::unique_ptr<Filter>> &filters,
           QTreeView *view);
    void FilterItems(const Items &items);
    void FromForm();
    void ToForm();
    void ResetForm();
    const QString &caption() const { return m_caption; }
    const Items &items() const { return m_items; }
    const std::vector<std::unique_ptr<Column>> &columns() const { return m_columns; }
    const std::vector<Bucket> &buckets() const;
    void RenameCaption(const QString &newName);
    QString GetCaption() const;
    // Sets this search as current, will display items in passed QTreeView.
    void Activate(const Items &items);
    void RestoreViewProperties();
    void SaveViewProperties();
    ItemLocation GetTabLocation(const QModelIndex &index) const;
    void SetViewMode(ViewMode mode);
    ViewMode GetViewMode() const { return m_current_mode; }
    bool has_bucket(int row) const;
    const Bucket &bucket(int row) const;
    const QModelIndex index(const std::shared_ptr<Item> &item) const;
    void SetRefreshReason(RefreshReason reason) { m_refresh_reason = reason; }
    void Sort(int column, Qt::SortOrder order);

private:
    std::vector<Bucket> &active_buckets();

    BuyoutManager &m_bo_manager;
    QTreeView &m_view;

    std::vector<std::unique_ptr<FilterData>> m_filters;
    std::vector<std::unique_ptr<Column>> m_columns;

    ItemsModel m_model;
    std::vector<Bucket> m_bucket_by_tab;
    std::vector<Bucket> m_bucket_by_item;

    QString m_caption;
    Items m_items;
    bool m_filtered;
    size_t m_filtered_item_count;
    std::set<QString> m_expanded_property;
    ViewMode m_current_mode;
    RefreshReason m_refresh_reason;
};

template<>
struct fmt::formatter<Search::ViewMode, char> : QtEnumFormatter<Search::ViewMode>
{};
