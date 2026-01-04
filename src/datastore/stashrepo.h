// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QObject>

#include <optional>
#include <vector>

#include "poe/types/stashtab.h"

class QSqlDatabase;
class QString;

class UserStore;

class StashRepo : public QObject
{
    Q_OBJECT
public:
    explicit StashRepo(QSqlDatabase &db, UserStore &store)
        : m_db(db)
        , m_store(store) {};

    std::optional<poe::StashTab> getStash(const QString &id,
                                          const QString &realm,
                                          const QString &league);
    std::vector<poe::StashTab> getStashList(const QString &realm,
                                            const QString &league,
                                            const std::optional<QString> type = {});

    std::vector<poe::StashTab> getStashChildren(const QString &id,
                                                const QString &realm,
                                                const QString &league);

    bool ensureSchema();
    bool reset();

public slots:
    bool saveStash(const poe::StashTab &stash, const QString &realm, const QString &league);
    bool saveStashList(const std::vector<poe::StashTab> &stashes,
                       const QString &realm,
                       const QString &league);

private:
    bool saveListTransaction(const std::vector<poe::StashTab> &stashes,
                             const QString &realm,
                             const QString &league);

    QSqlDatabase &m_db;
    UserStore &m_store;
};
