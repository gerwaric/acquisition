// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QObject>
#include <QString>

#include "util/json_utils.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

class QSqlDatabase;

class DataRepo : public QObject
{
    Q_OBJECT
public:
    explicit DataRepo(QSqlDatabase &db, QObject *parent);

    struct Key
    {
        QString name;
        QString scope{"*"};

        QString toString() const;
        bool isValid() const;
    };

    bool resetRepo();
    bool ensureSchema();

    bool contains(const DataRepo::Key &key);
    bool remove(const DataRepo::Key &key);

    bool setByteArray(const DataRepo::Key &key, const QByteArray &value);
    bool setString(const DataRepo::Key &key, const QString &value);
    bool setBool(const DataRepo::Key &key, bool value);
    bool setInt(const DataRepo::Key &key, int value);
    bool setUInt(const DataRepo::Key &key, uint value);
    bool setDouble(const DataRepo::Key &key, double value);
    bool setDateTime(const DataRepo::Key &key, const QDateTime &value);

    template<typename T>
    bool set(const DataRepo::Key &key, const T &value)
    {
        return setValue(key, write_json<T>(value));
    }

    QByteArray getByteArray(const DataRepo::Key &key, const QByteArray &default_value = {});
    QString getString(const DataRepo::Key &key, const QString &default_value = {});
    bool getBool(const DataRepo::Key &key, bool default_value = false);
    int getInt(const DataRepo::Key &key, int default_value = 0);
    uint getUInt(const DataRepo::Key &key, uint default_value = 0);
    double getDouble(const DataRepo::Key &key, double default_value = 0.0);
    QDateTime getDateTime(const DataRepo::Key &key, QDateTime default_value = QDateTime());

    template<typename T>
    void get(const DataRepo::Key &key, T &out)
    {
        QVariant value;
        if (getValue(key, value)) {
            auto result = read_json<T>(value.toByteArray());
            if (result) {
                out = *result;
            }
        }
    }

private:
    QSqlDatabase &m_db;

    bool getValue(const DataRepo::Key &key, QVariant &out);
    bool setValue(const DataRepo::Key &key, const QVariant &value);
};
