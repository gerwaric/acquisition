// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

#pragma once

#include <QComboBox>
#include <QCompleter>
#include <QProxyStyle>
#include <QTimer>

#include <util/spdlog_qt.h>

class QAbstractItemModel;

class SearchComboCompleter : public QCompleter
{
    Q_OBJECT
public:
    using QCompleter::QCompleter;
public slots:
    void complete(const QRect &rect = QRect());
};

class SearchComboStyle : public QProxyStyle
{
public:
    using QProxyStyle::QProxyStyle;
    int styleHint(StyleHint hint,
                  const QStyleOption *option = nullptr,
                  const QWidget *widget = nullptr,
                  QStyleHintReturn *returnData = nullptr) const override;

private:
    const int TOOLTIP_DELAY_MSEC = 50;
};

class SearchComboBox : public QComboBox
{
    Q_OBJECT
public:
    SearchComboBox(QAbstractItemModel *model, const QString &value, QWidget *parent = nullptr);
private slots:
    void OnTextEdited();
    void OnEditTimeout();
    void OnCompleterActivated(const QString &text);

private:
    SearchComboCompleter m_completer;
    QTimer m_edit_timer;
    bool m_skip_completer{false};
};

// TokenAndFilterProxy.h (or keep in main.cpp for a minimal example)
#include <QRegularExpression>
#include <QSortFilterProxyModel>

class TokenAndFilterProxy final : public QSortFilterProxyModel
{
    Q_OBJECT
public:
    explicit TokenAndFilterProxy(QObject *parent = nullptr)
        : QSortFilterProxyModel(parent)
    {
        // Optional: stable sorting not required; filtering only.
        setDynamicSortFilter(false);
    }

    void setQueryText(const QString &text)
    {
        beginFilterChange();
        m_tokens = text.split(s_sep, Qt::SkipEmptyParts);
        spdlog::info("FUZZY TOKENS: {}", m_tokens.join(", "));
        endFilterChange(QSortFilterProxyModel::Direction::Rows);
    }

protected:
    bool filterAcceptsRow(int sourceRow, const QModelIndex &sourceParent) const override
    {
        if (m_tokens.isEmpty()) {
            return true;
        }

        const int col = filterKeyColumn() < 0 ? 0 : filterKeyColumn();
        const QModelIndex idx = sourceModel()->index(sourceRow, col, sourceParent);
        const QString s = sourceModel()->data(idx, filterRole()).toString();

        // AND-of-substrings, case-insensitive.
        for (const QString &tok : m_tokens) {
            if (!s.contains(tok, Qt::CaseInsensitive)) {
                return false;
            }
            //spdlog::info("MATCH: {} in {}", tok, s);
        }
        return true;
    }

private:
    static const QRegularExpression s_sep;
    QStringList m_tokens;
};
