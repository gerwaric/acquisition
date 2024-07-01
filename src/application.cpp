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

#include <QApplication>
#include <QDir>
#include <QMessageBox>
#include <QNetworkAccessManager>
#include <QSettings>
#include <QtHttpServer/QHttpServer>

#include "buyoutmanager.h"
#include "sqlitedatastore.h"
#include "memorydatastore.h"
#include "filesystem.h"
#include "itemsmanager.h"
#include "currencymanager.h"
#include "shop.h"
#include "QsLog.h"
#include "ratelimiter.h"
#include "updatechecker.h"
#include "oauthmanager.h"
#include "version_defines.h"

Application::Application(bool test_mode) :
	test_mode_(test_mode)
{
	if (test_mode_) {
		global_data_ = std::make_unique<MemoryDataStore>();
		return;
	};

	const QString user_dir = Filesystem::UserDir();
	const QString settings_path = user_dir + "/settings.ini";
	const QString global_data_file = user_dir + "/data/" + SqliteDataStore::MakeFilename("", "");

	settings_ = std::make_unique<QSettings>(settings_path, QSettings::IniFormat);
	global_data_ = std::make_unique<SqliteDataStore>(global_data_file);
	network_manager_ = std::make_unique<QNetworkAccessManager>(this);
	update_checker_ = std::make_unique<UpdateChecker>(this, settings(), network_manager());
	oauth_manager_ = std::make_unique<OAuthManager>(this, network_manager(), global_data());
	rate_limiter_ = std::make_unique<RateLimiter>(this, network_manager(), oauth_manager());

	LoadTheme();
}

Application::~Application() {
	if (buyout_manager_) {
		buyout_manager_->Save();
	};
}

QSettings& Application::settings() const {
	if (!settings_) {
		FatalAccessError("settings is null");
	};
	return *settings_;
};

ItemsManager& Application::items_manager() const {
	if (!items_manager_) {
		FatalAccessError("items nanager");
	};
	return *items_manager_;
};

DataStore& Application::global_data() const {
	if (!global_data_) {
		FatalAccessError("global datastore");
	};
	return *global_data_;
};

DataStore& Application::data() const {
	if (!data_) {
		FatalAccessError("datastore");
	};
	return *data_;
};

BuyoutManager& Application::buyout_manager() const {
	if (!buyout_manager_) {
		FatalAccessError("buyout manager");
	};
	return *buyout_manager_;
};

QNetworkAccessManager& Application::network_manager() const {
	if (!network_manager_) {
		FatalAccessError("network access manager");
	};
	return *network_manager_;
}

Shop& Application::shop() const {
	if (!shop_) {
		FatalAccessError("shop");
	};
	return *shop_;
}

CurrencyManager& Application::currency_manager() const {
	if (!currency_manager_) {
		FatalAccessError("currency manager");
	};
	return *currency_manager_;
}

UpdateChecker& Application::update_checker() const {
	if (!update_checker_) {
		FatalAccessError("update checker");
	};
	return *update_checker_;
}

OAuthManager& Application::oauth_manager() const {
	if (!oauth_manager_) {
		FatalAccessError("OAuth manager");
	};
	return *oauth_manager_;
}

RateLimiter& Application::rate_limiter() const {
	if (!rate_limiter_) {
		FatalAccessError("rate limiter");
	};
	return *rate_limiter_;
}

void Application::FatalAccessError(const char* object_name) {
	const QString message = QString("Attempted to use the %1 before it was created.").arg(object_name);
	QLOG_FATAL() << message;
	QMessageBox errorMsg;
	errorMsg.setIcon(QMessageBox::Icon::Critical);
	errorMsg.setWindowTitle("Acquistion: Fatal Error");
	errorMsg.setText(message);
	errorMsg.setStandardButtons(QMessageBox::StandardButton::Abort);
	QApplication::quit();
}

void Application::LoadTheme() {
	// Load the appropriate theme.
	const QString theme = settings_->value("theme").toString();

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
		const QString theme_data = ts.readAll();
		qApp->setStyleSheet(theme_data);
		QPalette pal = QApplication::palette();
		pal.setColor(QPalette::WindowText, Qt::white);
		QApplication::setPalette(pal);
	};
}

void Application::InitLogin(POE_API mode)
{
	if (test_mode_) {
		// This is used in tests
		data_ = std::make_unique<MemoryDataStore>();
	} else {
		const std::string league = settings_->value("league").toString().toStdString();
		const std::string account = settings_->value("account").toString().toStdString();
		const QString data_dir = Filesystem::UserDir() + "/data/";
		const QString data_file = SqliteDataStore::MakeFilename(account, league);
		const QString data_path = data_dir + data_file;
		data_ = std::make_unique<SqliteDataStore>(data_path);
		SaveDbOnNewVersion();
	}

	buyout_manager_ = std::make_unique<BuyoutManager>(*data_);

	items_manager_ = std::make_unique<ItemsManager>(this,
		settings(),
		network_manager(),
		buyout_manager(),
		data(),
		rate_limiter());

	shop_ = std::make_unique<Shop>(this,
		settings(),
		network_manager(),
		data(),
		items_manager(),
		buyout_manager());

	currency_manager_ = std::make_unique<CurrencyManager>(nullptr,
		settings(),
		data(),
		items_manager());

	connect(items_manager_.get(), &ItemsManager::ItemsRefreshed, this, &Application::OnItemsRefreshed);

	if (test_mode_ == false) {
		items_manager_->Start(mode);
	};
}

void Application::OnItemsRefreshed(bool initial_refresh) {
	currency_manager_->Update();
	shop_->Update();
	if (!initial_refresh && shop_->auto_update())
		shop_->SubmitShopToForum();
}

void Application::SaveDbOnNewVersion() {
	//If user updated from a 0.5c db to a 0.5d, db exists but no "version" in it
	std::string version = data_->Get("version", "0.5c");
	// We call this just after login, so we didn't pulled tabs for the first time ; so "tabs" shouldn't exist in the DB
	// This way we don't create an useless data_save_version folder on the first time you run acquisition

	bool first_start = data_->Get("tabs", "first_time") == "first_time" &&
		data_->GetTabs(ItemLocationType::STASH).size() == 0 &&
		data_->GetTabs(ItemLocationType::CHARACTER).size() == 0;
	if (version != APP_VERSION_STRING && !first_start) {
		QString data_path = Filesystem::UserDir() + QString("/data");
		QString save_path = data_path + "_save_" + version.c_str();
		QDir src(data_path);
		QDir dst(save_path);
		if (!dst.exists())
			QDir().mkpath(dst.path());
		for (const auto& name : src.entryList()) {
			QFile::copy(data_path + QDir::separator() + name, save_path + QDir::separator() + name);
		}
		QLOG_INFO() << "I've created the folder " << save_path << "in your acquisition folder, containing a save of all your data";
	}
	data_->Set("version", APP_VERSION_STRING);
}
