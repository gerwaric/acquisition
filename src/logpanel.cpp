/*
	Copyright 2014 Ilya Zhuravlev

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

#include "logpanel.h"

#include <QObject>
#include <QPushButton>
#include <QTextEdit>
#include <QStatusBar>

#include "mainwindow.h"
#include "ui_mainwindow.h"

struct MessageType {
	QColor color;
	std::string desc;
};

// Colors for different message types. Unfortunately this is hard-coded
// and does not account for different themes.
static std::vector<MessageType> message_types{
	{ QColor(), "message" },
	{ QColor(174, 141, 28), "warning" },
	{ QColor(255, 0, 0), "error" }
};

LogPanel::LogPanel(MainWindow* window, Ui::MainWindow* ui) :
	status_button_(new QPushButton),
	output_(new QTextEdit),
	signal_handler_(*this)
{
	num_messages_.resize(message_types.size());

	QFont font("Monospace");
	font.setStyleHint(QFont::TypeWriter);
	output_->setReadOnly(true);
	output_->setFont(font);
	output_->setMaximumHeight(250); // TODO(xyz): remove magic numbers
	output_->insertPlainText("Errors and warnings will be printed here\n");
	output_->hide();

	status_button_->setFlat(true);
	window->statusBar()->addPermanentWidget(status_button_);
	UpdateStatusLabel();
	QObject::connect(status_button_, &QPushButton::clicked, &signal_handler_, &LogPanelSignalHandler::OnStatusLabelClicked);

	ui->mainLayout->addWidget(output_);
}

void LogPanel::UpdateStatusLabel() {
	QString stylesheet;
	QString text = "Event Log";
    for (size_t i = num_messages_.size(); i > 0; --i) {
        int num = num_messages_[i-1];
        auto& type = message_types[i-1];
		if (num > 0) {
			text = QString("%1 " + QString(type.desc.c_str())).arg(num) + (num > 1 ? "s" : "");
			stylesheet = "font-weight: bold; color: " + type.color.name();
			break;
		}
	}

	status_button_->setStyleSheet(stylesheet);
	status_button_->setText(text);
}

void LogPanel::write(const QString& message, QsLogging::Level level) {
	// this is done like this because write() can be called from different threads,
	// so we let Qt signals/slots system take care of boring synchronization stuff
	QMetaObject::invokeMethod(&signal_handler_, "OnMessage", Qt::QueuedConnection,
		Q_ARG(QString, message), Q_ARG(QsLogging::Level, level));
}

void LogPanel::AddLine(const QString& message, QsLogging::Level level) {
	QColor color;
	int type;
	switch (level) {
	case QsLogging::InfoLevel:
		type = 0;
		break;
	case QsLogging::WarnLevel:
		type = 1;
		break;
	case QsLogging::ErrorLevel:
		type = 2;
		break;
	default:
		return;
	}

	num_messages_[type]++;
	color = message_types[type].color;

	output_->moveCursor(QTextCursor::End);
	if (level != QsLogging::InfoLevel) {
		// Don't set the text color for basic info messages
		// because they may be unreadable on dark themes.
		//
		// The real solution is to have the colors depend
		// on the theme somehow.
		output_->setTextColor(color);
	};
	output_->insertPlainText(message + "\n");
	output_->ensureCursorVisible();

	if (output_->isVisible()) {
		num_messages_.clear();
		num_messages_.resize(message_types.size());
	}
	UpdateStatusLabel();
}

void LogPanel::ToggleOutputVisibility() {
	if (output_->isVisible()) {
		output_->hide();
	} else {
		output_->show();
		num_messages_.clear();
		num_messages_.resize(message_types.size());
		UpdateStatusLabel();
	}
}

void LogPanelSignalHandler::OnStatusLabelClicked() {
	parent_.ToggleOutputVisibility();
}

void LogPanelSignalHandler::OnMessage(const QString& message, const QsLogging::Level level) {
	parent_.AddLine(message, level);
}
