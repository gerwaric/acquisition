// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QObject>
#include <QSettings>
#include <QString>

#include <map>
#include <set>

#include "legacydatastore.h"

class LegacyBuyoutValidator : QObject
{
    Q_OBJECT
public:
    enum struct ValidationResult { Uninitialized, Valid, Invalid, Error };

    static const QString SettingsKey;

    LegacyBuyoutValidator(QSettings &settings, const QString &dataDir);
    ValidationResult validate();
    void notifyUser();

private:
    void validateTabBuyouts();
    void validateItemBuyouts();

    QSettings &m_settings;

    const QString m_filename;
    const LegacyDataStore m_datastore;
    ValidationResult m_status{ValidationResult::Uninitialized};

    std::map<QString, std::set<QString>> m_issues;
};
