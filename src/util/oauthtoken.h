/*
    Copyright (C) 2014-2025 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#pragma once

#include <QDateTime>
#include <QString>

#include <optional>

struct OAuthToken {

    QString access_token;
    int expires_in{ -1 };
    QString token_type;
    QString scope;
    QString sub;
    QString username;
    QString refresh_token;

    std::optional<QDateTime> birthday;
    std::optional<QDateTime> access_expiration;
    std::optional<QDateTime> refresh_expiration;
};
