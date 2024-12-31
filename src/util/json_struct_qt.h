#pragma once

#include <json_struct/json_struct.h>

#include <QByteArray>
#include <QDateTime>
#include <QString>

#include <string>

#include "util.h"

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
            const QByteArray value(context.token.value.data, context.token.value.size);
            const QByteArray fixed(Util::FixTimezone(value));
            to_type = QDateTime::fromString(fixed, Qt::RFC2822Date);
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

template<typename T>
T parseJson(const QByteArray& bytes) {
    const std::string json(bytes.toStdString());
    JS::ParseContext context(json);
    T result;
    if (context.parseTo(&T) != JS::Error::NoError) {
        const QString type_name(typeid(T).name());
        const std::string error_message = context.makeErrorString();
        QLOG_ERROR() << "Error parsing json into" << type_name << ":" << error_message;
    };
    return result;
}

template<typename T>
T parseJson(const QString& json) {
    const QByteArray bytes = json.toUtf8();
    return parseJson<T>(bytes);
}

template<typename T>
T parseJson(const QNetworkReply* reply) {
    const QByteArray bytes = reply->readAll();
    return parseJson<T>(bytes);
}
