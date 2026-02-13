// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QObject>

namespace app {

    struct SessionContext
    {
        QString account;
        QString realm;
        QString league;

        QString scope() const;
        bool isValid() const;
    };

} // namespace app
