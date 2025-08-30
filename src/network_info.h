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

#include "version_defines.h"

constexpr const char *USER_AGENT = APP_NAME "/" APP_VERSION_STRING " (contact: " APP_PUBLISHER_EMAIL
                                            ")";

constexpr const char *POE_COOKIE_NAME = "POESESSID";
constexpr const char *POE_COOKIE_DOMAIN = ".pathofexile.com";
constexpr const char *POE_COOKIE_PATH = "/";

enum class POE_API {
    LEGACY,
    OAUTH,
};
