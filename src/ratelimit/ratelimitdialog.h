// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Tom Holz

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
    explicit RateLimitDialog(QWidget *parent, RateLimiter *limiter);
public slots:
    void OnPause(int pause, const QString &policy_name);
    void OnPolicyUpdate(const RateLimitPolicy &policy);
    void OnQueueUpdate(const QString &policy_name, int queue_size);

private:
    QVBoxLayout *m_layout;
    QTreeWidget *m_treeWidget;
    QLabel *m_statusLabel;
};
