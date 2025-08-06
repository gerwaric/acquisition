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
#include <QString>
#include <QStringList>

class QNetworkAccessManager;
class QNetworkReply;

enum class ProgramState;

class RePoE : public QObject
{
    Q_OBJECT
public:
    RePoE(QNetworkAccessManager &network_manager);
    void Init(const QString &data_dir);
    bool IsInitialized() const { return m_initialized; };
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
    QNetworkAccessManager &m_network_manager;
    QString m_data_dir;
    std::vector<QString> m_needed_files;
};
