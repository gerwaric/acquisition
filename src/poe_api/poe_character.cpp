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

	Character::Character(const std::string& json) {
		JS::ParseContext context(json);
		JS::Error error = context.parseTo(*this);
		if (error != JS::Error::NoError) {
			QLOG_ERROR() << "Error parsing character:" << context.makeErrorString();
		};
	};

	struct CharacterListWrapper {
		CharacterListWrapper(const std::string& data) {
			JS::ParseContext context(data);
			JS::Error error = context.parseTo(*this);
			if (error != JS::Error::NoError) {
				QLOG_ERROR() << "Error parsing wrapped characters:" << context.makeErrorString();
				return;
			};
		};
		std::vector<PoE::Character> characters;
		JS_OBJ(characters);
	};

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
			// Run the callback in the context object's thread.
			QMetaObject::invokeMethod(object,
				[=]() {
					const CharacterListWrapper result(reply->readAll().toStdString());
					callback(result.characters);
				});
		};
		QMetaObject::invokeMethod(&rate_limiter,
			[=]() {
				rate_limiter.Submit("GET /character",
					QNetworkRequest(QUrl(LIST_CHARACTERS)),
					callback_wrapper);
			});
	}

	struct CharacterWrapper {
		CharacterWrapper(const std::string& data) {
			JS::ParseContext context(data);
			JS::Error error = context.parseTo(*this);
			if (error != JS::Error::NoError) {
				QLOG_ERROR() << "Error parsing wrapped character:" << context.makeErrorString();
				return;
			};
		};
		PoE::Character character;
		JS_OBJ(character);
	};

	void GetCharacter(QObject* object, GetCharacterCallback callback, const PoE::CharacterName& name)
	{
		static auto& rate_limiter = Application::instance().rate_limiter();

		if (!name) {
			QLOG_ERROR() << "PoE API: cannot get character: name is empty.";
			return;
		};

		const QString GET_CHARACTER("https://api.pathofexile.com/character/" + QString(name));

		auto callback_wrapper = [=](QNetworkReply* reply) {
			// Check for network errors.
			if (reply->error() != QNetworkReply::NoError) {
				const auto status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute);
				const auto message = reply->attribute(QNetworkRequest::HttpReasonPhraseAttribute);
				QLOG_ERROR() << "GetCharacter: network error (" << status << "):" << message;
				return;
			};
			// Run the callback in the context object's thread.
			QMetaObject::invokeMethod(object,
				[=]() {
					CharacterWrapper result(reply->readAll().toStdString());
					callback(result.character);
				});
		};
		QMetaObject::invokeMethod(&rate_limiter,
			[=]() {
				rate_limiter.Submit("GET /character/<name>",
					QNetworkRequest(QUrl(GET_CHARACTER)),
					callback_wrapper);
			});
	}
}
