// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QDir>
#include <QObject>
#include <QString>

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

private:
    struct Impl;
    std::unique_ptr<Impl> m_impl;
};
