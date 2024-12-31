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

#pragma once

#include <QDialog>

class QLabel;
class QTreeWidget;
class QVBoxLayout;

class QString;

class RateLimiter;
class RateLimitPolicy;

class RateLimitDialog : public QDialog
{
    Q_OBJECT
public:
    explicit RateLimitDialog(QWidget* parent, RateLimiter* limiter);
public slots:
    void OnPause(int pause, const QString& policy_name);
    void OnPolicyUpdate(const RateLimitPolicy& policy);
    void OnQueueUpdate(const QString& policy_name, int queue_size);
private:
    QVBoxLayout* m_layout;
    QTreeWidget* m_treeWidget;
    QLabel* m_statusLabel;
};
