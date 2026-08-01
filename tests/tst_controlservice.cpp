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
            items.OnItemsRefreshed(Items{first, second}, {location}, true);
            buyouts.manager->Set(*first, makeChaosBuyout(10));
        }

        BuyoutManagerFixture buyouts;
        QSettings settings;
        ItemsManager items;
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

QTEST_GUILESS_MAIN(ControlServiceTest)
#include "tst_controlservice.moc"
