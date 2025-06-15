/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include <QByteArrayView>
#include <QStringView>

#include <string>
#include <string_view>

#include <spdlog/spdlog.h>

#include <util/glaze_qt.h>

namespace json {

    // Serialization

    template <typename T>
    std::string to_string(const T& value) {
        std::string buffer;
        const auto result = glz::write_json(value, buffer);
        if (result) {
            const std::string type_name = typeid(T).name();
            const std::string error_message = glz::format_error(result, buffer);
            spdlog::error("Error serializing {} into json: {}", type_name, error_message);
            buffer = "";
        };
        return buffer;
    }

    template <typename T>
    inline QByteArray to_qbytearray(const T& value) {
        const std::string json = json::to_string(value);
        return QByteArray::fromStdString(json);
    }

    template <typename T>
    inline QString to_qstring(const T& value) {
        const QByteArray utf8 = json::to_qbytearray(value);
        return QString::fromUtf8(utf8);
    }

    // Deserialization

    template <typename T>
    void from_json_into(T& out, QByteArrayView json) {
        const std::string_view json_view(json.constData(), json.size());
        constexpr glz::opts options{.error_on_unknown_keys = false};
        glz::context ctx{};
        const auto result = glz::read<options>(out, json_view, ctx);
        if (result) {
            const std::string type_name = typeid(T).name();
            const std::string error_message = glz::format_error(result, json_view);
            spdlog::error("Error parsing json to {}: {}", type_name, error_message);
        }
    }

    template <typename T>
    void from_json_into_strict(T& out, QByteArrayView json) {
        const std::string_view json_view(json.constData(), json.size());
        const auto result = glz::read_json(out, json_view);
        if (result) {
            const std::string type_name = typeid(T).name();
            const std::string error_message = glz::format_error(result, json_view);
            spdlog::error("Error parsing json to {}: {}", type_name, error_message);
        }
    }

    template <typename T>
    inline T from_json(QByteArrayView json) {
        T out;
        json::from_json_into<T>(out, json);
        return out;
    }

    template <typename T>
    inline T from_json_strict(QByteArrayView json) {
        T out;
        json::from_json_into_strict<T>(out, json);
        return out;
    }

    template <typename T>
    inline void from_json_into(T& out, QStringView json) {
        const QByteArray utf8 = json.toUtf8();
        json::from_json_into(out, QByteArrayView(utf8));
    }

    template <typename T>
    inline void from_json_into_strict(T& out, QStringView json) {
        const QByteArray utf8 = json.toUtf8();
        json::from_json_into_strict(out, QByteArrayView(utf8));
    }

    template <typename T>
    inline T from_json(QStringView json) {
        const QByteArray utf8 = json.toUtf8();
        return from_json<T>(QByteArrayView(utf8));
    }

    template <typename T>
    inline T from_json_strict(QStringView json) {
        const QByteArray utf8 = json.toUtf8();
        return from_json_strict<T>(QByteArrayView(utf8));
    }
}
