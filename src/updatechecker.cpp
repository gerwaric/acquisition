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

namespace {
	const char* GITHUB_RELEASES_URL = "https://api.github.com/repos/gerwaric/acquisition/releases";
	const char* GITHUB_DOWNLOADS_URL = "https://github.com/gerwaric/acquisition/releases";
}

// Check for updates every 24 hours.
const int UpdateChecker::update_interval = 24 * 60 * 60 * 1000;

UpdateChecker::UpdateChecker(QObject* parent, QSettings& settings, QNetworkAccessManager& network_manager) :
	QObject(parent),
	settings_(settings),
	nm_(network_manager)
{
	timer_.setInterval(update_interval);
	timer_.start();
	connect(&timer_, &QTimer::timeout, this, &UpdateChecker::CheckForUpdates);
}

void UpdateChecker::CheckForUpdates() {
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
	QLOG_TRACE() << "UpdateChecker: received an update reply.";

	// Check for network errors.
	QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
	if (reply->error() != QNetworkReply::NoError) {
		QLOG_ERROR() << "The network reply came with an error:" << reply->errorString();
		return;
	};

	// Parse the reply as a json document.
	rapidjson::Document doc;
	const QByteArray bytes = reply->readAll();
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

	QString newest_tag;
	QString matched_tag;

	const auto app_version = QStringLiteral(APP_VERSION);
	const auto app_version_string = QStringLiteral(APP_VERSION_STRING);
	const bool app_is_prerelease = app_version != app_version_string;

	newest_release_ = "";
	newest_prerelease_ = "";
	upgrade_release_ = "";

	// Start iterating through the release objects.
	const auto& releases = doc.GetArray();
	for (const auto& release : releases) {

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

		const QString tag_name = release["tag_name"].GetString();
		const bool prerelease = release["prerelease"].GetBool();

		// Save the first tag we encounter, since this is the one that will trigger
		// the update signal regardless of wether it's a prerelease or not.
		if (newest_tag.isEmpty()) {
			newest_tag = tag_name;
		};

		// If the running version appears to be a pre-release, look for a matching release.
		if (app_is_prerelease) {
			if (!prerelease && tag_name.contains(app_version)) {
				if (upgrade_release_.isEmpty()) {
					QLOG_TRACE() << "UpdateChecker found an upgrade release:" << tag_name;
					upgrade_release_ = tag_name;
				};
			};
		};

		// Keep track of how many releases and prereleases we find before a match,
		// and keep track of the most recent ones.
		if (prerelease) {
			if (newest_prerelease_.isEmpty()) {
				QLOG_TRACE() << "UpdateChecker found a newer prerelease tag:" << tag_name;
				newest_prerelease_ = tag_name;
			};
		} else {
			if (newest_release_.isEmpty()) {
				QLOG_TRACE() << "UpdateChecker found a newer release tag:" << tag_name;
				newest_release_ = tag_name;
			};
		};

		// Try matching the release tag againg the running version.
		if (tag_name.contains(app_version_string, Qt::CaseInsensitive)) {
			QLOG_TRACE() << "UpdateChecker found a match for the running version" << tag_name;
			matched_tag = tag_name;
		};

		// End early if we've found all the things we are looking for.
		if (!matched_tag.isEmpty() && !newest_release_.isEmpty() && !newest_prerelease_.isEmpty()) {
			break;
		};
	};

	if (matched_tag.isEmpty()) {
		QLOG_WARN() << "Unable to match any github release against the running version!";
	};

	qsizetype k;
	const QVersionNumber newest_version = QVersionNumber::fromString(newest_tag, &k);
	const QString newest_postfix = newest_tag.sliced(k);

	const QVersionNumber last_checked_version = QVersionNumber::fromString(last_checked_tag_, &k);
	const QString last_checked_postfix = last_checked_tag_.sliced(k);

	// Update the last checked tag before announcing updates.
	last_checked_tag_ = newest_tag;

	// Send a signal if there's a new version from the last check.
	if ((newest_version > last_checked_version) || (newest_postfix != last_checked_postfix)) {
		emit UpdateAvailable();
	};
}

void UpdateChecker::AskUserToUpdate() {

	const auto skip_release = settings_.value("skip_release_version", "");
	const auto skip_prerelease = settings_.value("skip_prerelease_version", "");

	if ((newest_release_ == skip_release) && (newest_prerelease_ == skip_prerelease)) {
		QLOG_INFO() << "Skipping updates: no new versions";
		return;
	};

	// Only show the 'compatible version' update if it's present and is
	// different from the newest release. This cann happen if the user is
	// running an older pre-release, e.g. v0.10.3-alpha.3. In this case,
	// the user could update to either v0.10.3, or a newer release like
	// v0.10.4 or newer.
	const bool show_compatible =
		(upgrade_release_.isEmpty() == false) &&
		(upgrade_release_ != newest_release_);

	// Setup the update message.
	QStringList lines;
	lines.append("The latest official release is '" + newest_release_ + "'");
	lines.append("The latest pre-release is '" + newest_prerelease_ + "'");
	if (show_compatible) {
		lines.append("\nYou could also update your version to '" + upgrade_release_ + "'");
	};
	const QString message = lines.join("\n");

	// Create the dialog box.
	QMessageBox msgbox(nullptr);
	msgbox.setWindowTitle("Update [" APP_VERSION_STRING "]");
	msgbox.setText(message);
	auto accept_button = msgbox.addButton("  Go to Github  ", QMessageBox::AcceptRole);
	auto ignore_button = msgbox.addButton("  Ignore  ", QMessageBox::RejectRole);
	auto skip_button = msgbox.addButton("  Ignore (until new versions are available)", QMessageBox::RejectRole);
	msgbox.setDefaultButton(accept_button);

	// Resize the buttons so the text fits.
	accept_button->setMinimumWidth(accept_button->sizeHint().width());
	ignore_button->setMinimumWidth(ignore_button->sizeHint().width());
	skip_button->setMinimumWidth(skip_button->sizeHint().width());

	// Get the user's choice.
	msgbox.exec();

	const auto clicked = msgbox.clickedButton();
	if (clicked == accept_button) {
		QDesktopServices::openUrl(QUrl(GITHUB_DOWNLOADS_URL));
	} else if (clicked == skip_button) {
		settings_.setValue("skip_release_version", newest_release_);
		settings_.setValue("skip_prerelease_version", newest_prerelease_);
	};
}
