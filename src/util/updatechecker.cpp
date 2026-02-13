// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2015 Ilya Zhuravlev

#include "util/updatechecker.h"

#include <QDesktopServices>
#include <QMessageBox>
#include <QNetworkRequest>
#include <QPushButton>
#include <QRegularExpression>
#include <QSettings>
#include <QSizePolicy>
#include <QUrl>
#include <QWidget>

#include <semver/semver.hpp>

#include "app/usersettings.h"
#include "util/glaze_qt.h" // IWYU pragma: keep
#include "util/networkmanager.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "version_defines.h"

constexpr const char* GITHUB_RELEASES_URL = "https://api.github.com/repos/gerwaric/acquisition/releases";
constexpr const char* GITHUB_DOWNLOADS_URL = "https://github.com/gerwaric/acquisition/releases";

// This are just the fields used by acquisition. See https://docs.github.com/en/rest/releases/releases
struct GitHubReleaseTag
{
    QString tag_name;
    bool draft;
    bool prerelease;
};

// Check for updates every 24 hours.
constexpr int UPDATE_INTERVAL = 24 * 60 * 60 * 1000;

UpdateChecker::UpdateChecker(NetworkManager &network_manager, app::UserSettings &settings)
    : m_network_manager(network_manager)
    , m_settings(settings)
    , m_running_version(semver::version::parse(APP_VERSION_STRING))
{
    spdlog::debug("UpdateChecker: running version is {}", m_running_version.str());
    m_timer.setInterval(UPDATE_INTERVAL);
    connect(&m_timer, &QTimer::timeout, this, &UpdateChecker::CheckForUpdates);
}

void UpdateChecker::setLastSkippedUpdates(const semver::version &release,
                                          const semver::version &prerelease)
{
    m_previous_release = release;
    m_previous_prerelease = prerelease;
}

void UpdateChecker::CheckForUpdates() {
    // Get the releases from GitHub as a json object.
    spdlog::trace("UpdateChecker: requesting GitHub releases: {}", GITHUB_RELEASES_URL);

    m_previous_release = m_settings.lastSkippedRelease();
    m_previous_prerelease = m_settings.lastSkippedPreRelease();

    QNetworkRequest request{QUrl(GITHUB_RELEASES_URL)};
    QNetworkReply *reply = m_network_manager.get(request);
    connect(reply, &QNetworkReply::finished, this, &UpdateChecker::OnUpdateReplyReceived);

    // Make sure errors are logged.
    m_network_manager.logReplyErrors(reply, "UpdateChecker");
}

void UpdateChecker::OnUpdateReplyReceived() {
    // Process the releases received from GitHub.
    spdlog::trace("UpdateChecker: received an update reply from GitHub.");

    // Check for network errors.
    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    if (reply->error() != QNetworkReply::NoError) {
        spdlog::error("The network reply came with an error: {}", reply->errorString());
        return;
    }

    // Parse release tags from the json object.
    const QByteArray bytes = reply->readAll();
    const std::vector<ReleaseTag> releases = ParseReleaseTags(bytes);

    m_latest_release = semver::version();
    m_latest_prerelease = semver::version();

    for (const auto& release : releases) {
        if (release.prerelease) {
            if ((release.version > m_latest_prerelease)) {
                m_latest_prerelease = release.version;
            }
        } else {
            if ((release.version > m_latest_release)) {
                m_latest_release = release.version;
            }
        }
    }

    // Make sure at least one tag was found.
    if ((m_latest_release == semver::version()) && (m_latest_prerelease == semver::version())) {
        spdlog::warn("Unable to find any github releases or pre-releases!");
        return;
    }
    if (m_latest_release > semver::version()) {
        spdlog::debug("UpdateChecker: latest release found: {}", m_latest_release.str());
    }
    if (m_latest_prerelease > semver::version()) {
        spdlog::debug("UpdateChecker: latest prerelease found: {}", m_latest_prerelease.str());
    }

    // Send a signal if there's a new version from the last check.
    if (has_newer_release() || has_newer_prerelease()) {
        emit UpdateAvailable();
    }

    // Start the timer for the next udpate check.
    m_timer.start();
}

std::vector<UpdateChecker::ReleaseTag> UpdateChecker::ParseReleaseTags(const QByteArray& bytes)
{
    std::vector<GitHubReleaseTag> github_releases;

    constexpr const glz::opts permissive{.error_on_unknown_keys = false};
    const std::string_view sv{bytes, size_t(bytes.size())};
    const auto ec = glz::read<permissive>(github_releases, sv);
    if (ec) {
        const auto msg = glz::format_error(ec, sv);
        spdlog::error("Error parsing GitHub release tags: {}", msg);
        return {};
    }

    // Reserve the output vector.
    std::vector<ReleaseTag> releases;
    releases.reserve(github_releases.size());

    // Check each of the release objects.
    for (const auto &github_release : github_releases) {
        ReleaseTag release;

        // Parse the release version
        QString version_string = github_release.tag_name;
        if (version_string.startsWith("v", Qt::CaseInsensitive)) {
            version_string.remove(0, 1);
        }
        release.version = semver::version::parse(version_string.toStdString());

        // Make sure we found a parseable version number
        if (release.version == semver::version()) {
            spdlog::warn("Error parsing GitHub release version from '{}'", version_string);
        }

        // Parse the release flags
        release.draft = github_release.draft;
        release.prerelease = github_release.prerelease;

        // Add this release to the list.
        releases.push_back(release);
    }
    return releases;
}

bool UpdateChecker::has_newer_release() const {
    return (m_latest_release > m_previous_release) && (m_running_version < m_latest_release);
}

bool UpdateChecker::has_newer_prerelease() const {
    return (m_latest_prerelease > m_previous_prerelease) && (m_running_version < m_latest_prerelease);
}

void UpdateChecker::AskUserToUpdate() {

    if (!has_newer_release() && !has_newer_prerelease()) {
        spdlog::warn("UpdateChecker: no newer versions available");
        //return;
    }

    // Setup the update message.
    QStringList lines;
    if (has_newer_release()) {
        lines.append("A newer release is available:");
        lines.append("   " + QString::fromStdString(m_latest_release.str()));
    }
    if (has_newer_prerelease()) {
        if (m_latest_prerelease > m_latest_release) {
            if (!lines.isEmpty()) {
                lines.append("");
            }
            lines.append("A newer prerelease is available:");
            lines.append("   " + QString::fromStdString(m_latest_prerelease.str()));
        }
    }
    if (lines.isEmpty()) {
        QMessageBox::information(nullptr, "Acquisition Update Checker", "No updates appear to be available", QMessageBox::StandardButton::Ok);
        return;
    }
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

    // Save the latest releases into the settings file only if the
    // user asked to skip them in the future.
    if (clicked == ignore_button) {
        m_settings.lastSkippedRelease(m_latest_release);
        m_settings.lastSkippedPreRelease(m_latest_prerelease);
    }

    // Open a desktop web browser window if the user clicked that button.
    if (clicked == accept_button) {
        QDesktopServices::openUrl(QUrl(GITHUB_DOWNLOADS_URL));
    }
}
