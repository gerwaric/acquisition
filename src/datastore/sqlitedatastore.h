// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QMutex>
#include <QSet>
#include <QSqlDatabase>

#include "datastore.h"

class Application;
struct CurrencyUpdate;

class SqliteDataStore : public DataStore
{
public:
    SqliteDataStore(const QString &m_filename);
    ~SqliteDataStore();
    void Set(const QString &key, const QString &value);
    void SetTabs(const ItemLocationType type, const Locations &value);
    void SetItems(const ItemLocation &loc, const Items &value);
    QString Get(const QString &key, const QString &default_value = "");
    Locations GetTabs(const ItemLocationType type);
    Items GetItems(const ItemLocation &loc);
    void InsertCurrencyUpdate(const CurrencyUpdate &update);
    std::vector<CurrencyUpdate> GetAllCurrency();
    static QString MakeFilename(const QString &name, const QString &league);

private:
    void CreateTable(const QString &username, const QString &fields);
    void CleanItemsTable();

    QString m_filename;

    QString getThreadLocalConnectionName() const;
    QSqlDatabase getThreadLocalDatabase();
    mutable QMutex m_mutex;
    mutable QSet<QString> m_connection_names;
};
