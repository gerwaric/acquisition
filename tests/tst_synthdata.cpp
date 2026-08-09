#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QRegularExpression>
#include <QSet>
#include <QSqlDatabase>
#include <QSqlQuery>
#include <QTemporaryDir>
#include <QtTest/QtTest>

#include "datastore/characterrepo.h"
#include "datastore/stashrepo.h"
#include "datastore/userstore.h"
#include "poe/types/character.h"
#include "poe/types/stashtab.h"

// Validation for tools/synthdata: a generated userstore must load through
// the same repo layer the worker uses (itemsmanagerworker.cpp ParseCachedItems)
// with every payload parsing into the typed poe:: structs, every coverage and
// quirk probe recorded in the generator's manifest must be present in the raw
// bytes exactly once, and the generated schema must match what UserStore
// itself creates.
//
// Under ctest, the synthdata fixture (tools/synthdata/selftest.py) generates
// a small store from the checked-in RePoE fixtures into the build directory
// and points ACQ_SYNTH_DEFAULT_DIR/ACQ_SYNTH_DEFAULT_ACCOUNT at it. Set
// ACQ_SYNTH_DATA_DIR/ACQ_SYNTH_ACCOUNT to validate a bigger external store
// instead:
//
//   ACQ_SYNTH_DATA_DIR=/tmp/genout ACQ_SYNTH_ACCOUNT='SYNTH#0000' \
//       ./build/tests/tst_synthdata

namespace {

struct Target
{
    QString dir;
    QString account;
    QString dbPath() const
    {
        return QDir(dir).absoluteFilePath("userstore-" + account + ".db");
    }
};

Target resolveTarget()
{
    Target t{qEnvironmentVariable("ACQ_SYNTH_DATA_DIR"),
             qEnvironmentVariable("ACQ_SYNTH_ACCOUNT")};
    if (t.dir.isEmpty() || t.account.isEmpty()) {
        t = {qEnvironmentVariable("ACQ_SYNTH_DEFAULT_DIR"),
             qEnvironmentVariable("ACQ_SYNTH_DEFAULT_ACCOUNT")};
    }
    return t;
}

// Schema fingerprint: every non-internal object's DDL with all whitespace
// stripped (formatting-insensitive, drift-sensitive), each table's table_info
// rows, and the user_version.
QStringList schemaFingerprint(QSqlDatabase &db)
{
    QStringList out;
    QSqlQuery q(db);
    q.exec("SELECT type, name, sql FROM sqlite_master"
           " WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name");
    QStringList tables;
    while (q.next()) {
        QString sql = q.value(2).toString();
        sql.remove(QRegularExpression("\\s+"));
        out << q.value(0).toString() + " " + q.value(1).toString() + " " + sql;
        if (q.value(0).toString() == "table") {
            tables << q.value(1).toString();
        }
    }
    for (const auto &table : tables) {
        QSqlQuery ti(db);
        ti.exec("PRAGMA table_info(" + table + ")");
        while (ti.next()) {
            QStringList cols;
            for (int i = 0; i < 6; ++i) {
                cols << ti.value(i).toString();
            }
            out << table + ": " + cols.join("|");
        }
    }
    QSqlQuery uv(db);
    uv.exec("PRAGMA user_version");
    uv.next();
    out << "user_version=" + uv.value(0).toString();
    return out;
}

} // namespace

class SynthDataTest : public QObject
{
    Q_OBJECT

private slots:
    // Order matters: the schema check must fingerprint the generated file
    // BEFORE any slot opens it through UserStore, which would migrate an
    // out-of-date schema in place and hide exactly the generator drift the
    // check exists to catch.
    void schemaMatchesApplication();
    void manifestProbesPresent();
    void generatedStoreLoadsAndParses();
};

void SynthDataTest::generatedStoreLoadsAndParses()
{
    const Target t = resolveTarget();
    if (t.dir.isEmpty() || t.account.isEmpty()) {
        QSKIP("no generated store: run under ctest (synthdata fixture) or set"
              " ACQ_SYNTH_DATA_DIR and ACQ_SYNTH_ACCOUNT");
    }

    UserStore store(QDir(t.dir), t.account);

    // Enumerate rows with a side connection; parse them through the repos.
    {
        auto raw = QSqlDatabase::addDatabase("QSQLITE", "synthdata-enumerate");
        raw.setDatabaseName(t.dbPath());
        QVERIFY(raw.open());

        int stash_rows = 0, parsed_tabs = 0, items = 0, with_mods = 0, rares = 0;
        QSqlQuery q(raw);
        QVERIFY(q.exec("SELECT id, realm, league FROM stashes"
                       " WHERE json_data IS NOT NULL"));
        while (q.next()) {
            ++stash_rows;
            const auto tab = store.stashes().getStash(q.value(0).toString(),
                                                      q.value(1).toString(),
                                                      q.value(2).toString());
            QVERIFY2(tab.has_value(),
                     qPrintable("stash row failed to parse: " + q.value(0).toString()));
            ++parsed_tabs;
            if (tab->items) {
                items += static_cast<int>(tab->items->size());
                for (const auto &item : *tab->items) {
                    if (item.frameType == poe::FrameType::Rare) {
                        ++rares;
                    }
                    if (item.explicitMods && !item.explicitMods->empty()) {
                        ++with_mods;
                    }
                }
            }
        }
        QVERIFY(stash_rows > 0);
        QCOMPARE(parsed_tabs, stash_rows);
        QVERIFY(items > 0);
        // A store without rare equipment (e.g. a currency-only profile) is
        // valid; the mod pipeline is only required where rares exist.
        if (rares > 0) {
            QVERIFY(with_mods > 0);
        }

        int char_rows = 0, parsed_chars = 0;
        QVERIFY(q.exec("SELECT name, realm FROM characters"
                       " WHERE json_data IS NOT NULL"));
        while (q.next()) {
            ++char_rows;
            const auto character = store.characters().getCharacter(
                q.value(0).toString(), q.value(1).toString());
            QVERIFY2(character.has_value(),
                     qPrintable("character failed to parse: " + q.value(0).toString()));
            ++parsed_chars;
        }
        QCOMPARE(parsed_chars, char_rows);

        qInfo() << "synthdata:" << parsed_tabs << "tabs," << items << "items,"
                << with_mods << "with explicit mods," << parsed_chars << "characters";
        raw.close();
    }
    QSqlDatabase::removeDatabase("synthdata-enumerate");
}

// Every probe the generator recorded — one per coverage axis and per
// registry quirk — must exist in the store's raw bytes exactly once. An
// item probe's item_json is the item's exact serialization as embedded in
// its payload, so this also proves emitted nulls and deleted keys survived
// into the database unaltered.
void SynthDataTest::manifestProbesPresent()
{
    const Target t = resolveTarget();
    if (t.dir.isEmpty() || t.account.isEmpty()) {
        QSKIP("no generated store: run under ctest (synthdata fixture) or set"
              " ACQ_SYNTH_DATA_DIR and ACQ_SYNTH_ACCOUNT");
    }
    QFile mf(t.dbPath() + ".manifest.json");
    if (!mf.open(QIODevice::ReadOnly)) {
        QSKIP("no manifest next to the database (older generator output)");
    }
    QJsonParseError parseError;
    const auto doc = QJsonDocument::fromJson(mf.readAll(), &parseError);
    QVERIFY2(parseError.error == QJsonParseError::NoError,
             qPrintable("malformed manifest: " + parseError.errorString()));
    const auto manifest = doc.object();
    QVERIFY2(manifest.contains("repro") && manifest.contains("probes"),
             "manifest is missing its repro/probes sections");
    const auto probes = manifest["probes"].toArray();
    if (probes.isEmpty()) {
        // A store generated without --coverage legitimately has no probes;
        // anything else with an empty probe list is a generator bug.
        QVERIFY(manifest["repro"].toObject().contains("coverage"));
        QVERIFY(!manifest["repro"].toObject()["coverage"].toBool());
        QSKIP("manifest has no probes (store generated without --coverage)");
    }

    {
        auto db = QSqlDatabase::addDatabase("QSQLITE", "synthdata-manifest");
        db.setDatabaseName(t.dbPath());
        QVERIFY(db.open());

        // Multi-million-item stores are in scope, so this must stay one
        // streamed pass over the payload bytes with bounded memory: collect
        // the probe item ids up front, then count only those ids' "id":"..."
        // occurrences for the global exactly-once check — nothing else is
        // retained. The byte-exact item_json comparison then runs only
        // against each probe's own stash payload, fetched individually.
        QSet<QByteArray> probeIds;
        for (const auto &value : probes) {
            const auto probe = value.toObject();
            if (probe["kind"].toString() != "quirk-tab") {
                probeIds.insert(probe["item_id"].toString().toUtf8());
            }
        }
        QHash<QByteArray, int> idCounts;
        {
            static const QByteArray marker = "\"id\":\"";
            QSqlQuery q(db);
            QVERIFY(q.exec("SELECT json_data FROM stashes"
                           " WHERE json_data IS NOT NULL"));
            while (q.next()) {
                const QByteArray payload = q.value(0).toByteArray();
                for (auto at = payload.indexOf(marker); at >= 0;
                     at = payload.indexOf(marker, at + 1)) {
                    const auto start = at + marker.size();
                    const auto end = payload.indexOf('"', start);
                    if (end > start) {
                        const QByteArray id = payload.mid(start, end - start);
                        if (probeIds.contains(id)) {
                            ++idCounts[id];
                        }
                    }
                }
            }
        }

        const auto payloadFor = [&db](const QString &stashId) {
            QSqlQuery pq(db);
            pq.prepare("SELECT json_data FROM stashes WHERE id = ?");
            pq.addBindValue(stashId);
            return pq.exec() && pq.next() ? pq.value(0).toByteArray()
                                          : QByteArray();
        };

        int itemProbes = 0, tabProbes = 0;
        for (const auto &value : probes) {
            const auto probe = value.toObject();
            const QString id = probe["id"].toString();
            if (probe["kind"].toString() == "quirk-tab") {
                QSqlQuery tq(db);
                tq.prepare("SELECT name, meta_colour FROM stashes WHERE id = ?");
                tq.addBindValue(probe["stash_id"].toString());
                QVERIFY(tq.exec());
                QVERIFY2(tq.next(), qPrintable("quirk tab missing: " + id));
                QCOMPARE(tq.value(0).toString(), probe["name"].toString());
                if (!probe["colour"].isNull()) {
                    QCOMPARE(tq.value(1).toString(), probe["colour"].toString());
                }
                ++tabProbes;
                continue;
            }
            const QByteArray itemId = probe["item_id"].toString().toUtf8();
            const QByteArray needle = probe["item_json"].toString().toUtf8();
            QVERIFY2(!itemId.isEmpty() && !needle.isEmpty(),
                     qPrintable("probe without item_id/item_json: " + id));
            QVERIFY2(idCounts.value(itemId) == 1,
                     qPrintable(QString("probe '%1': item id found %2 times"
                                        " (want 1)")
                                    .arg(id)
                                    .arg(idCounts.value(itemId))));
            const QByteArray payload = payloadFor(probe["stash_id"].toString());
            QVERIFY2(payload.contains(needle),
                     qPrintable(QString("probe '%1': emitted bytes not found"
                                        " in its stash payload")
                                    .arg(id)));
            ++itemProbes;
        }
        qInfo() << "synthdata manifest:" << itemProbes << "item probes,"
                << tabProbes << "tab probes verified";
        db.close();
    }
    QSqlDatabase::removeDatabase("synthdata-manifest");
}

// The generator carries its own copy of the schema DDL; it must stay
// byte-equivalent (modulo whitespace) to what UserStore creates, or the
// synthetic store stops being the store the app would have written.
void SynthDataTest::schemaMatchesApplication()
{
    const Target t = resolveTarget();
    if (t.dir.isEmpty() || t.account.isEmpty()) {
        QSKIP("no generated store: run under ctest (synthdata fixture) or set"
              " ACQ_SYNTH_DATA_DIR and ACQ_SYNTH_ACCOUNT");
    }

    QTemporaryDir tmp;
    QVERIFY(tmp.isValid());
    QStringList expected;
    {
        // Scoped so the store's connection is gone before comparison.
        UserStore reference(QDir(tmp.path()), "SCHEMA#0000");
    }
    {
        auto db = QSqlDatabase::addDatabase("QSQLITE", "synthdata-schema-ref");
        db.setDatabaseName(
            QDir(tmp.path()).absoluteFilePath("userstore-SCHEMA#0000.db"));
        QVERIFY(db.open());
        expected = schemaFingerprint(db);
        db.close();
    }
    QSqlDatabase::removeDatabase("synthdata-schema-ref");

    QStringList actual;
    {
        auto db = QSqlDatabase::addDatabase("QSQLITE", "synthdata-schema-gen");
        db.setDatabaseName(t.dbPath());
        QVERIFY(db.open());
        actual = schemaFingerprint(db);
        db.close();
    }
    QSqlDatabase::removeDatabase("synthdata-schema-gen");

    QCOMPARE(actual, expected);
}

QTEST_GUILESS_MAIN(SynthDataTest)
#include "tst_synthdata.moc"
