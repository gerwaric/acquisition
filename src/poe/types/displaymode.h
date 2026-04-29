// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

#pragma once

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-DisplayMode

    enum class DisplayMode : unsigned {
        NameFirst = 0,      // Name should be followed by values
        ValuesFirst = 1,    // Values should be followed by name
        ProgressBar = 2,    // Progress bar
        InsertedValues = 3, // Values should be inserted into the string by index
        Separator = 4       // Separator
    };

} // namespace poe
