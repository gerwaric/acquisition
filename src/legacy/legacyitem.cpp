// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "legacy/legacyitem.h"

#include <QRegularExpression>

#include "util/spdlog_qt.h" // IWYU pragma: keep

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
