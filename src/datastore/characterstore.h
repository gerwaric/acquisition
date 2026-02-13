// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QObject>
#include <QString>

#include <optional>
#include <vector>

#include "poe/types/character.h"

class QString;

class CharacterStore : public QObject
{
    Q_OBJECT
public:
    explicit CharacterStore(QStringView connName);

    std::optional<poe::Character> getCharacter(const QString &name, const QString &realm);
    std::vector<poe::Character> getCharacterList(const QString &realm,
                                                 const std::optional<QString> league = {});

    bool resetRepo();
    bool ensureSchema();

public slots:
    bool saveCharacter(const poe::Character &character);
    bool saveCharacterList(const std::vector<poe::Character> &characters);

private:
    bool saveListTransaction(const std::vector<poe::Character> &characters);

    QString m_connName;
};
