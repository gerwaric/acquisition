// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QString>

struct LegacyBuyout
{
    double value{0.0};
    long long last_update{0};
    QString type;
    QString currency;
    QString source;
    bool inherited{false};
};
