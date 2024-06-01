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
	void UpdateAvailable(const QVersionNumber& version);
public slots:
	void CheckForUpdates();
	void AskUserToUpdate();
private slots:
	void OnUpdateReplyReceived();
	void OnUpdateErrorOccurred(QNetworkReply::NetworkError code);
	void OnUpdateSslErrors(const QList<QSslError>& errors);
private:
	QNetworkAccessManager& nm_;
	QTimer timer_;
	QVersionNumber last_checked_;
	static const int update_interval;
	static const QVersionNumber current_version;
	static const QRegularExpression version_regex;
};
