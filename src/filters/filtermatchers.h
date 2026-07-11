// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include "filters/filterspec.h"
#include "filters/filterstate.h"

class Item;

bool matches(const Item &item, const TextState &state, const TextPayload &payload);
bool matches(const Item &item, const ComboState &state, const ComboPayload &payload);
bool matches(const Item &item, const MinMaxState &state, const MinMaxPayload &payload);
bool matches(const Item &item, const ColorsState &state, const ColorsPayload &payload);
bool matches(const Item &item, const BoolState &state, const BoolPayload &payload);
bool matches(const Item &item, const ModsState &state, const ModsPayload &payload);
bool MatchesFilter(const Item &item, const FilterSpec &spec, const FilterState &state);
