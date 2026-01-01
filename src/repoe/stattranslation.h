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

#include <QString>

#include <optional>

namespace repoe {

    // This is bare minimun need to parse RePoE stat translations for the mods list generator.
    struct EnglishTranslation
    {
        std::vector<QString> format;
        QString string;

        // This was added with the change to process json files inside
        // the stat_translations directory. In this case, the necropolis
        // mods from 3.24 have some kind of duplicate formatting with
        // markup that acquisition has not had to deal with before.
        //
        // It's possible this is true for other files in the stat_translations
        // folder, but acquisition has never needed to load modifiers from those
        // files before.
        std::optional<bool> is_markup;
    };

    struct StatTranslation
    {
        std::vector<EnglishTranslation> English;
    };

} // namespace repoe
