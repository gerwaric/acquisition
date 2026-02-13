// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QLatin1StringView>
#include <QSettings>

#include <semver/semver.hpp>
#include <spdlog/spdlog.h>

#include "util/codecs.h"

namespace app {

    template<typename T>
    class Setting
    {
    public:
        Setting(QSettings &s, QLatin1StringView key)
            : m_settings(s)
            , m_key(key)
        {}

        // Get the value.
        T operator()() const
        {
            const QVariant v = m_settings.value(m_key);
            return VariantCodec<T>::decode(v);
        }

        // Set the value.
        void operator()(const T &value)
        {
            const QVariant v = VariantCodec<T>::encode(value);
            m_settings.setValue(m_key, v);
        }

        // Clear the value.
        void clear() { m_settings.remove(m_key); }

    private:
        QSettings &m_settings;
        QLatin1StringView m_key;
    };

} // namespace app
