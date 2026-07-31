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
    void sort(int column, Qt::SortOrder order);
    // Re-sorts on the current sort indicator unconditionally (M3 S2): a
    // buyout batch changes Price/Date order without touching the
    // indicator, so sort()'s already-sorted skip must not apply.
    void Resort();
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
    // The layout-preserving sort machinery shared by sort() and Resort():
    // snapshots persistent indexes, sorts the search's buckets, remaps.
    void ApplySort(int column, Qt::SortOrder order);

    BuyoutManager &m_bo_manager;
    Search &m_search;
    Qt::SortOrder m_sort_order;
    int m_sort_column;
    bool m_sorted;
};
