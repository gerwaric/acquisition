// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "util/json_writers.h"

#include "poe/types/character.h"
#include "poe/types/stashtab.h"
#include "util/json_utils.h"

QByteArray json::writeCharacter(const poe::Character &character)
{
    return write_json(character);
}

QByteArray json::writeCharacterList(const std::vector<poe::Character> &characters)
{
    return write_json(characters);
}

QByteArray json::writeStash(const poe::StashTab &stash)
{
    return write_json(stash);
}

QByteArray json::writeStashList(const std::vector<poe::StashTab> &stashes)
{
    return write_json(stashes);
}
