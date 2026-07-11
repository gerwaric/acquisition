// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "filters/filterstate.h"

#include "filters/filterspec.h"

#include <type_traits>

bool IsActive(const FilterState &state)
{
    return std::visit([](const auto &value) { return value.isActive(); }, state);
}

FilterState MakeDefaultState(const FilterSpec &spec)
{
    return std::visit(
        [](const auto &payload) -> FilterState {
            using Payload = std::decay_t<decltype(payload)>;
            if constexpr (std::is_same_v<Payload, LegacyPayload>) {
                return LegacyState{};
            } else if constexpr (std::is_same_v<Payload, TextPayload>) {
                return TextState{};
            } else if constexpr (std::is_same_v<Payload, ComboPayload>) {
                return ComboState{};
            } else if constexpr (std::is_same_v<Payload, MinMaxPayload>) {
                return MinMaxState{};
            } else if constexpr (std::is_same_v<Payload, BoolPayload>) {
                return BoolState{};
            } else if constexpr (std::is_same_v<Payload, ColorsPayload>) {
                return ColorsState{};
            } else {
                return ModsState{};
            }
        },
        spec.payload);
}
