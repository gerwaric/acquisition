// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QString>

#include <vector>

class Item;
struct ItemProperty;
struct ItemPropertyValue;

QString ColorPropertyValue(const ItemPropertyValue &value);
QString FormatProperty(const ItemProperty &prop);
QString GenerateProperties(const Item &item);
QString GenerateRequirements(const Item &item);
QString getTextMods(const Item &item, const QString &modType, const char *modColor);
std::vector<QString> GenerateMods(const Item &item);
QString GenerateItemInfo(const Item &item, const QString &key, bool fancy);
