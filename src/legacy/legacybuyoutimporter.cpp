// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "legacy/legacybuyoutimporter.h"

#include <QColor>
#include <QDateTime>
#include <QFileInfo>
#include <QStringList>

#include <algorithm>
#include <array>
#include <functional>
#include <optional>
#include <unordered_map>
#include <unordered_set>
#include <vector>

#include <xlsxdatavalidation.h>
#include <xlsxdocument.h>
#include <xlsxformat.h>

#include "buyout.h"
#include "currency.h"
#include "datastore/buyoutrepo.h"
#include "datastore/characterrepo.h"
#include "datastore/stashrepo.h"
#include "legacy/legacybuyout.h"
#include "legacy/legacydatastore.h"
#include "poe/types/character.h"
#include "poe/types/item.h"
#include "poe/types/stashtab.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

namespace {

    struct ImportTarget
    {
        QString id;
        QString location_id;
        ItemLocationType location_type{ItemLocationType::STASH};
        QString old_name;
        QString current_name;
        QString reason;
        QString item_name;

        bool operator==(const ImportTarget &other) const
        {
            return id == other.id && location_id == other.location_id
                   && location_type == other.location_type;
        }
    };

    struct PlanRow
    {
        QString action;
        QString reason;
        QString target_type;
        LegacyBuyout buyout;
        QString item_id;
        QString location_id;
        QString location_type;
        QString item_name;
        QString old_tab_label;
        QString current_tab_name;
        QString old_character;
        QString current_character;
        QString legacy_hash;
        std::optional<Buyout> existing;
    };

    const QStringList PLAN_HEADERS{
        "action",
        "outcome",
        "reason",
        "target_type",
        "value",
        "currency",
        "type",
        "source",
        "inherited",
        "last_update",
        "item_id",
        "location_id",
        "location_type",
        "item_name",
        "old_tab_label",
        "current_tab_name",
        "old_character",
        "current_character",
        "legacy_hash",
        "existing_value",
        "existing_source",
        "error",
    };

    void appendUnique(std::vector<ImportTarget> &targets, ImportTarget target)
    {
        if (std::ranges::find(targets, target) == targets.end()) {
            targets.push_back(std::move(target));
        }
    }

    QString locationTypeTag(ItemLocationType type)
    {
        return type == ItemLocationType::STASH ? "stash" : "character";
    }

    QString legacyStashLabel(const LegacyStash &stash)
    {
        return stash.n.value_or(stash.name);
    }

    QString fixedLegacyStashId(const QString &id)
    {
        return id.size() > 10 ? id.left(10) : id;
    }

    QString itemName(const LegacyItem &item)
    {
        if (item.name.isEmpty()) {
            return item.typeLine;
        }
        if (item.typeLine.isEmpty()) {
            return item.name;
        }
        return item.name + " " + item.typeLine;
    }

    void appendItemIds(const std::optional<std::vector<poe::Item>> &items,
                       std::unordered_set<QString> &ids)
    {
        if (!items) {
            return;
        }
        for (const poe::Item &item : *items) {
            if (item.id && !item.id->isEmpty()) {
                ids.insert(*item.id);
            }
            appendItemIds(item.socketedItems, ids);
        }
    }

    std::unordered_set<QString> currentCharacterItemIds(const poe::Character &character)
    {
        std::unordered_set<QString> ids;
        appendItemIds(character.equipment, ids);
        appendItemIds(character.skills, ids);
        appendItemIds(character.inventory, ids);
        appendItemIds(character.rucksack, ids);
        appendItemIds(character.jewels, ids);
        appendItemIds(character.guardian, ids);
        return ids;
    }

    QString prefillAction(const LegacyBuyout &buyout,
                          const QString &matching_reason,
                          const std::optional<Buyout> &existing)
    {
        if (matching_reason == "orphaned" || matching_reason == "needs-attention"
            || buyout.inherited || (existing && existing->source == Buyout::BUYOUT_SOURCE_MANUAL)) {
            return "skip";
        }
        return "import";
    }

    QString prefillReason(const LegacyBuyout &buyout,
                          const QString &matching_reason,
                          const std::optional<Buyout> &existing)
    {
        if (matching_reason == "orphaned" || matching_reason == "needs-attention") {
            return matching_reason;
        }
        if (buyout.inherited) {
            return "inherited";
        }
        if (existing && existing->source == Buyout::BUYOUT_SOURCE_MANUAL) {
            return "existing-manual";
        }
        return matching_reason;
    }

    bool writePlanWorkbook(const QString &filename,
                           const QString &source_filename,
                           const QString &db_version,
                           const std::vector<PlanRow> &rows)
    {
        QXlsx::Document document;
        if (!document.addSheet("plan")) {
            return false;
        }

        QXlsx::Format header_format;
        header_format.setFontBold(true);
        header_format.setPatternBackgroundColor(QColor("#d9eaf7"));
        header_format.setFillPattern(QXlsx::Format::PatternSolid);
        for (int column = 0; column < PLAN_HEADERS.size(); ++column) {
            if (!document.write(1, column + 1, PLAN_HEADERS.at(column), header_format)) {
                return false;
            }
        }

        const auto write = [&document](int row, int column, const QVariant &value) {
            return !value.isValid() || value.isNull() || document.write(row, column, value);
        };
        int row_number = 2;
        for (const PlanRow &row : rows) {
            const QVariant existing_value = row.existing ? QVariant(row.existing->value)
                                                         : QVariant();
            const QVariant existing_source = row.existing
                                                 ? QVariant(row.existing->BuyoutSourceAsTag())
                                                 : QVariant();
            const std::array values{
                QVariant(row.action),
                QVariant(QString()),
                QVariant(row.reason),
                QVariant(row.target_type),
                QVariant(row.buyout.value),
                QVariant(row.buyout.currency),
                QVariant(row.buyout.type),
                QVariant(row.buyout.source),
                QVariant(row.buyout.inherited),
                QVariant::fromValue(row.buyout.last_update),
                QVariant(row.item_id),
                QVariant(row.location_id),
                QVariant(row.location_type),
                QVariant(row.item_name),
                QVariant(row.old_tab_label),
                QVariant(row.current_tab_name),
                QVariant(row.old_character),
                QVariant(row.current_character),
                QVariant(row.legacy_hash),
                existing_value,
                existing_source,
                QVariant(QString()),
            };
            for (int column = 0; column < static_cast<int>(values.size()); ++column) {
                if (!write(row_number, column + 1, values.at(column))) {
                    return false;
                }
            }
            ++row_number;
        }

        QXlsx::DataValidation actions(QXlsx::DataValidation::List,
                                      QXlsx::DataValidation::Between,
                                      "\"import,skip\"");
        actions.setErrorMessage("Choose import or skip.", "Invalid action");
        actions.setErrorMessageVisible(true);
        actions.addRange(2, 1, std::max(row_number + 100, 1000), 1);
        if (!document.addDataValidation(actions)) {
            return false;
        }
        document.setColumnWidth(1, 4, 20);
        document.setColumnWidth(5, 10, 14);
        document.setColumnWidth(11, 22, 24);

        if (!document.addSheet("meta")) {
            return false;
        }
        const std::array<std::pair<QString, QVariant>, 4> metadata{
            std::pair{QString("format_version"), QVariant(QString("1"))},
            std::pair{QString("source_file"),
                      QVariant(QFileInfo(source_filename).absoluteFilePath())},
            std::pair{QString("exported_at"),
                      QVariant(QDateTime::currentDateTimeUtc().toString(Qt::ISODateWithMs))},
            std::pair{QString("source_db_version"), QVariant(db_version)},
        };
        int meta_row = 1;
        for (const auto &[key, value] : metadata) {
            if (!document.write(meta_row, 1, key, header_format)
                || !document.write(meta_row, 2, value)) {
                return false;
            }
            ++meta_row;
        }
        document.setColumnWidth(1, 24);
        document.setColumnWidth(2, 80);
        return document.saveAs(filename);
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

QString LegacyBuyoutPlanReport::summary() const
{
    return QString("Matched %1 of %2 buyouts (%3 ambiguous, %4 orphaned).\nPlan rows: %5\n"
                   "Malformed legacy entries skipped: %6")
        .arg(matched)
        .arg(total)
        .arg(ambiguous)
        .arg(orphaned)
        .arg(rows)
        .arg(skipped);
}

LegacyBuyoutPlanReport LegacyBuyoutImporter::createPlan(const QString &source_filename,
                                                        const QString &plan_filename)
{
    LegacyBuyoutPlanReport report;
    report.plan_file = plan_filename;
    if (!m_stashes || !m_characters) {
        report.error = "Legacy buyout planning requires the current stash and character stores.";
        return report;
    }

    const LegacyDataStore store(source_filename);
    report.skipped = store.skippedRowCount();
    if (!store.isValid()) {
        report.error = "The selected file is not a readable legacy Acquisition database.";
        return report;
    }
    const QString db_version = store.data().db_version.trimmed();
    if (db_version != "4" && db_version != "5") {
        report.error = QString("Unsupported legacy database version '%1'. This importer currently "
                               "supports db_version 4 and 5 only.")
                           .arg(db_version.isEmpty() ? "missing" : db_version);
        return report;
    }

    const auto current_item_buyouts = m_repo.getItemBuyouts();
    const auto current_location_buyouts = m_repo.getLocationBuyouts();

    std::unordered_map<QString, QString> current_stash_names;
    for (const poe::StashTab &stash : m_stashes->getStashList(m_realm, m_league)) {
        if (!stash.id.isEmpty()) {
            current_stash_names[stash.id] = stash.name;
        }
    }

    const std::vector<poe::Character> current_characters = m_characters->getCharacterList(m_realm);
    std::unordered_map<QString, std::unordered_set<QString>> current_character_item_ids;
    for (const poe::Character &character : current_characters) {
        const auto detail = m_characters->getCharacter(character.name, m_realm);
        if (detail) {
            current_character_item_ids[character.id] = currentCharacterItemIds(*detail);
        }
    }

    std::unordered_map<QString, std::unordered_set<QString>> old_character_item_ids;
    for (const auto &[row_location, items] : store.items()) {
        for (const LegacyItem &item : items) {
            if (item.id.isEmpty()) {
                continue;
            }
            if (item._character && !item._character->isEmpty()) {
                old_character_item_ids[*item._character].insert(item.id);
            } else if (!row_location.isEmpty() && !item._tab_label) {
                old_character_item_ids[row_location].insert(item.id);
            }
        }
    }

    std::unordered_map<QString, std::vector<ImportTarget>> character_targets;
    for (const LegacyCharacter &legacy : store.tabs().characters) {
        if (legacy.name.isEmpty()) {
            ++report.skipped;
            continue;
        }

        if (!legacy.id.isEmpty()) {
            QString current_name;
            for (const poe::Character &current : current_characters) {
                if (current.id == legacy.id) {
                    current_name = current.name;
                    break;
                }
            }
            appendUnique(character_targets[legacy.name],
                         ImportTarget{.id = legacy.id,
                                      .location_id = legacy.id,
                                      .location_type = ItemLocationType::CHARACTER,
                                      .old_name = legacy.name,
                                      .current_name = current_name});
            continue;
        }

        for (const poe::Character &current : current_characters) {
            if (current.name == legacy.name) {
                const QString id = current.id.isEmpty() ? current.name : current.id;
                appendUnique(character_targets[legacy.name],
                             ImportTarget{.id = id,
                                          .location_id = id,
                                          .location_type = ItemLocationType::CHARACTER,
                                          .old_name = legacy.name,
                                          .current_name = current.name});
            }
        }
        if (!character_targets[legacy.name].empty()) {
            continue;
        }

        const auto old_ids = old_character_item_ids.find(legacy.name);
        if (old_ids == old_character_item_ids.end()) {
            continue;
        }
        for (const poe::Character &current : current_characters) {
            const auto current_ids = current_character_item_ids.find(current.id);
            if (current_ids == current_character_item_ids.end()) {
                continue;
            }
            const bool shares_item = std::ranges::any_of(old_ids->second, [&](const QString &id) {
                return current_ids->second.contains(id);
            });
            if (shares_item) {
                const QString id = current.id.isEmpty() ? current.name : current.id;
                appendUnique(character_targets[legacy.name],
                             ImportTarget{.id = id,
                                          .location_id = id,
                                          .location_type = ItemLocationType::CHARACTER,
                                          .old_name = legacy.name,
                                          .current_name = current.name,
                                          .reason = "character-matched-by-items"});
            }
        }
    }

    std::unordered_map<QString, std::vector<ImportTarget>> item_targets;
    for (const auto &[row_location, items] : store.items()) {
        for (const LegacyItem &item : items) {
            const QString hash = item.hash();
            if (item.id.isEmpty() || hash.isEmpty()) {
                ++report.skipped;
                continue;
            }
            if (item._tab_label && !item._character) {
                if (row_location.isEmpty()) {
                    ++report.skipped;
                    continue;
                }
                appendUnique(item_targets[hash],
                             ImportTarget{.id = item.id,
                                          .location_id = row_location,
                                          .location_type = ItemLocationType::STASH,
                                          .old_name = *item._tab_label,
                                          .current_name = current_stash_names[row_location],
                                          .item_name = itemName(item)});
            } else if (item._character && !item._tab_label) {
                const auto characters = character_targets.find(*item._character);
                if (characters == character_targets.end()) {
                    continue;
                }
                for (const ImportTarget &character : characters->second) {
                    ImportTarget target = character;
                    target.id = item.id;
                    target.item_name = itemName(item);
                    appendUnique(item_targets[hash], std::move(target));
                }
            } else {
                ++report.skipped;
            }
        }
    }

    std::vector<PlanRow> rows;
    const auto append_item_row = [&](const QString &hash,
                                     const LegacyBuyout &legacy,
                                     const ImportTarget *target,
                                     std::size_t target_index,
                                     std::size_t target_count) {
        QString matching_reason = "orphaned";
        std::optional<Buyout> existing;
        if (target) {
            if (target->reason == "needs-attention") {
                matching_reason = target->reason;
            } else if (target_count > 1) {
                matching_reason = QString("ambiguous-%1-of-%2").arg(target_index).arg(target_count);
            } else if (!target->reason.isEmpty()) {
                matching_reason = target->reason;
            } else {
                matching_reason = "matched";
            }
            const auto found = current_item_buyouts.find(target->id);
            if (found != current_item_buyouts.end()) {
                existing = found->second;
            }
        }
        const QString reason = prefillReason(legacy, matching_reason, existing);
        rows.push_back(
            PlanRow{.action = prefillAction(legacy, matching_reason, existing),
                    .reason = reason,
                    .target_type = "item",
                    .buyout = legacy,
                    .item_id = target ? target->id : QString(),
                    .location_id = target ? target->location_id : QString(),
                    .location_type = target ? locationTypeTag(target->location_type) : QString(),
                    .item_name = target ? target->item_name : QString(),
                    .old_tab_label = target && target->location_type == ItemLocationType::STASH
                                         ? target->old_name
                                         : QString(),
                    .current_tab_name = target && target->location_type == ItemLocationType::STASH
                                            ? target->current_name
                                            : QString(),
                    .old_character = target && target->location_type == ItemLocationType::CHARACTER
                                         ? target->old_name
                                         : QString(),
                    .current_character = target && target->location_type == ItemLocationType::CHARACTER
                                             ? target->current_name
                                             : QString(),
                    .legacy_hash = hash,
                    .existing = existing});
    };

    for (const auto &[hash, legacy] : store.data().buyouts) {
        ++report.total;
        if (!convertBuyout(legacy)) {
            ++report.skipped;
            continue;
        }
        const auto targets = item_targets.find(hash);
        if (targets == item_targets.end() || targets->second.empty()) {
            ++report.orphaned;
            append_item_row(hash, legacy, nullptr, 0, 0);
            continue;
        }
        ++report.matched;
        if (targets->second.size() > 1) {
            ++report.ambiguous;
        }
        for (std::size_t index = 0; index < targets->second.size(); ++index) {
            append_item_row(hash,
                            legacy,
                            &targets->second.at(index),
                            index + 1,
                            targets->second.size());
        }
    }

    std::unordered_map<QString, std::vector<ImportTarget>> location_targets;
    const std::function<void(const LegacyStash &)> add_stash = [&](const LegacyStash &stash) {
        const QString label = legacyStashLabel(stash);
        const QString id = fixedLegacyStashId(stash.id);
        if (!label.isEmpty() && !id.isEmpty()) {
            const bool truncated = stash.id.size() > 10;
            const bool cross_checked = store.items().contains(id)
                                       || current_stash_names.contains(id);
            appendUnique(location_targets["stash:" + label],
                         ImportTarget{.id = id,
                                      .location_id = id,
                                      .location_type = ItemLocationType::STASH,
                                      .old_name = label,
                                      .current_name = current_stash_names[id],
                                      .reason = truncated && !cross_checked ? "needs-attention"
                                                                            : QString()});
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
    for (const auto &[name, targets] : character_targets) {
        for (const ImportTarget &target : targets) {
            appendUnique(location_targets["character:" + name], target);
        }
    }

    const auto append_location_row = [&](const QString &legacy_location,
                                         const LegacyBuyout &legacy,
                                         const ImportTarget *target,
                                         std::size_t target_index,
                                         std::size_t target_count) {
        QString matching_reason = "orphaned";
        std::optional<Buyout> existing;
        if (target) {
            if (target->reason == "needs-attention") {
                matching_reason = target->reason;
            } else if (target_count > 1) {
                matching_reason = QString("ambiguous-%1-of-%2").arg(target_index).arg(target_count);
            } else if (!target->reason.isEmpty()) {
                matching_reason = target->reason;
            } else {
                matching_reason = "matched";
            }
            const auto found = current_location_buyouts.find(target->id);
            if (found != current_location_buyouts.end()) {
                existing = found->second;
            }
        }
        rows.push_back(PlanRow{
            .action = prefillAction(legacy, matching_reason, existing),
            .reason = prefillReason(legacy, matching_reason, existing),
            .target_type = "location",
            .buyout = legacy,
            .location_id = target ? target->id : QString(),
            .location_type = target ? locationTypeTag(target->location_type) : QString(),
            .old_tab_label = target && target->location_type == ItemLocationType::STASH
                                 ? target->old_name
                                 : QString(),
            .current_tab_name = target && target->location_type == ItemLocationType::STASH
                                    ? target->current_name
                                    : QString(),
            .old_character = target && target->location_type == ItemLocationType::CHARACTER
                                 ? target->old_name
                                 : QString(),
            .current_character = target && target->location_type == ItemLocationType::CHARACTER
                                     ? target->current_name
                                     : QString(),
            .legacy_hash = legacy_location,
            .existing = existing,
        });
    };

    for (const auto &[legacy_location, legacy] : store.data().tab_buyouts) {
        ++report.total;
        if (!convertBuyout(legacy)) {
            ++report.skipped;
            continue;
        }
        const auto targets = location_targets.find(legacy_location);
        if (targets == location_targets.end() || targets->second.empty()) {
            ++report.orphaned;
            append_location_row(legacy_location, legacy, nullptr, 0, 0);
            continue;
        }
        ++report.matched;
        if (targets->second.size() > 1) {
            ++report.ambiguous;
        }
        for (std::size_t index = 0; index < targets->second.size(); ++index) {
            append_location_row(legacy_location,
                                legacy,
                                &targets->second.at(index),
                                index + 1,
                                targets->second.size());
        }
    }

    report.rows = static_cast<qint64>(rows.size());
    if (!writePlanWorkbook(plan_filename, source_filename, db_version, rows)) {
        report.error = QString("Could not write legacy buyout plan '%1'.").arg(plan_filename);
        return report;
    }
    report.success = true;
    return report;
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
