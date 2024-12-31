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
    std::string access_token() const { return m_access_token; };
    int expires_in() const { return m_expires_in; };
    std::string scope() const { return m_scope; };
    std::string username() const { return m_username; };
    std::string sub() const { return m_sub; };
    std::string refresh_token() const { return m_refresh_token; };
    QDateTime birthday() const { return m_birthday; };
    QDateTime access_expiration() const { return m_access_expiration; };
    QDateTime refresh_expiration() const { return m_refresh_expiration; };
    std::string toJson() const;
    std::string toJsonPretty() const;
private:
    static QDateTime getDate(const std::string& timestamp);
    rapidjson::Document toJsonDoc() const;

    std::string m_access_token;
    long long int m_expires_in;
    std::string m_token_type;
    std::string m_scope;
    std::string m_username;
    std::string m_sub;
    std::string m_refresh_token;

    QDateTime m_birthday;
    QDateTime m_access_expiration;
    QDateTime m_refresh_expiration;
};
