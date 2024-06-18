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

#pragma once

#include <QObject>
#include <QString>
#include <QDateTime>

#include <string>

class QNetworkAccessManager;
class QNetworkReply;
class QSettings;

class DataStore;
class ItemsManager;
class BuyoutManager;
class Shop;
class CurrencyManager;
class UpdateChecker;
class OAuthManager;
class RateLimiter;

enum class PoeApiMode { LEGACY, OAUTH };

class Application : public QObject {
	Q_OBJECT
public:
	explicit Application(bool mock_data = false);
	~Application();
	Application(const Application&) = delete;
	Application& operator=(const Application&) = delete;
	// Should be called by login dialog after login
	void InitLogin(PoeApiMode mode);
	QSettings& settings() { return *settings_; };
	ItemsManager& items_manager() { return *items_manager_; }
	DataStore& global_data() const { return *global_data_; }
	DataStore& data() const { return *data_; }
	BuyoutManager& buyout_manager() const { return *buyout_manager_; }
	QNetworkAccessManager& network_manager() const { return *network_manager_; }
	Shop& shop() const { return *shop_; }
	CurrencyManager& currency_manager() const { return *currency_manager_; }
	UpdateChecker& update_checker() const { return *update_checker_; }
	OAuthManager& oauth_manager() const { return *oauth_manager_; }
	RateLimiter& rate_limiter() const { return *rate_limiter_; }
public slots:
	void OnItemsRefreshed(bool initial_refresh);
private:
	void LoadTheme();
	bool test_mode_;
	std::unique_ptr<QSettings> settings_;
	std::unique_ptr<DataStore> global_data_;
	std::unique_ptr<DataStore> data_;
	std::unique_ptr<BuyoutManager> buyout_manager_;
	std::unique_ptr<Shop> shop_;
	std::unique_ptr<QNetworkAccessManager> network_manager_;
	std::unique_ptr<ItemsManager> items_manager_;
	std::unique_ptr<CurrencyManager> currency_manager_;
	std::unique_ptr<UpdateChecker> update_checker_;
	std::unique_ptr<OAuthManager> oauth_manager_;
	std::unique_ptr<RateLimiter> rate_limiter_;
	void SaveDbOnNewVersion();
};
