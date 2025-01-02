/*
    Copyright (C) 2024-2025 Gerwaric

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

#include <json_struct/json_struct.h>

#include <QByteArray>
#include <QDateTime>
#include <QNetworkReply>
#include <QString>

#include <array>

namespace JS {

    template<>
    struct TypeHandler<QByteArray>
    {
    public:
        static inline Error to(QByteArray& to_type, JS::ParseContext& context)
        {
            to_type = QByteArray(context.token.value.data, context.token.value.size);
            return Error::NoError;
        }
        static void from(const QByteArray& from_type, JS::Token& token, JS::Serializer& serializer)
        {
            token.value_type = JS::Type::String;
            token.value.data = from_type.constData();
            token.value.size = from_type.size();
            serializer.write(token);
        }
    };

    template<>
    struct TypeHandler<QString>
    {
    public:
        static inline Error to(QString& to_type, JS::ParseContext& context)
        {
            const QByteArray value(context.token.value.data, context.token.value.size);
            to_type = QString::fromUtf8(value);
            return Error::NoError;
        }
        static void from(const QString& from_type, JS::Token& token, JS::Serializer& serializer)
        {
            const QByteArray value = from_type.toUtf8();
            token.value_type = JS::Type::String;
            token.value.data = value.constData();
            token.value.size = value.size();
            serializer.write(token);
        }
    };

    template<>
    struct TypeHandler<QDateTime>
    {
    public:
        static inline Error to(QDateTime& to_type, JS::ParseContext& context)
        {
            // This is necessary because Qt 6.8 does not support obsolete time zone labels
            // that are still allowable in RFC2822.
            static const std::array<std::pair<QByteArray, QByteArray>, 10> obsolete_zones = { {
                {"GMT", "+0000"},
                {"UT" , "+0000"},
                {"EST", "-0005"},
                {"EDT", "-0004"},
                {"CST", "-0006"},
                {"CDT", "-0005"},
                {"MST", "-0007"},
                {"MDT", "-0006"},
                {"PST", "-0008"},
                {"PDT", "-0007"}
            } };
            QByteArray value(context.token.value.data, context.token.value.size);
            for (auto& pair : obsolete_zones) {
                const QByteArray& zone = pair.first;
                const QByteArray& offset = pair.second;
                if (value.endsWith(zone)) {
                    value.chop(zone.length());
                    value.append(offset);
                };
            };
            to_type = QDateTime::fromString(value, Qt::RFC2822Date);
            if (!to_type.isValid()) {
                return Error::UserDefinedErrors;
            };
            return Error::NoError;
        }
        static void from(const QDateTime& from_type, JS::Token& token, JS::Serializer& serializer)
        {
            const std::string buf = from_type.toString(Qt::RFC2822Date).toStdString();
            token.value_type = JS::Type::String;
            token.value.data = buf.data();
            token.value.size = buf.size();
            serializer.write(token);
        }
    };

    template <typename Value, typename Map>
    struct TypeHandlerMap<QString,Value,Map>
    {
        static inline Error to(Map& to_type, ParseContext& context)
        {
            if (context.token.value_type != Type::ObjectStart)
            {
                return JS::Error::ExpectedObjectStart;
            };
            Error error = context.nextToken();
            if (error != JS::Error::NoError) {
                return error;
            };
            while (context.token.value_type != Type::ObjectEnd)
            {
                std::string str;
                Internal::handle_json_escapes_in(context.token.name, str);
                const QByteArray bytes(str.data(), str.size());
                QString key = QString::fromUtf8(bytes);
                Value v;
                error = TypeHandler<Value>::to(v, context);
                to_type[std::move(key)] = std::move(v);
                if (error != JS::Error::NoError) {
                    return error;
                };
                error = context.nextToken();
            };
            return error;
        }

        static void from(const Map& from, Token& token, Serializer& serializer)
        {
            token.value_type = Type::ObjectStart;
            token.value = DataRef("{");
            serializer.write(token);
            for (auto it = from.begin(); it != from.end(); ++it) {
                const QByteArray bytes = it->first.toUtf8();
                token.name.data = bytes.constData();
                token.name.size = bytes.size();
                token.name_type = Type::String;
                TypeHandler<Value>::from(it->second, token, serializer);
            };
            token.name.size = 0;
            token.name.data = "";
            token.name_type = Type::String;
            token.value_type = Type::ObjectEnd;
            token.value = DataRef("}");
            serializer.write(token);
        }
    };
}
