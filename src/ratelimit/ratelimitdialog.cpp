// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

#include "ratelimit/ratelimitdialog.h"

#include <QCoreApplication>
#include <QLabel>
#include <QPushButton>
#include <QSizePolicy>
#include <QTreeWidget>
#include <QTreeWidgetItem>
#include <QVBoxLayout>

#include <utility>

#include "ratelimit/ratelimiter.h"
#include "ratelimit/ratelimitpolicy.h"
#include "util/util.h"

namespace {
    // Flush interval for coalesced queue updates (M1-M2/D10): small enough
    // that the displayed queue depth is never visibly stale — steady-state
    // arrivals are one per reply, seconds apart — while any single-loop-turn
    // burst collapses into one apply.
    constexpr int kQueueFlushIntervalMsec = 100;
} // namespace

RateLimitDialog::RateLimitDialog(QWidget *parent, RateLimiter *limiter)
    : QDialog(parent)
{
    setSizeGripEnabled(true);

    m_queue_flush_timer.setSingleShot(true);
    m_queue_flush_timer.setInterval(kQueueFlushIntervalMsec);
    connect(&m_queue_flush_timer, &QTimer::timeout, this, &RateLimitDialog::FlushQueueUpdates);

    // The spaces at the end of the header labels are intentional. Without them,
    // Qt was cutting off the last character or two. I could not figure out how
    // to avoid this.
    const QStringList columns = {"Policy / Rule  ",
                                 "Queue  ",
                                 "Hits / Limit  ",
                                 "Period (s)  ",
                                 "Timeout(s)  ",
                                 "Status  "};

    setWindowTitle("Acquisition : Rate Limit Status Window");

    m_treeWidget = new QTreeWidget;
    m_treeWidget->setSelectionMode(QAbstractItemView::NoSelection);
    m_treeWidget->setEditTriggers(QAbstractItemView::NoEditTriggers);
    m_treeWidget->setColumnCount(columns.size());
    m_treeWidget->setHeaderLabels(columns);
    m_treeWidget->setFrameShape(QFrame::StyledPanel);
    m_treeWidget->setFrameShadow(QFrame::Sunken);
    m_treeWidget->setSizeAdjustPolicy(QAbstractScrollArea::SizeAdjustPolicy::AdjustToContents);
    m_treeWidget->setSortingEnabled(false);
    m_treeWidget->setUniformRowHeights(true);
    m_treeWidget->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);

    for (int i = 0; i < m_treeWidget->columnCount(); ++i) {
        m_treeWidget->resizeColumnToContents(i);
    }

    m_statusLabel = new QLabel;
    m_statusLabel->setText("Rate limit status: UNKNOWN");
    m_statusLabel->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);

    m_layout = new QVBoxLayout(this);
    m_layout->setContentsMargins(15, 15, 15, 15);
    m_layout->addWidget(m_treeWidget);
    m_layout->addWidget(m_statusLabel);

    resize(600, 400);
    setMinimumWidth(600);
    setMinimumHeight(400);

    connect(limiter, &RateLimiter::PolicyUpdate, this, &RateLimitDialog::OnPolicyUpdate);
    connect(limiter, &RateLimiter::QueueUpdate, this, &RateLimitDialog::OnQueueUpdate);
    connect(limiter, &RateLimiter::Paused, this, &RateLimitDialog::OnPause);
}

void RateLimitDialog::OnPolicyUpdate(const RateLimitPolicy &policy)
{
    QString queued_state;

    // Look for an existing top-level item and remove it.
    for (int i = 0; i < m_treeWidget->topLevelItemCount(); ++i) {
        QTreeWidgetItem *item = m_treeWidget->topLevelItem(i);
        if (item->text(0) == policy.name()) {
            queued_state = item->text(1);
            delete m_treeWidget->takeTopLevelItem(i);
            break;
        }
    }
    // Create a new top-level item.
    QTreeWidgetItem *policy_item = new QTreeWidgetItem(m_treeWidget);
    policy_item->setFlags(Qt::ItemIsEnabled);
    policy_item->setText(0, policy.name());
    policy_item->setText(1, queued_state);
    for (const auto &rule : policy.rules()) {
        for (const auto &item : rule.items()) {
            const auto &limit = item.limit();
            const auto &state = item.state();
            QTreeWidgetItem *rule_item = new QTreeWidgetItem(policy_item);
            rule_item->setFlags(Qt::ItemIsEnabled);
            rule_item->setText(0, QString("%1 (%2s)").arg(rule.name()).arg(limit.period()));
            rule_item->setText(2, QString("%1 / %2").arg(state.hits()).arg(limit.hits()));
            rule_item->setText(3, QString::number(limit.period()));
            rule_item->setText(4, QString::number(limit.restriction()));
            rule_item->setText(5, Util::toString(item.status()));
            rule_item->setExpanded(true);
        }
    }
    policy_item->setExpanded(true);

    // Sort the rows by policy name.
    m_treeWidget->sortByColumn(0, Qt::SortOrder::AscendingOrder);

    // Resize the columns.
    for (int i = 0; i < m_treeWidget->columnCount(); ++i) {
        m_treeWidget->resizeColumnToContents(i);
    }
}

void RateLimitDialog::OnQueueUpdate(const QString &policy_name, int queued_requests)
{
    m_pending_queue_updates.insert(policy_name, queued_requests);
    // Started only when idle (never restarted): the display is at most one
    // interval behind no matter how the burst is shaped.
    if (!m_queue_flush_timer.isActive()) {
        m_queue_flush_timer.start();
    }
}

void RateLimitDialog::FlushQueueUpdates()
{
    // Rows are looked up by name at flush time, not captured at arrival:
    // OnPolicyUpdate replaces the row object wholesale, and a replacement
    // between arrival and flush would leave a captured pointer dangling.
    const auto pending = std::exchange(m_pending_queue_updates, {});
    for (auto it = pending.cbegin(); it != pending.cend(); ++it) {
        for (int i = 0; i < m_treeWidget->topLevelItemCount(); ++i) {
            QTreeWidgetItem *item = m_treeWidget->topLevelItem(i);
            if (item->text(0) == it.key()) {
                if (it.value() > 0) {
                    item->setText(1, QString::number(it.value()));
                } else {
                    item->setText(1, "");
                }
                break;
            }
        }
    }
}

void RateLimitDialog::OnPause(int pause, const QString &policy)
{
    if (pause <= 0) {
        m_statusLabel->setText("Not rate limited");
    } else {
        m_statusLabel->setText(
            QString("Paused for %1 seconds due to %2").arg(QString::number(pause), policy));
    }
}
