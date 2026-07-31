// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "items_model.h"

#include "bucket.h"
#include "buyoutmanager.h"
#include "itemlocation.h"
#include "locationinventory.h"
#include "search.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/util.h"

#include <algorithm>
#include <iterator>
#include <map>
#include <memory>
#include <set>
#include <vector>

ItemsModel::ItemsModel(BuyoutManager &bo_manager, Search &search)
    : m_bo_manager(bo_manager)
    , m_search(search)
    , m_sort_order(Qt::DescendingOrder)
    , m_sort_column(0)
    , m_sorted(false)
{}

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

int ItemsModel::rowCount(const QModelIndex &parent) const
{
    // Root element, contains buckets
    if (!parent.isValid()) {
        return static_cast<int>(m_search.buckets().size());
    }
    if ((parent.model() != this) || (parent.column() != 0)) {
        return 0;
    }
    // Bucket, contains elements
    if (!parent.parent().isValid()) {
        const int bucket_row = parent.row();
        if (m_search.has_bucket(bucket_row)) {
            return static_cast<int>(m_search.bucket(bucket_row).items().size());
        } else {
            return 0;
        }
    }
    // Element, contains nothing
    return 0;
}

int ItemsModel::columnCount(const QModelIndex &parent) const
{
    if (parent.isValid() && (parent.model() != this)) {
        return 0;
    }
    // Root element, contains buckets
    if (!parent.isValid()) {
        return static_cast<int>(m_search.columns().size());
    }
    // Bucket, contains elements
    if (!parent.parent().isValid()) {
        return static_cast<int>(m_search.columns().size());
    }
    // Element, contains nothing
    return 0;
}

QVariant ItemsModel::headerData(int section, Qt::Orientation orientation, int role) const
{
    if (orientation == Qt::Horizontal && role == Qt::DisplayRole && section >= 0
        && section < static_cast<int>(m_search.columns().size())) {
        return QString(m_search.columns()[section]->name());
    }
    return QVariant();
}

QVariant ItemsModel::data(const QModelIndex &index, int role) const
{
    // Bucket title
    if (!index.isValid()) {
        return QVariant();
    }
    if ((index.model() != this) || (index.column() < 0)
        || (index.column() >= static_cast<int>(m_search.columns().size()))) {
        return QVariant();
    }

    if (index.internalId() == 0) {
        if (index.column() > 0) {
            return QVariant();
        }

        const ItemLocation &location = m_search.GetTabLocation(index);
        if (role == Qt::CheckStateRole) {
            if (!location.IsValid()) {
                return QVariant();
            }
            if (m_bo_manager.GetRefreshLocked(location)) {
                return Qt::PartiallyChecked;
            }
            return (m_bo_manager.GetRefreshChecked(location) ? Qt::Checked : Qt::Unchecked);
        }
        if (role == Qt::DisplayRole) {
            if (!location.IsValid()) {
                return "All Items";
            }
            QString title(location.GetHeader());
            const auto bo = m_bo_manager.GetTab(location);
            if (bo.IsActive()) {
                title += QString(" [%1]").arg(bo.AsText());
            }
            return title;
        }
        if (location.IsValid() && location.type() == ItemLocationType::STASH) {
            if (role == Qt::BackgroundRole) {
                QColor backgroundColor(location.getR(), location.getG(), location.getB());
                if (backgroundColor.isValid()) {
                    return backgroundColor;
                }
            }
            if (role == Qt::ForegroundRole) {
                QColor backgroundColor(location.getR(), location.getG(), location.getB());
                return Util::recommendedForegroundTextColor(backgroundColor);
            }
        }
        return QVariant();
    }
    auto &column = m_search.columns()[index.column()];
    const int bucket_row = index.parent().row();
    if (m_search.has_bucket(bucket_row)) {
        const Bucket &bucket = m_search.bucket(bucket_row);
        const int item_row = index.row();
        if (bucket.has_item(item_row)) {
            const Item &item = *bucket.item(item_row);
            if (role == Qt::DisplayRole) {
                return column->value(item);
            } else if (role == Qt::ForegroundRole) {
                return column->color(item);
            } else if (role == Qt::DecorationRole) {
                return column->icon(item);
            }
        } else {
            spdlog::error("items model cannot get data: bucket {} does not have {} items",
                          bucket_row,
                          item_row);
        }
    } else {
        spdlog::error("items model cannot get data: bucket {} does not exist", bucket_row);
    }
    return QVariant();
}

Qt::ItemFlags ItemsModel::flags(const QModelIndex &index) const
{
    if (!index.isValid()) {
        return Qt::ItemFlags();
    }
    if ((index.model() != this) || (index.column() < 0)
        || (index.column() >= static_cast<int>(m_search.columns().size()))) {
        return Qt::ItemFlags();
    }
    if (index.column() == 0 && index.internalId() == 0) {
        const ItemLocation &location = m_search.GetTabLocation(index);
        if (location.IsValid() && !m_bo_manager.GetRefreshLocked(location)) {
            return Qt::ItemIsSelectable | Qt::ItemIsEnabled | Qt::ItemIsUserCheckable;
        }
    }
    return Qt::ItemIsSelectable | Qt::ItemIsEnabled;
}

bool ItemsModel::setData(const QModelIndex &index, const QVariant &value, int role)
{
    if (!index.isValid() || (index.model() != this)) {
        return false;
    }

    if (role == Qt::CheckStateRole && index.column() == 0 && index.internalId() == 0) {
        const ItemLocation &location = m_search.GetTabLocation(index);
        if (!location.IsValid() || m_bo_manager.GetRefreshLocked(location)) {
            return false;
        }
        m_bo_manager.SetRefreshChecked(location, value.toBool());

        // It's possible that our tabs can have the same name.  Right now we don't have a
        // way to differentiate these tabs so indicate dataChanged event for each tab with
        // the same name as the current checked tab so the 'check' is properly updated in
        // the layout
        QString target_hash = location.id();
        auto row_count = rowCount();
        for (int i = 0; i < row_count; ++i) {
            auto match_index = this->index(i);
            if (m_search.GetTabLocation(match_index).id() == target_hash) {
                emit dataChanged(match_index, match_index, {Qt::CheckStateRole});
            }
        }
        return true;
    }
    return false;
}

void ItemsModel::refreshCheckStates()
{
    const int rows = rowCount();
    if (rows > 0) {
        emit dataChanged(index(0, 0), index(rows - 1, 0), {Qt::CheckStateRole});
    }
}

void ItemsModel::sort(int column, Qt::SortOrder order)
{
    if ((column < 0) || (column >= columnCount())) {
        return;
    }

    // Ignore sort requests if we're already sorted
    if (m_sorted && (m_sort_column == column) && (m_sort_order == order)) {
        return;
    }

    spdlog::debug("Sorting items model by column {}", column);
    // D1 rules 2-3: a column switch discards every resident key vector
    // (keys are derived from the outgoing column); any (column, order)
    // change invalidates every bucket's order at once. The post-refilter
    // indicator pass arrives with neither changed and clears nothing —
    // fresh buckets are already unsorted, and untouched valid buckets
    // (a background search re-activating after a scoped buyout batch)
    // keep their skip.
    if (column != m_sort_column) {
        m_search.EvictResidentKeys();
    }
    if ((column != m_sort_column) || (order != m_sort_order)) {
        m_search.InvalidateAllOrder();
    }
    m_sort_order = order;
    m_sort_column = column;
    ApplySort(column, order);
}

void ItemsModel::Resort()
{
    if ((m_sort_column < 0) || (m_sort_column >= columnCount())) {
        return;
    }
    ApplySort(m_sort_column, m_sort_order);
}

void ItemsModel::ResortBucket(int row)
{
    if ((m_sort_column < 0) || (m_sort_column >= columnCount()) || !m_search.has_bucket(row)) {
        return;
    }
    ApplySort(m_sort_column, m_sort_order, row);
}

void ItemsModel::OnBucketExpanded(int row)
{
    if (!m_search.has_bucket(row)) {
        return;
    }
    m_search.MarkBucketExpanded(row);
    // D2 rule 2: an invalid flag sorts the bucket first (building its
    // keys, which stay resident — D1); a valid flag does no sort work and
    // builds no keys — the bucket stays sorted-but-keyless until a
    // key-consuming event hydrates it (R3-1).
    if (!m_search.bucket(row).sorted()) {
        ApplySort(m_sort_column, m_sort_order, row);
    }
}

void ItemsModel::OnBucketCollapsed(int row)
{
    if (m_search.GetViewMode() == Search::ViewMode::ByItem) {
        return; // the flat bucket is materialized by definition (D2 rule 6)
    }
    if (!m_search.has_bucket(row)) {
        return;
    }
    m_search.MarkBucketCollapsed(row);
}

void ItemsModel::ApplySort(int column, Qt::SortOrder order, int only_bucket)
{
    struct ItemIndexSnapshot
    {
        QModelIndex from;
        int bucket_row;
        int column;
        std::shared_ptr<Item> item;
    };

    // Emit before snapshotting: listeners such as QItemSelectionModel create
    // persistent indexes inside their layoutAboutToBeChanged handlers, and
    // those must be included in the remapping below. A scoped sort names
    // its bucket in the parents list, so listeners save and restore state
    // for that subtree only instead of the whole model.
    QList<QPersistentModelIndex> parents;
    if (only_bucket >= 0) {
        parents.append(QPersistentModelIndex(index(only_bucket, 0)));
    }
    emit layoutAboutToBeChanged(parents, QAbstractItemModel::VerticalSortHint);

    std::vector<ItemIndexSnapshot> snapshots;
    const auto persistent_indexes = persistentIndexList();
    snapshots.reserve(persistent_indexes.size());
    for (const QModelIndex &persistent_index : persistent_indexes) {
        if (!persistent_index.isValid() || persistent_index.internalId() == 0) {
            continue;
        }

        const int bucket_row = m_search.rowForSerial(persistent_index.internalId());
        if ((only_bucket >= 0) && (bucket_row != only_bucket)) {
            continue; // a scoped sort moves rows in one bucket only
        }
        if (!m_search.has_bucket(bucket_row)) {
            continue;
        }

        const Bucket &bucket = m_search.bucket(bucket_row);
        const int item_row = persistent_index.row();
        if (!bucket.has_item(item_row)) {
            continue;
        }

        snapshots.push_back(
            {persistent_index, bucket_row, persistent_index.column(), bucket.item(item_row)});
    }

    if (only_bucket >= 0) {
        m_search.SortBucket(only_bucket, column, order);
    } else {
        m_search.Sort(column, order);
    }

    QModelIndexList from;
    QModelIndexList to;
    from.reserve(snapshots.size());
    to.reserve(snapshots.size());
    for (const ItemIndexSnapshot &snapshot : snapshots) {
        if (!m_search.has_bucket(snapshot.bucket_row)) {
            continue;
        }

        const Bucket &bucket = m_search.bucket(snapshot.bucket_row);
        const auto &items = bucket.items();
        const auto item = std::find(items.begin(), items.end(), snapshot.item);
        if (item == items.end()) {
            continue;
        }

        const int item_row = static_cast<int>(std::distance(items.begin(), item));
        const QModelIndex parent_index = index(snapshot.bucket_row, 0);
        from.push_back(snapshot.from);
        to.push_back(index(item_row, snapshot.column, parent_index));
    }
    changePersistentIndexList(from, to);
    emit layoutChanged(parents, QAbstractItemModel::VerticalSortHint);
    if (only_bucket < 0) {
        // A scoped expand-sort says nothing about the materialized set as
        // a whole; only the view-wide pass marks the model sorted.
        SetSorted(true);
    }
}

void ItemsModel::RepaintBuyoutCells(const BuyoutChangeSet &changes)
{
    // Batching rule 5 (M3 R1-6): Price/Date cells render buyout state
    // unconditionally, so the affected visible rows repaint on any buyout
    // batch regardless of which column sorts the view. The buyout-
    // dependent columns are a contiguous span (Price, Date), so one
    // dataChanged rectangle per affected bucket covers them.
    const auto &columns = m_search.columns();
    int first_column = -1;
    int last_column = -1;
    for (int n = 0; n < static_cast<int>(columns.size()); ++n) {
        if (columns[n]->buyoutDependent()) {
            if (first_column < 0) {
                first_column = n;
            }
            last_column = n;
        }
    }
    if (first_column < 0) {
        return;
    }

    // Group the affected item ids by their stable bucket key through the
    // visible-id index so only affected buckets are scanned: the scoped
    // pricing pass rides the delta path, which stays O(delta + affected
    // bucket) (M2 hard constraint). The flat By-Item bucket holds the
    // whole visible result and is scanned once — the same O(n) shape as
    // D4's stated merge exception. An affected id the index cannot fully
    // represent — duplicated across buckets mid-refresh, or the empty id
    // shared by id-less items — forces the same every-bucket scan, so
    // every visible occurrence repaints; unique indexed ids (the steady
    // state) keep the bucket-scoped fast path.
    const bool by_item = (m_search.GetViewMode() == Search::ViewMode::ByItem);
    bool scan_every_bucket = by_item || changes.everything;
    std::map<LocationInventory::Key, std::set<QString>> ids_by_bucket;
    if (!scan_every_bucket) {
        for (const QString &id : changes.item_ids) {
            if (m_search.visibleIdUnindexed(id)) {
                scan_every_bucket = true;
                ids_by_bucket.clear();
                break;
            }
            if (const auto item = m_search.visibleItemById(id)) {
                ids_by_bucket[LocationInventory::KeyFor(item->location())].insert(id);
            }
        }
    }

    const auto &buckets = m_search.buckets();
    const int bucket_count = static_cast<int>(buckets.size());
    for (int bucket_row = 0; bucket_row < bucket_count; ++bucket_row) {
        const Bucket &bucket = buckets[bucket_row];
        const auto &items = bucket.items();
        const int item_count = static_cast<int>(items.size());
        const QModelIndex header = index(bucket_row, 0);

        // Whole-bucket repaint is reserved for the shapes where it is the
        // exact affected set: `everything`, and a tab-level change in
        // By-Tab, where the bucket IS the tab — its header renders the
        // tab buyout and every row can inherit it. The flat By-Item
        // bucket aggregates every tab, so a tab-level change resolves to
        // the affected tab's rows below (S5 review round 1 — never the
        // whole flat bucket), and its header renders no buyout.
        const bool whole_bucket = changes.everything
                                  || (!by_item
                                      && (changes.tab_ids.count(bucket.location().id()) > 0));
        if (whole_bucket) {
            if (!by_item) {
                emit dataChanged(header, header);
            }
            if (item_count > 0) {
                emit dataChanged(index(0, first_column, header),
                                 index(item_count - 1, last_column, header));
            }
            continue;
        }

        // Row-level repaint: affected rows are emitted as MAXIMAL
        // contiguous runs — O(affected runs) rectangles, never one
        // first-to-last span (S5 review round 1: with scattered affected
        // ids in the flat bucket, a spanning rectangle cost O(collection)
        // view-side work per priced delta — ~43 s at 1m).
        const std::set<QString> *bucket_ids = nullptr;
        if (!scan_every_bucket) {
            const auto it = ids_by_bucket.find(LocationInventory::KeyFor(bucket.location()));
            if (it == ids_by_bucket.end()) {
                continue; // fast path: no affected rows in this bucket
            }
            bucket_ids = &it->second;
        }
        const auto affected = [&](int n) {
            const Item &item = *items[n];
            if (bucket_ids) {
                return bucket_ids->count(item.id()) > 0;
            }
            if (changes.item_ids.count(item.id()) > 0) {
                return true;
            }
            return by_item && (changes.tab_ids.count(item.location().id()) > 0);
        };
        int run_first = -1;
        for (int n = 0; n <= item_count; ++n) {
            const bool hit = (n < item_count) && affected(n);
            if (hit && (run_first < 0)) {
                run_first = n;
            } else if (!hit && (run_first >= 0)) {
                emit dataChanged(index(run_first, first_column, header),
                                 index(n - 1, last_column, header));
                run_first = -1;
            }
        }
    }
}

QModelIndex ItemsModel::parent(const QModelIndex &index) const
{
    // bucket
    if (!index.isValid() || index.model() != this || index.internalId() == 0) {
        return QModelIndex();
    }
    // item: the internalId is the bucket's stable serial (M3 S4), so the
    // parent mapping survives top-level insert/remove/move operations.
    const int bucket_row = m_search.rowForSerial(index.internalId());
    if (bucket_row < 0) {
        return QModelIndex();
    }
    return createIndex(bucket_row, 0, static_cast<quintptr>(0));
}

QModelIndex ItemsModel::index(int row, int column, const QModelIndex &parent) const
{
    if ((row < 0) || (column < 0) || (parent.isValid() && parent.model() != this)
        || !hasIndex(row, column, parent)) {
        return QModelIndex();
    }

    if (parent.isValid()) {
        // item: carry the bucket's stable serial (nonzero by construction)
        // so a child index never encodes a display position (M3 S4).
        if (!m_search.has_bucket(parent.row())) {
            return QModelIndex();
        }
        return createIndex(row,
                           column,
                           static_cast<quintptr>(m_search.bucket(parent.row()).serial()));
    } else {
        return createIndex(row, column, static_cast<quintptr>(0));
    }
}

void ItemsModel::BeginRemoveItemRows(int bucket_row, int first, int last)
{
    beginRemoveRows(index(bucket_row, 0), first, last);
}

void ItemsModel::BeginInsertItemRows(int bucket_row, int first, int last)
{
    beginInsertRows(index(bucket_row, 0), first, last);
}

void ItemsModel::BeginInsertBucketRow(int row)
{
    beginInsertRows(QModelIndex(), row, row);
}

void ItemsModel::BeginRemoveBucketRow(int row)
{
    beginRemoveRows(QModelIndex(), row, row);
}

bool ItemsModel::BeginMoveBucketRow(int from, int to)
{
    // beginMoveRows takes the destination in pre-move coordinates: moving
    // down, the row lands before to+1.
    const int destination = (to > from) ? (to + 1) : to;
    return beginMoveRows(QModelIndex(), from, from, QModelIndex(), destination);
}

void ItemsModel::EmitBucketMetadataChanged(int row)
{
    const QModelIndex header = index(row, 0);
    emit dataChanged(header, header);
}
