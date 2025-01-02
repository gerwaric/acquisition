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

namespace libpoe {

    // https://www.pathofexile.com/developer/docs/reference#type-DisplayMode

    enum class DisplayMode : unsigned {
        NameFirst = 0, // Name should be followed by values
        ValuesFirst = 1, // Values should be followed by name
        ProgressBar = 2, // Progress bar
        InsertedValues = 3, // Values should be inserted into the string by index
        Separator = 4 // Separator
    };

}

JS_ENUM_NAMESPACE_DECLARE_VALUE_PARSER(libpoe, DisplayMode);
