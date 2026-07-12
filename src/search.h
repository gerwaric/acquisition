// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QObject>
#include <QString>

#include <memory>
#include <set>
#include <vector>

#include "bucket.h"
#include "column.h"
#include "filters/filterstate.h"
#include "item.h"
#include "items_model.h"
#include "util/util.h"

class BuyoutManager;
class FilterCatalog;
class ItemsModel;
class QModelIndex;

class Search
{
    Q_GADGET
public:
    enum class ViewMode : int { ByTab = 0, ByItem = 1 };
    Q_ENUM(ViewMode)

    Search(BuyoutManager &bo, const QString &caption, const FilterCatalog &catalog);
    ~Search();
    void FilterItems(const Items &items);
    const QString &caption() const { return m_caption; }
    const Items &items() const { return m_items; }
    const std::vector<std::unique_ptr<Column>> &columns() const { return m_columns; }
    ItemsModel &model() { return m_model; }
    const std::set<QString> &expandedHeaders() const { return m_expanded_property; }
    void setExpandedHeaders(std::set<QString> headers);
    bool defaultExpanded() const { return m_filtered || (m_current_mode == ViewMode::ByItem); }
    const std::vector<Bucket> &buckets() const;
    void RenameCaption(const QString &newName);
    QString GetCaption() const;
    ItemLocation GetTabLocation(const QModelIndex &index) const;
    void SetViewMode(ViewMode mode);
    ViewMode GetViewMode() const { return m_current_mode; }
    bool has_bucket(int row) const;
    const Bucket &bucket(int row) const;
    const QModelIndex index(const std::shared_ptr<Item> &item) const;
    void SetRefreshReason(RefreshReason reason) { m_refresh_reason = reason; }
    void Sort(int column, Qt::SortOrder order);

    const FilterCatalog &catalog() const { return m_filter_catalog; }
    qsizetype filterStateCount() const { return static_cast<qsizetype>(m_filter_states.size()); }
    FilterState &filterStateAt(qsizetype index);
    const FilterState &filterStateAt(qsizetype index) const;

private:
    std::vector<Bucket> &active_buckets();

    BuyoutManager &m_bo_manager;

    // Catalog and filter states are index-aligned. MainWindow owns the catalog
    // and outlives every Search.
    const FilterCatalog &m_filter_catalog;
    std::vector<FilterState> m_filter_states;
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
