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
#include "poe_api/poe_stash.h"

#include <QObject>

#include "QsLog.h"
#include "ratelimit.h"

namespace PoE {

	LegacyStashTab::LegacyStashTab(const StashTab& tab) :
		n(tab.name),
		i(tab.index.value_or(0)),
		id(tab.id),
		type(tab.type),
		selected(false)
	{
		colour.r = std::stoul(tab.metadata.colour->substr(0, 2));
		colour.g = std::stoul(tab.metadata.colour->substr(2, 2));
		colour.b = std::stoul(tab.metadata.colour->substr(4, 2));
	}

	void ListStashes(QObject* object, ListStashesCallback callback, const std::string& league)
	{
		static auto& rate_limiter = RateLimit::RateLimiter::instance();

		const QString LIST_STASHES("https://api.pathofexile.com/stash/" + QString::fromStdString(league));

		auto callback_wrapper = [=](QNetworkReply* reply) {
			// Check for network errors.
			if (reply->error() != QNetworkReply::NoError) {
				const auto status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute);
				const auto message = reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute);
				QLOG_ERROR() << "ListStashes: network error (" << status << "):" << message;
				return;
			};
			const QByteArray data = reply->readAll();
			ListStashesResult result;
			JS::ParseContext context(data);
			JS::Error error = context.parseTo(result);
			if (error != JS::Error::NoError) {
				QLOG_ERROR() << "ListStashes: json error:" << context.makeErrorString();
				return;
			};
			// Run the callback in the context object's thread.
			QMetaObject::invokeMethod(object, [=]() { callback(result); });
		};

		rate_limiter.Submit("GET /stash/<league>", QNetworkRequest(QUrl(LIST_STASHES)), callback_wrapper);
	}

	void GetStash(QObject* object, GetStashCallback callback,
		const std::string& league,
		const std::string& stash_id,
		const std::string& substash_id)
	{
		static auto& rate_limiter = RateLimit::RateLimiter::instance();

		const QString LIST_STASHES = substash_id.empty()
			? "https://api.pathofexile.com/stash/" + QString::fromStdString(league + "/" + stash_id)
			: "https://api.pathofexile.com/stash/" + QString::fromStdString(league + "/" + stash_id + "/" + substash_id);

		auto callback_wrapper = [=](QNetworkReply* reply) {
			// Check for network errors.
			if (reply->error() != QNetworkReply::NoError) {
				const auto status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute);
				const auto message = reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute);
				QLOG_ERROR() << "GetStash: network error (" << status << "):" << message;
				return;
			};
			const QByteArray data = reply->readAll();
			GetStashResult result;
			JS::ParseContext context(data);
			JS::Error error = context.parseTo(result);
			if (error != JS::Error::NoError) {
				QLOG_ERROR() << "GetStash: json error:" << context.makeErrorString();
				return;
			};
			// Run the callback in the context object's thread.
			QMetaObject::invokeMethod(object, [=]() { callback(result); });
		};

		rate_limiter.Submit("GET /stash/<league>/<stash_id>", QNetworkRequest(QUrl(LIST_STASHES)), callback_wrapper);
	}

}
