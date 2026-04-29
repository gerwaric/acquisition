// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QString>

#include <optional>

struct LegacyItemLocation
{
    int _type;
    std::optional<int> _tab;
    std::optional<QString> _tab_label;
    std::optional<QString> _character;
    std::optional<int> _x;
    std::optional<int> _y;
    bool _socketed;
    bool _removeonly;
};
