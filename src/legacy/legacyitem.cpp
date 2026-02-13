// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "legacy/legacyitem.h"

#include <QRegularExpression>

#include "util/spdlog_qt.h" // IWYU pragma: keep

QString LegacyItem::effectiveTypeLine() const
{
    QString result;

    if (hybrid && !hybrid->isVaalGem.value_or(false)) {
        // Use the base type for hybrid items (except Vaal gems).
        result = hybrid->baseTypeName;
    } else {
        // Otherwise use the typeline.
        result = typeLine;
    }

    // Remove legacy set information.
    static const QRegularExpression re("^(<<.*?>>)*");
    result.replace(re, "");
    return result;
}

QString LegacyItem::hash() const
{
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
    switch (_type) {
    case LocationType::STASH:
        input += "~stash:" + *_tab_label;
    case LocationType::CHARACTER:
        input += "~character:" + *_character;
    }

    const QByteArray utf8 = input.toUtf8();
    const QByteArray hash = QCryptographicHash::hash(utf8, QCryptographicHash::Md5);
    return QString(hash.toHex());
}
