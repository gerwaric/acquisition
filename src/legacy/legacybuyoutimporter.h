// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QString>

#include <utility>

class BuyoutRepo;
class CharacterRepo;
class StashRepo;

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

struct LegacyBuyoutApplyReport
{
    bool success{false};
    qint64 imported{0};
    qint64 already_present{0};
    qint64 protected_manual{0};
    qint64 skipped{0};
    qint64 errors{0};
    QString error;
    // Set when the buyouts were applied but a post-commit step (saving
    // the annotated workbook) failed; success stays true.
    QString warning;

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

    LegacyBuyoutPlanReport createPlan(const QString &source_filename, const QString &plan_filename);
    LegacyBuyoutApplyReport applyPlan(const QString &plan_filename);

private:
    BuyoutRepo &m_repo;
    StashRepo *m_stashes{nullptr};
    CharacterRepo *m_characters{nullptr};
    QString m_realm;
    QString m_league;
};
