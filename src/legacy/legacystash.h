/*
    Copyright (C) 2014-2024 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#pragma once

#include <QString>

#include <optional>

#include <json_struct/json_struct_qt.h>

struct LegacyStash {

    struct MapData {
        int series;
        JS_OBJ(series);
    };

    struct Metadata {
        std::optional<bool> public_;
        std::optional<bool> folder;
        QString colour;
        std::optional<LegacyStash::MapData> map;
        JS_OBJECT(
            JS_MEMBER_WITH_NAME(public_, "public"),
            JS_MEMBER(folder),
            JS_MEMBER(colour),
            JS_MEMBER(map));
    };

    struct Colour {
        int r;
        int g;
        int b;
        JS_OBJ(r, g, b);
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
    JS_OBJ(id, folder, name, type, index, metadata, children, i, n, colour);

};
