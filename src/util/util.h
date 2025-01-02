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
#include <QDebug>
#include <QMetaEnum>
#include <QObject>
#include <QString>
#include <QUrlQuery>

#include <json_struct/json_struct.h>
#include <QsLog/QsLog.h>
#include <rapidjson/document.h>

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

    QString Md5(const QString& value);
    double AverageDamage(const QString& s);
    void PopulateBuyoutTypeComboBox(QComboBox* combobox);
    void PopulateBuyoutCurrencyComboBox(QComboBox* combobox);

    int TextWidth(TextWidthId id);

    void ParseJson(QNetworkReply* reply, rapidjson::Document* doc);
    QString GetCsrfToken(const QByteArray& page, const QString& name);
    QString FindTextBetween(const QString& page, const QString& left, const QString& right);

    QString RapidjsonSerialize(const rapidjson::Value& val);
    QString RapidjsonPretty(const rapidjson::Value& val);
    void RapidjsonAddString(rapidjson::Value* object, const char* const name, const QString& value, rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>& alloc);
    void RapidjsonAddConstString(rapidjson::Value* object, const char* const name, const QString& value, rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>& alloc);
    void RapidjsonAddInt64(rapidjson::Value* object, const char* const name, qint64 value, rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>& alloc);


    void GetTabColor(rapidjson::Value& json, int& r, int& g, int& b);

    QString StringReplace(const QString& haystack, const QString& needle, const QString& replace);
    QColor recommendedForegroundTextColor(const QColor& backgroundColor);

    /*
        Example usage:
            MatchMod("+# to Life", "+12.3 to Life", &result);
        Will return true if matches and save average value to output.
    */
    bool MatchMod(const char* match, const char* mod, double* output);

    QString Capitalise(const QString& str);

    QString TimeAgoInWords(const QDateTime buyout_time);

    QString Decode(const QString& entity);

    QUrlQuery EncodeQueryItems(const std::list<std::pair<QString, QString>>& items);

    void unique_elements(std::vector<QString>& vec);

    QByteArray FixTimezone(const QByteArray& rfc2822_date);

    // Convert Q_ENUM and Q_ENUM_NS objects to their string value.
    template<typename T>
    QString toString(const T& value) {
        return QMetaEnum::fromType<T>().valueToKey(static_cast<std::underlying_type_t<T>>(value));
    };

    template<typename T>
    void parseJson(const std::string& json, T& out) {
        JS::ParseContext context(json);
        if (context.parseTo<T>(out) != JS::Error::NoError) {
            const QString type_name(typeid(T).name());
            const QString error_message = QString::fromStdString(context.makeErrorString());
            QLOG_ERROR() << "Error parsing json into" << type_name << ":" << error_message;
        };
    }

    template<typename T>
    inline void parseJson(const QByteArray& json, T& out) {
        parseJson<T>(json.toStdString(), out);
    }

    template<typename T>
    inline void parseJson(const QString& json, T& out) {
        parseJson<T>(json.toUtf8(), out);
    }

    template<typename T>
    inline T parseJson(const QByteArray& json) {
        T result;
        parseJson<T>(json, result);
        return result;
    }

    template<typename T>
    inline T parseJson(const QString& json) {
        T result;
        parseJson<T>(json.toUtf8(), result);
        return result;
    }

}
