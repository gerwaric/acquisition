// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <optional>
#include <vector>

#include <QString>

#include "poe/types/gempage.h"

namespace poe {

    // https://www.pathofexile.com/developer/docs/reference#type-GemTab

    struct GemTab
    {
        std::optional<QString> name;     // ?string
        std::vector<poe::GemPage> pages; // array of GemPage
    };

} // namespace poe
