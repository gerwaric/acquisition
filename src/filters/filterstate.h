// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QString>

#include <optional>
#include <variant>
#include <vector>

struct FilterSpec;

struct TextState
{
    QString query;
    bool isActive() const { return !query.isEmpty(); }
};

struct ComboState
{
    QString value;
    bool isActive() const { return !value.isEmpty(); }
};

struct MinMaxState
{
    std::optional<double> min;
    std::optional<double> max;
    bool isActive() const { return min.has_value() || max.has_value(); }
};

struct ColorsState
{
    std::optional<int> r;
    std::optional<int> g;
    std::optional<int> b;
    bool isActive() const { return r.has_value() || g.has_value() || b.has_value(); }
};

struct BoolState
{
    bool checked = false;
    bool isActive() const { return checked; }
};

struct ModRow
{
    QString mod;
    std::optional<double> min;
    std::optional<double> max;
};

struct ModsState
{
    std::vector<ModRow> rows;
    bool isActive() const { return !rows.empty(); }
};

using FilterState
    = std::variant<TextState, ComboState, MinMaxState, ColorsState, BoolState, ModsState>;

bool IsActive(const FilterState &state);
FilterState MakeDefaultState(const FilterSpec &spec);
