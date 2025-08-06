/*
    Copyright (C) 2014-2025 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "legacyitem.h"

#include <QRegularExpression>

#include <util/spdlog_qt.h>

QString LegacyItem::effectiveTypeLine() const
{
    QString result;
    // Acquisition uses the base typeline for vaal gems.
    if (hybrid && (!hybrid->isVaalGem || (*hybrid->isVaalGem == false))) {
        result = hybrid->baseTypeName;
    } else {
        result = typeLine;
    }
    // Remove legacy set information.
    static const QRegularExpression re("^(<<.*?>>)*");
    result.replace(re, "");
    return result;
}

QString LegacyItem::hash() const
{
    if (_character && _tab_label) {
        spdlog::error("LegacyItem::hash() item contains both '_character' and '_tab_label': {} {}",
                      name,
                      id);
        return QString();
    }

    // This code is intended to exactly replicate the hash calulculated by leqacy acquisition.
    QString input = name + "~" + effectiveTypeLine() + "~";

    // Add explicit mods.
    if (explicitMods) {
        for (const auto &mod : *explicitMods) {
            input += mod + "~";
        }
    }

    // Add implicit mods.
    if (implicitMods) {
        for (const auto &mod : *implicitMods) {
            input += mod + "~";
        }
    }

    // Add properties.
    if (properties) {
        for (const auto &prop : *properties) {
            input += prop.name + "~";
            for (const auto &val : prop.values) {
                input += std::get<0>(val) + "~";
            }
        }
    }
    input += "~";

    // Add additional properties.
    if (additionalProperties) {
        for (const auto &prop : *additionalProperties) {
            input += prop.name + "~";
            for (const auto &val : prop.values) {
                input += std::get<0>(val) + "~";
            }
        }
    }
    input += "~";

    // Add sockets.
    if (sockets) {
        for (const auto &socket : *sockets) {
            if (socket.attr) {
                input += QString::number(socket.group) + "~" + *socket.attr + "~";
            }
        }
    }

    // Finish with the location tag.
    if (_character) {
        input += "~character:" + *_character;
    } else {
        input += "~stash:" + *_tab_label;
    }

    const QString result = QString(
        QCryptographicHash::hash(input.toUtf8(), QCryptographicHash::Md5).toHex());
    return result;
}
