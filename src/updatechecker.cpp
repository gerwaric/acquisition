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

#include "QsLog.h"
#include "rapidjson/document.h"
#include "rapidjson/error/en.h"

#include "network_info.h"
#include "version_defines.h"

constexpr const char* GITHUB_RELEASES_URL = "https://api.github.com/repos/gerwaric/acquisition/releases";
constexpr const char* GITHUB_DOWNLOADS_URL = "https://github.com/gerwaric/acquisition/releases";

// Check for updates every 24 hours.
constexpr int UPDATE_INTERVAL = 24 * 60 * 60 * 1000;

UpdateChecker::UpdateChecker(QObject* parent,
    QSettings& settings,
    QNetworkAccessManager& network_manager)
    : QObject(parent)
    , settings_(settings)
    , nm_(network_manager)
{
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
    QStringList tags;
    std::vector<bool> prerelease_flags;
    ParseReleaseTags(reply->readAll(), tags, prerelease_flags);

    // Find the index of the first release tag.
    int kRelease = -1;
    for (auto i = 0; i < tags.size(); ++i) {
        if (!prerelease_flags[i]) {
            kRelease = i;
            break;
        };
    };

    // Find the index of the first pre-release tag.
    int kPrerelease = -1;
    for (auto i = 0; i < tags.size(); ++i) {
        if (prerelease_flags[i]) {
            kPrerelease = i;
            break;
        };
    };

    // Make sure at least one tag was found.
    if ((kRelease < 0) && (kPrerelease < 0)) {
        QLOG_WARN() << "Unable to find any github releases or pre-releases!";
        return;
    };

    // Find the index of the currently running build.
    const auto kApp = tags.indexOf(APP_VERSION_STRING, Qt::CaseInsensitive);
    if (kApp < 0) {
        QLOG_WARN() << "No github tag matches the running version (" APP_VERSION_STRING ")";
    };

    // Create helpful variable to keep track of what we know now.
    bool has_newer_release = (kRelease >= 0);
    bool has_newer_prerelease = (kPrerelease >= 0) && (kPrerelease < kRelease);

    // If we found the current build in the list from GitHub, make sure that
    // the "newer" releases are actually newer.
    //
    // Lower indexes mean newer releases, because that's the order GitHub
    // returns them in.
    if (kApp >= 0) {
        has_newer_release &= (kRelease < kApp);
        has_newer_prerelease &= (kPrerelease < kApp);
    };

    // Now save the appropriate tags.
    latest_release_ = (kRelease >= 0) ? tags[kRelease] : "";
    latest_prerelease_ = (kPrerelease >= 0) ? tags[kPrerelease] : "";

    // Send a signal if there's a new version from the last check.
    if (has_newer_release || has_newer_prerelease) {
        emit UpdateAvailable();
    };
}

void UpdateChecker::ParseReleaseTags(const QByteArray& bytes,
    QStringList& tag_names,
    std::vector<bool>& prerelease_flags)
{
    // Parse the reply as a json document.
    rapidjson::Document doc;
    doc.Parse(bytes.constData());

    // Check for json errors.
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Error parsing github releases:" << rapidjson::GetParseError_En(doc.GetParseError());
        return;
    };
    if (!doc.IsArray()) {
        QLOG_ERROR() << "Error parsing github releases: document was not an array";
        return;
    };

    // Prepare the output vectors.
    const auto n = doc.GetArray().Size();
    tag_names.reserve(n);
    prerelease_flags.reserve(n);

    // Check each of the release objects.
    for (const auto& release : doc.GetArray()) {

        // Skip draft releases.
        if (release.HasMember("draft") && release["draft"].IsBool() && release["draft"].GetBool()) {
            continue;
        };

        // Make sure the release object has the fields we need.
        if (!release.HasMember("tag_name") || !release["tag_name"].IsString()) {
            QLOG_ERROR() << "Encountered a github release without 'tag_name'.";
            continue;
        };
        if (!release.HasMember("prerelease") || !release["prerelease"].IsBool()) {
            QLOG_ERROR() << "Encountered a github release without 'prerelease'.";
            continue;
        };

        // Strip leading "v" or "V".
        QString tag_name = release["tag_name"].GetString();
        if (tag_name.startsWith("v") || tag_name.startsWith("V")) {
            tag_name.remove(0, 1);
        };

        // Add this release to the list.
        tag_names.push_back(tag_name);
        prerelease_flags.push_back(release["prerelease"].GetBool());
    };
}

void UpdateChecker::AskUserToUpdate() {

    const QString skip_release = settings_.value("skip_release").toString();
    const QString skip_prerelease = settings_.value("skip_prerelease").toString();

    // Check to see if we have new releases to advertise to the user.
    const bool new_release = (0 != skip_release.compare(latest_release_, Qt::CaseInsensitive));
    const bool new_prerelease = (0 != skip_prerelease.compare(latest_prerelease_, Qt::CaseInsensitive));

    if (!new_release && !new_prerelease) {
        QLOG_INFO() << "Skipping updates: no new versions";
        return;
    };

    // Setup the update message.
    QStringList lines;
    if (new_release) {
        lines.append("The latest version is:");
        lines.append("   " + latest_release_);
    };
    if (new_prerelease) {
        if (!lines.isEmpty()) {
            lines.append("");
        };
        lines.append("The latest pre-release is:");
        lines.append("   " + latest_prerelease_);
    };
    const QString message = lines.join("\n");

    // Create the dialog box.
    QMessageBox msgbox(nullptr);
    msgbox.setWindowTitle("Acquisition [" APP_VERSION_STRING "]: Update Available");
    msgbox.setText(message);
    auto accept_button = msgbox.addButton("  Go to Github  ", QMessageBox::AcceptRole);
    auto ignore_button = msgbox.addButton("  Ignore until newer versions  ", QMessageBox::RejectRole);
    auto remind_button = msgbox.addButton("  Ignore once  ", QMessageBox::RejectRole);
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
        settings_.setValue("skip_release", latest_release_);
        settings_.setValue("skip_prerelease", latest_prerelease_);
    } else {
        settings_.setValue("skip_release", "");
        settings_.setValue("skip_prerelease", "");
    };

    // Open a desktop web browser window if the user clicked that button.
    if (clicked == accept_button) {
        QDesktopServices::openUrl(QUrl(GITHUB_DOWNLOADS_URL));
    };
}
