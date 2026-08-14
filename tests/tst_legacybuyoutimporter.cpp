// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest/QtTest>

#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>
#include <QTemporaryDir>
#include <QUuid>

#include "legacy/legacybuyoutimporter.h"
#include "legacy/legacyitem.h"
#include "testfixtures.h"

namespace {

    void createLegacyDatabase(const QString &filename, const QString &db_version)
    {
        LegacyItem stash_item;
        stash_item.name = "";
        stash_item.typeLine = "Chaos Orb";
        stash_item._tab_label = "Priced";
        const QString stash_hash = stash_item.hash();

        LegacyItem character_item;
        character_item.name = "";
        character_item.typeLine = "Divine Orb";
        character_item._character = "Bob";
        const QString character_hash = character_item.hash();

        const QString buyout_json = QString(R"json({
                "%1":{"value":5,"last_update":1700000000,"type":"b/o","currency":"chaos","source":"manual","inherited":false},
                "%2":{"value":6,"last_update":1700000001,"type":"price","currency":"divine","source":"manual","inherited":false},
                "orphan-hash":{"value":7,"last_update":1700000002,"type":"b/o","currency":"chaos","source":"manual","inherited":false}
            })json")
                                        .arg(stash_hash, character_hash);
        const QString tab_buyout_json = R"json({
            "stash:Priced":{"value":8,"last_update":1700000003,"type":"b/o","currency":"chaos","source":"manual","inherited":false},
            "character:Bob":{"value":9,"last_update":1700000004,"type":"price","currency":"divine","source":"manual","inherited":false}
        })json";
        const QString stashes_json = R"json([
            {"id":"stash-a","name":"Priced","type":"PremiumStash","index":0,"metadata":{"colour":"7f7f7f"}},
            {"id":"stash-b","name":"Priced","type":"PremiumStash","index":1,"metadata":{"colour":"7f7f7f"}}
        ])json";
        const QString characters_json = R"json([
            {"id":"character-id-bob","name":"Bob","realm":"pc","class":"Marauder","league":"Standard","level":90,"experience":1}
        ])json";
        const QString stash_items_json = R"json([
            {"id":"item-a","name":"","typeLine":"Chaos Orb","_tab_label":"Priced","ilvl":0,"frameType":5},
            {"id":"item-b","name":"","typeLine":"Chaos Orb","_tab_label":"Priced","ilvl":0,"frameType":5}
        ])json";
        const QString character_items_json = R"json([
            {"id":"item-character","name":"","typeLine":"Divine Orb","_character":"Bob","inventoryId":"MainInventory"}
        ])json";

        const QString connection_name = "legacy-import-source:"
                                        + QUuid::createUuid().toString(QUuid::WithoutBraces);
        {
            QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", connection_name);
            db.setDatabaseName(filename);
            QVERIFY2(db.open(), qPrintable(db.lastError().text()));
            QSqlQuery query(db);
            QVERIFY(query.exec("CREATE TABLE data (key TEXT PRIMARY KEY, value BLOB)"));
            QVERIFY(query.exec("CREATE TABLE tabs (type INTEGER PRIMARY KEY, value BLOB)"));
            QVERIFY(query.exec("CREATE TABLE items (loc TEXT PRIMARY KEY, value BLOB)"));

            query.prepare("INSERT INTO data (key, value) VALUES (?, ?)");
            const auto insert_data = [&query](const QString &key, const QString &value) {
                query.bindValue(0, key);
                query.bindValue(1, value);
                return query.exec();
            };
            QVERIFY(insert_data("db_version", db_version));
            QVERIFY(insert_data("version", "0.15.0"));
            QVERIFY(insert_data("buyouts", buyout_json));
            QVERIFY(insert_data("tab_buyouts", tab_buyout_json));

            query.prepare("INSERT INTO tabs (type, value) VALUES (?, ?)");
            query.bindValue(0, 0);
            query.bindValue(1, stashes_json);
            QVERIFY(query.exec());
            query.bindValue(0, 1);
            query.bindValue(1, characters_json);
            QVERIFY(query.exec());

            query.prepare("INSERT INTO items (loc, value) VALUES (?, ?)");
            query.bindValue(0, "stash-a");
            query.bindValue(1, stash_items_json);
            QVERIFY(query.exec());
            query.bindValue(0, "Bob");
            query.bindValue(1, character_items_json);
            QVERIFY(query.exec());
            db.close();
        }
        QSqlDatabase::removeDatabase(connection_name);
    }

} // namespace

class LegacyBuyoutImporterTest : public QObject
{
    Q_OBJECT

private slots:
    void importsMatchesWithoutOverwritingAndIsIdempotent();
    void importsVersion5StampedFiles();
    void refusesPreVersion4Files();
};

void LegacyBuyoutImporterTest::importsMatchesWithoutOverwritingAndIsIdempotent()
{
    QTemporaryDir source_dir;
    QVERIFY(source_dir.isValid());
    const QString source_path = source_dir.filePath("legacy-v4.db");
    createLegacyDatabase(source_path, "4");

    BuyoutManagerFixture destination;
    const Buyout existing_item = makeChaosBuyout(99.0);
    const Buyout existing_location = makeChaosBuyout(98.0);
    QCOMPARE(destination.repo->saveItemBuyout(existing_item,
                                              "item-a",
                                              "stash-a",
                                              ItemLocationType::STASH,
                                              false),
             BuyoutSaveResult::Saved);
    QCOMPARE(destination.repo->saveLocationBuyout(existing_location,
                                                  "stash-a",
                                                  ItemLocationType::STASH,
                                                  false),
             BuyoutSaveResult::Saved);

    LegacyBuyoutImporter importer(*destination.repo);
    const LegacyBuyoutImportReport first = importer.importFile(source_path);

    QVERIFY2(first.success, qPrintable(first.error));
    QCOMPARE(first.imported, 4);
    QCOMPARE(first.ambiguous, 2);
    QCOMPARE(first.orphaned, 1);
    QCOMPARE(first.skipped, 2);

    bool reload_notified_everything = false;
    connect(destination.manager.get(),
            &BuyoutManager::BuyoutsChanged,
            this,
            [&reload_notified_everything](const BuyoutChangeSet &changes) {
                reload_notified_everything = changes.everything;
            });
    destination.manager->ReloadBuyouts();
    QVERIFY(reload_notified_everything);
    const Item imported_item = makeTestItem("item-b");
    QCOMPARE(destination.manager->Get(imported_item).value, 5.0);

    const auto item_buyouts = destination.repo->getItemBuyouts();
    QCOMPARE(item_buyouts.size(), std::size_t(3));
    QCOMPARE(item_buyouts.at("item-a").value, 99.0);
    QCOMPARE(item_buyouts.at("item-b").value, 5.0);
    QCOMPARE(item_buyouts.at("item-character").value, 6.0);

    const auto location_buyouts = destination.repo->getLocationBuyouts();
    QCOMPARE(location_buyouts.size(), std::size_t(3));
    QCOMPARE(location_buyouts.at("stash-a").value, 98.0);
    QCOMPARE(location_buyouts.at("stash-b").value, 8.0);
    QCOMPARE(location_buyouts.at("character-id-bob").value, 9.0);

    QSqlQuery location_query(*destination.db);
    location_query.prepare(
        "SELECT location_id, location_type FROM item_buyouts WHERE item_id = 'item-character'");
    QVERIFY(location_query.exec());
    QVERIFY(location_query.next());
    QCOMPARE(location_query.value(0).toString(), QString("character-id-bob"));
    QCOMPARE(location_query.value(1).toString(), QString("character"));

    const LegacyBuyoutImportReport second = importer.importFile(source_path);
    QVERIFY(second.success);
    QCOMPARE(second.imported, 0);
    QCOMPARE(second.ambiguous, 2);
    QCOMPARE(second.orphaned, 1);
    QCOMPARE(second.skipped, 6);
}

void LegacyBuyoutImporterTest::importsVersion5StampedFiles()
{
    // Master's MigrateBuyouts stamps db_version 5 into upgraded legacy
    // files while leaving the v4-generation hash keys intact (R1-1), so
    // a 5-stamped file must import exactly like a 4-stamped one.
    QTemporaryDir source_dir;
    QVERIFY(source_dir.isValid());
    const QString source_path = source_dir.filePath("legacy-v5.db");
    createLegacyDatabase(source_path, "5");

    BuyoutManagerFixture destination;
    LegacyBuyoutImporter importer(*destination.repo);
    const LegacyBuyoutImportReport report = importer.importFile(source_path);

    QVERIFY2(report.success, qPrintable(report.error));
    QCOMPARE(report.imported, 6);
    QCOMPARE(report.orphaned, 1);
    QCOMPARE(destination.repo->getItemBuyouts().size(), std::size_t(3));
    QCOMPARE(destination.repo->getLocationBuyouts().size(), std::size_t(3));
}

void LegacyBuyoutImporterTest::refusesPreVersion4Files()
{
    QTemporaryDir source_dir;
    QVERIFY(source_dir.isValid());
    const QString source_path = source_dir.filePath("legacy-v3.db");
    createLegacyDatabase(source_path, "3");

    BuyoutManagerFixture destination;
    LegacyBuyoutImporter importer(*destination.repo);
    const LegacyBuyoutImportReport report = importer.importFile(source_path);

    QVERIFY(!report.success);
    QVERIFY(report.error.contains("db_version 4"));
    QVERIFY(destination.repo->getItemBuyouts().empty());
    QVERIFY(destination.repo->getLocationBuyouts().empty());
}

QTEST_GUILESS_MAIN(LegacyBuyoutImporterTest)

#include "tst_legacybuyoutimporter.moc"
