/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include <QObject>
#include <QSettings>
#include <QString>

#include <map>
#include <set>

#include "legacydatastore.h"

class LegacyBuyoutValidator : QObject {
    Q_OBJECT
public:
    enum struct ValidationResult { Uninitialized, Valid, Invalid, Error };

    static const QString SettingsKey;

    LegacyBuyoutValidator(QSettings& settings, const QString& dataDir);
    ValidationResult validate();
    void notifyUser();

private:
    void validateTabBuyouts();
    void validateItemBuyouts();

    QSettings& m_settings;

    const QString m_filename;
    const LegacyDataStore m_datastore;
    ValidationResult m_status{ ValidationResult::Uninitialized };

    std::map<QString, std::set<QString>> m_issues;
};
