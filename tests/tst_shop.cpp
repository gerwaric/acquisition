// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest/QtTest>

#include <QSettings>
#include <QStandardPaths>

#include <memory>

#include "fakenetwork.h"
#include "fakenetworkmanager.h"
#include "itemsmanager.h"
#include "poe/poeapiclient.h"
#include "shop.h"
#include "testfixtures.h"
#include "util/networkmanager.h"

class ShopFixture
{
public:
    ShopFixture()
    {
        // NetworkManager creates a disk cache in AppLocalDataLocation.
        QStandardPaths::setTestModeEnabled(true);
        settings = std::make_unique<QSettings>(buyoutFixture.tempDir.filePath("settings.ini"),
                                               QSettings::IniFormat);
        // Offline: the success path goes on to fetch the forum edit-thread
        // page, which against a real NetworkManager would be a live GET to
        // pathofexile.com from a unit test.
        networkManager = std::make_unique<FakeNetworkManager>();
        rateLimiter = std::make_unique<FakeRateLimiter>(*networkManager);
        api = std::make_unique<PoeApiClient>(*rateLimiter);
        itemsManager = std::make_unique<ItemsManager>(*settings,
                                                      *buyoutFixture.manager,
                                                      *buyoutFixture.data);
        shop = makeShop();
    }

    // A fresh Shop over the same datastore, to exercise the constructor's
    // thread-list load against whatever a previous Shop persisted.
    std::unique_ptr<Shop> makeShop()
    {
        return std::make_unique<Shop>(*settings,
                                      *networkManager,
                                      *api,
                                      *buyoutFixture.data,
                                      *itemsManager,
                                      *buyoutFixture.manager);
    }

    BuyoutManagerFixture buyoutFixture;
    std::unique_ptr<QSettings> settings;
    std::unique_ptr<FakeNetworkManager> networkManager;
    std::unique_ptr<FakeRateLimiter> rateLimiter;
    std::unique_ptr<PoeApiClient> api;
    std::unique_ptr<ItemsManager> itemsManager;
    std::unique_ptr<Shop> shop;
};

class ShopTest : public QObject
{
    Q_OBJECT

private slots:
    void freshDatastoreHasNoThreads();
    void settingAndClearingThreads();
    void threadsRoundTripThroughDatastore();
    void stashIndexFailureReleasesTheSubmitLock();
    void stashIndexSuccessProceedsToShopSubmission();
    void continuationDoesNotRunAfterShopDestruction();

    // Items-pipeline M2, D8: the single-active-job foundation (stage 2
    // front-load), plus the delta-dependent capture pins.
    void olderSubmissionCannotCleanNewerInput();
    void activeJobUnaffectedByLocalEdits();
    void shopSubmissionUsesCapturedSnapshot();
    void manualSubmissionRendersCapturedPublishedState();
};

namespace {

    // A Shop ready to submit: threads set and a POESESSID present, the two
    // preconditions SubmitShopToForum checks before it indexes anything.
    void armForSubmission(ShopFixture &fixture)
    {
        fixture.shop->SetThread({"123"});
        fixture.settings->setValue("session_id", "fake-poesessid");
        fixture.settings->setValue("account", "someone");
        fixture.settings->setValue("realm", "pc");
        fixture.settings->setValue("league", "Standard");
    }

    void drainEvents()
    {
        for (int i = 0; i < 20; ++i) {
            QCoreApplication::processEvents();
            QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        }
    }

    // Publish one priced stash item the way a refresh does, and return the
    // published shared_ptr so a test can edit its buyout afterwards.
    std::shared_ptr<Item> publishPricedItem(ShopFixture &fixture, const char *id, double chaos)
    {
        auto item = std::make_shared<Item>(makeTestItem(id));
        const Items items{item};
        const std::vector<ItemLocation> tabs{item->location()};
        fixture.itemsManager->OnItemsRefreshed(items, tabs, false);
        fixture.buyoutFixture.manager->Set(*item, makeChaosBuyout(chaos));
        return item;
    }

    // The legacy stash index reply for makeTestStashLocation's tab. The uid
    // the shop keys on is the first 10 characters of the id.
    constexpr const char *kStashIndexJson = R"({"tabs":[{"n":"Test Tab","i":0,"id":"stash00001"}]})";

    // A forum edit-thread page carrying exactly the CSRF hash input and the
    // title input Shop::OnEditPageFinished parses.
    const QByteArray kEditThreadPage
        = "<input type=\"hidden\" name=\"hash\" value=\"fakecsrfhash\">\n"
          "<input type=\"text\" name=\"title\" id=\"title\" "
          "onkeypress=\"return&#x20;event.keyCode&#x21;&#x3D;13\" value=\"My Shop\">";

    // Walk the active job's forum flow to completion: finish the edit-page
    // GET at m_sent index `get_index`, wait out the 500ms CSRF delay for the
    // POST, and finish the POST cleanly.
    void completeForumSubmission(ShopFixture &fixture, int get_index)
    {
        QCOMPARE(fixture.networkManager->sent(get_index).op, QNetworkAccessManager::GetOperation);
        fixture.networkManager->sent(get_index).reply->finish({},
                                                              200,
                                                              QNetworkReply::NoError,
                                                              kEditThreadPage);
        QTRY_COMPARE_WITH_TIMEOUT(fixture.networkManager->count(), get_index + 2, 5000);
        QCOMPARE(fixture.networkManager->sent(get_index + 1).op,
                 QNetworkAccessManager::PostOperation);
        fixture.networkManager->sent(get_index + 1)
            .reply->finish({}, 200, QNetworkReply::NoError, QByteArray("ok"));
        drainEvents();
    }

} // namespace

// F45: an empty "shop" key must load as no threads, not as one empty
// thread. Before the fix, QString("").split(";") produced {""}, so
// m_threads.empty() was never true and the no-threads warning in
// SubmitShopToForum was unreachable.
void ShopTest::freshDatastoreHasNoThreads()
{
    ShopFixture fixture;
    QVERIFY(fixture.shop->threads().isEmpty());
}

void ShopTest::settingAndClearingThreads()
{
    ShopFixture fixture;

    fixture.shop->SetThread({"123", "456"});
    QCOMPARE(fixture.shop->threads(), QStringList({"123", "456"}));

    fixture.shop->SetThread({});
    QVERIFY(fixture.shop->threads().isEmpty());
}

void ShopTest::threadsRoundTripThroughDatastore()
{
    ShopFixture fixture;

    fixture.shop->SetThread({"123", "456"});
    const auto reloaded = fixture.makeShop();
    QCOMPARE(reloaded->threads(), QStringList({"123", "456"}));

    fixture.shop->SetThread({});
    const auto cleared = fixture.makeShop();
    QVERIFY(cleared->threads().isEmpty());
}

void ShopTest::stashIndexFailureReleasesTheSubmitLock()
{
    // A classified failure must release the submit lock. Before the facade
    // this function open-coded a transport check, a status check, and a
    // parse check; now it branches once on a FetchError — and the pin that
    // matters is that the branch still resets m_submitting, or the shop
    // could never be submitted again without a restart.
    ShopFixture fixture;
    armForSubmission(fixture);

    fixture.shop->SubmitShopToForum();
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(1));
    QCOMPARE(fixture.rateLimiter->pendingFuture(0).endpoint,
             QString("https://www.pathofexile.com/character-window/get-stash-items"));

    fixture.rateLimiter->reject(0, RateLimit::FetchError::Kind::Http, "HTTP status 503");
    drainEvents();

    // The failure stops the update: nothing is sent to the forum.
    QCOMPARE(fixture.networkManager->count(), 0);

    // The lock is released: a second submission starts a new index request.
    fixture.shop->SubmitShopToForum();
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(2));
}

void ShopTest::stashIndexSuccessProceedsToShopSubmission()
{
    // The success path parses below the facade and carries on into the forum
    // submission, which goes through the NetworkManager rather than the
    // limiter. The forum GET is the pin that actually proves the continuation
    // RAN: the submission lock staying held would look identical if the
    // continuation had never executed at all.
    ShopFixture fixture;
    armForSubmission(fixture);

    fixture.shop->SubmitShopToForum(true);
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(1));
    QCOMPARE(fixture.networkManager->count(), 0);

    fixture.rateLimiter->resolve(0, R"({"tabs":[{"n":"one","i":0,"id":"0123456789abcdef"}]})");
    drainEvents();

    // Exactly one forum request: the edit-thread page for the single
    // configured thread, fetched for its CSRF token.
    QCOMPARE(fixture.networkManager->count(), 1);
    QCOMPARE(fixture.networkManager->sent(0).op, QNetworkAccessManager::GetOperation);
    QCOMPARE(fixture.networkManager->sent(0).request.url().toString(),
             QString("https://www.pathofexile.com/forum/edit-thread/123"));

    // And the lock is still held while that submission is in flight.
    fixture.shop->SubmitShopToForum(true);
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(1));
}

void ShopTest::continuationDoesNotRunAfterShopDestruction()
{
    // The continuation is context-bound to the Shop (R6-2). Destroying the
    // Shop while its index request is in flight must drop the continuation
    // rather than run a handler against freed memory — the lifetime
    // property the design calls out explicitly. Completing the promise
    // afterwards must be safe and must reach nothing.
    ShopFixture fixture;
    armForSubmission(fixture);

    fixture.shop->SubmitShopToForum();
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(1));

    fixture.shop.reset();
    drainEvents();

    // Completing after destruction is safe, and the handler never runs —
    // proved directly: a continuation that ran would have gone on to fetch
    // the forum edit-thread page, so zero requests is the evidence. (No
    // crash alone would not distinguish "dropped" from "ran and happened
    // not to fault".)
    fixture.rateLimiter->resolve(0, R"({"tabs":[{"n":"one","i":0,"id":"0123456789abcdef"}]})");
    drainEvents();
    QCOMPARE(fixture.networkManager->count(), 0);
    // The promise did settle, so the test cannot pass by never completing.
    QVERIFY(fixture.rateLimiter->pendingFuture(0).promise == nullptr);
}

// M2 D8 (stage 2 foundation): monotonic input/cache revisions replace the
// single outdated flag. Completing job N advances only N's revision — an
// input expired to N+1 while N was active stays dirty through N's successful
// completion, and only a job that captured N+1 renders it clean. The old
// flag reset (shop.cpp's unconditional m_shop_data_outdated = false on
// render) would have marked the newer input clean here.
void ShopTest::olderSubmissionCannotCleanNewerInput()
{
    ShopFixture fixture;
    armForSubmission(fixture);
    publishPricedItem(fixture, "item-a", 1);

    // Fresh shop: nothing rendered yet, so the cache is outdated.
    QVERIFY(fixture.shop->shop_data_outdated());

    // Job N captures the current input revision, then the input moves on
    // (a local edit path calls ExpireShopData) while N is still active.
    fixture.shop->SubmitShopToForum();
    fixture.shop->ExpireShopData();

    fixture.rateLimiter->resolve(0, kStashIndexJson);
    drainEvents();

    // N rendered and published the preview cache for its own revision; the
    // newer input revision is still dirty.
    QVERIFY(!fixture.shop->shop_data().isEmpty());
    QVERIFY(fixture.shop->shop_data_outdated());

    // Finishing N successfully still cannot clean the newer revision.
    completeForumSubmission(fixture, 0);
    QVERIFY(fixture.shop->shop_data_outdated());

    // A job that captures the newer revision renders it clean. (Same items,
    // so the unchanged-hash exit releases it without posting.)
    fixture.shop->SubmitShopToForum();
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(2));
    fixture.rateLimiter->resolve(1, kStashIndexJson);
    drainEvents();
    QVERIFY(!fixture.shop->shop_data_outdated());
}

// M2 D8/R6-4 (stage 2 foundation): the active job's capture is immutable —
// a buyout edit made while the job is in flight does not reach its rendered
// output; the edit follows in a later submission. Today's continuation read
// live buyout state after the index arrived, so the edit would have leaked
// into the post.
void ShopTest::activeJobUnaffectedByLocalEdits()
{
    ShopFixture fixture;
    armForSubmission(fixture);
    const auto item = publishPricedItem(fixture, "item-a", 1);

    // The job captures the 1-chaos buyout at request time.
    fixture.shop->SubmitShopToForum();

    // A local edit while the job is active: reprice and expire, the
    // MainWindow buyout-edit path.
    fixture.buyoutFixture.manager->Set(*item, makeChaosBuyout(2));
    fixture.shop->ExpireShopData();

    fixture.rateLimiter->resolve(0, kStashIndexJson);
    drainEvents();

    // The rendered output is the captured state, not the edited state.
    QVERIFY(!fixture.shop->shop_data().isEmpty());
    QVERIFY(fixture.shop->shop_data().first().contains(" 1 chaos"));
    QVERIFY(!fixture.shop->shop_data().first().contains(" 2 chaos"));

    // The job completes without re-rendering: the posted state is unchanged.
    completeForumSubmission(fixture, 0);
    QVERIFY(fixture.shop->shop_data().first().contains(" 1 chaos"));
}

// M2 D8/R2-1 (stage 2): submission input is captured by value at request
// time. Submission for update N is requested, the stash-index future is held,
// N+1 streams a delta that empties the published state — and the generated
// shop still reflects N's captured input. (Value capture also makes the
// rebase hazard moot: no shared Item object is retained, so a later
// FinishUpdate rebasing locations in place cannot reach the capture.)
void ShopTest::shopSubmissionUsesCapturedSnapshot()
{
    ShopFixture fixture;
    armForSubmission(fixture);
    const auto item = publishPricedItem(fixture, "item-a", 1);

    // Request submission for N: the capture happens now, the index is held.
    fixture.shop->SubmitShopToForum();
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(1));

    // N+1 streams a delta emptying item-a's fetch source: the published
    // state no longer holds any postable item.
    fixture.itemsManager->OnTabRefreshed(item->location(), {});
    QVERIFY(fixture.itemsManager->items().empty());

    // The index arrives; the job renders its capture, not the published
    // state — the shop still holds the captured item.
    fixture.rateLimiter->resolve(0, kStashIndexJson);
    drainEvents();
    QVERIFY(!fixture.shop->shop_data().isEmpty());
    QVERIFY(fixture.shop->shop_data().first().contains(" 1 chaos"));
    QVERIFY(fixture.shop->shop_data().first().contains("linkItem"));
}

// M2 D8/R3-1 (stage 2): a forced manual request during an item refresh
// renders its captured published state even when the preview cache's
// revision still appears current — deltas deliberately do not advance the
// input revision (R4-2), so the old cached-data shortcut would have posted
// pre-delta text.
void ShopTest::manualSubmissionRendersCapturedPublishedState()
{
    ShopFixture fixture;
    armForSubmission(fixture);
    const auto item_a = publishPricedItem(fixture, "item-a", 1);

    // A completed submission leaves a preview cache rendered from the
    // current input revision — the cache "appears current".
    fixture.shop->SubmitShopToForum();
    fixture.rateLimiter->resolve(0, kStashIndexJson);
    drainEvents();
    completeForumSubmission(fixture, 0);
    QVERIFY(!fixture.shop->shop_data_outdated());
    QVERIFY(fixture.shop->shop_data().first().contains(" 1 chaos"));

    // A streamed delta replaces the published item with a repriced one; the
    // cache revision still appears current.
    auto item_b = std::make_shared<Item>(makeTestItem("item-b"));
    fixture.itemsManager->OnTabRefreshed(item_a->location(), {item_b});
    fixture.buyoutFixture.manager->Set(*item_b, makeChaosBuyout(2));
    QVERIFY(!fixture.shop->shop_data_outdated());

    // A forced manual submission renders its own capture of the current
    // published state instead of reusing the cached pre-delta text.
    fixture.shop->SubmitShopToForum(true);
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(2));
    fixture.rateLimiter->resolve(1, kStashIndexJson);
    drainEvents();
    QVERIFY(fixture.shop->shop_data().first().contains(" 2 chaos"));
    QVERIFY(!fixture.shop->shop_data().first().contains(" 1 chaos"));
}

QTEST_MAIN(ShopTest)

#include "tst_shop.moc"
