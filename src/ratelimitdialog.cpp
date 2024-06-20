/*
	Copyright 2024 Gerwaric

	This file is part of Acquisition.

	Acquisition is free software: you can redistribute it and/or modify
	it under the terms of the GNU General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	Acquisition is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU General Public License for more details.

	You should have received a copy of the GNU General Public License
	along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/
#include "ratelimitdialog.h"

#include <QCoreApplication>
#include <QPushButton>
#include <QSizePolicy>
#include <QTreeWidget>
#include <QTreeWidgetItem>
#include <QLabel>
#include <QVBoxLayout>

#include "ratelimit.h"
#include "ratelimiter.h"
#include "util.h"

RateLimitDialog::RateLimitDialog(QWidget* parent, RateLimiter* limiter)
	: QDialog(parent)
{
	setSizeGripEnabled(true);

	// The spaces at the end of the header labels are intentional. Without them,
	// Qt was cutting off the last character or two. I could not figure out how
	// to avoid this.
	const QStringList columns = { "Policy / Rule  ", "Hits / Limit  ", "Period (s)  ", "Timeout (s)  ", "Status  " };

	setWindowTitle("Acquisition : Rate Limit Status Window");

	treeWidget = new QTreeWidget;
	treeWidget->setSelectionMode(QAbstractItemView::NoSelection);
	treeWidget->setEditTriggers(QAbstractItemView::NoEditTriggers);
	treeWidget->setColumnCount(columns.size());
	treeWidget->setHeaderLabels(columns);
	treeWidget->setFrameShape(QFrame::StyledPanel);
	treeWidget->setFrameShadow(QFrame::Sunken);
	treeWidget->setSizeAdjustPolicy(QAbstractScrollArea::SizeAdjustPolicy::AdjustToContents);
	treeWidget->setSortingEnabled(false);
	treeWidget->setUniformRowHeights(true);
	treeWidget->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);

	for (int i = 0; i < treeWidget->columnCount(); ++i) {
		treeWidget->resizeColumnToContents(i);
	};

	statusLabel = new QLabel;
	statusLabel->setText("Rate limit status: UNKNOWN");
	statusLabel->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);

	refreshButton = new QPushButton;
	refreshButton->setText("Refresh");
	refreshButton->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);
	connect(refreshButton, &QPushButton::clicked, this, &RateLimitDialog::OnRefreshRequested);

	layout = new QVBoxLayout(this);
	layout->setContentsMargins(15, 15, 15, 15);
	layout->addWidget(treeWidget);
	layout->addWidget(statusLabel);
	layout->addWidget(refreshButton);

	resize(600, 400);
	setMinimumWidth(600);
	setMinimumHeight(400);

	connect(limiter, &RateLimiter::PolicyUpdate, this, &RateLimitDialog::OnPolicyUpdate);
	connect(limiter, &RateLimiter::Paused, this, &RateLimitDialog::OnPause);
	connect(this, &RateLimitDialog::RequestUpdate, limiter, &RateLimiter::OnUpdateRequested);
}

void RateLimitDialog::OnRefreshRequested()
{
	emit RequestUpdate();
}

void RateLimitDialog::OnPolicyUpdate(const RateLimit::Policy& policy)
{
	// Look for an existing top-level item and remove it.
	for (int i = 0; i < treeWidget->topLevelItemCount(); ++i) {
		QTreeWidgetItem* item = treeWidget->topLevelItem(i);
		if (item->text(0) == policy.name()) {
			delete treeWidget->takeTopLevelItem(i);
			break;
		};
	};
	// Create a new top-level item.
	QTreeWidgetItem* policy_item = new QTreeWidgetItem(treeWidget);
	policy_item->setText(0, policy.name());
	policy_item->setExpanded(true);
	for (const auto& rule : policy.rules()) {
		for (const auto& item : rule.items()) {
			QTreeWidgetItem* rule_item = new QTreeWidgetItem(policy_item);
			rule_item->setText(0, QString("%1 (%2s)").arg(rule.name()).arg(item.limit().period()));
			rule_item->setText(1, QString("%1 / %2").arg(
				QString::number(item.state().hits()),
				QString::number(item.limit().hits())));
			rule_item->setText(2, QString::number(item.limit().period()));
			rule_item->setText(3, QString::number(item.limit().restriction()));
			rule_item->setText(4, Util::toString(item.status()));
			rule_item->setExpanded(true);
		};
	};

	// Sort the rows by policy name.
	treeWidget->sortByColumn(0, Qt::SortOrder::AscendingOrder);

	// Resize the columns.
	for (int i = 0; i < treeWidget->columnCount(); ++i) {
		treeWidget->resizeColumnToContents(i);
	};
}

void RateLimitDialog::OnPause(int pause, const QString& policy)
{
	if (pause <= 0) {
		statusLabel->setText("Not rate limited");
	} else {
		statusLabel->setText(QString("Paused for %1 seconds due to %2").arg(QString::number(pause), policy));
	};
}
