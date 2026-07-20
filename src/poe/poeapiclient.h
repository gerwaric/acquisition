// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <expected>
#include <stop_token>

#include <QFuture>
#include <QString>

#include "poe/types/character.h"
#include "poe/types/stashtab.h"
#include "poe/types/website/webstashtab.h"
#include "ratelimit/fetcherror.h"

class RateLimiter;

// The typed boundary between the Path of Exile domain and the network
// (network-redesign spec, the facade section). Callers name what they want —
// a realm, a league, a stash id — and get back a future carrying either the
// parsed payload or a FetchError. Above this line nothing sees
// QNetworkRequest, QNetworkReply, or bytes; below it nothing sees items.
//
// Deliberately boring: not a QObject, no coroutines, and no state beyond a
// reference to the limiter. Account, realm, and league are call parameters,
// not remembered settings, which is what makes the class genuinely stateless
// (R7).
//
// The methods are virtual for exactly one reason: the worker suite substitutes
// a fake facade for them (testing-plan item 5), so worker tests can say "when
// asked for stash X, return this tab" instead of crafting response bytes. That
// is the whole intent of the vtable — this is not an extension point, and
// production has and should have exactly one implementation. Request building
// and endpoint labels are pinned against the real class in tst_poeapiclient.
//
// The facade owns the transfer-timeout invariant (R5-3): every request it
// builds carries the 10 s inactivity timeout the gate's liveness depends on
// (D5). That is not a convention here — a request without it can hold a gate
// slot indefinitely and stall the whole hub.
//
// Everything runs on the main thread: the parse continuations attach with no
// threading context, so they run wherever the promise is completed, which is
// the main thread by construction (D2-D6 depend on this).
class PoeApiClient
{
public:
    template<typename T>
    using Result = QFuture<std::expected<T, RateLimit::FetchError>>;

    explicit PoeApiClient(RateLimiter &rate_limiter)
        : m_rate_limiter(rate_limiter)
    {}

    virtual ~PoeApiClient() = default;

    virtual Result<poe::StashListWrapper> listStashes(const QString &realm,
                                                      const QString &league,
                                                      std::stop_token token = {});

    virtual Result<poe::StashWrapper> getStash(const QString &realm,
                                               const QString &league,
                                               const QString &stash_id,
                                               const QString &substash_id = {},
                                               std::stop_token token = {});

    virtual Result<poe::CharacterListWrapper> listCharacters(const QString &realm,
                                                             std::stop_token token = {});

    virtual Result<poe::CharacterWrapper> getCharacter(const QString &realm,
                                                       const QString &name,
                                                       std::stop_token token = {});

    // The legacy character-window stash index the forum shop uses. It is not
    // an OAuth endpoint and has no poe:: builder, so the request is built
    // here — WITH the transfer timeout, which the hand-rolled request in
    // Shop never had (F60).
    virtual Result<poe::WebStashListWrapper> getLegacyStashIndex(const QString &account,
                                                                 const QString &realm,
                                                                 const QString &league,
                                                                 std::stop_token token = {});

private:
    RateLimiter &m_rate_limiter;
};
