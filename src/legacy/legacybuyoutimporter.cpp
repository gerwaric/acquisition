// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "legacy/legacybuyoutimporter.h"

#include <QDateTime>

#include <algorithm>
#include <functional>
#include <optional>
#include <unordered_map>
#include <vector>

#include "buyout.h"
#include "currency.h"
#include "datastore/buyoutrepo.h"
#include "legacy/legacybuyout.h"
#include "legacy/legacydatastore.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

namespace {

    struct ImportTarget
    {
        QString id;
        QString location_id;
        ItemLocationType location_type{ItemLocationType::STASH};

        bool operator==(const ImportTarget &) const = default;
    };

    void appendUnique(std::vector<ImportTarget> &targets, ImportTarget target)
    {
        if (std::ranges::find(targets, target) == targets.end()) {
            targets.push_back(std::move(target));
        }
    }

    std::optional<Buyout> convertBuyout(const LegacyBuyout &legacy)
    {
        Buyout buyout;
        buyout.value = legacy.value;
        buyout.last_update = QDateTime::fromSecsSinceEpoch(legacy.last_update);
        buyout.type = Buyout::TagAsBuyoutType(legacy.type);
        buyout.currency = Currency::FromTag(legacy.currency);
        buyout.source = Buyout::TagAsBuyoutSource(legacy.source);
        buyout.inherited = legacy.inherited;

        const bool known_type = buyout.BuyoutTypeAsTag() == legacy.type;
        const bool known_currency = buyout.CurrencyAsTag() == legacy.currency;
        const bool known_source = buyout.BuyoutSourceAsTag() == legacy.source;
        if (!known_type || !known_currency || !known_source || !buyout.IsValid()) {
            return std::nullopt;
        }
        return buyout;
    }

    void countSave(BuyoutSaveResult result, LegacyBuyoutImportReport &report)
    {
        if (result == BuyoutSaveResult::Saved) {
            ++report.imported;
        } else {
            ++report.skipped;
        }
    }

} // namespace

QString LegacyBuyoutImportReport::summary() const
{
    return QString("Imported: %1\nAmbiguous: %2\nOrphaned: %3\nSkipped: %4")
        .arg(imported)
        .arg(ambiguous)
        .arg(orphaned)
        .arg(skipped);
}

LegacyBuyoutImportReport LegacyBuyoutImporter::importFile(const QString &filename)
{
    LegacyBuyoutImportReport report;
    const LegacyDataStore store(filename);
    report.skipped = store.skippedRowCount();
    if (!store.isValid()) {
        report.error = "The selected file is not a readable legacy Acquisition database.";
        return report;
    }
    // Both 4 and 5 hold v4-generation hash keys: master's MigrateBuyouts
    // stamps db_version 5 into upgraded legacy files without touching the
    // buyouts, so the live file of a <=0.15 upgrader reads 5 (R1-1). Only
    // pre-4 files used the <<set:...>>-prefixed hash this importer does
    // not compute.
    const QString db_version = store.data().db_version.trimmed();
    if (db_version != "4" && db_version != "5") {
        report.error = QString("Unsupported legacy database version '%1'. This importer currently "
                               "supports db_version 4 and 5 only.")
                           .arg(db_version.isEmpty() ? "missing" : db_version);
        return report;
    }

    std::unordered_map<QString, QString> character_ids;
    for (const LegacyCharacter &character : store.tabs().characters) {
        if (!character.name.isEmpty() && !character.id.isEmpty()) {
            character_ids.emplace(character.name, character.id);
        } else {
            ++report.skipped;
        }
    }

    std::unordered_map<QString, std::vector<ImportTarget>> item_targets;
    for (const auto &[row_location, items] : store.items()) {
        for (const LegacyItem &item : items) {
            ImportTarget target{.id = item.id};
            if (item._tab_label && !item._character) {
                target.location_id = row_location;
                target.location_type = ItemLocationType::STASH;
            } else if (item._character && !item._tab_label) {
                const auto character = character_ids.find(*item._character);
                if (character == character_ids.end()) {
                    ++report.skipped;
                    continue;
                }
                target.location_id = character->second;
                target.location_type = ItemLocationType::CHARACTER;
            } else {
                ++report.skipped;
                continue;
            }

            const QString hash = item.hash();
            if (target.id.isEmpty() || target.location_id.isEmpty() || hash.isEmpty()) {
                ++report.skipped;
                continue;
            }
            appendUnique(item_targets[hash], std::move(target));
        }
    }

    for (const auto &[hash, legacy_buyout] : store.data().buyouts) {
        const auto buyout = convertBuyout(legacy_buyout);
        if (!buyout) {
            spdlog::warn("Legacy buyout import: invalid item buyout '{}'; skipping", hash);
            ++report.skipped;
            continue;
        }
        const auto match = item_targets.find(hash);
        if (match == item_targets.end()) {
            ++report.orphaned;
            continue;
        }
        if (match->second.size() > 1) {
            ++report.ambiguous;
        }
        for (const ImportTarget &target : match->second) {
            countSave(m_repo.saveItemBuyout(*buyout,
                                            target.id,
                                            target.location_id,
                                            target.location_type,
                                            false),
                      report);
        }
    }

    std::unordered_map<QString, std::vector<ImportTarget>> location_targets;
    const std::function<void(const LegacyStash &)> add_stash = [&](const LegacyStash &stash) {
        if (!stash.name.isEmpty() && !stash.id.isEmpty()) {
            appendUnique(location_targets["stash:" + stash.name],
                         ImportTarget{.id = stash.id,
                                      .location_id = stash.id,
                                      .location_type = ItemLocationType::STASH});
        } else {
            ++report.skipped;
        }
        if (stash.children) {
            for (const LegacyStash &child : *stash.children) {
                add_stash(child);
            }
        }
    };
    for (const LegacyStash &stash : store.tabs().stashes) {
        add_stash(stash);
    }
    for (const LegacyCharacter &character : store.tabs().characters) {
        if (!character.name.isEmpty() && !character.id.isEmpty()) {
            appendUnique(location_targets["character:" + character.name],
                         ImportTarget{.id = character.id,
                                      .location_id = character.id,
                                      .location_type = ItemLocationType::CHARACTER});
        }
    }

    for (const auto &[legacy_location, legacy_buyout] : store.data().tab_buyouts) {
        const auto buyout = convertBuyout(legacy_buyout);
        if (!buyout) {
            spdlog::warn("Legacy buyout import: invalid location buyout '{}'; skipping",
                         legacy_location);
            ++report.skipped;
            continue;
        }
        const auto match = location_targets.find(legacy_location);
        if (match == location_targets.end()) {
            ++report.orphaned;
            continue;
        }
        if (match->second.size() > 1) {
            ++report.ambiguous;
        }
        for (const ImportTarget &target : match->second) {
            countSave(m_repo.saveLocationBuyout(*buyout, target.id, target.location_type, false),
                      report);
        }
    }

    report.success = true;
    return report;
}
