// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <memory>
#include <variant>
#include <vector>

#include "filters.h"
#include "filters/filterspec.h"

class BuyoutManager;
class QAbstractItemModel;
class QLayout;
class QVBoxLayout;

using FormSlot = std::variant<std::unique_ptr<Filter>>;

class SearchForm
{
public:
    SearchForm(QVBoxLayout &layout,
               const FilterCatalog &catalog,
               BuyoutManager &buyoutManager,
               const FilterCallbacks &callbacks);
    ~SearchForm();

    SearchForm(const SearchForm &) = delete;
    SearchForm &operator=(const SearchForm &) = delete;

    const std::vector<Filter *> &legacyFilters() const { return m_legacyFilters; }

private:
    void addSearchGroup(QLayout *layout, const QString &name = {});

    QVBoxLayout &m_layout;
    const FilterCatalog &m_catalog;
    std::vector<std::unique_ptr<QAbstractItemModel>> m_models;
    std::vector<FormSlot> m_slots;
    std::vector<Filter *> m_legacyFilters;
};
