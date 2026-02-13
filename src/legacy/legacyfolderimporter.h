// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QDir>
#include <QObject>

class UserStore;

class LegacyFolderImporter : public QObject
{
    Q_OBJECT
public:
    LegacyFolderImporter(UserStore &store);

    void import(const QDir &folder);

private:
    UserStore &m_store;
};
