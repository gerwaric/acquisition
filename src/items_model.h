// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QAbstractItemModel>

class BuyoutManager;
class Search;

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
    void sort();
    Qt::SortOrder GetSortOrder() const { return m_sort_order; }
    int GetSortColumn() const { return m_sort_column; }
    void SetSorted(bool val) { m_sorted = val; }

private:
    BuyoutManager &m_bo_manager;
    Search &m_search;
    Qt::SortOrder m_sort_order;
    int m_sort_column;
    bool m_sorted;
};
