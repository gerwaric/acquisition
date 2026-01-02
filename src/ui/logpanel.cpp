// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "ui/logpanel.h"

#include <QFontDatabase>
#include <QObject>
#include <QPushButton>
#include <QStatusBar>
#include <QTextEdit>

#include <spdlog/sinks/callback_sink.h>
#include <spdlog/sinks/qt_sinks.h>

#include "ui/mainwindow.h"
#include "ui_mainwindow.h"

constexpr QColor ERROR_COLOR = QColor(255, 0, 0);
constexpr QColor WARNING_COLOR = QColor(174, 141, 28);

constexpr int MAX_LINES = 200;
constexpr bool DARK_COLORS = false;
constexpr bool IS_UTF8 = true;

LogPanel::LogPanel(MainWindow *window, Ui::MainWindow *ui)
    : QObject(window)
{
    // Create the UI widgets.
    m_status_button = new QPushButton(window);
    m_output = new QTextEdit(window);

    // Setup the output panel.
    m_output->hide();
    m_output->setReadOnly(true);
    m_output->setFont(QFontDatabase::systemFont(QFontDatabase::FixedFont));
    m_output->insertPlainText("Errors and warnings will be printed here\n");
    m_output->setMaximumHeight(250); // TODO(xyz): remove magic numbers

    // Setup the status button.
    m_status_button->setFlat(false);
    window->statusBar()->addPermanentWidget(m_status_button);

    // Get the status label ready.
    UpdateStatusLabel();

    QObject::connect(m_status_button, &QPushButton::clicked, this, &LogPanel::TogglePanelVisibility);

    // Create a sinke for the text panel.
    auto panel_sink = std::make_shared<spdlog::sinks::qt_color_sink_mt>(m_output,
                                                                        MAX_LINES,
                                                                        DARK_COLORS,
                                                                        IS_UTF8);
    panel_sink->set_level(spdlog::level::warn);

    // Create a custom callback sink to change the status button label.
    auto callback_sink = std::make_shared<spdlog::sinks::callback_sink_mt>(
        [this](const spdlog::details::log_msg &msg) {
            if (msg.level >= spdlog::level::err) {
                ++m_num_errors;
            } else if (msg.level == spdlog::level::warn) {
                ++m_num_warnings;
            }
            UpdateStatusLabel();
        });
    callback_sink->set_level(spdlog::level::warn);

    // Attach the sinks to the logger.
    auto logger = spdlog::get("main");
    logger->sinks().push_back(panel_sink);
    logger->sinks().push_back(callback_sink);

    ui->mainLayout->addWidget(m_output);
}

void LogPanel::UpdateStatusLabel()
{
    QString label = "Event Log";
    QString style = "";
    unsigned int k = 0;

    if (m_num_errors > 0) {
        label = "error(s)";
        style = "font-weight: bold; color: " + ERROR_COLOR.name();
        k = m_num_errors;
    } else if (m_num_warnings > 0) {
        label = "warning(s)";
        style = "font-weight: bold; color: " + WARNING_COLOR.name();
        k = m_num_warnings;
    } else if (m_num_messages > 0) {
        label = "message(s)";
        k = m_num_messages;
    }

    if (k > 0) {
        label = QString("%1 %2").arg(QString::number(k), label);
    }

    m_status_button->setStyleSheet(style);
    m_status_button->setText(label);
}

void LogPanel::TogglePanelVisibility()
{
    if (m_output->isVisible()) {
        m_output->hide();
    } else {
        // Since we'll be showing the panel to the user, we can
        // reset the message counters.
        m_output->show();
        m_num_messages = 0;
        m_num_warnings = 0;
        m_num_errors = 0;
        UpdateStatusLabel();
    }
}
