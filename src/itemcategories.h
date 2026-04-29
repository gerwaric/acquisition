// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2022 testpushpleaseignore <elnino2k10@gmail.com>

#pragma once

#include <QByteArray>
#include <QString>
#include <QStringList>

void InitItemClasses(const QByteArray &classes);
void InitItemBaseTypes(const QByteArray &baseTypes);

QString GetItemCategory(const QString &baseType);

const QStringList &GetItemCategories();
