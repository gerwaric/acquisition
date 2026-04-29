// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

#pragma once

#include <QString>

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-Guild

    struct Guild
    {
        unsigned id;  // uint
        QString name; // string
        QString tag;  // string
    };

} // namespace poe
