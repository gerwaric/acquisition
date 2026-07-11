// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "filters/filtermatchers.h"

#include <QtGlobal>

#include <type_traits>

bool matches(const Item &, const TextState &, const TextPayload &)
{
    return true;
}
bool matches(const Item &, const ComboState &, const ComboPayload &)
{
    return true;
}
bool matches(const Item &, const MinMaxState &, const MinMaxPayload &)
{
    return true;
}
bool matches(const Item &, const ColorsState &, const ColorsPayload &)
{
    return true;
}
bool matches(const Item &, const BoolState &, const BoolPayload &)
{
    return true;
}
bool matches(const Item &, const ModsState &, const ModsPayload &)
{
    return true;
}

bool MatchesFilter(const Item &item, const FilterSpec &spec, const FilterState &state)
{
    return std::visit(
        [&item, &state](const auto &payload) {
            using Payload = std::decay_t<decltype(payload)>;
            if constexpr (std::is_same_v<Payload, LegacyPayload>) {
                Q_ASSERT(std::holds_alternative<LegacyState>(state));
                return true;
            } else {
                using State = std::conditional_t<
                    std::is_same_v<Payload, TextPayload>,
                    TextState,
                    std::conditional_t<
                        std::is_same_v<Payload, ComboPayload>,
                        ComboState,
                        std::conditional_t<
                            std::is_same_v<Payload, MinMaxPayload>,
                            MinMaxState,
                            std::conditional_t<std::is_same_v<Payload, ColorsPayload>,
                                               ColorsState,
                                               std::conditional_t<std::is_same_v<Payload, BoolPayload>,
                                                                  BoolState,
                                                                  ModsState>>>>>;
                const auto *typedState = std::get_if<State>(&state);
                Q_ASSERT(typedState);
                return typedState ? matches(item, *typedState, payload) : false;
            }
        },
        spec.payload);
}
