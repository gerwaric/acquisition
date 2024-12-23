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

#include "updatechecker.h"

#include <QDesktopServices>
#include <QMessageBox>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QPushButton>
#include <QRegularExpression>
#include <QSettings>
#include <QSizePolicy>
#include <QUrl>
#include <QWidget>

#include <QsLog/QsLog.h>
#include <rapidjson/document.h>
#include <rapidjson/error/en.h>
#include <cpp-semver/semver.hpp>

#include "network_info.h"
#include "util.h"
#include "version_defines.h"

constexpr const char* GITHUB_RELEASES_URL = "https://api.github.com/repos/gerwaric/acquisition/releases";
constexpr const char* GITHUB_DOWNLOADS_URL = "https://github.com/gerwaric/acquisition/releases";

// Check for updates every 24 hours.
constexpr int UPDATE_INTERVAL = 24 * 60 * 60 * 1000;

static const semver::version NULL_VERSION = semver::version();

UpdateChecker::UpdateChecker(
    QSettings& settings,
    QNetworkAccessManager& network_manager)
    : settings_(settings)
    , nm_(network_manager)
    , running_version_(semver::version::parse(APP_VERSION_STRING))
{
    const QString skip_release = settings_.value("skip_release").toString();
    const QString skip_prerelease = settings_.value("skip_prerelease").toString();
    previous_release_ = skip_release.isEmpty() ? semver::version() : semver::version::parse(skip_release.toStdString());
    previous_prerelease_ = skip_prerelease.isEmpty() ? semver::version() : semver::version::parse(skip_prerelease.toStdString());

    QLOG_DEBUG() << "UpdateChecker: running version is" << running_version_.str();
    QLOG_DEBUG() << "UpdateChecker: skipped release is" << previous_release_.str();
    QLOG_DEBUG() << "UpdateChecker: skipped prerelease is" << previous_prerelease_.str();

    timer_.setInterval(UPDATE_INTERVAL);
    timer_.start();
    connect(&timer_, &QTimer::timeout, this, &UpdateChecker::CheckForUpdates);
}

void UpdateChecker::CheckForUpdates() {
    // Get the releases from GitHub as a json object.
    QLOG_TRACE() << "UpdateChecker: requesting GitHub releases:" << GITHUB_RELEASES_URL;
    QNetworkRequest request = QNetworkRequest(QUrl(GITHUB_RELEASES_URL));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* reply = nm_.get(request);
    connect(reply, &QNetworkReply::errorOccurred, this, &UpdateChecker::OnUpdateErrorOccurred);
    connect(reply, &QNetworkReply::sslErrors, this, &UpdateChecker::OnUpdateSslErrors);
    connect(reply, &QNetworkReply::finished, this, &UpdateChecker::OnUpdateReplyReceived);
}

void UpdateChecker::OnUpdateErrorOccurred(QNetworkReply::NetworkError code) {
    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    QLOG_ERROR() << "Network error" << code << "occurred while checking for an update : " << reply->errorString();
}

void UpdateChecker::OnUpdateSslErrors(const QList<QSslError>& errors) {
    const int n = errors.size();
    QLOG_ERROR() << n << "SSL error(s) checking for an update:";
    for (int i = 0; i < n; ++i) {
        QLOG_ERROR() << "SSL error #" << i << "is" << errors[i].errorString();
    };
}

void UpdateChecker::OnUpdateReplyReceived() {
    // Process the releases received from GitHub.
    QLOG_TRACE() << "UpdateChecker: received an update reply from GitHub.";

    // Check for network errors.
    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    if (reply->error() != QNetworkReply::NoError) {
        QLOG_ERROR() << "The network reply came with an error:" << reply->errorString();
        return;
    };

    // Parse release tags from the json object.
    const QByteArray bytes = reply->readAll();
    const std::vector<ReleaseTag> releases = ParseReleaseTags(bytes);

    latest_release_ = NULL_VERSION;
    latest_prerelease_ = NULL_VERSION;

    for (const auto& release : releases) {
        if (release.prerelease) {
            if ((release.version > latest_prerelease_)) {
                latest_prerelease_ = release.version;
            };
        } else {
            if ((release.version > latest_release_)) {
                latest_release_ = release.version;
            };
        };
    };

    // Make sure at least one tag was found.
    if ((latest_release_ == NULL_VERSION) && (latest_prerelease_ == NULL_VERSION)) {
        QLOG_WARN() << "Unable to find any github releases or pre-releases!";
        return;
    };
    if (latest_release_ > NULL_VERSION) {
        QLOG_DEBUG() << "UpdateChecker: latest release found:" << latest_release_.str();
    };
    if (latest_prerelease_ > NULL_VERSION) {
        QLOG_DEBUG() << "UpdateChecker: latest prerelease found:" << latest_prerelease_.str();
    };

    // Send a signal if there's a new version from the last check.
    if (has_newer_release() || has_newer_prerelease()) {
        emit UpdateAvailable();
    };
}

std::vector<UpdateChecker::ReleaseTag> UpdateChecker::ParseReleaseTags(const QByteArray& bytes)
{
    // Parse the reply as a json document.
    rapidjson::Document doc;
    doc.Parse(bytes.constData());

    // Check for json errors.
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Error parsing github releases:" << rapidjson::GetParseError_En(doc.GetParseError());
        return {};
    };
    if (!doc.IsArray()) {
        QLOG_ERROR() << "Error parsing github releases: document was not an array";
        return {};
    };

    // Reserve the output vector.
    std::vector<ReleaseTag> releases;
    releases.reserve(doc.GetArray().Size());

    // Check each of the release objects.
    for (const auto& json : doc.GetArray()) {

        ReleaseTag release;

        // Parse the release version
        if (json.HasMember("tag_name") && json["tag_name"].IsString()) {
            QString version_string = json["tag_name"].GetString();
            if (version_string.startsWith("v", Qt::CaseInsensitive)) {
                version_string.remove(0, 1);
            };
            release.version = semver::version::parse(version_string.toStdString());
        };

        // Make sure we found a parseable version number
        if (release.version == NULL_VERSION) {
            QLOG_WARN() << "Github release does not contain a name:" << Util::RapidjsonSerialize(json);
        };

        // Parse the release flags
        release.draft = (json.HasMember("draft") && json["draft"].IsBool() && json["draft"].GetBool());
        release.prerelease = (json.HasMember("prerelease") && json["prerelease"].IsBool() && json["prerelease"].GetBool());

        // Add this release to the list.
        releases.push_back(release);
    };
    return releases;
}

bool UpdateChecker::has_newer_release() const {
    return (latest_release_ > previous_release_) && (running_version_ < latest_release_);
}

bool UpdateChecker::has_newer_prerelease() const {
    return (latest_prerelease_ > previous_prerelease_) && (running_version_ < latest_prerelease_);
}

void UpdateChecker::AskUserToUpdate() {

    if (!has_newer_release() && !has_newer_prerelease()) {
        QLOG_WARN() << "UpdateChecker: no newer versions available";
        //return;
    };

    // Setup the update message.
    QStringList lines;
    if (has_newer_release()) {
        lines.append("A newer release is available:");
        lines.append("   " + QString::fromStdString(latest_release_.str()));
    };
    if (has_newer_prerelease()) {
        if (latest_prerelease_ > latest_release_) {
            if (!lines.isEmpty()) {
                lines.append("");
            };
            lines.append("A newer prerelease is available:");
            lines.append("   " + QString::fromStdString(latest_prerelease_.str()));
        };
    };
    if (lines.isEmpty()) {
        QMessageBox::information(nullptr, "Acquisition Update Checker", "No updates appear to be available", QMessageBox::StandardButton::Ok);
        return;
    };
    const QString message = lines.join("\n");

    // Create the dialog box.
    QMessageBox msgbox(nullptr);
    msgbox.setWindowTitle("Acquisition Update Checker");
    msgbox.setText(message);
    auto accept_button = msgbox.addButton("  Go to Github  ", QMessageBox::AcceptRole);
    auto remind_button = msgbox.addButton("  Ignore  ", QMessageBox::RejectRole);
    auto ignore_button = msgbox.addButton("  Ignore, and don't ask again  ", QMessageBox::RejectRole);
    msgbox.setDefaultButton(ignore_button);

    // Resize the buttons so the text fits.
    accept_button->setMinimumWidth(accept_button->sizeHint().width());
    ignore_button->setMinimumWidth(ignore_button->sizeHint().width());
    remind_button->setMinimumWidth(remind_button->sizeHint().width());

    // Get the user's choice.
    msgbox.exec();
    const auto clicked = msgbox.clickedButton();

    // Save the latest releases into the settings file oinly if the
    // user asked to skip them in the future.
    if (clicked == ignore_button) {
        settings_.setValue("skip_release", QString::fromStdString(latest_release_.str()));
        settings_.setValue("skip_prerelease", QString::fromStdString(latest_prerelease_.str()));
    } else {
        settings_.setValue("skip_release", "");
        settings_.setValue("skip_prerelease", "");
    };

    // Open a desktop web browser window if the user clicked that button.
    if (clicked == accept_button) {
        QDesktopServices::openUrl(QUrl(GITHUB_DOWNLOADS_URL));
    };
}
