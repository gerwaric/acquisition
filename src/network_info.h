#pragma once

#include "version_defines.h"

#define USER_AGENT  (APP_NAME "/" APP_VERSION_STRING "(contact: " APP_PUBLISHER_EMAIL ")")

constexpr const char* POE_COOKIE_NAME = "POESESSID";
constexpr const char* POE_COOKIE_DOMAIN = ".pathofexile.com";
constexpr const char* POE_COOKIE_PATH = "/";
