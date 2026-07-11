// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <memory>
#include <variant>
#include <vector>

#include "filters.h"
#include "filters/filterspec.h"
#include "filters/filterstate.h"

class QAbstractItemModel;
class QLayout;
class QVBoxLayout;
class Search;

class FilterFormAdapter
{
public:
    virtual ~FilterFormAdapter() = default;

    virtual void saveTo(FilterState &state) const = 0;
    virtual void loadFrom(const FilterState &state) = 0;
    virtual void reset() = 0;
};

using FormSlot = std::variant<std::unique_ptr<Filter>, std::unique_ptr<FilterFormAdapter>>;

class SearchForm
{
public:
    SearchForm(QVBoxLayout &layout, const FilterCatalog &catalog, const FilterCallbacks &callbacks);
    ~SearchForm();

    SearchForm(const SearchForm &) = delete;
    SearchForm &operator=(const SearchForm &) = delete;

    // Catalog-indexed; migrated adapter slots contain nullptr.
    const std::vector<Filter *> &legacyFilters() const { return m_legacyFilters; }
    void saveTo(Search &search);
    void loadFrom(Search &search);
    void reset();

private:
    void addSearchGroup(QLayout *layout, const QString &name = {});
    void saveBoundState(qsizetype index);

    QVBoxLayout &m_layout;
    const FilterCatalog &m_catalog;
    std::vector<std::unique_ptr<QAbstractItemModel>> m_models;
    std::vector<FormSlot> m_slots;
    std::vector<Filter *> m_legacyFilters;
    // The search currently represented by the form. Dynamic rows save into it
    // before the delayed refresh runs, so an immediate tab switch cannot drop them.
    Search *m_boundSearch = nullptr;
};
