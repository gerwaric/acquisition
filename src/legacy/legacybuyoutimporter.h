// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QString>

class BuyoutRepo;

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

class LegacyBuyoutImporter
{
public:
    explicit LegacyBuyoutImporter(BuyoutRepo &repo)
        : m_repo(repo)
    {}

    LegacyBuyoutImportReport importFile(const QString &filename);

private:
    BuyoutRepo &m_repo;
};
