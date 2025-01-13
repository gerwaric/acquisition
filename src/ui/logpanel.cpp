/*
    Copyright (C) 2014-2024 Acquisition Contributors

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
    const QColor color;
    const char* desc;
};

// Colors for different message types. Unfortunately this is hard-coded
// and does not account for different themes.
constexpr std::array<MessageType, 3> message_types{ {
    { QColor(), "message" },
    { QColor(174, 141, 28), "warning" },
    { QColor(255, 0, 0), "error" }} };

LogPanel::LogPanel(MainWindow* window, Ui::MainWindow* ui)
    : m_status_button(new QPushButton)
    , m_output(new QTextEdit)
    , m_signal_handler(*this)
{
    m_num_messages.resize(message_types.size());

    QFont font("Monospace");
    font.setStyleHint(QFont::TypeWriter);
    m_output->setReadOnly(true);
    m_output->setFont(font);
    m_output->setMaximumHeight(250); // TODO(xyz): remove magic numbers
    m_output->insertPlainText("Errors and warnings will be printed here\n");
    m_output->hide();

    m_status_button->setFlat(false);
    window->statusBar()->addPermanentWidget(m_status_button);
    UpdateStatusLabel();
    QObject::connect(m_status_button, &QPushButton::clicked, &m_signal_handler, &LogPanelSignalHandler::OnStatusLabelClicked);

    ui->mainLayout->addWidget(m_output);
}

void LogPanel::UpdateStatusLabel() {
    QString stylesheet;
    QString text = "Event Log";
    for (size_t i = m_num_messages.size(); i > 0; --i) {
        int num = m_num_messages[i - 1];
        auto& type = message_types[i - 1];
        if (num > 0) {
            text = QString("%1 " + QString(type.desc)).arg(num) + (num > 1 ? "s" : "");
            stylesheet = "font-weight: bold; color: " + type.color.name();
            break;
        }
    }

    m_status_button->setStyleSheet(stylesheet);
    m_status_button->setText(text);
}

void LogPanel::write(const QString& message, QsLogging::Level level) {
    // this is done like this because write() can be called from different threads,
    // so we let Qt signals/slots system take care of boring synchronization stuff
    QMetaObject::invokeMethod(&m_signal_handler, "OnMessage", Qt::QueuedConnection,
        Q_ARG(QString, message), Q_ARG(QsLogging::Level, level));
}

void LogPanel::AddLine(const QString& message, QsLogging::Level level) {
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

    m_num_messages[type]++;

    const auto weight = m_output->fontWeight();
    const auto color = m_output->textColor();

    // Insert the message with the right weight and color.
    QString line = message;
    if (level != QsLogging::InfoLevel) {
        line = "<b>" + line + "</b>";
    };
    if (level == QsLogging::ErrorLevel) {
        line = "<font color='red'>" + line + "</font>";
    };
    m_output->moveCursor(QTextCursor::End);
    m_output->insertHtml(line + "<br>");
    m_output->ensureCursorVisible();

    // Restore the original font weight and color.
    m_output->setFontWeight(weight);
    m_output->setTextColor(color);

    if (m_output->isVisible()) {
        m_num_messages.clear();
        m_num_messages.resize(message_types.size());
    }
    UpdateStatusLabel();
}

void LogPanel::ToggleOutputVisibility() {
    if (m_output->isVisible()) {
        m_output->hide();
    } else {
        m_output->show();
        m_num_messages.clear();
        m_num_messages.resize(message_types.size());
        UpdateStatusLabel();
    }
}

void LogPanelSignalHandler::OnStatusLabelClicked() {
    m_parent.ToggleOutputVisibility();
}

void LogPanelSignalHandler::OnMessage(const QString& message, const QsLogging::Level level) {
    m_parent.AddLine(message, level);
}
