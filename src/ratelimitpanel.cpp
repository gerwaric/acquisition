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

#include "ratelimitpanel.h"

#include <QPushButton>
#include <QTextEdit>

#include "mainwindow.h"
#include "ui_mainwindow.h"

// Modeled after LogPanel
RateLimitStatusPanel::RateLimitStatusPanel(MainWindow* window, Ui::MainWindow* ui) :
	QObject(window),
	status_button_(new QPushButton),
	output_(new QTextEdit)
{
	QFont font("Monospace");
	font.setStyleHint(QFont::TypeWriter);
	output_->setReadOnly(true);
	output_->setFont(font);
	output_->setMaximumHeight(200); // TODO(xyz): remove magic numbers
	output_->setText("Rate limit status will be displayed here.\n");
	output_->hide();

	status_button_->setFlat(true);
	status_button_->setText("Rate Limit Status");
	window->statusBar()->addPermanentWidget(status_button_);
	QObject::connect(status_button_, &QPushButton::clicked, this, &RateLimitStatusPanel::OnStatusLabelClicked);

	ui->mainLayout->addWidget(output_);
}

void RateLimitStatusPanel::OnStatusLabelClicked() {
	if (output_->isVisible()) {
		output_->hide();
	} else {
		output_->show();
	};
}

void RateLimitStatusPanel::OnStatusUpdate(const QString& message) {
	output_->setText(message);
}