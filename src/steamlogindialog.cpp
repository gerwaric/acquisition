/*
	Copyright 2014 Ilya Zhuravlev

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

#include "steamlogindialog.h"
#include "ui_steamlogindialog.h"

#include <QCloseEvent>
#include <memory>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkCookie>
#include <QNetworkCookieJar>
#include <QNetworkRequest>
#include <QUrlQuery>
#include <QFile>

#include "network_info.h"

extern const char* POE_COOKIE_NAME;

SteamLoginDialog::SteamLoginDialog(QWidget* parent) :
	QDialog(parent),
	ui(new Ui::SteamLoginDialog)
{
	ui->setupUi(this);
}

SteamLoginDialog::~SteamLoginDialog() {
	delete ui;
}

void SteamLoginDialog::closeEvent(QCloseEvent* e) {
	if (!completed_)
		emit Closed();
	QDialog::closeEvent(e);
}

void SteamLoginDialog::Init() {
	completed_ = false;
}
