// Manual Release-only scale harness for the local-control viewing path.

#include <QCommandLineParser>
#include <QCoreApplication>
#include <QElapsedTimer>
#include <QJsonArray>
#include <QSettings>

#include <algorithm>
#include <cstdio>

#include "control/controlservice.h"
#include "itemcategories.h"
#include "itemsmanager.h"
#include "spikedataset.h"
#include "testfixtures.h"

int main(int argc, char **argv)
{
    QCoreApplication app(argc, argv);
    QCommandLineParser parser;
    parser.addHelpOption();
    QCommandLineOption preset_option("preset", "Dataset preset: smoke, 100k, or 1m.", "name", "100k");
    parser.addOption(preset_option);
    parser.process(app);

    const auto config = SpikeDataset::Config::Preset(parser.value(preset_option));
    if (!config) {
        std::fprintf(stderr, "unknown preset\n");
        return 2;
    }

    InitItemClasses(R"json({"TestClass":{"name":"Weapons"}})json");
    InitItemBaseTypes(
        R"json({"Metadata/Items/TestSword":{"item_class":"TestClass","name":"Test Sword","release_state":"released"}})json");

    SpikeDataset dataset(*config);
    BuyoutManagerFixture buyouts;
    QSettings settings(buyouts.tempDir.filePath("settings.ini"), QSettings::IniFormat);
    ItemsManager items_manager(settings, *buyouts.manager, *buyouts.data);
    control::ControlService service("benchmark");
    service.AttachSession(items_manager, nullptr, *buyouts.manager, "benchmark", "benchmark");

    Items items = dataset.allItems();
    const qsizetype expected = items.size();
    items_manager.OnItemsRefreshed(std::move(items), dataset.locations(), true);

    QString cursor;
    qsizetype received = 0;
    qsizetype pages = 0;
    qint64 maximum_page_ns = 0;
    QElapsedTimer total;
    total.start();
    while (true) {
        QJsonObject params{{"limit", 100}};
        if (!cursor.isEmpty()) {
            // The opaque cursor carries the original limit and filters; the
            // protocol rejects combining a cursor with those parameters.
            params = QJsonObject{{"cursor", cursor}};
        }
        QElapsedTimer page_timer;
        page_timer.start();
        const QJsonObject response = service.Handle(
            control::Request{"benchmark", "items", params});
        maximum_page_ns = std::max(maximum_page_ns, page_timer.nsecsElapsed());
        if (!response.value("ok").toBool()) {
            std::fprintf(stderr, "view request failed\n");
            return 1;
        }
        const QJsonObject result = response.value("result").toObject();
        received += result.value("items").toArray().size();
        ++pages;
        cursor = result.value("next_cursor").toString();
        if (cursor.isEmpty()) {
            break;
        }
    }

    if (received != expected) {
        std::fprintf(stderr,
                     "item count mismatch: expected %lld, received %lld\n",
                     static_cast<long long>(expected),
                     static_cast<long long>(received));
        return 1;
    }
    std::printf("preset=%s items=%lld pages=%lld total_ms=%.3f max_page_ms=%.3f\n",
                qPrintable(parser.value(preset_option)),
                static_cast<long long>(received),
                static_cast<long long>(pages),
                double(total.nsecsElapsed()) / 1'000'000.0,
                double(maximum_page_ns) / 1'000'000.0);
    return 0;
}
