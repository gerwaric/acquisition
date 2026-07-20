// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest/QtTest>

#include <QSettings>
#include <QStandardPaths>

#include <memory>

#include "fakenetwork.h"
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
        networkManager = std::make_unique<NetworkManager>();
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
    std::unique_ptr<NetworkManager> networkManager;
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

    // The lock is released: a second submission starts a new index request.
    fixture.shop->SubmitShopToForum();
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(2));
}

void ShopTest::stashIndexSuccessProceedsToShopSubmission()
{
    // The success path parses below the facade and carries on into the
    // forum submission, which does not go through the limiter — so the
    // observable is that the lock is still HELD (no new index request),
    // the exact opposite of the failure case above.
    ShopFixture fixture;
    armForSubmission(fixture);

    fixture.shop->SubmitShopToForum(true);
    QCOMPARE(fixture.rateLimiter->futureCount(), size_t(1));

    fixture.rateLimiter->resolve(0, R"({"tabs":[{"n":"one","i":0,"id":"0123456789abcdef"}]})");
    drainEvents();

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

    // Completing after destruction is safe; the handler never runs.
    fixture.rateLimiter->resolve(0, R"({"tabs":[{"n":"one","i":0,"id":"0123456789abcdef"}]})");
    drainEvents();
    // Reaching here without a crash IS the assertion; assert the promise
    // did settle so the test cannot pass by never completing at all.
    QVERIFY(fixture.rateLimiter->pendingFuture(0).promise == nullptr);
}

QTEST_MAIN(ShopTest)

#include "tst_shop.moc"
