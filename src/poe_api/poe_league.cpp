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
#include "poe_api/poe_league.h"

#include <QObject>

#include "QsLog.h"
#include "ratelimit.h"

namespace PoE {

	void GetLeagues(QObject* object, GetLeaguesCallback callback)
	{
		static auto& rate_limiter = RateLimit::RateLimiter::instance();
		
		const QString GET_LEAGUES("https://api.pathofexile.com/account/leagues");

		auto callback_wrapper = [=](QNetworkReply* reply) {
			// Check for network errors.
			if (reply->error() != QNetworkReply::NoError) {
				const auto status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute);
				const auto message = reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute);
				QLOG_ERROR() << "GetLeagues: network error (" << status << "):" << message;
				return;
			};
			const QByteArray data = reply->readAll();
			GetLeaguesResult result;
			JS::ParseContext context(data);
			JS::Error error = context.parseTo(result);
			if (error != JS::Error::NoError) {
				QLOG_ERROR() << "GetLeagues: json error:" << context.makeErrorString();
				return;
			};
			// Run the callback in the context object's thread.
			QMetaObject::invokeMethod(object, [=]() { callback(result); });
		};

		rate_limiter.Submit("GET /account/leagues", QNetworkRequest(QUrl(GET_LEAGUES)), callback_wrapper);
	}

}
