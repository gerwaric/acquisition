// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "legacydatastore.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QProcess>
#include <QSqlError>
#include <QSqlQuery>
#include <QTemporaryDir>

namespace {

    static bool getByteArray(QSqlDatabase db, const QString &query, QByteArray &value)
    {
        QSqlQuery q(db);
        q.setForwardOnly(true);
        q.prepare(query);
        if (!q.exec()) {
            spdlog::error("Database error calling exec(): {}: {}", query, db.lastError().text());
            return false;
        }
        if (!q.next()) {
            spdlog::error("Database error calling next(): {}: {}", query, db.lastError().text());
            return false;
        }
        value = q.value(0).toByteArray();
        return true;
    }

    static bool getString(QSqlDatabase db, const QString &query, QString &value)
    {
        QSqlQuery q(db);
        q.setForwardOnly(true);
        q.prepare(query);
        if (!q.exec()) {
            spdlog::error("Database error calling exec(): {}: {}", query, db.lastError().text());
            return false;
        }
        if (!q.next()) {
            spdlog::error("Database error calling next(): {}: {}", query, db.lastError().text());
            return false;
        }
        value = q.value(0).toString();
        return true;
    }

    // Strict read: not null-terminated input; error on unknown keys.
    template <typename T>
    static bool getStruct(QSqlDatabase db, const QString& query, T& value)
    {
        QByteArray data;
        if (!getByteArray(db, query, data)) {
            return false;
        }

        // Create a view over the QByteArray (it may contain '\0', so don't assume C-strings)
        const std::string_view sv{data.constData(),
                                  static_cast<std::size_t>(data.size())};

        // Glaze options:
        // - null_terminated = false: we pass a size-aware view
        // - error on unknown keys: depending on Glaze minor version, either use
        //     `.unknown_keys = glz::unknown_keys::error`
        //   or older `.error_on_unknown_keys = true`.
        constexpr glz::opts opts{
            .null_terminated = false,
            .error_on_unknown_keys = true
        };

        if (auto ec = glz::read<opts>(value, sv); ec) {
            spdlog::error("Json error parsing {} from '{}': {}",
                          typeid(T).name(),
                          query.toStdString(),
                          glz::format_error(ec, sv));
            return false;
        }
        return true;
    }

} // namespace

//-------------------------------------------------------------------------------------------

LegacyDataStore::LegacyDataStore(const QString &filename)
{
    if (!QFile::exists(filename)) {
        spdlog::error("BuyoutCollection: file not found: {}", filename);
        return;
    }

    QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", "LegacyDataStore");
    db.setConnectOptions("QSQLITE_OPEN_READONLY");
    db.setDatabaseName(filename);
    if (!db.open()) {
        spdlog::error("BuyoutCollection: cannot open {} due to error: {}",
                      filename,
                      db.lastError().text());
        return;
    }

    bool ok = true;
    ok &= getString(db, "SELECT value FROM data WHERE (key = 'db_version')", m_data.db_version);
    ok &= getString(db, "SELECT value FROM data WHERE (key = 'version')", m_data.version);
    ok &= getStruct(db, "SELECT value FROM data WHERE (key = 'buyouts')", m_data.buyouts);
    ok &= getStruct(db, "SELECT value FROM data WHERE (key = 'tab_buyouts')", m_data.tab_buyouts);
    ok &= getStruct(db, "SELECT value FROM tabs WHERE (type = 0)", m_tabs.stashes);
    ok &= getStruct(db, "SELECT value FROM tabs WHERE (type = 1)", m_tabs.characters);
    if (!ok) {
        spdlog::error("LegacyDataStore: unable to load all data from {}", filename);
        return;
    }

    const QString statement = "SELECT loc, value FROM items";
    QSqlQuery query(db);
    query.setForwardOnly(true);
    query.prepare(statement);
    if (!query.exec()) {
        spdlog::error("LegacyDataStore: error executing '{}': {}",
                      statement,
                      query.lastError().text());
        return;
    }

    m_item_count = 0;
    while (query.next()) {
        const QString loc   = query.value(0).toString();
        const QByteArray ba = query.value(1).toByteArray();

        std::vector<LegacyItem> result;

        // Parse from a size-aware view (don't assume null-terminated input)
        const std::string_view sv{ba.constData(), static_cast<std::size_t>(ba.size())};

        // Strictish read; set unknown_keys to error if you want to reject extra fields
        constexpr glz::opts opts{
            .null_terminated = false,
            .error_on_unknown_keys = true
        };

        if (auto ec = glz::read<opts>(result, sv); ec) {
            spdlog::error("LegacyDataStore: error parsing 'items' for '{}': {}",
                          loc.toStdString(),
                          glz::format_error(ec, sv));
            return; // or 'continue' if you want to skip bad rows
        }

        m_item_count += static_cast<qint64>(result.size());
        m_items[loc] = std::move(result); // requires your QString adapter in glaze_qt.h
    }

    if (query.lastError().isValid()) {
        spdlog::error("LegacyDataStore: error moving to next record in 'items': {}",
                      query.lastError().text());
        return;
    }

    query.finish();
    db.close();
    db.removeDatabase("LegacyDataStore");

    m_valid = true;
}

bool LegacyDataStore::exportJson(const QString &filename) const
{
    QFile file(filename);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        spdlog::error("Export failed: could not open json file: {}", file.errorString());
        return false;
    }

    std::string out;
    if (auto ec = glz::write_json(*this, out); ec) {
        // For write errors, format_error still gives a readable message.
        spdlog::error("Export failed: {}", glz::format_error(ec, out));
        return false;
    }

    const QByteArray data = QByteArray::fromStdString(out);
    file.write(data);
    file.close();
    return true;
}

bool LegacyDataStore::exportTgz(const QString &filename) const
{
    // Use a temporary working directory.
    QTemporaryDir dir;
    if (!dir.isValid()) {
        spdlog::error("Export failed: could not create a temporary directory: {}",
                      dir.errorString());
        return false;
    }

    // First export to a temporary .json file.
    const QString tempfile = dir.filePath("export.json");
    if (!exportJson(tempfile)) {
        return false;
    }

    // Next compress the temporary file into a tgz.
    const QString command = "tar";
    const QStringList arguments = {"czvf", filename, "-C", dir.path(), "export.json"};
    QProcess process;
    process.start(command, arguments);
    if (!process.waitForFinished()) {
        spdlog::error("Export failed: process failed: {}", process.errorString());
        return false;
    }
    if (process.exitCode() != 0) {
        spdlog::error("Export failed: tar error: {}", process.errorString());
        return false;
    }

    // Remove the temporary .json file.
    if (!QFile(tempfile).remove()) {
        spdlog::warn("Error removing temporary json file: {}", tempfile);
    }
    return true;
}
