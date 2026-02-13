// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "datastore/datastore_utils.h"

#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>

#include "util/spdlog_qt.h" // IWYU pragma: keep

QDateTime ds::timestamp()
{
    // QDateTime::toString does not output timezone:
    // https://qt-project.atlassian.net/browse/QTBUG-26161
    //
    // To get around that bug, we feed the UTC offset back into the datetime.
    QDateTime now = QDateTime::currentDateTime();
    now = now.toOffsetFromUtc(now.offsetFromUtc());
    return now;
}

void ds::logQueryError(const char *context, const QSqlQuery &q)
{
    const auto err = q.lastError();

    const std::vector<std::pair<QString, QString>> errors = {
        {"error", err.text()},
        {"driverError", err.driverText()},
        {"databaseError", err.databaseText()},
        {"nativeErrorCode", err.nativeErrorCode()},
    };
    QStringList error_msgs;
    for (const auto &[name, text] : errors) {
        if (!text.isEmpty()) {
            error_msgs.push_back(QString("%1='%2'").arg(name, text));
        }
    }

    // For some drivers, executedQuery() can be more informative than lastQuery()
    const QString sql = (q.executedQuery().isEmpty() ? q.lastQuery() : q.executedQuery())
                            .simplified();

    const QStringList names = q.boundValueNames();
    const QVariantList values = q.boundValues();
    QStringList binds;
    binds.reserve(std::max(names.size(), values.size()));

    if (names.isEmpty()) {
        for (int i = 0; i < values.size(); ++i) {
            binds.push_back(QString(":%1='%2'").arg(i).arg(summarizeVariant(values[i])));
        }
    } else {
        const int n = std::min(names.size(), values.size());
        for (int i = 0; i < n; ++i) {
            binds.push_back(QString("%1='%2'").arg(names[i], summarizeVariant(values[i])));
        }
        // If mismatch, include what we can without crashing or losing everything
        if (names.size() != values.size()) {
            binds.push_back(
                QString("<mismatch names=%1 values=%2>").arg(names.size()).arg(values.size()));
        }
    }

    spdlog::error("{} query failed: '{}' for query='{}' and {}",
                  context,
                  error_msgs.join(", "),
                  sql,
                  binds.join(", "));
}

QString ds::summarizeVariant(const QVariant &v)
{
    if (!v.isValid() || v.isNull()) {
        return "<NULL>";
    }

    // QByteArray / BLOB
    if (v.metaType() == QMetaType::fromType<QByteArray>()) {
        const auto bytes = v.toByteArray();
        return QString("<blob %1 bytes>").arg(bytes.size());
    }

    // Default string-ish summary with truncation
    QString s = v.toString();
    constexpr int kMaxLen = 200;
    if (s.size() > kMaxLen) {
        s.truncate(kMaxLen);
        s += "...";
    }

    // Avoid multiline spam
    s.replace('\n', "\\n");
    s.replace('\r', "\\r");
    s.replace('\t', "\\t");
    return s;
}
