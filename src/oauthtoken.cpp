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

#include "oauthtoken.h"

#include <QNetworkReply>

#include "QsLog.h"
#include "rapidjson/error/en.h"

#include "util.h"

OAuthToken::OAuthToken() :
    expires_in_(-1)
{
    QLOG_TRACE() << "OAuthToken::OAuthToken(json) entered";
}

OAuthToken::OAuthToken(const std::string& json) :
    expires_in_(-1)
{
    QLOG_TRACE() << "OAuthToken::OAuthToken(json) entered";
    rapidjson::Document doc;
    doc.Parse(json.c_str());
    if (doc.HasParseError()) {
        QLOG_ERROR() << "Error parsing OAuthToken from json:" << rapidjson::GetParseError_En(doc.GetParseError());
        return;
    };
    if (doc.IsObject() == false) {
        QLOG_ERROR() << "OAuthToken json is not an object.";
        return;
    };
    if (doc.HasMember("access_token") && doc["access_token"].IsString()) {
        access_token_ = doc["access_token"].GetString();
    };
    if (doc.HasMember("expires_in") && doc["expires_in"].IsInt64()) {
        expires_in_ = doc["expires_in"].GetInt64();
    };
    if (doc.HasMember("token_type") && doc["token_type"].IsString()) {
        token_type_ = doc["token_type"].GetString();
    };
    if (doc.HasMember("scope") && doc["scope"].IsString()) {
        scope_ = doc["scope"].GetString();
    };
    if (doc.HasMember("username") && doc["username"].IsString()) {
        username_ = doc["username"].GetString();
    };
    if (doc.HasMember("sub") && doc["sub"].IsString()) {
        sub_ = doc["sub"].GetString();
    };
    if (doc.HasMember("refresh_token") && doc["refresh_token"].IsString()) {
        refresh_token_ = doc["refresh_token"].GetString();
    };

    if (doc.HasMember("birthday") && doc["birthday"].IsString()) {
        birthday_ = getDate(doc["birthday"].GetString());
    } else {
        QLOG_WARN() << "Constructing OAuth token without a birthday.";
    };

    if (doc.HasMember("expiration") && doc["expiration"].IsString()) {
        expiration_ = getDate(doc["expiration"].GetString());
    } else {
        QLOG_WARN() << "Constructing OAuth token without an expiration";
    };
}

OAuthToken::OAuthToken(QNetworkReply& reply) :
    OAuthToken(reply.readAll().toStdString())
{
    QLOG_TRACE() << "OAuthToken::OAuthToken(reply) entered";
    // Determine birthday and expiration time.
    const QString reply_timestamp = Util::FixTimezone(reply.rawHeader("Date"));
    const QDateTime reply_birthday = QDateTime::fromString(reply_timestamp, Qt::RFC2822Date).toLocalTime();
    QLOG_TRACE() << "OAuthToken::OAuthToken(reply) reply date is" << reply_birthday.toString();

    if (birthday_.isValid()) {
        QLOG_ERROR() << "The OAuth token already has a birthday";
    };
    if (expiration_.isValid()) {
        QLOG_ERROR() << "The OAuth token already has an expiration";
    };
    birthday_ = reply_birthday;
    expiration_ = reply_birthday.addSecs(expires_in_);
}

bool OAuthToken::isValid() const {
    if (access_token_.empty()) {
        return false;
    } else if (!expiration_.isValid()) {
        return false;
    } else {
        return (expiration_ > QDateTime::currentDateTime());
    };
}

std::string OAuthToken::toJson() const {
    return Util::RapidjsonSerialize(toJsonDoc());
}

std::string OAuthToken::toJsonPretty() const {
    return Util::RapidjsonPretty(toJsonDoc());
}

rapidjson::Document OAuthToken::toJsonDoc() const {
    rapidjson::Document doc(rapidjson::kObjectType);
    auto& allocator = doc.GetAllocator();
    Util::RapidjsonAddString(&doc, "access_token", access_token_, allocator);
    Util::RapidjsonAddInt64(&doc, "expires_in", expires_in_, allocator);
    Util::RapidjsonAddString(&doc, "token_type", token_type_, allocator);
    Util::RapidjsonAddString(&doc, "scope", scope_, allocator);
    Util::RapidjsonAddString(&doc, "username", username_, allocator);
    Util::RapidjsonAddString(&doc, "sub", sub_, allocator);
    Util::RapidjsonAddString(&doc, "refresh_token", refresh_token_, allocator);
    if (birthday_.isValid()) {
        const std::string birthday = birthday_.toString(Qt::RFC2822Date).toStdString();
        Util::RapidjsonAddString(&doc, "birthday", birthday, allocator);
    };
    if (expiration_.isValid()) {
        const std::string expiration = expiration_.toString(Qt::RFC2822Date).toStdString();
        Util::RapidjsonAddString(&doc, "expiration", expiration, allocator);
    };
    return doc;
}

QDateTime OAuthToken::getDate(const std::string& timestamp) {
    QByteArray value = QByteArray::fromStdString(timestamp);
    value = Util::FixTimezone(value);
    return QDateTime::fromString(value, Qt::RFC2822Date);
}
