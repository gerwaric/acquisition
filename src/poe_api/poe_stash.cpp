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
#include "application.h"
#include "ratelimit.h"

namespace PoE {

	StashTab::StashTab(const std::string& json) {
		JS::ParseContext context(json);
		JS::Error error = context.parseTo(*this);
		if (error != JS::Error::NoError) {
			QLOG_ERROR() << "Error parsing stash tab:" << context.makeErrorString();
		};
	};

	struct StashTabListWrapper {
		StashTabListWrapper(const std::string& data) {
			JS::ParseContext context(data);
			JS::Error error = context.parseTo(*this);
			if (error != JS::Error::NoError) {
				QLOG_ERROR() << "Error parsing wrapped stashes:" << context.makeErrorString();
				return;
			};
		};
		std::vector<PoE::StashTab> stashes;
		JS_OBJ(stashes);
	};

	void ListStashes(QObject* object, ListStashesCallback callback, const PoE::LeagueName& league)
	{
		static auto& rate_limiter = Application::instance().rate_limiter();

		if (!league) {
			QLOG_ERROR() << "PoE API: cannot list stashes: league is empty";
			return;
		};

		const QString LIST_STASHES("https://api.pathofexile.com/stash/" + QString(league));

		auto callback_wrapper = [=](QNetworkReply* reply) {
			// Check for network errors.
			if (reply->error() != QNetworkReply::NoError) {
				const auto status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute);
				const auto message = reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute);
				QLOG_ERROR() << "ListStashes: network error (" << status << "):" << message;
				return;
			};
			// Run the callback in the context object's thread.
			QMetaObject::invokeMethod(object,
				[=]() {
					const StashTabListWrapper result(reply->readAll().toStdString());
					callback(result.stashes);
				});
		};
		QMetaObject::invokeMethod(&rate_limiter,
			[=]() { 
				rate_limiter.Submit("GET /stash/<league>",
					QNetworkRequest(QUrl(LIST_STASHES)),
					callback_wrapper);
			});
	}

	struct StashTabWrapper {
		StashTabWrapper(const std::string& data) {
			JS::ParseContext context(data);
			JS::Error error = context.parseTo(*this);
			if (error != JS::Error::NoError) {
				QLOG_ERROR() << "Error parsing wrapped stash:" << context.makeErrorString();
				return;
			};
		};
		PoE::StashTab stash;
		JS_OBJ(stash);
	};

	void GetStash(QObject* object, GetStashCallback callback,
		const PoE::LeagueName& league,
		const PoE::StashId& stash_id,
		const PoE::StashId& substash_id)
	{
		static auto& rate_limiter = Application::instance().rate_limiter();

		if (!league) {
			QLOG_ERROR() << "PoE API: cannot get stash: league is empty";
			return;
		};
		if (!stash_id) {
			QLOG_ERROR() << "PoE API: cannot get stash: stash_id is empty";
			return;
		};

		const QString path = (substash_id)
			? QString(league) + "/" + QString(stash_id) + "/" + QString(substash_id)
			: QString(league) + "/" + QString(stash_id);

		const QString GET_STASH = "https://api.pathofexile.com/stash/" + path;

		auto callback_wrapper = [=](QNetworkReply* reply) {
			// Check for network errors.
			if (reply->error() != QNetworkReply::NoError) {
				const auto status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute);
				const auto message = reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute);
				QLOG_ERROR() << "GetStash: network error (" << status << "):" << message;
				return;
			};
			// Parse the reply.
			// Run the callback in the context object's thread.
			QMetaObject::invokeMethod(object,
				[=]() {
					const StashTabWrapper result(reply->readAll().toStdString());
					callback(result.stash);
				});
		};
		QMetaObject::invokeMethod(&rate_limiter,
			[=]() {
				rate_limiter.Submit("GET /stash/<league>/<stash_id>",
					QNetworkRequest(QUrl(GET_STASH)),
					callback_wrapper);			
			});
	}

}
