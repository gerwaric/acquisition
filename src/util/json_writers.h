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

namespace json {

    // Wire format of the JSON these writers produce, which is what the
    // datastore stores in stashes.json_data / characters.json_data.
    //
    //   1  pre-3.29
    //   2  3.29: implicitMods/explicitMods became arrays of poe::ItemMod;
    //            craftedMods/fracturedMods/mutatedMods/desecratedMods removed
    //
    // Bump this whenever a change to the poe:: types makes an older blob
    // unreadable or, worse, readable as something it no longer means: a
    // renamed or retyped member, or a member whose meaning changed. Adding a
    // new optional field does not need a bump — old blobs simply lack it.
    //
    // Rows carry the version they were written with, and readers compare it
    // for EQUALITY, never `<`. A blob written by a newer Acquisition must be
    // refetched rather than misparsed when a user downgrades, so "newer than
    // I understand" has to fail the same way "older" does.
    //
    // This versions our serialization of the poe:: structs, not GGG's API:
    // nothing in the API responses carries a patch version, and the structs
    // can change without one (see F62 in docs/cleanup/findings.md).
    constexpr int PAYLOAD_VERSION = 2;

    QByteArray writeCharacter(const poe::Character &character);
    QByteArray writeCharacterList(const std::vector<poe::Character> &json);

    QByteArray writeStash(const poe::StashTab &stash);
    QByteArray writeStashList(const std::vector<poe::StashTab> &json);

} // namespace json
