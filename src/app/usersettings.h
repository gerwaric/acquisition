// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QDir>
#include <QObject>
#include <QSettings>
#include <QString>

#include <spdlog/spdlog.h>

#include <semver/semver.hpp>

#include "util/codecs.h"

namespace app {

    class UserSettings : public QObject
    {
        Q_OBJECT
    public:
        explicit UserSettings(const QDir &dir);
        ~UserSettings();

        QDir userDir() const { return m_user_dir; };
        QString sessionKey() { return username() + "/" + realm() + "/" + league(); }
        void clear();

    private:
        // Because c++ initializes members in the order they are declared, we need
        // to declare m_settings here so it can be used in the constructor to initialize
        // the individual app::Setting objects declared below.
        QSettings m_settings;

        template<typename T>
        class Setting
        {
        public:
            Setting(QSettings &s, QLatin1StringView key)
                : m_settings(s)
                , m_key(key)
            {}

            // Get the value.
            T operator()() const { return VariantCodec<T>::decode(m_settings.value(m_key)); }

            // Set the value.
            void operator()(const T &value)
            {
                m_settings.setValue(m_key, VariantCodec<T>::encode(value));
            }

            // Clear the value.
            void clear() { m_settings.remove(m_key); }

        private:
            QSettings &m_settings;
            QLatin1StringView m_key;
        };

    public:
        // Session settings.
        UserSettings::Setting<QString> username;
        UserSettings::Setting<QString> realm;
        UserSettings::Setting<QString> league;

        // Startup settings.
        UserSettings::Setting<bool> showStartupOptions;
        UserSettings::Setting<bool> rememberUser;
        UserSettings::Setting<bool> useSystemProxy;
        UserSettings::Setting<spdlog::level::level_enum> logLevel;
        UserSettings::Setting<QString> theme;

        // Update settings.
        UserSettings::Setting<semver::version> lastSkippedRelease;
        UserSettings::Setting<semver::version> lastSkippedPreRelease;

    private:
        QDir m_user_dir;
    };

} // namespace app
