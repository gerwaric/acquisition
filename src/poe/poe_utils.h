// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QNetworkRequest>
#include <QString>

namespace poe {

    struct StashTab;

    bool isSpecialStash(const poe::StashTab &stash);

    std::pair<const QString &, QNetworkRequest> MakeStashListRequest(const QString &realm,
                                                                     const QString &league);

    std::pair<const QString &, QNetworkRequest> MakeStashRequest(const QString &realm,
                                                                 const QString &league,
                                                                 const QString &stash_id,
                                                                 const QString &substash_id = {});

    std::pair<const QString &, QNetworkRequest> MakeCharacterListRequest(const QString &realm);

    std::pair<const QString &, QNetworkRequest> MakeCharacterRequest(const QString &realm,
                                                                     const QString &name);

} // namespace poe
