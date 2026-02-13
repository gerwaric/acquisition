// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QLatin1StringView>
#include <QObject>
#include <QString>

#include "util/codecs.h"

namespace app {
    class UserSettings;
}

class SessionStore : public QObject
{
    Q_OBJECT
public:
    SessionStore(QStringView connName, app::UserSettings &settings);

    bool resetRepo();
    bool ensureSchema();

private:
    // These need to be declared here so we can use them in to declare the
    // public session settings that follow. However, they are only meant for
    // use by this class, so they are private.
    QVariant get(QLatin1StringView key) const;
    void set(QLatin1StringView key, const QVariant &value);
    void clear(QLatin1StringView key);

    template<typename T>
    class Setting
    {
    public:
        Setting(SessionStore &store, QLatin1StringView key)
            : m_store(store)
            , m_key(key)
        {}

        // Get the value.
        T operator()() const { return VariantCodec<T>::decode(m_store.get(m_key)); }

        // Set the value.
        void operator()(const T &value) { m_store.set(m_key, VariantCodec<T>::encode(value)); }

        // Clear the value.
        void clear() { m_store.clear(m_key); }

    private:
        SessionStore &m_store;
        QLatin1StringView m_key;
    };

public:
    // Refresh settings.
    SessionStore::Setting<bool> autoupdate;
    SessionStore::Setting<uint> autoupdateInterval;
    SessionStore::Setting<bool> fetchMapStashes;
    SessionStore::Setting<bool> fetchUniqueStashes;

    // Shop settings
    SessionStore::Setting<bool> shopAutoupdate;
    SessionStore::Setting<QString> shopThreads;
    SessionStore::Setting<QString> shopHash;
    SessionStore::Setting<QString> shopTemplate;

    SessionStore::Setting<QByteArray> refreshChecked;

private:
    QString sessionScope() const;

    app::UserSettings &m_settings;

    QString m_connName;
};
