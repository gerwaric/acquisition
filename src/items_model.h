// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QAbstractItemModel>

#include "modelprobes.h"

class BuyoutManager;
class Search;
struct BuyoutChangeSet;

class ItemsModel : public QAbstractItemModel
{
    Q_OBJECT
public:
    explicit ItemsModel(BuyoutManager &bo_manager, Search &search);
    int rowCount(const QModelIndex &parent = QModelIndex()) const;
    int columnCount(const QModelIndex &parent = QModelIndex()) const;
    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const;
    QModelIndex parent(const QModelIndex &index) const;
    QModelIndex index(int row, int column = 0, const QModelIndex &parent = QModelIndex()) const;
    QVariant headerData(int section, Qt::Orientation orientation, int role) const;
    Qt::ItemFlags flags(const QModelIndex &index) const;
    bool setData(const QModelIndex &index, const QVariant &value, int role = Qt::EditRole);
    // The view-wide sort entry (header clicks, the post-refilter
    // indicator pass). This is also where D1's invalidation rules 2-3
    // act: switching the column evicts every resident key vector, and any
    // (column, order) change clears every bucket's sorted flag — then the
    // materialized buckets re-establish order (Search::Sort) and collapsed
    // ones defer to expansion.
    void sort(int column, Qt::SortOrder order);
    // Re-sorts on the current sort indicator unconditionally (M3 S2): a
    // buyout batch changes Price/Date order without touching the
    // indicator, so sort()'s already-sorted skip must not apply. The
    // caller (OnBuyoutsChanged) invalidates the affected buckets first
    // (R3-2), so the per-bucket flags make this exact.
    void Resort();
    // The single-bucket form (S3 review round 1): a batch whose affected
    // materialized set is one bucket scopes the whole layout operation —
    // signals, persistent-index walk, sort — to that bucket instead of
    // paying the view-wide dance.
    void ResortBucket(int row);
    // The view's expand/collapse signals, per top-level row (D2 rules
    // 2-3): expanding marks the bucket materialized and sorts it under
    // the layout-change protocol iff its flag is invalid; collapsing
    // unmarks it and evicts its keys (By-Item's flat bucket ignores
    // collapse — it is materialized by definition). Both are idempotent.
    void OnBucketExpanded(int row);
    void OnBucketCollapsed(int row);
    // Batching rule 5 (M3 R1-6): repaint the affected visible Price/Date
    // cells — and affected bucket headers, which render tab buyouts —
    // under any active sort column.
    void RepaintBuyoutCells(const BuyoutChangeSet &changes);
    Qt::SortOrder GetSortOrder() const { return m_sort_order; }
    int GetSortColumn() const { return m_sort_column; }
    void SetSorted(bool val) { m_sorted = val; }
    void beginUpdate()
    {
        if (auto &probes = ModelProbes::instance(); probes.enabled) {
            ++probes.model_resets;
            ++probes.model_resets_by_model[this];
        }
        beginResetModel();
    }
    void endUpdate() { endResetModel(); }
    void refreshCheckStates();

private:
    // The layout-preserving sort machinery shared by sort(), Resort(),
    // and the expand path: snapshots persistent indexes, sorts the
    // search's buckets, remaps. `only_bucket` scopes the whole dance to
    // one bucket (sort-on-expand); -1 sorts the materialized set and
    // marks the model sorted.
    void ApplySort(int column, Qt::SortOrder order, int only_bucket = -1);

    BuyoutManager &m_bo_manager;
    Search &m_search;
    Qt::SortOrder m_sort_order;
    int m_sort_column;
    bool m_sorted;
};
