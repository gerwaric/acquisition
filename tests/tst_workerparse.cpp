#include <QtTest/QtTest>

#include <QSettings>

#include <glaze/glaze.hpp>

#include "datastore/stashrepo.h"
#include "datastore/userstore.h"
#include "itemsmanagerworker.h"
#include "poe/types/item.h"
#include "ratelimit/ratelimiter.h"
#include "testfixtures.h"
#include "util/glaze_qt.h" // IWYU pragma: keep
#include "util/networkmanager.h"

class WorkerParseTest : public QObject
{
    Q_OBJECT

private slots:
    void parsesCachedStashItems();
};

static poe::Item makePoeItem(const char *id)
{
    const QByteArray json = QString(R"({
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
                                .arg(id)
                                .toUtf8();
    auto item = glz::read_json<poe::Item>(std::string_view(json.constData(), json.size()));
    Q_ASSERT(item);
    return *item;
}

void WorkerParseTest::parsesCachedStashItems()
{
    BuyoutManagerFixture fixture;
    const QString account = "worker-parse-account";
    const QString realm = "pc";
    const QString league = "Worker Parse League";
    const QString dataDir = fixture.tempDir.filePath("data");

    poe::StashTab stash;
    stash.id = "stash00001";
    stash.name = "Parse Test";
    stash.type = "PremiumStash";
    stash.index = 0;
    stash.items = std::vector<poe::Item>{makePoeItem("worker-parse-item")};

    {
        UserStore store(QDir(dataDir), account);
        QVERIFY(store.stashes().saveStashList({stash}, realm, league));
        QVERIFY(store.stashes().saveStash(stash, realm, league));
    }

    QSettings settings(fixture.tempDir.filePath("settings.ini"), QSettings::IniFormat);
    settings.setValue("account", account);
    settings.setValue("realm", realm);
    settings.setValue("league", league);
    settings.sync();

    NetworkManager network;
    RateLimiter rateLimiter(network);
    ItemsManagerWorker worker(settings, *fixture.manager, rateLimiter);

    const ParseResult result = worker.ParseCachedItems(dataDir);

    QCOMPARE(result.tabs.size(), 1);
    QCOMPARE(result.tabs[0].id(), stash.id);
    QCOMPARE(result.tabs[0].tab_label(), stash.name);
    QVERIFY(result.tab_id_index.contains(stash.id));
    QCOMPARE(result.items.size(), 1);
    QCOMPARE(result.items[0]->id(), QString("worker-parse-item"));
    QCOMPARE(result.items[0]->location().id(), stash.id);
}

QTEST_GUILESS_MAIN(WorkerParseTest)

#include "tst_workerparse.moc"
