// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "app/usersettings.h"

#include <QApplication>
#include <QDir>
#include <QSettings>

#include "util/spdlog_qt.h" // IWYU pragma: keep

using namespace Qt::StringLiterals;

// clang-format off
app::UserSettings::UserSettings(const QDir &dir)
    : m_settings{dir.filePath("settings.ini"), QSettings::IniFormat}
    , username                  {m_settings, "session/username"_L1}
    , realm                     {m_settings, "session/realm"_L1}
    , league                    {m_settings, "session/league"_L1}
    , showStartupOptions        {m_settings, "startup/show_options"_L1}
    , rememberUser              {m_settings, "startup/remember_user"_L1}
    , useSystemProxy            {m_settings, "statup/use_system_proxy"_L1}
    , logLevel                  {m_settings, "app/log_level"_L1}
    , theme                     {m_settings, "app/theme"_L1}
    , lastSkippedRelease        {m_settings, "app/last_skipped_release"_L1}
    , lastSkippedPreRelease     {m_settings, "app/last_skipped_prerelease"_L1}
    , m_user_dir(dir)
// clang-format on
{}

app::UserSettings::~UserSettings() {}

void ::app::UserSettings::clear()
{
    m_settings.clear();
}
