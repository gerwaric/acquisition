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
#include "poe_api/poe_character.h"

#include <QObject>

#include "QsLog.h"
#include "application.h"
#include "ratelimit.h"

namespace PoE {

	LegacyCharacter::LegacyCharacter(const Character& character) :
		name(character.name),
		realm(character.realm),
		class_name(character.class_name),
		league(character.league.value_or("")),
		level(character.level),
		pinnable(true),
		i(0) {}

	void ListCharacters(QObject* object, ListCharactersCallback callback)
	{
		static auto& rate_limiter = Application::instance().rate_limiter();

		const QString LIST_CHARACTERS("https://api.pathofexile.com/character");

		auto callback_wrapper = [=](QNetworkReply* reply) {
			// Check for network errors.
			if (reply->error() != QNetworkReply::NoError) {
				const auto status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute);
				const auto message = reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute);
				QLOG_ERROR() << "ListCharacters: network error (" << status << "):" << message;
				return;
			};
			const QByteArray data = reply->readAll();
			ListCharactersResult result;
			JS::ParseContext context(data);
			JS::Error error = context.parseTo(result);
			if (error != JS::Error::NoError) {
				QLOG_ERROR() << "ListCharacters: json error:" << context.makeErrorString();
				return;
			};
			// Run the callback in the context object's thread.
			QMetaObject::invokeMethod(object, [=]() { callback(result); });
		};

		rate_limiter.Submit("GET /character", QNetworkRequest(QUrl(LIST_CHARACTERS)), callback_wrapper);
	}

	void GetCharacter(QObject* object, GetCharacterCallback callback, const std::string& name)
	{
		static auto& rate_limiter = Application::instance().rate_limiter();

		const QString GET_CHARACTER("https://api.pathofexile.com/character/" + QString::fromStdString(name));

		auto callback_wrapper = [=](QNetworkReply* reply) {
			// Check for network errors.
			if (reply->error() != QNetworkReply::NoError) {
				const auto status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute);
				const auto message = reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute);
				QLOG_ERROR() << "GetCharacter: network error (" << status << "):" << message;
				return;
			};
			const QByteArray data = reply->readAll();
			GetCharacterResult result;
			JS::ParseContext context(data);
			JS::Error error = context.parseTo(result);
			if (error != JS::Error::NoError) {
				QLOG_ERROR() << "GetCharacter: json error:" << context.makeErrorString();
				return;
			};
			// Run the callback in the context object's thread.
			QMetaObject::invokeMethod(object, [=]() { callback(result); });
		};

		rate_limiter.Submit("GET /character/<name>", QNetworkRequest(QUrl(GET_CHARACTER)), callback_wrapper);
	}
}
