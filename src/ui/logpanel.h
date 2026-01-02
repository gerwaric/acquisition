// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QObject>

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
    virtual bool isValid() { return true; }
public slots:
    void TogglePanelVisibility();

private:
    void UpdateStatusLabel();

    QPushButton *m_status_button;
    QTextEdit *m_output;

    unsigned int m_num_errors{0};
    unsigned int m_num_warnings{0};
    unsigned int m_num_messages{0};
};
