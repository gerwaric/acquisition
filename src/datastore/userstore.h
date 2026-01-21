// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QDir>
#include <QObject>
#include <QSqlDatabase>
#include <QString>

class BuyoutRepo;
class CharacterRepo;
class StashRepo;

class UserStore : public QObject
{
    Q_OBJECT
public:
    explicit UserStore(const QDir &dir, const QString &username);
    ~UserStore();

    StashRepo &stashes() { return *m_stashes; };
    CharacterRepo &characters() { return *m_characters; }
    BuyoutRepo &buyouts() { return *m_buyouts; };

private:
    int userVersion();
    void migrate();

    QSqlDatabase m_db;
    std::unique_ptr<StashRepo> m_stashes;
    std::unique_ptr<CharacterRepo> m_characters;
    std::unique_ptr<BuyoutRepo> m_buyouts;
};
