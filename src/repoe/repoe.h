// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QObject>
#include <QString>
#include <QStringList>

class QNetworkReply;

class NetworkManager;
enum class ProgramState;

class RePoE : public QObject
{
    Q_OBJECT
public:
    explicit RePoE(NetworkManager &network_manager, const QString &dataDir);
    void start();
    bool initialized() const { return m_initialized; }
signals:
    void StatusUpdate(ProgramState state, const QString &status);
    void finished();
public slots:
    void OnVersionReceived();
    void OnFileReceived();

private:
    void BeginUpdate();
    void RequestNextFile();
    void FinishUpdate();
    QByteArray ReadFile(const QString &filename);
    QString ParseVersion(const QString &contents);

    bool m_initialized;
    NetworkManager &m_network_manager;
    QString m_data_dir;
    std::vector<QString> m_needed_files;
};
