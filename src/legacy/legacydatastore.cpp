// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "legacy/legacydatastore.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QProcess>
#include <QScopeGuard>
#include <QSqlError>
#include <QSqlQuery>
#include <QTemporaryDir>
#include <QUuid>

namespace {

    enum class ReadResult { Loaded, Skipped, Error };

    static ReadResult getByteArray(QSqlDatabase db, const QString &query, QByteArray &value)
    {
        QSqlQuery q(db);
        q.setForwardOnly(true);
        q.prepare(query);
        if (!q.exec()) {
            spdlog::error("Database error calling exec(): {}: {}", query, db.lastError().text());
            return ReadResult::Error;
        }
        if (!q.next()) {
            spdlog::warn("LegacyDataStore: no row found for '{}'; skipping", query);
            return ReadResult::Skipped;
        }
        value = q.value(0).toByteArray();
        return ReadResult::Loaded;
    }

    static ReadResult getString(QSqlDatabase db, const QString &query, QString &value)
    {
        QSqlQuery q(db);
        q.setForwardOnly(true);
        q.prepare(query);
        if (!q.exec()) {
            spdlog::error("Database error calling exec(): {}: {}", query, db.lastError().text());
            return ReadResult::Error;
        }
        if (!q.next()) {
            spdlog::warn("LegacyDataStore: no row found for '{}'; skipping", query);
            return ReadResult::Skipped;
        }
        value = q.value(0).toString();
        return ReadResult::Loaded;
    }

    template<typename T>
    static ReadResult getStruct(QSqlDatabase db, const QString &query, T &value)
    {
        QByteArray data;
        const ReadResult read_result = getByteArray(db, query, data);
        if (read_result != ReadResult::Loaded) {
            return read_result;
        }

        // Create a view over the QByteArray (it may contain '\0', so don't assume C-strings)
        const std::string_view sv{data.constData(), static_cast<std::size_t>(data.size())};

        constexpr glz::opts opts{.null_terminated = false, .error_on_unknown_keys = false};

        if (auto ec = glz::read<opts>(value, sv); ec) {
            spdlog::warn("LegacyDataStore: JSON error parsing {} from '{}'; skipping: {}",
                         typeid(T).name(),
                         query.toStdString(),
                         glz::format_error(ec, sv));
            return ReadResult::Skipped;
        }
        return ReadResult::Loaded;
    }

} // namespace

//-------------------------------------------------------------------------------------------

LegacyDataStore::LegacyDataStore(const QString &filename)
{
    if (!QFile::exists(filename)) {
        spdlog::error("BuyoutCollection: file not found: {}", filename);
        return;
    }

    const QString connection_name = "LegacyDataStore:"
                                    + QUuid::createUuid().toString(QUuid::WithoutBraces);
    QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", connection_name);
    const auto close_database = qScopeGuard([&db, &connection_name] {
        db.close();
        db = QSqlDatabase();
        QSqlDatabase::removeDatabase(connection_name);
    });
    db.setConnectOptions("QSQLITE_OPEN_READONLY");
    db.setDatabaseName(filename);
    if (!db.open()) {
        spdlog::error("BuyoutCollection: cannot open {} due to error: {}",
                      filename,
                      db.lastError().text());
        return;
    }

    const auto load = [this](ReadResult result) {
        if (result == ReadResult::Skipped) {
            ++m_skipped_row_count;
        }
        return result != ReadResult::Error;
    };

    bool structurally_valid = true;
    structurally_valid &= load(
        getString(db, "SELECT value FROM data WHERE (key = 'db_version')", m_data.db_version));
    structurally_valid &= load(
        getString(db, "SELECT value FROM data WHERE (key = 'version')", m_data.version));
    structurally_valid &= load(
        getStruct(db, "SELECT value FROM data WHERE (key = 'buyouts')", m_data.buyouts));
    structurally_valid &= load(
        getStruct(db, "SELECT value FROM data WHERE (key = 'tab_buyouts')", m_data.tab_buyouts));
    structurally_valid &= load(
        getStruct(db, "SELECT value FROM tabs WHERE (type = 0)", m_tabs.stashes));
    structurally_valid &= load(
        getStruct(db, "SELECT value FROM tabs WHERE (type = 1)", m_tabs.characters));
    if (!structurally_valid) {
        spdlog::error("LegacyDataStore: required database tables could not be read from {}",
                      filename);
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
        const QString loc = query.value(0).toString();
        const QByteArray ba = query.value(1).toByteArray();

        std::vector<LegacyItem> result;

        // Parse from a size-aware view (don't assume null-terminated input)
        const std::string_view sv{ba.constData(), static_cast<std::size_t>(ba.size())};

        constexpr glz::opts opts{.null_terminated = false, .error_on_unknown_keys = false};

        if (auto ec = glz::read<opts>(result, sv); ec) {
            spdlog::warn("LegacyDataStore: error parsing 'items' for '{}'; skipping row: {}",
                         loc.toStdString(),
                         glz::format_error(ec, sv));
            ++m_skipped_row_count;
            continue;
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
