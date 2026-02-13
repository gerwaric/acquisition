// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2015 Ilya Zhuravlev

#pragma once

#include <QObject>
#include <QTimer>

#include <semver/semver.hpp>

class NetworkManager;
namespace app {
    class UserSettings;
}

class UpdateChecker : public QObject
{
    Q_OBJECT
public:
    explicit UpdateChecker(NetworkManager &network_manager, app::UserSettings &settings);

public slots:
    void setLastSkippedUpdates(const semver::version &release, const semver::version &prerelease);
    void CheckForUpdates();
    void AskUserToUpdate();

private slots:
    void OnUpdateReplyReceived();

signals:
    void UpdateAvailable();
    void updatesSkipped(const semver::version &release, const semver::version &prerelease);

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

    NetworkManager &m_network_manager;
    app::UserSettings &m_settings;

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
