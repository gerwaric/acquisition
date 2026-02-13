// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "sessioncontext.h"

QString app::SessionContext::scope() const
{
    return account + "/" + realm + "/" + league;
}

bool app::SessionContext::isValid() const
{
    if (account.isEmpty()) {
        return false;
    }
    if (realm.isEmpty()) {
        return false;
    }
    if (league.isEmpty()) {
        return false;
    }
    return true;
}
