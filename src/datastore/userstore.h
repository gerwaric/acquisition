// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

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

    void saveCharacter(const poe::Character &character);
    void saveCharacterList(const std::vector<poe::Character> &characters, const QString &realm);

    std::optional<poe::Character> getCharacter(const QString &name, const QString &realm);
    std::vector<poe::Character> getCharacterList(const QString &realm);

    void saveStash(const poe::StashTab &stash, const QString &realm, const QString &league);
    void saveStashList(const std::vector<poe::StashTab> &stashes,
                       const QString &realm,
                       const QString &league);

    std::optional<poe::StashTab> getStash(const QString &id,
                                          const QString &realm,
                                          const QString &league);
    std::vector<poe::StashTab> getStashList(const QString &realm, const QString &league);

private:
    static void logError(const QSqlQuery &q);

    int userVersion();
    bool setUserVersion(int v);
    void migrate();
    void attemptRollback();

    template<typename T>
    inline QVariant optionalValue(std::optional<T> wrapper)
    {
        return wrapper ? *wrapper : QVariant{QMetaType::fromType<T>()};
    };

    QSqlDatabase m_db;
};
