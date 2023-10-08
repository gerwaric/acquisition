/*
	Copyright 2023 Tom Holz

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

#include <QObject>

class QPushButton;
class QTextEdit;

#include "mainwindow.h"

// Modeled after LogPanel
class RateLimitStatusPanel : public QObject {
	Q_OBJECT
public:
	RateLimitStatusPanel(MainWindow* window, Ui::MainWindow* ui);
public slots:
	void OnStatusLabelClicked();
	void OnStatusUpdate(const QString& message);
private:
	QPushButton* status_button_;
	QTextEdit* output_;
};
