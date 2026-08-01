// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/viewprojection.h"

#include <QJsonArray>
#include <QJsonValue>

#include "buyout.h"
#include "buyoutmanager.h"
#include "item.h"
#include "itemconstants.h"
#include "itemlocation.h"

namespace control {

namespace {

    QString BuyoutTypeName(Buyout::BuyoutType type)
    {
        switch (type) {
        case Buyout::BUYOUT_TYPE_IGNORE:
            return "ignore";
        case Buyout::BUYOUT_TYPE_BUYOUT:
            return "buyout";
        case Buyout::BUYOUT_TYPE_FIXED:
            return "fixed";
        case Buyout::BUYOUT_TYPE_CURRENT_OFFER:
            return "current_offer";
        case Buyout::BUYOUT_TYPE_NO_PRICE:
            return "no_price";
        case Buyout::BUYOUT_TYPE_INHERIT:
            return "inherit";
        }
        return "inherit";
    }

    QJsonValue ProjectFrameType(int type)
    {
        switch (type) {
        case FrameType::FRAME_TYPE_NORMAL:
            return "normal";
        case FrameType::FRAME_TYPE_MAGIC:
            return "magic";
        case FrameType::FRAME_TYPE_RARE:
            return "rare";
        case FrameType::FRAME_TYPE_UNIQUE:
            return "unique";
        case FrameType::FRAME_TYPE_GEM:
            return "gem";
        case FrameType::FRAME_TYPE_CURRENCY:
            return "currency";
        case FrameType::FRAME_TYPE_DIVINATION_CARD:
            return "divination_card";
        case FrameType::FRAME_TYPE_QUEST_ITEM:
            return "quest_item";
        case FrameType::FRAME_TYPE_PROPHECY:
            return "prophecy";
        case FrameType::FRAME_TYPE_FOIL:
            return "foil";
        case FrameType::FRAME_TYPE_SUPPORTER_FOIL:
            return "supporter_foil";
        case FrameType::FRAME_TYPE_NECROPOLIS:
            return "necropolis";
        case FrameType::FRAME_TYPE_GOLD:
            return "gold";
        case FrameType::FRAME_TYPE_BREACH_SKILL:
            return "breach_skill";
        default:
            return QJsonValue::Null;
        }
    }

    QString BuyoutSourceName(Buyout::BuyoutSource source)
    {
        switch (source) {
        case Buyout::BUYOUT_SOURCE_NONE:
            return "none";
        case Buyout::BUYOUT_SOURCE_MANUAL:
            return "manual";
        case Buyout::BUYOUT_SOURCE_GAME:
            return "game";
        case Buyout::BUYOUT_SOURCE_AUTO:
            return "auto";
        }
        return "none";
    }

    QJsonObject ProjectBuyout(const Buyout &buyout)
    {
        QJsonObject result{{"type", BuyoutTypeName(buyout.type)},
                           {"source", BuyoutSourceName(buyout.source)},
                           {"value", buyout.value},
                           {"inherited", buyout.inherited}};
        if (buyout.currency.type == Currency::CURRENCY_NONE) {
            result.insert("currency", QJsonValue::Null);
        } else {
            result.insert("currency", buyout.currency.AsTag());
        }
        if (buyout.last_update.isNull()) {
            result.insert("last_updated", QJsonValue::Null);
        } else {
            result.insert("last_updated", buyout.last_update.toUTC().toString(Qt::ISODateWithMs));
        }
        return result;
    }

    QJsonArray ProjectProperties(const std::vector<ItemProperty> &properties)
    {
        QJsonArray result;
        for (const auto &property : properties) {
            QJsonArray values;
            for (const auto &value : property.values) {
                values.append(QJsonObject{{"text", value.str}, {"type", value.type}});
            }
            result.append(QJsonObject{{"name", property.name},
                                      {"display_mode", property.display_mode},
                                      {"values", values}});
        }
        return result;
    }

    QJsonArray ProjectRequirements(const std::vector<ItemRequirement> &requirements)
    {
        QJsonArray result;
        for (const auto &requirement : requirements) {
            result.append(QJsonObject{{"name", requirement.name},
                                      {"value",
                                       QJsonObject{{"text", requirement.value.str},
                                                   {"type", requirement.value.type}}}});
        }
        return result;
    }

    QJsonObject ProjectMods(const std::map<QString, ItemMods> &mods)
    {
        QJsonObject result;
        for (const auto &[group, entries] : mods) {
            QJsonArray values;
            for (const QString &entry : entries) {
                values.append(entry);
            }
            result.insert(group, values);
        }
        return result;
    }

    QJsonArray ProjectSockets(const std::vector<ItemSocket> &sockets)
    {
        QJsonArray result;
        for (const auto &socket : sockets) {
            result.append(QJsonObject{{"group", int(socket.group)},
                                      {"attribute", QString(QChar(uchar(socket.attr)))}});
        }
        return result;
    }

    QJsonArray ProjectInfluences(const Item &item)
    {
        constexpr std::array influences{
            std::pair{Item::SHAPER, "shaper"},
            std::pair{Item::ELDER, "elder"},
            std::pair{Item::CRUSADER, "crusader"},
            std::pair{Item::REDEEMER, "redeemer"},
            std::pair{Item::HUNTER, "hunter"},
            std::pair{Item::WARLORD, "warlord"},
            std::pair{Item::SEARING_EXARCH, "searing_exarch"},
            std::pair{Item::EATER_OF_WORLDS, "eater_of_worlds"},
        };
        QJsonArray result;
        for (const auto &[type, name] : influences) {
            if (item.hasInfluence(type)) {
                result.append(name);
            }
        }
        return result;
    }

    QJsonObject ProjectLocation(const ItemLocation &embedded, const ItemLocation &canonical)
    {
        QJsonObject result{{"kind", LocationKind(canonical)},
                           {"id", canonical.id()},
                           {"fetch_source_id", embedded.fetch_id()},
                           {"x", embedded.x()},
                           {"y", embedded.y()},
                           {"socketed", embedded.socketed()}};
        if (canonical.type() == ItemLocationType::STASH) {
            result.insert("tab_index", canonical.tab_index());
            result.insert("tab_label", canonical.tab_label());
            result.insert("tab_type", canonical.tab_type());
            result.insert("remove_only", canonical.removeonly());
            result.insert("color",
                          QJsonObject{{"red", canonical.getR()},
                                      {"green", canonical.getG()},
                                      {"blue", canonical.getB()}});
        } else {
            result.insert("character", canonical.character());
            result.insert("inventory_id", embedded.inventory_id());
        }
        return result;
    }

} // namespace

QString LocationKind(const ItemLocation &location)
{
    return location.type() == ItemLocationType::STASH ? "stash" : "character";
}

QJsonObject ProjectItem(const Item &item,
                        const ItemLocation &canonical_location,
                        const Buyout &effective_buyout)
{
    QJsonObject result{{"id", item.id()},
                       {"name", item.name()},
                       {"type_line", item.typeLine()},
                       {"category", item.category()},
                       {"icon", item.icon()},
                       {"identified", item.identified()},
                       {"item_level", item.ilvl()},
                       {"stack_count", item.count()},
                       {"width", item.w()},
                       {"height", item.h()},
                       {"flags",
                        QJsonObject{{"corrupted", item.corrupted()},
                                    {"crafted", item.crafted()},
                                    {"enchanted", item.enchanted()},
                                    {"fractured", item.fractured()},
                                    {"split", item.split()},
                                    {"synthesized", item.synthesized()},
                                    {"mutated", item.mutated()}}},
                       {"influences", ProjectInfluences(item)},
                       {"properties", ProjectProperties(item.text_properties())},
                       {"requirements", ProjectRequirements(item.text_requirements())},
                       {"mods", ProjectMods(item.text_mods())},
                       {"sockets", ProjectSockets(item.text_sockets())},
                       {"location", ProjectLocation(item.location(), canonical_location)},
                       {"effective_price", ProjectBuyout(effective_buyout)}};
    if (item.note().isNull()) {
        result.insert("note", QJsonValue::Null);
    } else {
        result.insert("note", item.note());
    }
    result.insert("frame_type", ProjectFrameType(item.frameType()));
    return result;
}

QJsonObject ProjectTab(const ItemLocation &location,
                       const BuyoutManager &buyout_manager,
                       qsizetype item_count)
{
    QJsonObject result{{"kind", LocationKind(location)},
                       {"id", location.id()},
                       {"item_count", int(item_count)},
                       {"refresh_checked", buyout_manager.GetRefreshChecked(location)},
                       {"refresh_locked", buyout_manager.GetRefreshLocked(location)},
                       {"effective_price", ProjectBuyout(buyout_manager.GetTab(location))}};
    if (location.type() == ItemLocationType::STASH) {
        result.insert("tab_index", location.tab_index());
        result.insert("tab_label", location.tab_label());
        result.insert("tab_type", location.tab_type());
        result.insert("remove_only", location.removeonly());
        result.insert("color",
                      QJsonObject{{"red", location.getR()},
                                  {"green", location.getG()},
                                  {"blue", location.getB()}});
    } else {
        result.insert("character", location.character());
    }
    return result;
}

} // namespace control
