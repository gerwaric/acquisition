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

#include <QByteArray>
#include <QMetaEnum>
#include <QNetworkReply>
#include <QString>
#include <QVariant>

#include <spdlog/spdlog.h>

// Create a specialized formatter for spdlog levels.

template<>
struct fmt::formatter<spdlog::level::level_enum> {
    constexpr auto parse(fmt::format_parse_context& ctx) -> decltype(ctx.begin()) {
        return ctx.end();
    };
    auto format(const spdlog::level::level_enum& level, fmt::format_context& ctx) const -> decltype(ctx.out()) {
        return fmt::format_to(ctx.out(), "{}", spdlog::level::to_string_view(level));
    };
};

// Create a specialized formatter for QString.

template <>
struct fmt::formatter<QString> {
    constexpr auto parse(fmt::format_parse_context& ctx) -> decltype(ctx.begin()) {
        return ctx.end();
    };
    auto format(const QString& str, fmt::format_context& ctx) const -> decltype(ctx.out()) {
        const auto utf8 = str.toUtf8();
        return fmt::format_to(ctx.out(), "{}", std::string_view(utf8.data(), utf8.size()));
    };
};

// Create a specialized formatter for QByteArray.

template <>
struct fmt::formatter<QByteArray> {
    constexpr auto parse(fmt::format_parse_context& ctx) -> decltype(ctx.begin()) {
        return ctx.end();
    };
    auto format(const QByteArray& arr, fmt::format_context& ctx) const -> decltype(ctx.out()) {
        const auto str = arr.toStdString();
        return fmt::format_to(ctx.out(), "{}", str);
    };
};

// Create a specialized formatter for QVariant.

template <>
struct fmt::formatter<QVariant> {
    constexpr auto parse(fmt::format_parse_context& ctx) -> decltype(ctx.begin()) {
        return ctx.end();
    }
    auto format(const QVariant& var, fmt::format_context& ctx) const -> decltype(ctx.out()) {
        if (!var.isValid()) {
            return fmt::format_to(ctx.out(), "<invalid>");
        };
        if (var.isNull()) {
            return fmt::format_to(ctx.out(), "<null>");
        };
        const std::string str = var.toString().toStdString();
        return fmt::format_to(ctx.out(), "{}", str);
    }
};

// Use a base class for Qt Enum formatters to subclass.

template <typename T>
struct QtEnumFormatter {
    static_assert(std::is_enum<T>::value, "T must be an enum type");

    template <typename ParseContext>
    constexpr auto parse(ParseContext& ctx) -> decltype(ctx.begin()) {
        return ctx.begin();
    };

    template <typename FormatContext>
    auto format(const T& value, FormatContext& ctx) const -> decltype(ctx.out()) {
        const QMetaEnum metaEnum = QMetaEnum::fromType<T>();
        const char* key = metaEnum.valueToKey(static_cast<int>(value));
        if (key) {
            return fmt::format_to(ctx.out(), "{}", key);
        } else {
            return fmt::format_to(ctx.out(), "{}({})", metaEnum.name(), static_cast<int>(value));
        };
    };
};

// Create a formatter for QnetworkReply::NetworkError.

template <>
struct fmt::formatter<QNetworkReply::NetworkError, char> : QtEnumFormatter<QNetworkReply::NetworkError> {};

// Create a formatter for Qt::CheckState.

template <>
struct fmt::formatter<Qt::CheckState, char> : QtEnumFormatter<Qt::CheckState> {};
