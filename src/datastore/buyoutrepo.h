// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QObject>

class QSqlDatabase;

class BuyoutRepo : public QObject {
    Q_OBJECT
public:
    explicit BuyoutRepo(QSqlDatabase &db)
        : m_db(db) {};

    bool reset();
    bool ensureSchema();

private:
    QSqlDatabase &m_db;
};
