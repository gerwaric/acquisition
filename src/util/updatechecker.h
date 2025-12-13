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

#include <QList>
#include <QNetworkReply>
#include <QObject>
#include <QSslError>
#include <QTimer>
#include <QVersionNumber>

#include <semver/semver.hpp>

class QSettings;
class QWidget;

class NetworkManager;

class UpdateChecker : public QObject
{
    Q_OBJECT
public:
    explicit UpdateChecker(QSettings &settings, NetworkManager &network_manager);
signals:
    void UpdateAvailable();
public slots:
    void CheckForUpdates();
    void AskUserToUpdate();
private slots:
    void OnUpdateReplyReceived();
    void OnUpdateErrorOccurred(QNetworkReply::NetworkError code);
    void OnUpdateSslErrors(const QList<QSslError> &errors);

private:
    struct ReleaseTag
    {
        semver::version version;
        bool draft{false};
        bool prerelease{false};
    };
    std::vector<ReleaseTag> ParseReleaseTags(const QByteArray &bytes);

    bool has_newer_release() const;
    bool has_newer_prerelease() const;

    QSettings &m_settings;
    NetworkManager &m_nm;

    // Trigger periodic update checks.
    QTimer m_timer;

    // The running version
    const semver::version m_running_version;

    // The latest github release
    semver::version m_latest_release;
    semver::version m_latest_prerelease;

    semver::version m_previous_release;
    semver::version m_previous_prerelease;
};
