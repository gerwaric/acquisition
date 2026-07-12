// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <functional>
#include <memory>
#include <vector>

#include <QMetaObject>

#include "ui/searchform.h"

class QGridLayout;
class QLayout;
class QPushButton;
class QWidget;

class ModsFilterForm final : public FilterFormAdapter
{
public:
    using StateChangedCallback = std::function<void()>;

    ModsFilterForm(QLayout *parent,
                   const FilterCallbacks &callbacks,
                   RefreshMode mode,
                   StateChangedCallback stateChanged);
    ~ModsFilterForm() override;

    void saveTo(FilterState &state) const override;
    void loadFrom(const FilterState &state) override;
    void reset() override;

private:
    class Row;

    void addRow(const ModRow &state);
    void addNewRow();
    void deleteRow(Row &row);
    void clearRows();
    void repackRows();
    void onRowsChanged();
    void updateRowContainerVisibility();

    const FilterCallbacks m_callbacks;
    const RefreshMode m_mode;
    const StateChangedCallback m_stateChanged;
    QGridLayout *m_rowsLayout = nullptr;
    QWidget *m_rowsContainer = nullptr;
    QPushButton *m_addButton = nullptr;
    QMetaObject::Connection m_addButtonConnection;
    std::vector<std::unique_ptr<Row>> m_rows;
    bool m_loading = false;
};
