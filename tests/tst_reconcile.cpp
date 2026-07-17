#include <QSqlDatabase>
#include <QSqlError>
#include <QUuid>
#include <QtTest/QtTest>

#include <memory>
#include <optional>

#include "datastore/characterrepo.h"
#include "datastore/stashrepo.h"
#include "poe/types/character.h"
#include "poe/types/stashtab.h"

// Repo-level pins for the F53 list reconciliation: rows the server no
// longer lists are deleted so deleted tabs and characters cannot resurrect
// from the cache at the next startup. Map/Unique children survive list
// reconciliation (they never appear in any list) and are reconciled
// against their parent's replies instead; folder children ride the
// flattened top-level list. Scoping: league for stashes, realm-wide for
// characters (the character list endpoint spans leagues).

class ReconcileTest : public QObject
{
    Q_OBJECT

private slots:
    void stashRowsAbsentFromFreshListAreDeleted();
    void folderChildrenFollowTheFlattenedList();
    void specialChildrenSurviveListReconcile();
    void deletedSpecialParentCascades();
    void emptyStashListDeletesLeagueRows();
    void parentReplyReconcilesChildRows();
    void emptyChildListDeletesAllChildren();
    void characterRowsAbsentFromFreshListAreDeleted();
    void emptyCharacterListDeletesRealmRows();
};

namespace {

    constexpr const char *kRealm = "pc";
    constexpr const char *kLeague = "Reconcile League";

    struct RepoFixture
    {
        RepoFixture()
            : connectionName("reconcile-test-" + QUuid::createUuid().toString(QUuid::WithoutBraces))
        {
            db.emplace(QSqlDatabase::addDatabase("QSQLITE", connectionName));
            db->setDatabaseName(":memory:");
            if (!db->open()) {
                qFatal("Failed to open test QSQLITE database: %s",
                       qPrintable(db->lastError().text()));
            }
            stashes = std::make_unique<StashRepo>(*db);
            characters = std::make_unique<CharacterRepo>(*db);
            if (!stashes->ensureSchema() || !characters->ensureSchema()) {
                qFatal("Failed to create test schema");
            }
        }

        ~RepoFixture()
        {
            characters.reset();
            stashes.reset();
            db->close();
            db.reset();
            QSqlDatabase::removeDatabase(connectionName);
        }

        QString connectionName;
        std::optional<QSqlDatabase> db;
        std::unique_ptr<StashRepo> stashes;
        std::unique_ptr<CharacterRepo> characters;
    };

    poe::StashTab makeTab(const QString &id,
                          const QString &type = "PremiumStash",
                          const std::optional<QString> &parent = {})
    {
        poe::StashTab tab;
        tab.id = id;
        tab.name = "Tab " + id;
        tab.type = type;
        tab.parent = parent;
        return tab;
    }

    poe::Character makeCharacter(const QString &id,
                                 const QString &name,
                                 const QString &realm = kRealm,
                                 const QString &league = kLeague)
    {
        poe::Character character;
        character.id = id;
        character.name = name;
        character.realm = realm;
        character.class_ = "Witch";
        character.league = league;
        character.level = 1;
        character.experience = 0;
        return character;
    }

    QStringList stashIds(StashRepo &repo, const QString &league = kLeague)
    {
        QStringList ids;
        for (const auto &stash : repo.getStashList(kRealm, league)) {
            ids.append(stash.id);
        }
        ids.sort();
        return ids;
    }

    QStringList characterIds(CharacterRepo &repo, const QString &realm = kRealm)
    {
        QStringList ids;
        for (const auto &character : repo.getCharacterList(realm)) {
            ids.append(character.id);
        }
        ids.sort();
        return ids;
    }

} // namespace

void ReconcileTest::stashRowsAbsentFromFreshListAreDeleted()
{
    RepoFixture f;
    QVERIFY(f.stashes->saveStashList({makeTab("tab-a"), makeTab("tab-b")}, kRealm, kLeague));

    QVERIFY(f.stashes->reconcileStashList({makeTab("tab-a")}, kRealm, kLeague));

    QCOMPARE(stashIds(*f.stashes), QStringList({"tab-a"}));
}

// Folder children arrive inline in the top-level list, so the flattening
// keeps a listed child and drops one the server no longer lists.
void ReconcileTest::folderChildrenFollowTheFlattenedList()
{
    RepoFixture f;
    QVERIFY(f.stashes->saveStashList({makeTab("folder-1", "Folder"),
                                      makeTab("child-old", "PremiumStash", QString("folder-1")),
                                      makeTab("child-new", "PremiumStash", QString("folder-1"))},
                                     kRealm,
                                     kLeague));

    poe::StashTab folder = makeTab("folder-1", "Folder");
    folder.children = {makeTab("child-new", "PremiumStash", QString("folder-1"))};
    QVERIFY(f.stashes->reconcileStashList({folder}, kRealm, kLeague));

    QCOMPARE(stashIds(*f.stashes), QStringList({"child-new", "folder-1"}));
}

// Map/Unique children never appear in any stash list; while their parent
// survives, list reconciliation must leave them alone.
void ReconcileTest::specialChildrenSurviveListReconcile()
{
    RepoFixture f;
    QVERIFY(f.stashes->saveStashList({makeTab("mapstash-1", "MapStash")}, kRealm, kLeague));
    QVERIFY(f.stashes->saveStash(makeTab("mapchild-1", "MapStash", QString("mapstash-1")),
                                 kRealm,
                                 kLeague));

    QVERIFY(f.stashes->reconcileStashList({makeTab("mapstash-1", "MapStash")}, kRealm, kLeague));

    QCOMPARE(stashIds(*f.stashes), QStringList({"mapchild-1", "mapstash-1"}));
}

// When the special parent itself disappears from the list, its child rows
// lose their protection and are deleted with it.
void ReconcileTest::deletedSpecialParentCascades()
{
    RepoFixture f;
    QVERIFY(f.stashes->saveStashList({makeTab("mapstash-1", "MapStash"), makeTab("tab-a")},
                                     kRealm,
                                     kLeague));
    QVERIFY(f.stashes->saveStash(makeTab("mapchild-1", "MapStash", QString("mapstash-1")),
                                 kRealm,
                                 kLeague));

    QVERIFY(f.stashes->reconcileStashList({makeTab("tab-a")}, kRealm, kLeague));

    QCOMPARE(stashIds(*f.stashes), QStringList({"tab-a"}));
}

// "Everything was deleted" must be expressible, and reconciliation must
// not leak across leagues.
void ReconcileTest::emptyStashListDeletesLeagueRows()
{
    RepoFixture f;
    QVERIFY(f.stashes->saveStashList({makeTab("tab-a"), makeTab("tab-b")}, kRealm, kLeague));
    QVERIFY(f.stashes->saveStashList({makeTab("tab-other")}, kRealm, "Other League"));

    QVERIFY(f.stashes->reconcileStashList({}, kRealm, kLeague));

    QCOMPARE(stashIds(*f.stashes), QStringList());
    QCOMPARE(stashIds(*f.stashes, "Other League"), QStringList({"tab-other"}));
}

void ReconcileTest::parentReplyReconcilesChildRows()
{
    RepoFixture f;
    QVERIFY(f.stashes->saveStashList({makeTab("mapstash-1", "MapStash"),
                                      makeTab("mapstash-2", "MapStash")},
                                     kRealm,
                                     kLeague));
    QVERIFY(f.stashes->saveStash(makeTab("child-a1", "MapStash", QString("mapstash-1")),
                                 kRealm,
                                 kLeague));
    QVERIFY(f.stashes->saveStash(makeTab("child-a2", "MapStash", QString("mapstash-1")),
                                 kRealm,
                                 kLeague));
    QVERIFY(f.stashes->saveStash(makeTab("child-b1", "MapStash", QString("mapstash-2")),
                                 kRealm,
                                 kLeague));

    QVERIFY(f.stashes->reconcileStashChildren("mapstash-1", {"child-a1"}, kRealm, kLeague));

    QCOMPARE(stashIds(*f.stashes),
             QStringList({"child-a1", "child-b1", "mapstash-1", "mapstash-2"}));
}

// With child fetching disabled the worker passes no children, deleting the
// cached child rows: re-enabling the setting requires a refetch instead of
// showing stale cache (documented F53 policy).
void ReconcileTest::emptyChildListDeletesAllChildren()
{
    RepoFixture f;
    QVERIFY(f.stashes->saveStashList({makeTab("mapstash-1", "MapStash")}, kRealm, kLeague));
    QVERIFY(f.stashes->saveStash(makeTab("child-a1", "MapStash", QString("mapstash-1")),
                                 kRealm,
                                 kLeague));

    QVERIFY(f.stashes->reconcileStashChildren("mapstash-1", {}, kRealm, kLeague));

    QCOMPARE(stashIds(*f.stashes), QStringList({"mapstash-1"}));
}

void ReconcileTest::characterRowsAbsentFromFreshListAreDeleted()
{
    RepoFixture f;
    QVERIFY(f.characters->saveCharacterList(
        {makeCharacter("charid0001", "CharOne"), makeCharacter("charid0002", "CharTwo")}));
    QVERIFY(f.characters->saveCharacterList({makeCharacter("charid0003", "OtherRealm", "sony")}));

    QVERIFY(f.characters->reconcileCharacterList({makeCharacter("charid0001", "CharOne")}, kRealm));

    QCOMPARE(characterIds(*f.characters), QStringList({"charid0001"}));
    QCOMPARE(characterIds(*f.characters, "sony"), QStringList({"charid0003"}));
}

void ReconcileTest::emptyCharacterListDeletesRealmRows()
{
    RepoFixture f;
    QVERIFY(f.characters->saveCharacterList({makeCharacter("charid0001", "CharOne")}));

    QVERIFY(f.characters->reconcileCharacterList({}, kRealm));

    QCOMPARE(characterIds(*f.characters), QStringList());
}

QTEST_GUILESS_MAIN(ReconcileTest)

#include "tst_reconcile.moc"
