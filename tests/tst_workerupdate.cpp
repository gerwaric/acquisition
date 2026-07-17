#include <QtTest/QtTest>

#include <QSettings>

#include "datastore/characterrepo.h"
#include "datastore/stashrepo.h"
#include "datastore/userstore.h"
#include "fakenetwork.h"
#include "itemsmanagerworker.h"
#include "testfixtures.h"
#include "util/json_readers.h"
#include "util/networkmanager.h"

// Offline update-cycle tests for ItemsManagerWorker, driven through the
// fake-network harness (items-pipeline M1). These pin the F28 semantics —
// a failed update loses nothing, stale replies from a superseded update
// are discarded — and the F48 no-duplicate-characters fix, none of which
// can be produced reliably by hand against the live API.

// NOTE: this class must be declared before the JSON helpers below: moc's
// lexer treats the // in a raw string literal (the icon URL) as a comment
// and loses any Q_OBJECT class that follows.
class WorkerUpdateTest : public QObject
{
    Q_OBJECT

private slots:
    void failedUpdateKeepsItemsIntact();
    void staleReplyFromSupersededUpdateIsDiscarded();
    void partialUpdateDoesNotDuplicateCharacters();
    void renamedTabMetadataRefreshesWithoutFetch();
};

namespace {

    constexpr const char *kRealm = "pc";
    constexpr const char *kLeague = "Worker Update League";

    QString itemJson(const QString &id)
    {
        return QString(R"({
            "verified": true,
            "w": 1,
            "h": 1,
            "icon": "https://web.poecdn.com/image/test.png",
            "id": "%1",
            "name": "",
            "typeLine": "Test Item",
            "baseType": "Test Item",
            "identified": true,
            "ilvl": 1,
            "x": 0,
            "y": 0
        })")
            .arg(id);
    }

    QString joinItems(const QStringList &item_ids)
    {
        QStringList parts;
        for (const auto &id : item_ids) {
            parts.append(itemJson(id));
        }
        return parts.join(",");
    }

    QByteArray stashJson(const QString &id,
                         const QString &name,
                         unsigned index,
                         const std::optional<QStringList> &item_ids = {})
    {
        QString json = QString(R"({"id":"%1","name":"%2","type":"PremiumStash","index":%3)")
                           .arg(id, name, QString::number(index));
        if (item_ids) {
            json += QString(R"(,"items":[%1])").arg(joinItems(*item_ids));
        }
        json += "}";
        return json.toUtf8();
    }

    QByteArray stashListBody(const QList<QByteArray> &stashes)
    {
        QByteArrayList list(stashes.cbegin(), stashes.cend());
        return R"({"stashes":[)" + list.join(",") + "]}";
    }

    QByteArray stashBody(const QByteArray &stash)
    {
        return R"({"stash":)" + stash + "}";
    }

    QByteArray characterJson(const QString &id,
                             const QString &name,
                             const std::optional<QStringList> &item_ids = {})
    {
        QString json
            = QString(
                  R"({"id":"%1","name":"%2","realm":"%3","class":"Witch","league":"%4","level":1,"experience":0)")
                  .arg(id, name, kRealm, kLeague);
        if (item_ids) {
            json += QString(R"(,"equipment":[%1])").arg(joinItems(*item_ids));
        }
        json += "}";
        return json.toUtf8();
    }

    QByteArray characterListBody(const QList<QByteArray> &characters)
    {
        QByteArrayList list(characters.cbegin(), characters.cend());
        return R"({"characters":[)" + list.join(",") + "]}";
    }

    QByteArray characterBody(const QByteArray &character)
    {
        return R"({"character":)" + character + "}";
    }

    QStringList sortedItemIds(const Items &items)
    {
        QStringList ids;
        for (const auto &item : items) {
            ids.append(item->id());
        }
        ids.sort();
        return ids;
    }

    // A worker wired to the fake network, with the last ItemsRefreshed
    // emission captured for inspection. Deliveries are synchronous, so
    // outside of initialize() no event loop runs and every assertion sees
    // the worker's settled state.
    struct WorkerFixture
    {
        explicit WorkerFixture(const QString &account)
            : settings(bm.tempDir.filePath("settings.ini"), QSettings::IniFormat)
            , rate_limiter(network)
        {
            settings.setValue("account", account);
            settings.setValue("realm", kRealm);
            settings.setValue("league", kLeague);
            settings.sync();
        }

        // Call after seeding the datastore: the worker reads the cache on
        // construction paths keyed off the settings file's directory.
        void start()
        {
            worker = std::make_unique<ItemsManagerWorker>(settings, *bm.manager, rate_limiter);
            QObject::connect(worker.get(),
                             &ItemsManagerWorker::ItemsRefreshed,
                             worker.get(),
                             [this](const Items &items,
                                    const std::vector<ItemLocation> &tabs,
                                    bool initial) {
                                 last_items = items;
                                 last_tabs = tabs;
                                 last_initial = initial;
                                 ++refresh_count;
                             });
            worker->OnRePoEReady();
        }

        QString dataDir() const { return bm.tempDir.filePath("data"); }

        BuyoutManagerFixture bm;
        QSettings settings;
        NetworkManager network;
        FakeRateLimiter rate_limiter;
        std::unique_ptr<ItemsManagerWorker> worker;

        Items last_items;
        std::vector<ItemLocation> last_tabs;
        bool last_initial{false};
        int refresh_count{0};
    };

} // namespace

// A terminal failure mid-update must lose nothing: items already replaced
// stay replaced, items whose fetch never landed stay cached (F28).
void WorkerUpdateTest::failedUpdateKeepsItemsIntact()
{
    WorkerFixture f("worker-update-account-1");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1", "a2"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    QVERIFY(tab_a && tab_b);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-1");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QVERIFY(f.last_initial);
    QCOMPARE(f.last_items.size(), size_t(3));

    const QByteArray fresh_list = stashListBody(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("stashbbbb1", "Tab B", 1)});

    // A full update fetches both lists, then one request per tab.
    f.worker->Update(TabSelection::All);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    QCOMPARE(f.rate_limiter.request(0).endpoint, QString("List Stashes"));
    f.rate_limiter.deliver(0, fresh_list);

    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    QCOMPARE(f.rate_limiter.request(1).endpoint, QString("List Characters"));
    f.rate_limiter.deliver(1, R"({"characters":[]})");

    // Tab A lands with a new set of items...
    QCOMPARE(f.rate_limiter.requestCount(), size_t(3));
    QCOMPARE(f.rate_limiter.request(2).endpoint, QString("Get Stash"));
    QVERIFY(f.rate_limiter.request(2).request.url().path().endsWith("stashaaaa1"));
    f.rate_limiter.deliver(2,
                           stashBody(stashJson("stashaaaa1",
                                               "Tab A",
                                               0,
                                               QStringList{"a1-new", "a2-new", "a3-new"})));

    // ...then tab B's fetch dies, terminating the update without an emit.
    QCOMPARE(f.rate_limiter.requestCount(), size_t(4));
    QVERIFY(f.rate_limiter.request(3).request.url().path().endsWith("stashbbbb1"));
    f.rate_limiter.deliver(3, {}, QNetworkReply::ConnectionRefusedError);
    QCOMPARE(f.refresh_count, 1);

    // A follow-up update of tab A alone succeeds and shows that the failed
    // update lost nothing: tab B's cached item is still present.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(5));
    f.rate_limiter.deliver(4, fresh_list);

    QCOMPARE(f.rate_limiter.requestCount(), size_t(6));
    QVERIFY(f.rate_limiter.request(5).request.url().path().endsWith("stashaaaa1"));
    f.rate_limiter.deliver(5,
                           stashBody(stashJson("stashaaaa1",
                                               "Tab A",
                                               0,
                                               QStringList{"a1-new", "a2-new", "a3-new"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1-new", "a2-new", "a3-new", "b1"}));
}

// Replies left in flight by a failed update must be discarded, both while
// the worker is idle and after a new update has begun (F28). Re-delivering
// the same recorded reply works because no event loop runs in between, so
// the receiver's deleteLater() calls have not executed yet.
void WorkerUpdateTest::staleReplyFromSupersededUpdateIsDiscarded()
{
    WorkerFixture f("worker-update-account-2");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    QVERIFY(tab_a);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-2");
        QVERIFY(store.stashes().saveStashList({*tab_a}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_items.size(), size_t(1));

    const ItemLocation loc_a(*tab_a);
    const QByteArray fresh_list = stashListBody({stashJson("stashaaaa1", "Tab A", 0)});

    // Update 1: the item fetch for tab A fails, terminating the update.
    f.worker->Update(TabSelection::Selected, {loc_a});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0, fresh_list);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    f.rate_limiter.deliver(1, {}, QNetworkReply::TimeoutError);
    QCOMPARE(f.refresh_count, 1);

    // The same reply surfacing again while no update is running must be
    // discarded without disturbing anything.
    f.rate_limiter.deliver(1, {}, QNetworkReply::TimeoutError);

    // Update 2 begins; the stale failure from update 1 arrives mid-update.
    // If it were processed, it would count as a request failure and abort
    // update 2 with no emit.
    f.worker->Update(TabSelection::Selected, {loc_a});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(3));
    f.rate_limiter.deliver(1, {}, QNetworkReply::TimeoutError);
    f.rate_limiter.deliver(2, fresh_list);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(4));
    f.rate_limiter.deliver(3,
                           stashBody(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1-after"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1-after"}));
}

// A partial update that includes a character must fetch only the selected
// characters and keep exactly one tab entry per character. The old
// skip-check compared names against an id-keyed index and never matched,
// so every character in the league was re-added and re-fetched (F48).
void WorkerUpdateTest::partialUpdateDoesNotDuplicateCharacters()
{
    WorkerFixture f("worker-update-account-3");

    const auto char_1 = json::readCharacter(
        characterJson("charid0001", "CharOne", QStringList{"c1-item"}));
    const auto char_2 = json::readCharacter(
        characterJson("charid0002", "CharTwo", QStringList{"c2-item"}));
    QVERIFY(char_1 && char_2);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-3");
        QVERIFY(store.characters().saveCharacterList({*char_1, *char_2}));
        QVERIFY(store.characters().saveCharacter(*char_1));
        QVERIFY(store.characters().saveCharacter(*char_2));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_tabs.size(), size_t(2));
    QCOMPARE(f.last_items.size(), size_t(2));

    // Refresh only CharOne. The fresh list mentions both characters.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*char_1, 0)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    QCOMPARE(f.rate_limiter.request(0).endpoint, QString("List Characters"));
    f.rate_limiter.deliver(0,
                           characterListBody({characterJson("charid0001", "CharOne"),
                                              characterJson("charid0002", "CharTwo")}));

    // Only the selected character is fetched.
    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    QCOMPARE(f.rate_limiter.request(1).endpoint, QString("Get Character"));
    QVERIFY(f.rate_limiter.request(1).request.url().path().endsWith("CharOne"));
    f.rate_limiter.deliver(1,
                           characterBody(
                               characterJson("charid0001", "CharOne", QStringList{"c1-item-new"})));

    QCOMPARE(f.refresh_count, 2);

    // Exactly one tab entry per character, and CharTwo's cached item intact.
    QCOMPARE(f.last_tabs.size(), size_t(2));
    QStringList tab_ids;
    for (const auto &tab : f.last_tabs) {
        tab_ids.append(tab.id());
    }
    tab_ids.sort();
    QCOMPARE(tab_ids, QStringList({"charid0001", "charid0002"}));
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"c1-item-new", "c2-item"}));
}

// Tab-list reconciliation gives every listed tab fresh metadata, even tabs
// outside the update selection: a tab renamed in-game updates its label on
// any partial refresh, with no extra fetch (the M1 behavior change that
// absorbed the F15 sketch), and its cached items are kept.
void WorkerUpdateTest::renamedTabMetadataRefreshesWithoutFetch()
{
    WorkerFixture f("worker-update-account-4");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Old Name", 0, QStringList{"a1"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    QVERIFY(tab_a && tab_b);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-4");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_items.size(), size_t(2));

    // Refresh only tab B; the fresh list shows tab A renamed in-game.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_b)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0,
                           stashListBody({stashJson("stashaaaa1", "New Name", 0),
                                          stashJson("stashbbbb1", "Tab B", 1)}));

    // Only tab B is fetched; the rename costs no extra request.
    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    QVERIFY(f.rate_limiter.request(1).request.url().path().endsWith("stashbbbb1"));
    f.rate_limiter.deliver(1, stashBody(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1-new"})));

    QCOMPARE(f.refresh_count, 2);

    // Tab A carries the new label and its cached item survived.
    const auto renamed = std::find_if(f.last_tabs.cbegin(),
                                      f.last_tabs.cend(),
                                      [](const ItemLocation &tab) {
                                          return tab.id() == "stashaaaa1";
                                      });
    QVERIFY(renamed != f.last_tabs.cend());
    QCOMPARE(renamed->tab_label(), QString("New Name"));
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "b1-new"}));
}

QTEST_GUILESS_MAIN(WorkerUpdateTest)

#include "tst_workerupdate.moc"
