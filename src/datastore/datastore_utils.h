// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QDateTime>
#include <QMetaType>
#include <QString>
#include <QVariant>

#include <optional>

class QSqlDatabase;
class QSqlQuery;

namespace ds {

    template<typename T>
    inline QVariant optionalAsNull(std::optional<T> wrapper)
    {
        return wrapper ? *wrapper : QVariant{QMetaType::fromType<T>()};
    };

    QDateTime timestamp();

    void logQueryError(const char *context, const QSqlQuery &q);

    QString summarizeVariant(const QVariant &v);

} // namespace ds
