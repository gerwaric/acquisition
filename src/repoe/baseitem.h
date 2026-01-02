// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QString>

namespace repoe {

    // The full schema is here https://repoe-fork.github.io/data-formats/base_items.json
    //
    // Only the parts used by acquisition are defined here.
    struct BaseItem
    {
        QString item_class;
        QString name;
        QString release_state;
    };

} // namespace repoe
