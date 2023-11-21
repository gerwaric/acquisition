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
#pragma once

#include <QDialog>

class QEvent;
class QLabel;
class QPushButton;
class QTreeWidget;
class QVBoxLayout;

namespace RateLimit {
	struct Policy;
}

class RateLimitDialog : public QDialog
{
	Q_OBJECT
public:
	explicit RateLimitDialog();
signals:
	void RequestUpdate();
public slots:
	void OnStatusUpdate(const QString& status);
	void OnPolicyUpdate(const RateLimit::Policy& policy);
private:
	QVBoxLayout* layout;
	QTreeWidget* treeWidget;
	QPushButton* refreshButton;
	QLabel* statusLabel;
};
