// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QDir>
#include <QObject>
#include <QString>

class BuyoutRepo;
class CharacterRepo;
class StashRepo;

class UserStore : public QObject
{
    Q_OBJECT
public:
    explicit UserStore(const QDir &dir, const QString &username);
    ~UserStore();

    CharacterRepo &characters();
    StashRepo &stashes();
    BuyoutRepo &buyouts();

private:
    struct Impl;
    std::unique_ptr<UserStore::Impl> m_impl;
};
