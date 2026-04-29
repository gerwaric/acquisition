// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QString>

#include <optional>

#include "util/glaze_qt.h" // IWYU pragma: keep

struct LegacyStash
{
    struct MapData
    {
        int series;
    };

    struct Metadata
    {
        std::optional<bool> public_;
        std::optional<bool> folder;
        QString colour;
        std::optional<LegacyStash::MapData> map;
    };

    struct Colour
    {
        int r;
        int g;
        int b;
    };

    QString id;
    std::optional<QString> folder;
    QString name;
    QString type;
    int index;
    LegacyStash::Metadata metadata;
    std::optional<std::vector<LegacyStash>> children;
    std::optional<int> i;
    std::optional<QString> n;
    std::optional<LegacyStash::Colour> colour;
};

template<>
struct glz::meta<LegacyStash::Metadata>
{
    using T = LegacyStash::Metadata;
    static constexpr auto modify = glz::object("public", &T::public_);
};
