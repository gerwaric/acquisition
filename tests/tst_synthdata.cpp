#include <QDir>
#include <QSqlDatabase>
#include <QSqlQuery>
#include <QtTest/QtTest>

#include "datastore/characterrepo.h"
#include "datastore/stashrepo.h"
#include "datastore/userstore.h"
#include "poe/types/character.h"
#include "poe/types/stashtab.h"

// Validation for tools/synthdata: a generated userstore must load through
// the same repo layer the worker uses (itemsmanagerworker.cpp ParseCachedItems)
// with every payload parsing into the typed poe:: structs. Skipped unless
// ACQ_SYNTH_DATA_DIR and ACQ_SYNTH_ACCOUNT point at a generated database,
// so the regular suite is unaffected:
//
//   ACQ_SYNTH_DATA_DIR=/tmp/genout ACQ_SYNTH_ACCOUNT='SYNTH#0000' \
//       ./build/tests/tst_synthdata

class SynthDataTest : public QObject
{
    Q_OBJECT

private slots:
    void generatedStoreLoadsAndParses();
};

void SynthDataTest::generatedStoreLoadsAndParses()
{
    const QString dir = qEnvironmentVariable("ACQ_SYNTH_DATA_DIR");
    const QString account = qEnvironmentVariable("ACQ_SYNTH_ACCOUNT");
    if (dir.isEmpty() || account.isEmpty()) {
        QSKIP("set ACQ_SYNTH_DATA_DIR and ACQ_SYNTH_ACCOUNT to validate a"
              " generated userstore");
    }

    UserStore store(QDir(dir), account);

    // Enumerate rows with a side connection; parse them through the repos.
    {
        auto raw = QSqlDatabase::addDatabase("QSQLITE", "synthdata-enumerate");
        raw.setDatabaseName(QDir(dir).absoluteFilePath("userstore-" + account + ".db"));
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

QTEST_GUILESS_MAIN(SynthDataTest)
#include "tst_synthdata.moc"
