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

#pragma once

#include <QDateTime>

#include <string>

#include <rapidjson/document.h>

class QNetworkReply;

class OAuthToken {
public:
    OAuthToken();
    OAuthToken(const std::string& json);
    OAuthToken(QNetworkReply& reply);
    std::string access_token() const { return access_token_; };
    int expires_in() const { return expires_in_; };
    std::string scope() const { return scope_; };
    std::string username() const { return username_; };
    std::string sub() const { return sub_; };
    std::string refresh_token() const { return refresh_token_; };
    QDateTime birthday() const { return birthday_; };
    QDateTime access_expiration() const { return access_expiration_; };
    QDateTime refresh_expiration() const { return refresh_expiration_; };
    std::string toJson() const;
    std::string toJsonPretty() const;
private:
    static QDateTime getDate(const std::string& timestamp);
    rapidjson::Document toJsonDoc() const;

    std::string access_token_;
    long long int expires_in_;
    std::string token_type_;
    std::string scope_;
    std::string username_;
    std::string sub_;
    std::string refresh_token_;

    QDateTime birthday_;
    QDateTime access_expiration_;
    QDateTime refresh_expiration_;
};
