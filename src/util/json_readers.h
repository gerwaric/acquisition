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

poe::Character readCharacter(const QByteArray &json);
poe::StashTab readStash(const QByteArray &json);

std::vector<poe::Character> readCharacterList(const QByteArray &json);
std::vector<poe::StashTab> readStashList(const QByteArray &json);
