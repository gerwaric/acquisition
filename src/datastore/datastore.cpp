// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "datastore/datastore.h"

void DataStore::SetInt(const QString &key, int value)
{
    Set(key, QString::number(value));
}

int DataStore::GetInt(const QString &key, int default_value)
{
    return Get(key, QString::number(default_value)).toInt();
}
