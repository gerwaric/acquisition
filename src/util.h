/*
    Copyright 2014 Ilya Zhuravlev

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

#include <string>
#include <QDateTime>
#include "rapidjson/document.h"
#include <QDebug>
#include <QMetaEnum>
#include <QObject>
#include <QUrlQuery>

#include "QsLogLevel.h"

class QComboBox;
class QNetworkReply;

struct Buyout;

enum class TextWidthId {
    WIDTH_MIN_MAX,
    WIDTH_LABEL,
    WIDTH_RGB,
    WIDTH_GROUP,    // Unused?
    WIDTH_BOOL_LABEL
};

// Reflection example for an ENUM in QT 5.4.x
class RefreshReason {
    Q_GADGET
public:
    enum Type {
        Unknown,
        ItemsChanged,
        SearchFormChanged,
        TabCreated,
        TabChanged
    };
    Q_ENUM(Type)
private:
    Type type;
};
QDebug& operator<<(QDebug& os, const RefreshReason::Type& obj);

class TabSelection {
    Q_GADGET
public:
    enum Type {
        All,
        Checked,
        Selected,
    };
    Q_ENUM(Type)
private:
    Type type;
};
QDebug& operator<<(QDebug& os, const TabSelection::Type& obj);

QDebug& operator<<(QDebug& os, const QsLogging::Level& obj);

namespace Util {

    QsLogging::Level TextToLogLevel(const QString& level);
    QString LogLevelToText(QsLogging::Level level);

    std::string Md5(const std::string& value);
    double AverageDamage(const std::string& s);
    void PopulateBuyoutTypeComboBox(QComboBox* combobox);
    void PopulateBuyoutCurrencyComboBox(QComboBox* combobox);

    int TextWidth(TextWidthId id);

    void ParseJson(QNetworkReply* reply, rapidjson::Document* doc);
    std::string GetCsrfToken(const QByteArray& page, const std::string& name);
    std::string FindTextBetween(const std::string& page, const std::string& left, const std::string& right);

    std::string RapidjsonSerialize(const rapidjson::Value& val);
    std::string RapidjsonPretty(const rapidjson::Value& val);
    void RapidjsonAddString(rapidjson::Value* object, const char* const name, const std::string& value, rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>& alloc);
    void RapidjsonAddConstString(rapidjson::Value* object, const char* const name, const std::string& value, rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>& alloc);
    void RapidjsonAddInt64(rapidjson::Value* object, const char* const name, qint64 value, rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>& alloc);

    std::string StringReplace(const std::string& haystack, const std::string& needle, const std::string& replace);
    std::string StringJoin(const std::vector<std::string>& array, const std::string& separator);
    std::vector<std::string> StringSplit(const std::string& str, char delim);
    QColor recommendedForegroundTextColor(const QColor& backgroundColor);
    std::string hexStr(const uint8_t* data, int len);

    /*
        Example usage:
            MatchMod("+# to Life", "+12.3 to Life", &result);
        Will return true if matches and save average value to output.
    */
    bool MatchMod(const char* match, const char* mod, double* output);

    std::string Capitalise(const std::string& str);

    std::string TimeAgoInWords(const QDateTime buyout_time);

    std::string Decode(const std::string& entity);

    QUrlQuery EncodeQueryItems(const std::list<std::pair<QString, QString>>& items);

    void unique_elements(std::vector<std::string>& vec);

    QByteArray FixTimezone(const QByteArray& rfc2822_date);

    std::string ConvertAsciiToUtf(const std::string& asciiString);

    // Convert Q_ENUM and Q_ENUM_NS objects to their string value.
    template<typename T>
    QString toString(const T& value) {
        return QMetaEnum::fromType<T>().valueToKey(static_cast<std::underlying_type_t<T>>(value));
    };
}
