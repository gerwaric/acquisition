// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QObject>

#include <atomic>
#include <memory>

#include <spdlog/logger.h>
#include <spdlog/sinks/sink.h>

class MainWindow;
class QPushButton;
class QTextEdit;

namespace Ui {
    class MainWindow;
}

class LogPanel : public QObject
{
    Q_OBJECT
public:
    LogPanel(MainWindow *window, Ui::MainWindow *ui);
    ~LogPanel() override;
    virtual bool isValid() { return true; }
public slots:
    void TogglePanelVisibility();

private:
    void UpdateStatusLabel();

    QPushButton *m_status_button;
    QTextEdit *m_output;

    std::shared_ptr<spdlog::logger> m_logger;
    spdlog::sink_ptr m_panel_sink;
    spdlog::sink_ptr m_callback_sink;

    std::atomic_uint m_num_errors{0};
    std::atomic_uint m_num_warnings{0};
    std::atomic_uint m_num_messages{0};
};
