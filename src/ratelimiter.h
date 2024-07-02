/*
	Copyright 2023 Gerwaric

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
#include <QNetworkRequest>
#include <QString>
#include <QTimer>

#include <list>
#include <map>
#include <memory>

#include "network_info.h"
#include "ratelimit.h"
#include "ratelimitmanager.h"

class QNetworkAccessManager;
class QNetworkReply;

class OAuthManager;
class RateLimitManager;

class RateLimiter : public QObject {
	Q_OBJECT

public:
	// Create a rate limiter.
	RateLimiter(QObject* parent,
		QNetworkAccessManager& network_manager,
		OAuthManager& oauth_manager,
		POE_API mode);

	void Init(POE_API mode);

	// Submit a request-callback pair to the rate limiter. Note that the callback function
	// should not delete the QNetworkReply. That is handled after the callback finishes.
	RateLimit::RateLimitedReply* Submit(
		const QString& endpoint,
		QNetworkRequest network_request);

public slots:
	// Used by the GUI to request a manual refresh.
	void OnUpdateRequested();

signals:
	// Emitted when one of the policy managers has signalled a policy update.
	void PolicyUpdate(const RateLimit::Policy& policy);

	// Signal sent to the UI so the user can see what's going on.
	void Paused(int seconds, const QString& policy_name);

private slots:

	void SendStatusUpdate();

	// Received signals from the policy managers.
	void OnPolicyUpdated(const RateLimit::Policy& policy);

private:
	// Process the first request for an endpoint we haven't encountered before.
	void SetupEndpoint(
		const QString& endpoint,
		QNetworkRequest network_request,
		RateLimit::RateLimitedReply* reply,
		QNetworkReply* network_reply);

	RateLimitManager& GetManager(
		const QString& endpoint,
		const QString& policy_name);

	// Reference to the Application's network access manager.
	QNetworkAccessManager& network_manager_;

	// Reference to the Application's OAuth manager.
	OAuthManager& oauth_manager_;

	POE_API mode_;

	QTimer update_timer_;

	std::list<std::unique_ptr<RateLimitManager>> managers_;
	std::map<const QString, RateLimitManager&> manager_by_policy_;
	std::map<const QString, RateLimitManager&> manager_by_endpoint_;

};
