/*
    Copyright (C) 2024-2025 Gerwaric

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

#include <json_struct/json_struct_qt.h>

#include <libpoe/type/item.h>

#include <optional>
#include <vector>

namespace libpoe{

    // https://www.pathofexile.com/developer/docs/reference#type-StashTab

    struct StashTab {

        struct Metadata {
            std::optional<bool> public_; // ? bool always true if present
            std::optional<bool> folder; // ? bool always true if present
            std::optional<QString> colour; // ? string 6 digit hex colour (NOTE: might be only 2 or 4 characters).
            JS_OBJECT(
                JS_MEMBER_WITH_NAME(public_, "public"),
                JS_MEMBER(folder),
                JS_MEMBER(colour));
        };

        QString id; // string a 10 digit hexadecimal string
        std::optional<QString> parent; // ? string a 10 digit hexadecimal string
        QString name; // string
        QString type; // string
        std::optional<unsigned> index; // ? uint
        libpoe::StashTab::Metadata metadata; // metadata object
        std::optional<std::vector<libpoe::StashTab>> children; // ? array of StashTab
        std::optional<std::vector<libpoe::Item>> items; // ? array of Item
        JS_OBJ(id, parent, name, type, index, metadata, children, items);

        inline bool operator<(const StashTab& other) const {
            const unsigned a = index.value_or(0);
            const unsigned b = other.index.value_or(0);
            return a < b;
        };
    };

    using StashTabList = std::vector<std::unique_ptr<libpoe::StashTab>>;

    struct StashListWrapper {
        StashTabList stashes;
        JS_OBJ(stashes);
    };

    struct StashWrapper {
        std::optional<libpoe::StashTab> stash;
        JS_OBJ(stash);
    };


};
