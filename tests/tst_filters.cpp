#include <QtTest/QtTest>

#include <QStringListModel>
#include <QVBoxLayout>
#include <QWidget>

#include "filters.h"
#include "testfixtures.h"

class FiltersTest : public QObject
{
    Q_OBJECT

private slots:
    void nameFilter();
    void minMaxFilter();
    void socketColorFilters();
    void booleanFilter();
    void rarityFilter();
};

struct FilterHarness
{
    QWidget host;
    QVBoxLayout layout{&host};
    QStringListModel rarityModel{RaritySearchFilter::RARITY_LIST};
    QObject receiver;
    FilterCallbacks callbacks{
        &receiver,
        [] {},
        [] {},
    };
};

static std::shared_ptr<Item> makeFilterItem(const QString &id,
                                            const QString &extraJson,
                                            int frameType = 2,
                                            const QString &frameTypeId = "Rare")
{
    const QByteArray json = QString(R"json({
        "baseType": "Test Item",
        "frameType": %3,
        "frameTypeId": "%4",
        "h": 1,
        "icon": "https://web.poecdn.com/image/test.png",
        "id": "%1",
        "identified": true,
        "ilvl": 1,
        "name": "Alpha Bite",
        "typeLine": "Test Item",
        "verified": false,
        "w": 1,
        "x": 0,
        "y": 0%2
    })json")
                                .arg(id, extraJson, QString::number(frameType), frameTypeId)
                                .toUtf8();
    return std::make_shared<Item>(makeTestItem(json.constData(), makeTestStashLocation()));
}

void FiltersTest::nameFilter()
{
    FilterHarness harness;
    NameSearchFilter filter(&harness.layout, harness.callbacks);
    FilterData data(&filter);
    data.text_query = "alpha";

    QVERIFY(filter.Matches(makeFilterItem("name-match", ""), &data));

    data.text_query = "ALPHA";
    QVERIFY(filter.Matches(makeFilterItem("name-match-case", ""), &data));

    data.text_query = "missing";
    QVERIFY(!filter.Matches(makeFilterItem("name-miss", ""), &data));
}

void FiltersTest::minMaxFilter()
{
    FilterHarness harness;
    SimplePropertyFilter filter(&harness.layout, "Quality", harness.callbacks);
    const auto withQuality = makeFilterItem("quality",
                                            R"json(,
        "properties": [
            {
                "displayMode": 0,
                "name": "Quality",
                "type": 6,
                "values": [["+20%", 1]]
            }
        ])json");
    const auto withoutQuality = makeFilterItem("missing-quality", "");

    FilterData data(&filter);
    data.min_filled = true;
    data.min = 10.0;
    QVERIFY(filter.Matches(withQuality, &data));

    data.min = 25.0;
    QVERIFY(!filter.Matches(withQuality, &data));

    data.min_filled = false;
    data.max_filled = true;
    data.max = 20.0;
    QVERIFY(filter.Matches(withQuality, &data));

    data.max = 19.0;
    QVERIFY(!filter.Matches(withQuality, &data));

    data.min_filled = true;
    data.min = 1.0;
    data.max_filled = false;
    QVERIFY(!filter.Matches(withoutQuality, &data));

    data.min_filled = false;
    QVERIFY(filter.Matches(withoutQuality, &data));
}

void FiltersTest::socketColorFilters()
{
    FilterHarness harness;
    const auto item = makeFilterItem("sockets",
                                     R"json(,
        "sockets": [
            {"group": 0, "attr": "S", "sColour": "R"},
            {"group": 0, "attr": "G", "sColour": "W"},
            {"group": 1, "attr": "I", "sColour": "B"}
        ])json");

    SocketsColorsFilter socketsFilter(&harness.layout, harness.callbacks);
    FilterData socketsData(&socketsFilter);
    socketsData.r_filled = true;
    socketsData.r = 1;
    socketsData.b_filled = true;
    socketsData.b = 1;
    QVERIFY(socketsFilter.Matches(item, &socketsData));

    socketsData.b = 3;
    QVERIFY(!socketsFilter.Matches(item, &socketsData));

    LinksColorsFilter linksFilter(&harness.layout, harness.callbacks);
    FilterData linksData(&linksFilter);
    linksData.r_filled = true;
    linksData.r = 1;
    linksData.b_filled = true;
    linksData.b = 1;
    QVERIFY(linksFilter.Matches(item, &linksData));

    linksData.b = 2;
    QVERIFY(!linksFilter.Matches(item, &linksData));
}

void FiltersTest::booleanFilter()
{
    FilterHarness harness;
    CorruptedFilter filter(&harness.layout, "", "Corrupted", harness.callbacks);
    FilterData data(&filter);
    data.checked = true;

    QVERIFY(filter.Matches(makeFilterItem("corrupted", R"json(, "corrupted": true)json"), &data));
    QVERIFY(!filter.Matches(makeFilterItem("not-corrupted", ""), &data));

    data.checked = false;
    QVERIFY(filter.Matches(makeFilterItem("unchecked", ""), &data));
}

void FiltersTest::rarityFilter()
{
    FilterHarness harness;
    RaritySearchFilter filter(&harness.layout, &harness.rarityModel, harness.callbacks);
    FilterData data(&filter);

    const auto normal = makeFilterItem("normal", "", 0, "Normal");
    const auto rare = makeFilterItem("rare", "", 2, "Rare");
    const auto unique = makeFilterItem("unique", "", 3, "Unique");
    const auto foil = makeFilterItem("foil", "", 9, "Foil");

    data.text_query = "Any Non-Unique";
    QVERIFY(filter.Matches(normal, &data));
    QVERIFY(filter.Matches(rare, &data));
    QVERIFY(!filter.Matches(unique, &data));

    data.text_query = "Unique";
    QVERIFY(filter.Matches(unique, &data));
    QVERIFY(!filter.Matches(foil, &data));

    data.text_query = "Unique (Foil)";
    QVERIFY(filter.Matches(foil, &data));
    QVERIFY(!filter.Matches(unique, &data));
}

QTEST_MAIN(FiltersTest)

#include "tst_filters.moc"
