// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QString>

#include <optional>

#include <util/glaze_qt.h>

struct LegacyCharacter
{
    QString id;
    QString name;
    QString realm;
    QString class_;
    QString league;
    unsigned int level;
    unsigned long experience;
    std::optional<bool> current;
};

template<>
struct glz::meta<LegacyCharacter>
{
    using T = LegacyCharacter;
    static constexpr auto modify = glz::object("class", &T::class_);
};
