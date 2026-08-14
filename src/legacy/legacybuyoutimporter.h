// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QString>

#include <utility>

class BuyoutRepo;
class CharacterRepo;
class StashRepo;

struct LegacyBuyoutImportReport
{
    bool success{false};
    qint64 imported{0};
    qint64 ambiguous{0};
    qint64 orphaned{0};
    qint64 skipped{0};
    QString error;

    QString summary() const;
};

struct LegacyBuyoutPlanReport
{
    bool success{false};
    qint64 total{0};
    qint64 matched{0};
    qint64 ambiguous{0};
    qint64 orphaned{0};
    qint64 skipped{0};
    qint64 rows{0};
    QString plan_file;
    QString error;

    QString summary() const;
};

class LegacyBuyoutImporter
{
public:
    explicit LegacyBuyoutImporter(BuyoutRepo &repo)
        : m_repo(repo)
    {}

    LegacyBuyoutImporter(BuyoutRepo &repo,
                         StashRepo &stashes,
                         CharacterRepo &characters,
                         QString realm,
                         QString league)
        : m_repo(repo)
        , m_stashes(&stashes)
        , m_characters(&characters)
        , m_realm(std::move(realm))
        , m_league(std::move(league))
    {}

    LegacyBuyoutImportReport importFile(const QString &filename);
    LegacyBuyoutPlanReport createPlan(const QString &source_filename, const QString &plan_filename);

private:
    BuyoutRepo &m_repo;
    StashRepo *m_stashes{nullptr};
    CharacterRepo *m_characters{nullptr};
    QString m_realm;
    QString m_league;
};
