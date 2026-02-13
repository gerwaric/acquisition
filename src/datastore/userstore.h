// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QDir>
#include <QObject>
#include <QSqlDatabase>
#include <QString>

class BuyoutStore;
class CharacterStore;
class StashStore;
class DataRepo;

class UserStore : public QObject
{
    Q_OBJECT
public:
    explicit UserStore(const QDir &dir, const QString &username);
    ~UserStore();

    QDir dir() const { return m_dir; }
    QString username() const { return m_username; }

    DataRepo &data() const { return *m_data; }
    StashStore &stashes() const { return *m_stashes; }
    CharacterStore &characters() const { return *m_characters; }
    BuyoutStore &buyouts() const { return *m_buyouts; }

private:
    int userVersion();
    void migrate();

    const QString m_username;
    const QDir m_dir;

    QSqlDatabase m_db;

    DataRepo *m_data;
    StashStore *m_stashes;
    CharacterStore *m_characters;
    BuyoutStore *m_buyouts;
};
