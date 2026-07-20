#pragma once

#include <functional>
#include <memory>
#include <stop_token>
#include <variant>
#include <vector>

#include <QFuture>
#include <QPromise>
#include <QString>

#include "poe/poeapiclient.h"
#include "ratelimit/fetcherror.h"
// Only for the unused base-constructor argument the fake is handed; nothing
// here calls into the limiter.
#include "ratelimit/ratelimiter.h"

// A controllable stand-in for the typed API facade (testing-plan item 5).
//
// Worker tests say "when asked for stash X, return this tab / this error"
// instead of crafting response bytes: every call records the domain arguments
// the worker asked with — realm, league, stash id, substash id, character
// name — and returns a future that stays pending until the test settles it.
// That keeps delivery order under the test's control, which is what the
// worker's sequencing pins depend on, while removing JSON and URLs from the
// worker suite entirely. Request building and endpoint labels are pinned
// against the real facade in tst_poeapiclient.
//
// Deliveries are settled by call index, so a test that fetches the same stash
// id in two successive updates simply settles the two calls differently — no
// per-id scripting needed.
//
// Each call is settled exactly once: not because a QFuture forbids more (it is
// a multi-result container — addResult() may be called repeatedly), but because
// settling a call clears its slot and a second attempt qFatals. This mirrors
// the pump's own one-shot completion. The legacy signal boundary this replaced
// could re-emit the same reply; nothing here reproduces that, deliberately.
class FakePoeApiClient : public PoeApiClient
{
public:
    // The payload the base's Result<T> future carries. Named differently on
    // purpose: PoeApiClient::Result<T> is the future, Value<T> is what it
    // resolves to.
    template<typename T>
    using Value = std::expected<T, RateLimit::FetchError>;

    // The limiter is never touched: every method below is overridden. It
    // exists only to satisfy the base constructor.
    explicit FakePoeApiClient(RateLimiter &unused)
        : PoeApiClient(unused)
    {}

    struct Call
    {
        enum class Kind { ListStashes, GetStash, ListCharacters, GetCharacter, LegacyStashIndex };

        Kind kind;
        QString account;
        QString realm;
        QString league;
        QString stash_id;
        QString substash_id;
        QString name;
        // Recorded for phase 5, when the worker starts passing real tokens.
        // Every phase-4b call site passes a default token (D7), so there is
        // nothing here worth asserting yet.
        std::stop_token token;
    };

    // --- the facade ------------------------------------------------------

    PoeApiClient::Result<poe::StashListWrapper> listStashes(const QString &realm,
                                                            const QString &league,
                                                            std::stop_token token = {}) override
    {
        Call call{.kind = Call::Kind::ListStashes, .realm = realm, .league = league, .token = token};
        return record<poe::StashListWrapper>(std::move(call));
    }

    PoeApiClient::Result<poe::StashWrapper> getStash(const QString &realm,
                                                     const QString &league,
                                                     const QString &stash_id,
                                                     const QString &substash_id = {},
                                                     std::stop_token token = {}) override
    {
        Call call{.kind = Call::Kind::GetStash,
                  .realm = realm,
                  .league = league,
                  .stash_id = stash_id,
                  .substash_id = substash_id,
                  .token = token};
        return record<poe::StashWrapper>(std::move(call));
    }

    PoeApiClient::Result<poe::CharacterListWrapper> listCharacters(
        const QString &realm, std::stop_token token = {}) override
    {
        Call call{.kind = Call::Kind::ListCharacters, .realm = realm, .token = token};
        return record<poe::CharacterListWrapper>(std::move(call));
    }

    PoeApiClient::Result<poe::CharacterWrapper> getCharacter(const QString &realm,
                                                             const QString &name,
                                                             std::stop_token token = {}) override
    {
        Call call{.kind = Call::Kind::GetCharacter, .realm = realm, .name = name, .token = token};
        return record<poe::CharacterWrapper>(std::move(call));
    }

    PoeApiClient::Result<poe::WebStashListWrapper> getLegacyStashIndex(
        const QString &account,
        const QString &realm,
        const QString &league,
        std::stop_token token = {}) override
    {
        Call call{.kind = Call::Kind::LegacyStashIndex,
                  .account = account,
                  .realm = realm,
                  .league = league,
                  .token = token};
        return record<poe::WebStashListWrapper>(std::move(call));
    }

    // --- inspection ------------------------------------------------------

    size_t callCount() const { return m_pending.size(); }
    const Call &call(size_t i) const { return m_pending.at(i).call; }

    // --- delivery --------------------------------------------------------

    // Settle call i with a value. T must match the call's payload type; a
    // mismatch is a test bug and aborts loudly rather than silently doing
    // nothing.
    template<typename T>
    void resolve(size_t i, T value)
    {
        settle<T>(i, Value<T>(std::move(value)));
    }

    // Convenience for the two wrappers the worker fetches one of.
    void resolveStash(size_t i, poe::StashTab stash)
    {
        resolve(i, poe::StashWrapper{.stash = std::move(stash)});
    }

    void resolveStashList(size_t i, std::vector<poe::StashTab> stashes)
    {
        resolve(i, poe::StashListWrapper{.stashes = std::move(stashes)});
    }

    void resolveCharacter(size_t i, poe::Character character)
    {
        resolve(i, poe::CharacterWrapper{.character = std::move(character)});
    }

    void resolveCharacterList(size_t i, std::vector<poe::Character> characters)
    {
        resolve(i, poe::CharacterListWrapper{.characters = std::move(characters)});
    }

    // Settle call i with a failure of the given kind. Every flavour the
    // network can produce reaches the worker through this one shape.
    void reject(size_t i, RateLimit::FetchError::Kind kind, const QString &message = {})
    {
        Slot &slot = m_pending.at(i);
        if (!slot.reject) {
            qFatal("FakePoeApiClient::reject: call %zu was already settled", i);
        }
        RateLimit::FetchError error;
        error.kind = kind;
        error.message = message;
        auto reject_fn = std::move(slot.reject);
        slot.resolve_type = nullptr;
        slot.reject = nullptr;
        reject_fn(std::move(error));
        // See settle(): drop the settled promise so its payload is not
        // retained past completion.
        slot.promise.reset();
    }

private:
    // Type-erased settle hooks, so calls of different payload types can live
    // in one indexed list.
    struct Slot
    {
        Call call;
        // Identifies the payload type, so resolve<T> can reject a mismatch.
        const void *resolve_type{nullptr};
        std::function<void(RateLimit::FetchError)> reject;
        // Holds the typed promise; erased behind the two hooks above.
        std::shared_ptr<void> promise;
    };

    template<typename T>
    static const void *typeTag()
    {
        static const char tag{};
        return &tag;
    }

    template<typename T>
    PoeApiClient::Result<T> record(Call call)
    {
        auto promise = std::make_shared<QPromise<Value<T>>>();
        promise->start();
        QFuture<Value<T>> future = promise->future();

        Slot slot;
        slot.call = std::move(call);
        slot.resolve_type = typeTag<T>();
        slot.promise = promise;
        slot.reject = [promise](RateLimit::FetchError error) {
            promise->addResult(Value<T>(std::unexpected(std::move(error))));
            promise->finish();
        };
        m_pending.push_back(std::move(slot));
        return future;
    }

    template<typename T>
    void settle(size_t i, Value<T> result)
    {
        Slot &slot = m_pending.at(i);
        if (!slot.reject) {
            qFatal("FakePoeApiClient: call %zu was already settled", i);
        }
        if (slot.resolve_type != typeTag<T>()) {
            qFatal("FakePoeApiClient: call %zu was settled with the wrong payload type", i);
        }
        auto promise = std::static_pointer_cast<QPromise<Value<T>>>(slot.promise);
        slot.resolve_type = nullptr;
        slot.reject = nullptr;
        promise->addResult(std::move(result));
        promise->finish();
        // Drop the settled promise: the returned QFuture keeps the result
        // alive for as long as a consumer holds it, so nothing here needs to.
        // Holding it would retain every settled payload (whole stash bodies)
        // until fixture teardown and skew phase-5 batch/memory tests.
        slot.promise.reset();
    }

    std::vector<Slot> m_pending;
};
