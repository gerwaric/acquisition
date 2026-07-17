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
    void vanishedMapChildItemsAreRemovedOnParentRefresh();
    void failedUpdateDoesNotRebasePublishedLocations();
    void listedButNeverFetchedTabIsFetchedOnNextUpdate();
    void failedFirstFetchDoesNotConsumeNewness();
    void failedChildFetchKeepsParentNew();
    void cachedParentWithMissingChildRowStaysNew();
    void datastoreFollowsServerDeletions();
    void datastoreFollowsCharacterDeletions();
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
                         const std::optional<QStringList> &item_ids = {},
                         const QString &type = "PremiumStash",
                         const QString &parent = {},
                         const QList<QByteArray> &children = {})
    {
        QString json = QString(R"({"id":"%1","name":"%2","type":"%3","index":%4)")
                           .arg(id, name, type, QString::number(index));
        if (!parent.isEmpty()) {
            json += QString(R"(,"parent":"%1")").arg(parent);
        }
        if (item_ids) {
            json += QString(R"(,"items":[%1])").arg(joinItems(*item_ids));
        }
        if (!children.isEmpty()) {
            QByteArrayList list(children.cbegin(), children.cend());
            json += R"(,"children":[)" + list.join(",") + "]";
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

    // Wire the worker's persistence signals to a store the way
    // Application does, including the F53 reconciliation connections.
    void connectPersistence(ItemsManagerWorker *worker, UserStore &store)
    {
        auto stashes = &store.stashes();
        auto characters = &store.characters();
        QObject::connect(worker,
                         &ItemsManagerWorker::stashListReceived,
                         stashes,
                         &StashRepo::saveStashList);
        QObject::connect(worker, &ItemsManagerWorker::stashReceived, stashes, &StashRepo::saveStash);
        QObject::connect(worker,
                         &ItemsManagerWorker::characterListReceived,
                         characters,
                         &CharacterRepo::saveCharacterList);
        QObject::connect(worker,
                         &ItemsManagerWorker::characterReceived,
                         characters,
                         &CharacterRepo::saveCharacter);
        QObject::connect(worker,
                         &ItemsManagerWorker::stashListReplaced,
                         stashes,
                         &StashRepo::reconcileStashList);
        QObject::connect(worker,
                         &ItemsManagerWorker::characterListReplaced,
                         characters,
                         &CharacterRepo::reconcileCharacterList);
        QObject::connect(worker,
                         &ItemsManagerWorker::stashChildrenReplaced,
                         stashes,
                         &StashRepo::reconcileStashChildren);
    }

    QStringList storedStashIds(UserStore &store)
    {
        QStringList ids;
        for (const auto &stash : store.stashes().getStashList(kRealm, kLeague)) {
            ids.append(stash.id);
        }
        ids.sort();
        return ids;
    }

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

    // Refresh only CharOne. The fresh list mentions both characters, with
    // CharTwo renamed in-game (same id, new name).
    f.worker->Update(TabSelection::Selected, {ItemLocation(*char_1, 0)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    QCOMPARE(f.rate_limiter.request(0).endpoint, QString("List Characters"));
    f.rate_limiter.deliver(0,
                           characterListBody({characterJson("charid0001", "CharOne"),
                                              characterJson("charid0002", "CharTwoRenamed")}));

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

    // The unfetched character's surviving item carries the fresh name —
    // forum codes and sort order read the item's embedded location.
    const auto item_c2 = std::find_if(f.last_items.cbegin(),
                                      f.last_items.cend(),
                                      [](const std::shared_ptr<Item> &item) {
                                          return item->id() == "c2-item";
                                      });
    QVERIFY(item_c2 != f.last_items.cend());
    QCOMPARE((*item_c2)->location().character(), QString("CharTwoRenamed"));
}

// Tab-list reconciliation gives every listed tab fresh metadata, even tabs
// outside the update selection: a tab renamed or moved in-game updates its
// label and position on any partial refresh, with no extra fetch (the M1
// behavior change that absorbed the F15 sketch), and its cached items are
// kept. The surviving items' embedded locations are rebased too — search
// buckets are built from item locations, so a stale embedded copy would
// show the old name (and a moved tab would bucket twice, once per index).
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

    // Refresh only tab B; the fresh list shows tab A renamed and moved
    // in-game (index 0 -> 2).
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_b)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0,
                           stashListBody({stashJson("stashbbbb1", "Tab B", 1),
                                          stashJson("stashaaaa1", "New Name", 2)}));

    // Only tab B is fetched; the rename costs no extra request.
    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    QVERIFY(f.rate_limiter.request(1).request.url().path().endsWith("stashbbbb1"));
    f.rate_limiter.deliver(1, stashBody(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1-new"})));

    QCOMPARE(f.refresh_count, 2);

    // Tab A carries the new label and position, and its cached item survived.
    const auto renamed = std::find_if(f.last_tabs.cbegin(),
                                      f.last_tabs.cend(),
                                      [](const ItemLocation &tab) {
                                          return tab.id() == "stashaaaa1";
                                      });
    QVERIFY(renamed != f.last_tabs.cend());
    QCOMPARE(renamed->tab_label(), QString("New Name"));
    QCOMPARE(renamed->tab_index(), 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "b1-new"}));

    // The surviving item's embedded location was rebased to match.
    const auto item_a = std::find_if(f.last_items.cbegin(),
                                     f.last_items.cend(),
                                     [](const std::shared_ptr<Item> &item) {
                                         return item->id() == "a1";
                                     });
    QVERIFY(item_a != f.last_items.cend());
    QCOMPARE((*item_a)->location().tab_label(), QString("New Name"));
    QCOMPARE((*item_a)->location().tab_index(), 2);
}

// A parent's reply is authoritative for its children: cached items fetched
// from a Map/Unique child the parent no longer lists must be removed when
// the parent is refreshed. Without this reconciliation they would survive
// every refresh, even a full one — the per-reply replacement only erases
// fetch ids it is about to re-fetch, and the tab-list cleanup keys on the
// display id, which for child items is the (still-listed) parent.
void WorkerUpdateTest::vanishedMapChildItemsAreRemovedOnParentRefresh()
{
    WorkerFixture f("worker-update-account-5");
    f.settings.setValue("get_map_stashes", true);

    // Cached state: a MapStash whose child holds one item. Child stashes
    // are cached under their own ids with a parent reference, and are not
    // part of the stash list (matching the live API).
    const auto parent_tab = json::readStash(
        stashJson("mapstash01", "Maps", 0, QStringList{"p1"}, "MapStash"));
    const auto child_tab = json::readStash(
        stashJson("mapchild01", "Tier 1", 0, QStringList{"m1"}, "MapStash", "mapstash01"));
    QVERIFY(parent_tab && child_tab);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-5");
        QVERIFY(store.stashes().saveStashList({*parent_tab}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*parent_tab, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*child_tab, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"m1", "p1"}));
    QCOMPARE(f.last_tabs.size(), size_t(1));

    // Refresh the map stash. Its reply lists a different child: the old
    // child's cached item must go, the new child's contents arrive.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*parent_tab)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0, stashListBody({stashJson("mapstash01", "Maps", 0, {}, "MapStash")}));

    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    QVERIFY(f.rate_limiter.request(1).request.url().path().endsWith("mapstash01"));
    f.rate_limiter.deliver(
        1,
        stashBody(stashJson("mapstash01",
                            "Maps",
                            0,
                            QStringList{"p1-new"},
                            "MapStash",
                            {},
                            {stashJson("mapchild02", "Tier 2", 0, {}, "MapStash", "mapstash01")})));

    // The new child is fetched by its own id...
    QCOMPARE(f.rate_limiter.requestCount(), size_t(3));
    QVERIFY(f.rate_limiter.request(2).request.url().path().endsWith("mapchild02"));
    f.rate_limiter.deliver(2,
                           stashBody(stashJson("mapchild02",
                                               "Tier 2",
                                               0,
                                               QStringList{"m2"},
                                               "MapStash",
                                               "mapstash01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"m2", "p1-new"}));

    // ...and its items display under the parent tab.
    const auto item_m2 = std::find_if(f.last_items.cbegin(),
                                      f.last_items.cend(),
                                      [](const std::shared_ptr<Item> &item) {
                                          return item->id() == "m2";
                                      });
    QVERIFY(item_m2 != f.last_items.cend());
    QCOMPARE((*item_m2)->location().id(), QString("mapstash01"));
    QCOMPARE((*item_m2)->location().fetch_id(), QString("mapchild02"));
}

// The emitted Items share Item objects with ItemsManager and the UI, so
// rebasing surviving item locations may only happen when an update
// finishes successfully (in FinishUpdate, just before the emit). Rebasing
// at list receipt would mutate the already-published snapshot mid-update —
// and a terminal failure would leave it mutated indefinitely, with no emit
// to rebuild the search buckets around the new metadata.
void WorkerUpdateTest::failedUpdateDoesNotRebasePublishedLocations()
{
    WorkerFixture f("worker-update-account-6");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Old Name", 0, QStringList{"a1"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    QVERIFY(tab_a && tab_b);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-6");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    // Hold the published item the way ItemsManager and the UI do: a
    // shared_ptr into the emitted snapshot.
    std::shared_ptr<Item> item_a;
    for (const auto &item : f.last_items) {
        if (item->id() == "a1") {
            item_a = item;
        }
    }
    QVERIFY(item_a);

    const QByteArray fresh_list = stashListBody(
        {stashJson("stashaaaa1", "New Name", 0), stashJson("stashbbbb1", "Tab B", 1)});

    // Refresh only tab B; the fresh list renames tab A, then tab B's fetch
    // dies, terminating the update without an emit.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_b)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0, fresh_list);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    f.rate_limiter.deliver(1, {}, QNetworkReply::ConnectionRefusedError);
    QCOMPARE(f.refresh_count, 1);

    // The published snapshot is untouched by the failed update.
    QCOMPARE(item_a->location().tab_label(), QString("Old Name"));

    // A subsequent successful update applies the rename to the same shared
    // object the UI holds.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_b)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(3));
    f.rate_limiter.deliver(2, fresh_list);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(4));
    f.rate_limiter.deliver(3,
                           stashBody(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1-new"})));
    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(item_a->location().tab_label(), QString("New Name"));
}

// A tab whose metadata was cached but whose contents were never fetched —
// the durable footprint of an update that died between list receipt and
// the tab's first item request — must still count as "new" after a
// restart, so the next content update fetches it (F55).
void WorkerUpdateTest::listedButNeverFetchedTabIsFetchedOnNextUpdate()
{
    WorkerFixture f("worker-update-account-7");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    const auto tab_n = json::readStash(stashJson("stashnnnn1", "New Tab", 1));
    QVERIFY(tab_a && tab_n);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-7");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_n}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        // tab N: listed, never fetched.
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_tabs.size(), size_t(2));
    QCOMPARE(f.last_items.size(), size_t(1));

    // Refresh only tab A: tab N must be fetched anyway, because its
    // contents were never seen.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0,
                           stashListBody({stashJson("stashaaaa1", "Tab A", 0),
                                          stashJson("stashnnnn1", "New Tab", 1)}));

    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    QVERIFY(f.rate_limiter.request(1).request.url().path().endsWith("stashaaaa1"));
    f.rate_limiter.deliver(1, stashBody(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));

    QCOMPARE(f.rate_limiter.requestCount(), size_t(3));
    QVERIFY(f.rate_limiter.request(2).request.url().path().endsWith("stashnnnn1"));
    f.rate_limiter.deliver(2, stashBody(stashJson("stashnnnn1", "New Tab", 1, QStringList{"n1"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "n1"}));
}

// The ledger-specified F55 pin: an update that fails after list receipt
// but before a newly discovered tab's first reply must not consume the
// tab's newness — the next partial refresh still fetches it. (Before the
// fix, list receipt alone marked the tab previously-known, so a later
// partial refresh skipped it and published the tab empty.)
void WorkerUpdateTest::failedFirstFetchDoesNotConsumeNewness()
{
    WorkerFixture f("worker-update-account-8");

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    QVERIFY(tab_a);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-8");
        QVERIFY(store.stashes().saveStashList({*tab_a}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    const QByteArray list_with_new_tab = stashListBody(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("stashnnnn1", "New Tab", 1)});

    // Update 1: the list reveals new tab N; its first fetch dies.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0, list_with_new_tab);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    QVERIFY(f.rate_limiter.request(1).request.url().path().endsWith("stashaaaa1"));
    f.rate_limiter.deliver(1, stashBody(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    QCOMPARE(f.rate_limiter.requestCount(), size_t(3));
    QVERIFY(f.rate_limiter.request(2).request.url().path().endsWith("stashnnnn1"));
    f.rate_limiter.deliver(2, {}, QNetworkReply::ConnectionRefusedError);
    QCOMPARE(f.refresh_count, 1);

    // Update 2 selects only tab A again: tab N is still new and must be
    // fetched.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(4));
    f.rate_limiter.deliver(3, list_with_new_tab);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(5));
    QVERIFY(f.rate_limiter.request(4).request.url().path().endsWith("stashaaaa1"));
    f.rate_limiter.deliver(4, stashBody(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    QCOMPARE(f.rate_limiter.requestCount(), size_t(6));
    QVERIFY(f.rate_limiter.request(5).request.url().path().endsWith("stashnnnn1"));
    f.rate_limiter.deliver(5, stashBody(stashJson("stashnnnn1", "New Tab", 1, QStringList{"n1"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "n1"}));
}

// A Map/Unique parent's contents are complete only when every enabled
// child fetch lands: marking the parent known at its own reply would let
// a failed child fetch strand the children, since they never appear in a
// top-level list to be retried (F55, review follow-up).
void WorkerUpdateTest::failedChildFetchKeepsParentNew()
{
    WorkerFixture f("worker-update-account-11");
    f.settings.setValue("get_map_stashes", true);

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    QVERIFY(tab_a);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-11");
        QVERIFY(store.stashes().saveStashList({*tab_a}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);

    const QByteArray list_with_new_map = stashListBody(
        {stashJson("stashaaaa1", "Tab A", 0), stashJson("mapstash01", "Maps", 1, {}, "MapStash")});
    const QByteArray map_parent_body = stashBody(
        stashJson("mapstash01",
                  "Maps",
                  1,
                  QStringList{"p1"},
                  "MapStash",
                  {},
                  {stashJson("mapchild01", "Tier 1", 0, {}, "MapStash", "mapstash01")}));

    // Update 1: the list reveals new Map stash M; its parent fetch lands,
    // but the child fetch it queues dies.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0, list_with_new_map);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    f.rate_limiter.deliver(1, stashBody(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    QCOMPARE(f.rate_limiter.requestCount(), size_t(3));
    QVERIFY(f.rate_limiter.request(2).request.url().path().endsWith("mapstash01"));
    f.rate_limiter.deliver(2, map_parent_body);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(4));
    QVERIFY(f.rate_limiter.request(3).request.url().path().endsWith("mapchild01"));
    f.rate_limiter.deliver(3, {}, QNetworkReply::ConnectionRefusedError);
    QCOMPARE(f.refresh_count, 1);

    // Update 2 selects only tab A again: the parent is still incomplete,
    // so it is refetched and its child fetch retried.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(5));
    f.rate_limiter.deliver(4, list_with_new_map);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(6));
    f.rate_limiter.deliver(5, stashBody(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    QCOMPARE(f.rate_limiter.requestCount(), size_t(7));
    QVERIFY(f.rate_limiter.request(6).request.url().path().endsWith("mapstash01"));
    f.rate_limiter.deliver(6, map_parent_body);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(8));
    QVERIFY(f.rate_limiter.request(7).request.url().path().endsWith("mapchild01"));
    f.rate_limiter.deliver(7,
                           stashBody(stashJson("mapchild01",
                                               "Tier 1",
                                               0,
                                               QStringList{"m1"},
                                               "MapStash",
                                               "mapstash01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m1", "p1"}));
}

// The restart shape of the same gap: a cached Map parent whose reply
// recorded children must stay "new" at startup if a child row is missing,
// so the next content update refetches it.
void WorkerUpdateTest::cachedParentWithMissingChildRowStaysNew()
{
    WorkerFixture f("worker-update-account-12");
    f.settings.setValue("get_map_stashes", true);

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    const auto map_parent = json::readStash(
        stashJson("mapstash01",
                  "Maps",
                  1,
                  QStringList{"p1"},
                  "MapStash",
                  {},
                  {stashJson("mapchild01", "Tier 1", 0, {}, "MapStash", "mapstash01")}));
    QVERIFY(tab_a && map_parent);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-12");
        QVERIFY(store.stashes().saveStashList({*tab_a, *map_parent}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*map_parent, kRealm, kLeague));
        // The child row is missing: its fetch never landed before restart.
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "p1"}));

    // Refresh only tab A: the incomplete parent must be fetched anyway.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*tab_a)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0,
                           stashListBody({stashJson("stashaaaa1", "Tab A", 0),
                                          stashJson("mapstash01", "Maps", 1, {}, "MapStash")}));
    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    f.rate_limiter.deliver(1, stashBody(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));
    QCOMPARE(f.rate_limiter.requestCount(), size_t(3));
    QVERIFY(f.rate_limiter.request(2).request.url().path().endsWith("mapstash01"));
    f.rate_limiter.deliver(
        2,
        stashBody(stashJson("mapstash01",
                            "Maps",
                            1,
                            QStringList{"p1"},
                            "MapStash",
                            {},
                            {stashJson("mapchild01", "Tier 1", 0, {}, "MapStash", "mapstash01")})));
    QCOMPARE(f.rate_limiter.requestCount(), size_t(4));
    QVERIFY(f.rate_limiter.request(3).request.url().path().endsWith("mapchild01"));
    f.rate_limiter.deliver(3,
                           stashBody(stashJson("mapchild01",
                                               "Tier 1",
                                               0,
                                               QStringList{"m1"},
                                               "MapStash",
                                               "mapstash01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m1", "p1"}));
}

// End-to-end F53: with the persistence wiring in place, a fresh top-level
// list deletes the rows of server-deleted tabs (a surviving Map child row
// is untouched — it is never in any list), and a Map parent's reply
// replaces its child rows.
void WorkerUpdateTest::datastoreFollowsServerDeletions()
{
    WorkerFixture f("worker-update-account-9");
    f.settings.setValue("get_map_stashes", true);

    const auto tab_a = json::readStash(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"}));
    const auto tab_b = json::readStash(stashJson("stashbbbb1", "Tab B", 1, QStringList{"b1"}));
    const auto map_parent = json::readStash(
        stashJson("mapstash01", "Maps", 2, QStringList{"p1"}, "MapStash"));
    const auto map_child = json::readStash(
        stashJson("mapchild01", "Tier 1", 0, QStringList{"m1"}, "MapStash", "mapstash01"));
    QVERIFY(tab_a && tab_b && map_parent && map_child);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-9");
        QVERIFY(store.stashes().saveStashList({*tab_a, *tab_b, *map_parent}, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_a, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*tab_b, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*map_parent, kRealm, kLeague));
        QVERIFY(store.stashes().saveStash(*map_child, kRealm, kLeague));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "b1", "m1", "p1"}));

    UserStore store(QDir(f.dataDir()), "worker-update-account-9");
    connectPersistence(f.worker.get(), store);

    // Server-side, Tab B was deleted; the fresh list omits it.
    f.worker->Update(TabSelection::All);
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0,
                           stashListBody({stashJson("stashaaaa1", "Tab A", 0),
                                          stashJson("mapstash01", "Maps", 1, {}, "MapStash")}));

    // Tab B's row is gone at list receipt; the Map child row survives.
    QCOMPARE(storedStashIds(store), QStringList({"mapchild01", "mapstash01", "stashaaaa1"}));

    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    f.rate_limiter.deliver(1, R"({"characters":[]})");

    QCOMPARE(f.rate_limiter.requestCount(), size_t(3));
    QVERIFY(f.rate_limiter.request(2).request.url().path().endsWith("stashaaaa1"));
    f.rate_limiter.deliver(2, stashBody(stashJson("stashaaaa1", "Tab A", 0, QStringList{"a1"})));

    // The Map parent's reply lists a different child: the old child row is
    // replaced in the datastore.
    QCOMPARE(f.rate_limiter.requestCount(), size_t(4));
    QVERIFY(f.rate_limiter.request(3).request.url().path().endsWith("mapstash01"));
    f.rate_limiter.deliver(
        3,
        stashBody(stashJson("mapstash01",
                            "Maps",
                            1,
                            QStringList{"p1"},
                            "MapStash",
                            {},
                            {stashJson("mapchild02", "Tier 2", 0, {}, "MapStash", "mapstash01")})));
    QCOMPARE(storedStashIds(store), QStringList({"mapstash01", "stashaaaa1"}));

    QCOMPARE(f.rate_limiter.requestCount(), size_t(5));
    QVERIFY(f.rate_limiter.request(4).request.url().path().endsWith("mapchild02"));
    f.rate_limiter.deliver(4,
                           stashBody(stashJson("mapchild02",
                                               "Tier 2",
                                               0,
                                               QStringList{"m2"},
                                               "MapStash",
                                               "mapstash01")));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(storedStashIds(store), QStringList({"mapchild02", "mapstash01", "stashaaaa1"}));
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"a1", "m2", "p1"}));
}

// End-to-end F53 for characters: a fresh character list deletes the rows
// of server-deleted characters.
void WorkerUpdateTest::datastoreFollowsCharacterDeletions()
{
    WorkerFixture f("worker-update-account-10");

    const auto char_1 = json::readCharacter(
        characterJson("charid0001", "CharOne", QStringList{"c1-item"}));
    const auto char_2 = json::readCharacter(
        characterJson("charid0002", "CharTwo", QStringList{"c2-item"}));
    QVERIFY(char_1 && char_2);
    {
        UserStore store(QDir(f.dataDir()), "worker-update-account-10");
        QVERIFY(store.characters().saveCharacterList({*char_1, *char_2}));
        QVERIFY(store.characters().saveCharacter(*char_1));
        QVERIFY(store.characters().saveCharacter(*char_2));
    }

    f.start();
    QTRY_COMPARE_WITH_TIMEOUT(f.refresh_count, 1, 10000);
    QCOMPARE(f.last_items.size(), size_t(2));

    UserStore store(QDir(f.dataDir()), "worker-update-account-10");
    connectPersistence(f.worker.get(), store);

    // Server-side, CharTwo was deleted; the fresh list omits it.
    f.worker->Update(TabSelection::Selected, {ItemLocation(*char_1, 0)});
    QCOMPARE(f.rate_limiter.requestCount(), size_t(1));
    f.rate_limiter.deliver(0, characterListBody({characterJson("charid0001", "CharOne")}));

    QStringList ids;
    for (const auto &character : store.characters().getCharacterList(kRealm)) {
        ids.append(character.id);
    }
    QCOMPARE(ids, QStringList({"charid0001"}));

    QCOMPARE(f.rate_limiter.requestCount(), size_t(2));
    f.rate_limiter.deliver(1,
                           characterBody(
                               characterJson("charid0001", "CharOne", QStringList{"c1-item"})));

    QCOMPARE(f.refresh_count, 2);
    QCOMPARE(sortedItemIds(f.last_items), QStringList({"c1-item"}));
}

QTEST_GUILESS_MAIN(WorkerUpdateTest)

#include "tst_workerupdate.moc"
