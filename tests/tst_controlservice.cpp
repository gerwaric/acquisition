#include <QJsonArray>
#include <QJsonDocument>
#include <QSettings>
#include <QtTest>

#include "control/controlservice.h"
#include "itemcategories.h"
#include "itemsmanager.h"
#include "testfixtures.h"

class ControlServiceTest : public QObject
{
    Q_OBJECT
private slots:
    void initTestCase();
    void reportsPreLoginStatus();
    void rejectsUnknownCommand();
    void viewingRequiresReadySession();
    void viewsPublishedItemsAndEffectivePrices();
    void tabsArePaginated();
    void itemLookupTracksPublishedDeltas();
    void itemLookupSurvivesDestinationFirstMove();
    void paginationRejectsOldRevision();
    void filteredPagesBoundSourceScan();
    void pagesBeforeResponseBudget();
    void rejectsOversizedItemProjection();
    void rejectsMalformedCursor();
    void rejectsMalformedQueryParameters();
    void refreshJobOutlivesStartRequest();
    void statusPreservesLatestTerminalRefresh();
    void queuedJobIgnoresUnrelatedSignals();
    void busyRefreshIsNotAccepted();
    void refreshOutcomesRemainTyped();
    void refreshHistoryIsBounded();
};

namespace {

    struct ViewingFixture
    {
        ViewingFixture()
            : settings(buyouts.tempDir.filePath("settings.ini"), QSettings::IniFormat)
            , items(settings, *buyouts.manager, *buyouts.data)
            , service("test-version")
        {
            location = makeTestStashLocation("stash-view", "Viewing Tab", 4);
            location.setFetchId(location.id());
            // Create the fixture items under this fixture's tab.
            const QByteArray first_json = R"({
                "w":1,"h":1,"id":"item-one","name":"Doom Grip",
                "typeLine":"Amethyst Ring","identified":true,"ilvl":84,"x":3,"y":7,"frameType":2,
                "synthesised":true,"fractured":true,
                "properties":[{"name":"Quality: {0}","displayMode":3,"values":[["+20%",1]]}],
                "requirements":[{"name":"Level","displayMode":0,"values":[["84",2]]}]
            })";
            const QByteArray second_json = R"({
                "w":1,"h":1,"id":"item-two","name":"",
                "typeLine":"Chaos Orb","identified":true,"ilvl":1,"x":1,"y":2,"note":""
            })";
            first = std::make_shared<Item>(makeTestItem(first_json.constData(), location));
            second = std::make_shared<Item>(makeTestItem(second_json.constData(), location));

            service.AttachSession(items, nullptr, *buyouts.manager, "Account#1", "League");
            service.ConfigureRefresh([this] { return readiness; }, [this] { ++starts; });
            items.OnItemsRefreshed(Items{first, second}, {location}, true);
            buyouts.manager->Set(*first, makeChaosBuyout(10));
        }

        BuyoutManagerFixture buyouts;
        QSettings settings;
        ItemsManager items;
        control::ControlService::RefreshReadiness readiness{
            control::ControlService::RefreshReadiness::Ready};
        int starts{0};
        control::ControlService service;
        ItemLocation location;
        std::shared_ptr<Item> first;
        std::shared_ptr<Item> second;
    };

    QJsonObject resultOf(const QJsonObject &response)
    {
        return response.value("result").toObject();
    }

} // namespace

void ControlServiceTest::initTestCase()
{
    InitItemClasses(R"json({"TestClass":{"name":"Weapons"}})json");
    InitItemBaseTypes(
        R"json({"Metadata/Items/TestSword":{"item_class":"TestClass","name":"Test Sword","release_state":"released"}})json");
}

void ControlServiceTest::reportsPreLoginStatus()
{
    control::ControlService service("test-version");
    service.SetNeedsLogin();
    const QJsonObject response = service.Handle(control::Request{"request", "status", {}});

    QVERIFY(response.value("ok").toBool());
    QCOMPARE(response.value("request_id").toString(), "request");
    const QJsonObject result = resultOf(response);
    QCOMPARE(result.value("application_version").toString(), "test-version");
    QCOMPARE(result.value("service_state").toString(), "needs_login");
    QCOMPARE(result.value("refresh_state").toString(), "unavailable");
    QVERIFY(!result.value("instance_id").toString().isEmpty());
}

void ControlServiceTest::rejectsUnknownCommand()
{
    control::ControlService service("test-version");
    const QJsonObject response = service.Handle(control::Request{"request", "nope", {}});

    QVERIFY(!response.value("ok").toBool(true));
    QCOMPARE(response.value("error").toObject().value("code").toString(), "unknown_command");
}

void ControlServiceTest::viewingRequiresReadySession()
{
    control::ControlService service("test-version");
    service.SetNeedsLogin();
    const QJsonObject response = service.Handle(control::Request{"request", "tabs", {}});
    QVERIFY(!response.value("ok").toBool(true));
    QCOMPARE(response.value("error").toObject().value("code").toString(), "not_ready");
}

void ControlServiceTest::viewsPublishedItemsAndEffectivePrices()
{
    ViewingFixture fixture;

    const QJsonObject status = resultOf(
        fixture.service.Handle(control::Request{"status", "status", {}}));
    QCOMPARE(status.value("service_state").toString(), "ready");
    QCOMPARE(status.value("item_count").toInt(), 2);

    const QJsonObject tabs_response = fixture.service.Handle(
        control::Request{"tabs", "tabs", {}});
    QVERIFY(tabs_response.value("ok").toBool());
    const QJsonArray tabs = resultOf(tabs_response).value("tabs").toArray();
    QCOMPARE(tabs.size(), 1);
    QCOMPARE(tabs.at(0).toObject().value("id").toString(), "stash-view");
    QCOMPARE(tabs.at(0).toObject().value("item_count").toInt(), 2);

    const QJsonObject item_response = fixture.service.Handle(
        control::Request{"item", "item", QJsonObject{{"id", "item-one"}}});
    QVERIFY(item_response.value("ok").toBool());
    const QJsonObject item = resultOf(item_response).value("item").toObject();
    QCOMPARE(item.value("name").toString(), "Doom Grip");
    QCOMPARE(item.value("item_level").toInt(), 84);
    QCOMPARE(item.value("frame_type").toString(), "rare");
    QVERIFY(item.value("flags").toObject().value("synthesized").toBool());
    QVERIFY(item.value("flags").toObject().value("fractured").toBool());
    QVERIFY(!item.value("influences").toArray().contains("synthesized"));
    QVERIFY(!item.value("influences").toArray().contains("fractured"));
    QCOMPARE(item.value("location").toObject().value("tab_label").toString(), "Viewing Tab");
    const QJsonObject price = item.value("effective_price").toObject();
    QCOMPARE(price.value("value").toDouble(), 10.0);
    QCOMPARE(price.value("currency").toString(), "chaos");
    QVERIFY(item.value("note").isNull());
    const QJsonObject property = item.value("properties").toArray().at(0).toObject();
    QCOMPARE(property.value("display_mode").toString(), "indexed");
    QCOMPARE(property.value("values").toArray().at(0).toObject().value("type").toString(),
             "augmented");
    const QJsonObject requirement = item.value("requirements").toArray().at(0).toObject();
    QCOMPARE(requirement.value("value").toObject().value("type").toString(), "unmet");

    const QJsonObject second_response = fixture.service.Handle(
        control::Request{"second", "item", QJsonObject{{"id", "item-two"}}});
    const QJsonValue empty_note = resultOf(second_response).value("item").toObject().value("note");
    QVERIFY(empty_note.isString());
    QVERIFY(empty_note.toString().isEmpty());
}

void ControlServiceTest::tabsArePaginated()
{
    ViewingFixture fixture;
    const ItemLocation second_location = makeTestStashLocation("second", "Second", 5);
    const ItemLocation third_location = makeTestStashLocation("third", "Third", 6);
    fixture.items.OnItemsRefreshed(Items{fixture.first, fixture.second},
                                   {fixture.location, second_location, third_location},
                                   false);

    const QJsonObject first = fixture.service.Handle(
        control::Request{"first", "tabs", QJsonObject{{"limit", 1}}});
    QVERIFY(first.value("ok").toBool());
    QCOMPARE(resultOf(first).value("tabs").toArray().size(), 1);
    const QString cursor = resultOf(first).value("next_cursor").toString();
    QVERIFY(!cursor.isEmpty());

    const QJsonObject second = fixture.service.Handle(
        control::Request{"second", "tabs", QJsonObject{{"cursor", cursor}}});
    QVERIFY(second.value("ok").toBool());
    QCOMPARE(resultOf(second).value("tabs").toArray().size(), 1);
    QVERIFY(!resultOf(second).value("next_cursor").toString().isEmpty());

    QString tampered = cursor;
    tampered[tampered.size() - 1]
        = tampered.back() == QChar('0') ? QChar('1') : QChar('0');
    const QJsonObject rejected = fixture.service.Handle(
        control::Request{"tampered", "tabs", QJsonObject{{"cursor", tampered}}});
    QCOMPARE(rejected.value("error").toObject().value("code").toString(),
             "invalid_cursor");
}

void ControlServiceTest::itemLookupTracksPublishedDeltas()
{
    ViewingFixture fixture;
    const auto replacement = std::make_shared<Item>(makeTestItem(
        R"({"w":1,"h":1,"id":"replacement","typeLine":"Orb","identified":true})",
        fixture.location));
    fixture.items.OnTabRefreshed(fixture.location, Items{replacement});

    const QJsonObject removed = fixture.service.Handle(
        control::Request{"removed", "item", QJsonObject{{"id", "item-one"}}});
    QCOMPARE(removed.value("error").toObject().value("code").toString(), "item_not_found");
    const QJsonObject added = fixture.service.Handle(
        control::Request{"added", "item", QJsonObject{{"id", "replacement"}}});
    QVERIFY(added.value("ok").toBool());

    ItemLocation child = makeTestStashLocation("parent", "Parent", 2);
    child.setFetchId("child-source");
    const auto child_item = std::make_shared<Item>(makeTestItem(
        R"({"w":1,"h":1,"id":"child-item","typeLine":"Orb","identified":true})",
        child));
    fixture.items.OnTabRefreshed(child, Items{child_item});
    QVERIFY(fixture.service
                .Handle(control::Request{
                    "child", "item", QJsonObject{{"id", "child-item"}}})
                .value("ok")
                .toBool());
    fixture.items.OnChildrenReconciled(child, {});
    const QJsonObject removed_child = fixture.service.Handle(
        control::Request{"removed-child", "item", QJsonObject{{"id", "child-item"}}});
    QCOMPARE(removed_child.value("error").toObject().value("code").toString(),
             "item_not_found");
}

void ControlServiceTest::itemLookupSurvivesDestinationFirstMove()
{
    ViewingFixture fixture;
    ItemLocation destination = makeTestStashLocation("destination", "Destination", 5);
    destination.setFetchId(destination.id());
    const auto moved = std::make_shared<Item>(makeTestItem(
        R"({"w":1,"h":1,"id":"item-one","typeLine":"Ring","identified":true})",
        destination));

    fixture.items.OnTabRefreshed(destination, Items{moved});
    fixture.items.OnTabRefreshed(fixture.location, Items{fixture.second});

    const QJsonObject response = fixture.service.Handle(
        control::Request{"moved", "item", QJsonObject{{"id", "item-one"}}});
    QVERIFY(response.value("ok").toBool());
    QCOMPARE(resultOf(response)
                 .value("item")
                 .toObject()
                 .value("location")
                 .toObject()
                 .value("id")
                 .toString(),
             "destination");

    ViewingFixture reverse_fixture;
    const auto reverse_moved = std::make_shared<Item>(makeTestItem(
        R"({"w":1,"h":1,"id":"item-one","typeLine":"Ring","identified":true})",
        destination));
    reverse_fixture.items.OnTabRefreshed(destination, Items{reverse_moved});
    reverse_fixture.items.OnTabRefreshed(destination, {});
    const QJsonObject restored = reverse_fixture.service.Handle(
        control::Request{"restored", "item", QJsonObject{{"id", "item-one"}}});
    QVERIFY(restored.value("ok").toBool());
    QCOMPARE(resultOf(restored)
                 .value("item")
                 .toObject()
                 .value("location")
                 .toObject()
                 .value("id")
                 .toString(),
             "stash-view");
}

void ControlServiceTest::paginationRejectsOldRevision()
{
    ViewingFixture fixture;
    const QJsonObject first_page = fixture.service.Handle(
        control::Request{"page", "items", QJsonObject{{"limit", 1}}});
    QVERIFY(first_page.value("ok").toBool());
    const QString cursor = resultOf(first_page).value("next_cursor").toString();
    QVERIFY(!cursor.isEmpty());
    QCOMPARE(resultOf(first_page).value("items").toArray().size(), 1);

    QString tampered = cursor;
    tampered[tampered.size() - 1]
        = tampered.back() == QChar('0') ? QChar('1') : QChar('0');
    const QJsonObject rejected = fixture.service.Handle(
        control::Request{"tampered", "items", QJsonObject{{"cursor", tampered}}});
    QCOMPARE(rejected.value("error").toObject().value("code").toString(),
             "invalid_cursor");

    fixture.buyouts.manager->Set(*fixture.first, makeChaosBuyout(11));
    const QJsonObject stale = fixture.service.Handle(
        control::Request{"stale", "items", QJsonObject{{"cursor", cursor}}});
    QVERIFY(!stale.value("ok").toBool(true));
    QCOMPARE(stale.value("error").toObject().value("code").toString(), "revision_changed");
}

void ControlServiceTest::filteredPagesBoundSourceScan()
{
    BuyoutManagerFixture buyouts;
    QSettings settings(buyouts.tempDir.filePath("settings.ini"), QSettings::IniFormat);
    ItemsManager items_manager(settings, *buyouts.manager, *buyouts.data);
    control::ControlService service("test-version");
    const ItemLocation first_location = makeTestStashLocation("first", "First", 0);
    const ItemLocation target_location = makeTestStashLocation("target", "Target", 1);
    Items items;
    items.reserve(10'001);
    for (int index = 0; index < 10'000; ++index) {
        const QByteArray json = QString(
                                    R"({"w":1,"h":1,"id":"item-%1","typeLine":"Orb","identified":true})")
                                    .arg(index)
                                    .toUtf8();
        items.push_back(std::make_shared<Item>(makeTestItem(json.constData(), first_location)));
    }
    items.push_back(std::make_shared<Item>(makeTestItem(
        R"({"w":1,"h":1,"id":"target-item","typeLine":"Orb","identified":true})",
        target_location)));
    service.AttachSession(items_manager, nullptr, *buyouts.manager, "Account#1", "League");
    items_manager.OnItemsRefreshed(std::move(items), {first_location, target_location}, true);

    const QJsonObject first_page = service.Handle(control::Request{
        "first-page",
        "items",
        QJsonObject{{"limit", 1}, {"tab_id", "target"}, {"kind", "stash"}}});
    QVERIFY(first_page.value("ok").toBool());
    QCOMPARE(resultOf(first_page).value("items").toArray().size(), 0);
    const QString cursor = resultOf(first_page).value("next_cursor").toString();
    QVERIFY(!cursor.isEmpty());

    const QJsonObject second_page = service.Handle(
        control::Request{"second-page", "items", QJsonObject{{"cursor", cursor}}});
    QCOMPARE(resultOf(second_page).value("items").toArray().size(), 1);
    QCOMPARE(resultOf(second_page)
                 .value("items")
                 .toArray()
                 .at(0)
                 .toObject()
                 .value("id")
                 .toString(),
             "target-item");
}

void ControlServiceTest::pagesBeforeResponseBudget()
{
    BuyoutManagerFixture buyouts;
    QSettings settings(buyouts.tempDir.filePath("settings.ini"), QSettings::IniFormat);
    ItemsManager items_manager(settings, *buyouts.manager, *buyouts.data);
    control::ControlService service("test-version");
    const ItemLocation location = makeTestStashLocation("large", "Large", 0);
    Items items;
    for (int index = 0; index < 3; ++index) {
        const QByteArray json = QJsonDocument(
                                    QJsonObject{{"w", 1},
                                                {"h", 1},
                                                {"id", QString("large-%1").arg(index)},
                                                {"typeLine", "Orb"},
                                                {"identified", true},
                                                {"note", QString(1'500'000, 'x')}})
                                    .toJson(QJsonDocument::Compact);
        items.push_back(std::make_shared<Item>(makeTestItem(json.constData(), location)));
    }
    service.AttachSession(items_manager, nullptr, *buyouts.manager, "Account#1", "League");
    items_manager.OnItemsRefreshed(std::move(items), {location}, true);

    const QJsonObject first = service.Handle(
        control::Request{"first", "items", QJsonObject{{"limit", 3}}});
    QVERIFY(first.value("ok").toBool());
    QCOMPARE(resultOf(first).value("items").toArray().size(), 2);
    const QString cursor = resultOf(first).value("next_cursor").toString();
    QVERIFY(!cursor.isEmpty());

    const QJsonObject second = service.Handle(
        control::Request{"second", "items", QJsonObject{{"cursor", cursor}}});
    QVERIFY(second.value("ok").toBool());
    QCOMPARE(resultOf(second).value("items").toArray().size(), 1);
    QVERIFY(resultOf(second).value("next_cursor").isNull());
}

void ControlServiceTest::rejectsOversizedItemProjection()
{
    BuyoutManagerFixture buyouts;
    QSettings settings(buyouts.tempDir.filePath("settings.ini"), QSettings::IniFormat);
    ItemsManager items_manager(settings, *buyouts.manager, *buyouts.data);
    control::ControlService service("test-version");
    const ItemLocation location = makeTestStashLocation("oversized", "Oversized", 0);
    const QByteArray json = QJsonDocument(
                                QJsonObject{{"w", 1},
                                            {"h", 1},
                                            {"id", "oversized-item"},
                                            {"typeLine", "Orb"},
                                            {"identified", true},
                                            {"note", QString(5'000'000, 'x')}})
                                .toJson(QJsonDocument::Compact);
    auto item = std::make_shared<Item>(makeTestItem(json.constData(), location));
    service.AttachSession(items_manager, nullptr, *buyouts.manager, "Account#1", "League");
    items_manager.OnItemsRefreshed(Items{item}, {location}, true);

    const QJsonObject response = service.Handle(
        control::Request{"item", "item", QJsonObject{{"id", "oversized-item"}}});
    QVERIFY(!response.value("ok").toBool(true));
    QCOMPARE(response.value("error").toObject().value("code").toString(),
             "item_too_large");
}

void ControlServiceTest::rejectsMalformedCursor()
{
    ViewingFixture fixture;
    const QJsonObject status = resultOf(
        fixture.service.Handle(control::Request{"status", "status", {}}));
    const QJsonObject cursor_object{{"instance_id", status.value("instance_id").toString()},
                                    {"revision", status.value("inventory_revision").toString()},
                                    {"source_kind", "stash"},
                                    {"source_id", "stash-view"},
                                    {"offset", "0"},
                                    {"limit", 1},
                                    {"tab_id", "stash-view"},
                                    {"kind", ""}};
    const QString cursor = QString::fromLatin1(
        QJsonDocument(cursor_object)
            .toJson(QJsonDocument::Compact)
            .toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals));
    const QJsonObject response = fixture.service.Handle(
        control::Request{"bad-cursor", "items", QJsonObject{{"cursor", cursor}}});
    QVERIFY(!response.value("ok").toBool(true));
    QCOMPARE(response.value("error").toObject().value("code").toString(), "invalid_cursor");

}

void ControlServiceTest::rejectsMalformedQueryParameters()
{
    ViewingFixture fixture;
    const QJsonObject empty_cursor = fixture.service.Handle(
        control::Request{"empty", "items", QJsonObject{{"cursor", ""}}});
    QCOMPARE(empty_cursor.value("error").toObject().value("code").toString(), "invalid_cursor");

    const QJsonObject numeric_tab = fixture.service.Handle(
        control::Request{"tab", "items", QJsonObject{{"tab_id", 123}}});
    QCOMPARE(numeric_tab.value("error").toObject().value("code").toString(),
             "invalid_request");
}

void ControlServiceTest::refreshJobOutlivesStartRequest()
{
    ViewingFixture fixture;
    const QJsonObject started = fixture.service.Handle(
        control::Request{"start", "refresh.start", {}});
    QVERIFY(started.value("ok").toBool());
    const QJsonObject start_result = resultOf(started);
    QVERIFY(start_result.value("accepted").toBool());
    const QString operation_id = start_result.value("operation_id").toString();
    QVERIFY(!operation_id.isEmpty());
    QCOMPARE(operation_id, "start");
    QCOMPARE(fixture.starts, 0);

    const QJsonObject duplicate = fixture.service.Handle(
        control::Request{"start", "refresh.start", {}});
    QVERIFY(resultOf(duplicate).value("accepted").toBool());
    QCOMPARE(resultOf(duplicate).value("operation_id").toString(), operation_id);
    QCOMPARE(fixture.starts, 0);

    QCoreApplication::processEvents();
    QCOMPARE(fixture.starts, 1);
    fixture.items.OnStatusUpdate(ProgramState::Busy, "Received 1/2 stash tabs");

    const auto status_request = control::Request{
        "job", "refresh.status", QJsonObject{{"operation_id", operation_id}}};
    QJsonObject operation = resultOf(fixture.service.Handle(status_request))
                                .value("operation")
                                .toObject();
    QCOMPARE(operation.value("state").toString(), "running");
    QCOMPARE(operation.value("progress").toString(), "Received 1/2 stash tabs");

    fixture.items.RefreshFinished(RefreshOutcome{CompletedRefresh{}});
    operation = resultOf(fixture.service.Handle(status_request)).value("operation").toObject();
    QCOMPARE(operation.value("state").toString(), "completed");
    QVERIFY(operation.value("outcome").toObject().value("clean").toBool());
}

void ControlServiceTest::statusPreservesLatestTerminalRefresh()
{
    ViewingFixture fixture;
    const QString completed_id = resultOf(fixture.service.Handle(
                                             control::Request{"first", "refresh.start", {}}))
                                     .value("operation_id")
                                     .toString();
    QCoreApplication::processEvents();
    fixture.items.RefreshFinished(RefreshOutcome{CompletedRefresh{}});

    const QString active_id = resultOf(fixture.service.Handle(
                                          control::Request{"second", "refresh.start", {}}))
                                  .value("operation_id")
                                  .toString();
    const QJsonObject status = resultOf(
        fixture.service.Handle(control::Request{"status", "status", {}}));
    QCOMPARE(status.value("active_refresh_id").toString(), active_id);
    QCOMPARE(status.value("latest_refresh").toObject().value("operation_id").toString(),
             completed_id);
    QCOMPARE(status.value("latest_refresh").toObject().value("state").toString(), "completed");
}

void ControlServiceTest::queuedJobIgnoresUnrelatedSignals()
{
    ViewingFixture fixture;
    const QJsonObject started = fixture.service.Handle(
        control::Request{"start", "refresh.start", {}});
    const QString id = resultOf(started).value("operation_id").toString();

    fixture.items.OnStatusUpdate(ProgramState::Busy, "unrelated progress");
    fixture.items.RefreshFinished(RefreshOutcome{CompletedRefresh{}});
    QCoreApplication::processEvents();
    QCOMPARE(fixture.starts, 1);

    const QJsonObject operation = resultOf(fixture.service.Handle(control::Request{
        "job", "refresh.status", QJsonObject{{"operation_id", id}}}))
                                      .value("operation")
                                      .toObject();
    QCOMPARE(operation.value("state").toString(), "running");
    QVERIFY(operation.value("progress").toString().isEmpty());
}

void ControlServiceTest::busyRefreshIsNotAccepted()
{
    ViewingFixture fixture;
    fixture.readiness = control::ControlService::RefreshReadiness::Busy;
    const QJsonObject response = fixture.service.Handle(
        control::Request{"start", "refresh.start", {}});
    QVERIFY(response.value("ok").toBool());
    QVERIFY(!resultOf(response).value("accepted").toBool(true));
    QCOMPARE(resultOf(response).value("state").toString(), "busy");
    QCOMPARE(fixture.starts, 0);

    fixture.readiness = control::ControlService::RefreshReadiness::Ready;
    const QJsonObject retry = fixture.service.Handle(
        control::Request{"start", "refresh.start", {}});
    QVERIFY(!resultOf(retry).value("accepted").toBool(true));
    QCOMPARE(resultOf(retry).value("state").toString(), "busy");
    QCoreApplication::processEvents();
    QCOMPARE(fixture.starts, 0);
}

void ControlServiceTest::refreshOutcomesRemainTyped()
{
    ViewingFixture fixture;
    const QJsonObject start = fixture.service.Handle(
        control::Request{"start", "refresh.start", {}});
    const QString id = resultOf(start).value("operation_id").toString();
    QCoreApplication::processEvents();

    RateLimit::FetchError parse_error;
    parse_error.kind = RateLimit::FetchError::Kind::Parse;
    parse_error.message = "bad payload";
    CompletedRefresh completed;
    completed.skipped.push_back(
        SkippedSource{FetchSourceKey{ItemLocationType::STASH, "stash-view"}, parse_error});
    fixture.items.RefreshFinished(RefreshOutcome{completed});

    const QJsonObject operation = resultOf(fixture.service.Handle(control::Request{
        "job", "refresh.status", QJsonObject{{"operation_id", id}}}))
                                      .value("operation")
                                      .toObject();
    QCOMPARE(operation.value("state").toString(), "completed");
    const QJsonObject outcome = operation.value("outcome").toObject();
    QVERIFY(!outcome.value("clean").toBool(true));
    QCOMPARE(outcome.value("skipped").toArray().size(), 1);
    QCOMPARE(outcome.value("skipped")
                 .toArray()
                 .at(0)
                 .toObject()
                 .value("error")
                 .toObject()
                 .value("kind")
                 .toString(),
             "parse");
}

void ControlServiceTest::refreshHistoryIsBounded()
{
    ViewingFixture fixture;
    QString first_id;
    QString second_id;
    QString last_id;
    for (int index = 0; index < 33; ++index) {
        const QJsonObject start = fixture.service.Handle(
            control::Request{QString("start-%1").arg(index), "refresh.start", {}});
        const QString id = resultOf(start).value("operation_id").toString();
        QVERIFY(!id.isEmpty());
        if (index == 0) {
            first_id = id;
        } else if (index == 1) {
            second_id = id;
        }
        last_id = id;
        QCoreApplication::processEvents();
        fixture.items.RefreshFinished(RefreshOutcome{CompletedRefresh{}});
    }

    const QJsonObject expired = fixture.service.Handle(control::Request{
        "old", "refresh.status", QJsonObject{{"operation_id", first_id}}});
    QCOMPARE(expired.value("error").toObject().value("code").toString(),
             "operation_not_found");
    const QJsonObject oldest_retained = fixture.service.Handle(control::Request{
        "oldest-retained", "refresh.status", QJsonObject{{"operation_id", second_id}}});
    QVERIFY(oldest_retained.value("ok").toBool());
    const QJsonObject newest_retained = fixture.service.Handle(control::Request{
        "newest-retained", "refresh.status", QJsonObject{{"operation_id", last_id}}});
    QVERIFY(newest_retained.value("ok").toBool());
}

QTEST_GUILESS_MAIN(ControlServiceTest)
#include "tst_controlservice.moc"
