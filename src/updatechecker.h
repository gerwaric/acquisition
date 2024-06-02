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

class QNetworkAccessManager;
class QWidget;

class UpdateChecker : public QObject {
	Q_OBJECT
public:
	UpdateChecker(QNetworkAccessManager& network_manager, QObject* parent = nullptr);
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
	QNetworkAccessManager& nm_;

	// Ammount of time between update checks (milliseconds)
	static const int update_interval;

	// Trigger periodic update checks.
	QTimer timer_;

	// The last release tag checked by the UpdateChecker
	QString last_checked_tag_;
	
	// The newest github release
	QString newest_release_;

	// The newest github pre-release
	QString newest_prerelease_;
	
	// If the user is running an older pre-release with a version suffix,
	// this will be the corresponding final release version.
	QString upgrade_release_;
};
