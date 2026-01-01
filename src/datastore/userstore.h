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

#include <QDir>
#include <QObject>
#include <QSqlDatabase>
#include <QString>
#include <QStringList>

#include "poe/types/character.h"
#include "poe/types/stashtab.h"

class UserStore : public QObject
{
    Q_OBJECT
public:
    explicit UserStore(const QDir &dir, const QString &username, QObject *parent = nullptr);
    ~UserStore();

    poe::Character getCharacter(const QString &name);
    std::vector<poe::Character> getCharacterList(const QString &realm);

    poe::StashTab getStash(const QString &id);
    std::vector<poe::StashTab> getStashList(const QString &realm, const QString &league);

    void saveCharacter(const poe::Character &character);
    void saveCharacterList(const std::vector<poe::Character> &characters, const QString &realm);

    void saveStash(const poe::StashTab &stash, const QString &realm, const QString &league);
    void saveStashList(const std::vector<poe::StashTab> &stashes,
                       const QString &realm,
                       const QString &league);

private:
    int userVersion();
    bool setUserVersion(int v);
    void migrate();

    QString m_connection;
    QString m_filename;
    QSqlDatabase m_db;
};
