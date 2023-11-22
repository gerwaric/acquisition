/*
	Copyright 2023 Gerwaric

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

#include <QEvent>
#include <QPushButton>
#include <QTreeWidget>
#include <QTreeWidgetItem>
#include <QLabel>
#include <QVBoxLayout>

#include "ratelimit.h"

RateLimitDialog::RateLimitDialog() : QDialog(nullptr)
{
	setSizeGripEnabled(true);

	// The spaces at the end of the header labels are intentional. Without them,
	// Qt was cutting off the last character or two. I could not figure out how
	// to avoid this.

	setWindowTitle("Acquisition : Rate Limit Status Window");

	treeWidget = new QTreeWidget;
	treeWidget->setColumnCount(4);
	treeWidget->setHeaderLabels({ "Policy/Rule  ", "Limit  ", "Last Known State  ", "Last Known Status  " });
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
	refreshButton->setText("Refresh\n(TBD waiting for a change coming in patch 3.22.3 or 3.23)");
	refreshButton->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);

	layout = new QVBoxLayout(this);
	layout->setContentsMargins(15, 15, 15, 15);
	layout->addWidget(treeWidget);
	layout->addWidget(statusLabel);
	layout->addWidget(refreshButton);

	resize(500, 400);
}

void RateLimitDialog::OnStatusUpdate(const QString& status)
{
	statusLabel->setText(status);
}

void RateLimitDialog::OnPolicyUpdate(const RateLimit::Policy& policy)
{
	// Look for an existing top-level item and remove it.
	for (int i = 0; i < treeWidget->topLevelItemCount(); ++i) {
		QTreeWidgetItem* item = treeWidget->topLevelItem(i);
		if (item->text(0) == policy.name) {
			delete treeWidget->takeTopLevelItem(i);
			break;
		};
	};
	// Create a new top-level item.
	QTreeWidgetItem* policy_item = new QTreeWidgetItem(treeWidget);
	policy_item->setText(0, policy.name);
	policy_item->setExpanded(true);
	for (const auto& rule : policy.rules) {
		for (const auto& item : rule.items) {
			QTreeWidgetItem* rule_item = new QTreeWidgetItem(policy_item);
			rule_item->setText(0, rule.name + " (" + QString::number(item.limit.period) + "s)");
			rule_item->setText(1, QString(item.limit));
			rule_item->setText(2, QString(item.state));
			rule_item->setText(3, RateLimit::POLICY_STATE[item.status]);
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
