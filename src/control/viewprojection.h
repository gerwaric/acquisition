// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#pragma once

#include <QJsonObject>

struct Buyout;
class BuyoutManager;
class Item;
class ItemLocation;

namespace control {

QString LocationKind(const ItemLocation &location);
QJsonObject ProjectItem(const Item &item,
                        const ItemLocation &canonical_location,
                        const Buyout &effective_buyout);
QJsonObject ProjectTab(const ItemLocation &location,
                       const BuyoutManager &buyout_manager,
                       qsizetype item_count);

} // namespace control
