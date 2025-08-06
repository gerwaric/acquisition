/*
    Copyright (C) 2014-2025 Acquisition Contributors

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
