/*
    Copyright 2015 Ilya Zhuravlev

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

#include "cpp-semver/semver.hpp"

class QNetworkAccessManager;
class QSettings;
class QWidget;

class UpdateChecker : public QObject {
    Q_OBJECT
public:
    explicit UpdateChecker(QObject* parent,
        QSettings& settings,
        QNetworkAccessManager& network_manager);
signals:
    void UpdateAvailable();
public slots:
    void CheckForUpdates();
    void AskUserToUpdate();
private slots:
    void OnUpdateReplyReceived();
    void OnUpdateErrorOccurred(QNetworkReply::NetworkError code);
    void OnUpdateSslErrors(const QList<QSslError>& errors);
private:

    struct ReleaseTag {
        semver::version version;
        bool draft{ false };
        bool prerelease{ false };
    };
    std::vector<ReleaseTag> ParseReleaseTags(const QByteArray& bytes);

    bool has_newer_release() const;
    bool has_newer_prerelease() const;

    QSettings& settings_;
    QNetworkAccessManager& nm_;

    // Trigger periodic update checks.
    QTimer timer_;

    // The running version
    const semver::version running_version_;

    // The latest github release
    semver::version latest_release_;
    semver::version latest_prerelease_;

    semver::version previous_release_;
    semver::version previous_prerelease_;
};
