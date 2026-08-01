#include <QJsonArray>
#include <QJsonDocument>
#include <QSettings>
#include <QtTest>

#include "control/controlservice.h"
#include "itemsmanager.h"
#include "testfixtures.h"

class ControlServiceTest : public QObject
{
    Q_OBJECT
private slots:
    void reportsPreLoginStatus();
    void rejectsUnknownCommand();
    void viewingRequiresReadySession();
    void viewsPublishedItemsAndEffectivePrices();
    void paginationRejectsOldRevision();
    void rejectsMalformedCursor();
    void rejectsMalformedQueryParameters();
    void refreshJobOutlivesStartRequest();
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
                "typeLine":"Amethyst Ring","identified":true,"ilvl":84,"x":3,"y":7
            })";
            const QByteArray second_json = R"({
                "w":1,"h":1,"id":"item-two","name":"",
                "typeLine":"Chaos Orb","identified":true,"ilvl":1,"x":1,"y":2
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
    QCOMPARE(item.value("location").toObject().value("tab_label").toString(), "Viewing Tab");
    const QJsonObject price = item.value("effective_price").toObject();
    QCOMPARE(price.value("value").toDouble(), 10.0);
    QCOMPARE(price.value("currency").toString(), "chaos");
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

    fixture.buyouts.manager->Set(*fixture.first, makeChaosBuyout(11));
    const QJsonObject stale = fixture.service.Handle(
        control::Request{"stale", "items", QJsonObject{{"cursor", cursor}}});
    QVERIFY(!stale.value("ok").toBool(true));
    QCOMPARE(stale.value("error").toObject().value("code").toString(), "revision_changed");
}

void ControlServiceTest::rejectsMalformedCursor()
{
    ViewingFixture fixture;
    const QJsonObject status = resultOf(
        fixture.service.Handle(control::Request{"status", "status", {}}));
    const QJsonObject cursor_object{{"instance_id", status.value("instance_id").toString()},
                                    {"revision", status.value("inventory_revision").toString()},
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
            control::Request{"start", "refresh.start", {}});
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
