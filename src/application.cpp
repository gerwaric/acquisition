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

#include "application.h"

#include <QNetworkAccessManager>
#include <QtHttpServer/QHttpServer>

#include "poe_api/poe_character.h"
#include "poe_api/poe_stash.h"

#include "buyoutmanager.h"
#include "datastore.h"
#include "filesystem.h"
#include "itemsmanager.h"
#include "currencymanager.h"
#include "porting.h"
#include "shop.h"
#include "QsLog.h"
#include "ratelimit.h"
#include "updatechecker.h"
#include "version_defines.h"

Application& Application::instance() {
	static Application app;
	return app;
}

Application& Application::test_instance() {
	static Application app(true);
	return app;
}

Application::Application(bool test_mode) :
	test_mode_(test_mode)
{
	InitGlobalData();
	LoadTheme();

	network_manager_ = std::make_unique<QNetworkAccessManager>(this);
	update_checker_ = std::make_unique<UpdateChecker>(*network_manager_, this);
	oauth_manager_ = std::make_unique<OAuthManager>(*network_manager_, this);
	rate_limiter_ = std::make_unique<RateLimit::RateLimiter>();

	// Make sure the rate limiter gets tokens when they are updated.
	connect(oauth_manager_.get(), &OAuthManager::accessGranted,
		rate_limiter_.get(), &RateLimit::RateLimiter::SetAccessToken);

	// Look for a stored OAuth access token.
	const std::string token_str = global_data_->Get("oauth_token", "");
	if (token_str != "") {
		OAuthToken token;
		JS::ParseContext context(token_str);
		JS::Error error = context.parseTo(token);
		if (error != JS::Error::NoError) {
			QLOG_ERROR() << "Error parsing stored OAuthToken:" << context.makeErrorString();
		} else {
			oauth_manager_->setToken(token);
		};
	};
}

Application::~Application() {
	if (buyout_manager_) {
		buyout_manager_->Save();
	};
}

void Application::InitGlobalData() {

	// The global datastore holds things like the selected theme and the oath token.
	if (test_mode_) {
		global_data_ = std::make_unique<DataStore>(":memory:");
		return;
	};

	const QDir data_path(GetDataPath());
	const QString file_name = "global";
	const QString file_path = data_path.absoluteFilePath(file_name);
	global_data_ = std::make_unique<DataStore>(file_path);
	if (data_path.exists()) {
		if (global_data_->Get("version") != APP_VERSION_STRING) {
			SaveDbOnNewVersion();
			global_data_->Set("version", APP_VERSION_STRING);
		};
	};
}

void Application::LoadTheme() {
	// Load the appropriate theme.
	const std::string theme = global_data_->Get("theme", "default");

	// Do nothing for the default theme.
	if (theme == "default") {
		return;
	};

	// Determine which qss file to use.
	QString stylesheet;
	if (theme == "dark") {
		stylesheet = ":qdarkstyle/dark/darkstyle.qss";
	} else if (theme == "light") {
		stylesheet = ":qdarkstyle/light/lightstyle.qss";
	} else {
		QLOG_ERROR() << "Invalid theme:" << theme;
		return;
	};

	// Load the theme.
	QFile f(stylesheet);
	if (!f.exists()) {
		QLOG_ERROR() << "Theme stylesheet not found:" << stylesheet;
	} else {
		f.open(QFile::ReadOnly | QFile::Text);
		QTextStream ts(&f);
		const QString stylesheet = ts.readAll();
		qApp->setStyleSheet(stylesheet);
		QPalette pal = QApplication::palette();
		pal.setColor(QPalette::WindowText, Qt::white);
		QApplication::setPalette(pal);
	};
}

void Application::InitLogin(const std::string& league, const std::string& account)
{
	account_ = PoE::AccountName(account);
	league_ = PoE::LeagueName(league);

	if (test_mode_ == false) {
		const QString file_name = DataStore::MakeFilename(account_.value(), league_.value());
		const QString file_path = QDir(GetDataPath()).absoluteFilePath(file_name);
		data_ = std::make_unique<DataStore>(file_path);
		if (data_->Get("version") != APP_VERSION_STRING) {
			// Backups were made when the global data store was created.
			data_->Set("version", APP_VERSION_STRING);
		};
	} else {
		data_ = std::make_unique<DataStore>(":memory:");
	};

	buyout_manager_ = std::make_unique<BuyoutManager>(*data_);
	shop_ = std::make_unique<Shop>(*this);
	items_manager_ = std::make_unique<ItemsManager>(*this);
	currency_manager_ = std::make_unique<CurrencyManager>(*this);

	connect(items_manager_.get(), &ItemsManager::ItemsRefreshed, this, &Application::OnItemsRefreshed);

	if (test_mode_ == false) {
		items_manager_->Start();
	};
}

QString Application::GetDataPath() const {
	return Filesystem::UserDir() + QDir::separator() + "data";
}

void Application::OnItemsRefreshed(bool initial_refresh) {
	currency_manager_->Update();
	shop_->Update();
	if (!initial_refresh && shop_->auto_update()) {
		shop_->SubmitShopToForum();
	};
}

void Application::SaveDbOnNewVersion() {

	// If user updated from a 0.5c db to a 0.5d, db exists but no "version" in it
	const QString version = QString::fromStdString(global_data_->Get("version", "0.5c"));
	QLOG_INFO() << "Preparing to backup your data because version" << APP_VERSION_STRING << "was detected.";

	const QString data_path = GetDataPath();
	QString save_path;
	int i = 0;
	do {
		save_path = data_path + "_" + version + "_save_" + QString::number(++i);
		if (i > 10) {
			QLOG_ERROR() << "There are too many existing backups. Something might be wrong. Not making another.";
			return;
		};
	} while (QDir(save_path).exists());

	if (!QDir().mkpath(save_path)) {
		QLOG_ERROR() << "Error creating backup folder:" << save_path;
		return;
	};

	QLOG_INFO() << "Backing your data folder to" << save_path;
	const QDir src(data_path);
	const QDir dst(save_path);
	for (auto name : src.entryList(QDir::Files)) {
		const QString src_file = src.absoluteFilePath(name);
		const QString dst_file = dst.absoluteFilePath(name);
		QFile::copy(src_file, dst_file);
	};
}
