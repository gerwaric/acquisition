// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QByteArray>

#include <vector>

namespace poe {
    struct Character;
    struct StashTab;
} // namespace poe

// Writers and readers are broken up to avoid an issue on Windows where
// too many templated functions in one file causes a build error and would
// require using /bigobj.

QByteArray writeCharacter(const poe::Character &character);
QByteArray writeCharacterList(const std::vector<poe::Character> &json);

QByteArray writeStash(const poe::StashTab &stash);
QByteArray writeStashList(const std::vector<poe::StashTab> &json);
