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
#include <QRegularExpression>
#include <QUrl>
#include <QWidget>

#include "QsLog.h"

#include "network_info.h"
#include "version_defines.h"

namespace {
	const char* CMAKELISTS = "https://raw.githubusercontent.com/gerwaric/acquisition/master/CMakeLists.txt";
	const char* UPDATE_DOWNLOAD_LOCATION = "https://github.com/gerwaric/acquisition/releases";
}

// Check for updates every 24 hours.
const int UpdateChecker::update_interval = 24 * 60 * 60 * 1000;

// Use APP_VERSION to hard-code the running version of acquisition.
const QVersionNumber UpdateChecker::current_version = QVersionNumber::fromString(APP_VERSION);

// Use a regular expression to capture the most recent gitub release version.
const QRegularExpression UpdateChecker::version_regex = QRegularExpression(
	R"REGEX(^ \s* project \s* \( .*? version \s+ (\S+))REGEX",
	QRegularExpression::CaseInsensitiveOption |
	QRegularExpression::MultilineOption |
	QRegularExpression::DotMatchesEverythingOption |
	QRegularExpression::ExtendedPatternSyntaxOption);

const QRegularExpression UpdateChecker::postfix_regex = QRegularExpression(
	R"REGEX(^ \s* set \s* \( .*? version_postfix \s+ "(.*?)")REGEX",
	QRegularExpression::CaseInsensitiveOption |
	QRegularExpression::MultilineOption |
	QRegularExpression::DotMatchesEverythingOption |
	QRegularExpression::ExtendedPatternSyntaxOption);

UpdateChecker::UpdateChecker(QNetworkAccessManager& network_manager, QObject* parent) :
	QObject(parent),
	nm_(network_manager),
	last_version_(current_version)
{
	timer_.setInterval(update_interval);
	timer_.start();
	connect(&timer_, &QTimer::timeout, this, &UpdateChecker::CheckForUpdates);
}

void UpdateChecker::CheckForUpdates() {
	QNetworkRequest request = QNetworkRequest(QUrl(CMAKELISTS));
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
	QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
	if (reply->error() != QNetworkReply::NoError) {
		QLOG_ERROR() << "The network reply came with an error:" << reply->errorString();
		return;
	};
	const QByteArray bytes = reply->readAll();
	QRegularExpressionMatch match = version_regex.match(bytes);
	if (!match.hasMatch() || !match.hasCaptured(1)) {
		QLOG_ERROR() << "Unable to parse project version from CMakeLists.txt";
		return;
	};
	const QVersionNumber github_version = QVersionNumber::fromString(match.captured(1));
	match = postfix_regex.match(bytes);
	if (!match.hasMatch() || !match.hasCaptured(1)) {
		QLOG_DEBUG() << "No version postfix found.";
	};
	const QString github_postfix = match.captured(1);
	if ((github_version > last_version_) || (github_postfix != last_postfix_)) {
		emit UpdateAvailable(github_version, github_postfix);
	};
	last_version_ = github_version;
	last_postfix_ = github_postfix;
}

void UpdateChecker::AskUserToUpdate() {
	QMessageBox::StandardButton result = QMessageBox::information(nullptr, "Update",
	    "Acquisition version " + last_version_.toString() + last_postfix_ + " is available. "
		"Would you like to navigate to GitHub to download it?",
		QMessageBox::Yes | QMessageBox::No,
		QMessageBox::Yes);
	if (result == QMessageBox::Yes) {
		QDesktopServices::openUrl(QUrl(UPDATE_DOWNLOAD_LOCATION));
	};
}
