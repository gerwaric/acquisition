/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include <QsLog/QsLog.h>
#include <rapidjson/error/en.h>

#include "util.h"

// Hard-code the token lifetimes for a public client.
constexpr long int ACCESS_TOKEN_LIFETIME_SECS = 10 * 3600; // 10 Hours
constexpr long int REFRESH_TOKEN_LIFETIME_SECS = 7 * 24 * 3600; // 7 Days

OAuthToken::OAuthToken()
    : expires_in_(-1)
{
    QLOG_TRACE() << "OAuthToken::OAuthToken(json) entered";
}

OAuthToken::OAuthToken(const std::string& json)
    : expires_in_(-1)
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
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `access_token`";
    };
    if (doc.HasMember("expires_in") && doc["expires_in"].IsInt64()) {
        expires_in_ = doc["expires_in"].GetInt64();
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `expires_in`";
    };
    if (doc.HasMember("token_type") && doc["token_type"].IsString()) {
        token_type_ = doc["token_type"].GetString();
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `token_type`";
    };
    if (doc.HasMember("scope") && doc["scope"].IsString()) {
        scope_ = doc["scope"].GetString();
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `scope`";
    };
    if (doc.HasMember("username") && doc["username"].IsString()) {
        username_ = doc["username"].GetString();
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `username`";
    };
    if (doc.HasMember("sub") && doc["sub"].IsString()) {
        sub_ = doc["sub"].GetString();
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `sub`";
    };
    if (doc.HasMember("refresh_token") && doc["refresh_token"].IsString()) {
        refresh_token_ = doc["refresh_token"].GetString();
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `refresh_token`";
    };

    if (doc.HasMember("birthday") && doc["birthday"].IsString()) {
        birthday_ = getDate(doc["birthday"].GetString());
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `birthday`";
    };

    if (doc.HasMember("access_expiration") && doc["access_expiration"].IsString()) {
        access_expiration_ = getDate(doc["access_expiration"].GetString());
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `access_expiration`";
    };

    if (doc.HasMember("refresh_expiration") && doc["refresh_expiration"].IsString()) {
        refresh_expiration_ = getDate(doc["refresh_expiration"].GetString());
    } else {
        QLOG_ERROR() << "OAuth: serialized token does not have `refresh_expiration`";
    };
}

OAuthToken::OAuthToken(QNetworkReply& reply)
    : OAuthToken(reply.readAll().toStdString())
{
    QLOG_TRACE() << "OAuthToken::OAuthToken(reply) entered";
    // Determine birthday and expiration time.
    const QString reply_timestamp = Util::FixTimezone(reply.rawHeader("Date"));
    const QDateTime reply_birthday = QDateTime::fromString(reply_timestamp, Qt::RFC2822Date).toLocalTime();
    QLOG_TRACE() << "OAuthToken::OAuthToken(reply) reply date is" << reply_birthday.toString();

    if (birthday_.isValid()) {
        QLOG_ERROR() << "The OAuth token already has a birthday";
    };
    if (access_expiration_.isValid()) {
        QLOG_ERROR() << "The OAuth token already has an expiration";
    };
    birthday_ = reply_birthday;
    access_expiration_ = reply_birthday.addSecs(expires_in_);
    refresh_expiration_ = reply_birthday.addSecs(expires_in_
        + REFRESH_TOKEN_LIFETIME_SECS
        - ACCESS_TOKEN_LIFETIME_SECS);
}

/*
bool OAuthToken::isValid() const {
    if (access_token_.empty()) {
        return false;
    } else if (!access_expiration_.isValid()) {
        return false;
    } else {
        return (access_expiration_ > QDateTime::currentDateTime());
    };
}
*/

std::string OAuthToken::toJson() const {
    return Util::RapidjsonSerialize(toJsonDoc());
}

std::string OAuthToken::toJsonPretty() const {
    return Util::RapidjsonPretty(toJsonDoc());
}

rapidjson::Document OAuthToken::toJsonDoc() const {
    const std::string birthday = birthday_.toString(Qt::RFC2822Date).toStdString();
    const std::string access_expiration = access_expiration_.toString(Qt::RFC2822Date).toStdString();
    const std::string refresh_expiration = refresh_expiration_.toString(Qt::RFC2822Date).toStdString();
    rapidjson::Document doc(rapidjson::kObjectType);
    auto& allocator = doc.GetAllocator();
    Util::RapidjsonAddString(&doc, "access_token", access_token_, allocator);
    Util::RapidjsonAddInt64(&doc, "expires_in", expires_in_, allocator);
    Util::RapidjsonAddString(&doc, "token_type", token_type_, allocator);
    Util::RapidjsonAddString(&doc, "scope", scope_, allocator);
    Util::RapidjsonAddString(&doc, "username", username_, allocator);
    Util::RapidjsonAddString(&doc, "sub", sub_, allocator);
    Util::RapidjsonAddString(&doc, "refresh_token", refresh_token_, allocator);
    Util::RapidjsonAddString(&doc, "birthday", birthday, allocator);
    Util::RapidjsonAddString(&doc, "access_expiration", access_expiration, allocator);
    Util::RapidjsonAddString(&doc, "refresh_expiration", refresh_expiration, allocator);
    return doc;
}

QDateTime OAuthToken::getDate(const std::string& timestamp) {
    QByteArray value = QByteArray::fromStdString(timestamp);
    value = Util::FixTimezone(value);
    return QDateTime::fromString(value, Qt::RFC2822Date);
}
