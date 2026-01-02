// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QByteArray>
#include <QMetaEnum>
#include <QNetworkReply>
#include <QString>
#include <QVariant>

#include <spdlog/spdlog.h>

// This is a helper define to avoid Qt Creator's warnings that this header is unused.
[[maybe_unused]] inline constexpr bool ACQUISITION_USE_SPDLOG = true;

// Define an inline helper function to convert levels into QStrings.

inline QString to_qstring(spdlog::level::level_enum level)
{
    const auto sv = spdlog::level::to_string_view(level);
    return QString::fromUtf8(sv.data(), sv.size());
}

// Create a specialized formatter for spdlog levels.

template<>
struct fmt::formatter<spdlog::level::level_enum>
{
    constexpr auto parse(fmt::format_parse_context &ctx) -> decltype(ctx.begin())
    {
        return ctx.end();
    }
    auto format(const spdlog::level::level_enum &level, fmt::format_context &ctx) const
        -> decltype(ctx.out())
    {
        return fmt::format_to(ctx.out(), "{}", spdlog::level::to_string_view(level));
    }
};

// Create a specialized formatter for QString.

template<>
struct fmt::formatter<QString>
{
    constexpr auto parse(fmt::format_parse_context &ctx) -> decltype(ctx.begin())
    {
        return ctx.end();
    }
    auto format(const QString &str, fmt::format_context &ctx) const -> decltype(ctx.out())
    {
        const auto utf8 = str.toUtf8();
        return fmt::format_to(ctx.out(), "{}", std::string_view(utf8.data(), utf8.size()));
    }
};

// Create a specialized formatter for QByteArray.

template<>
struct fmt::formatter<QByteArray>
{
    constexpr auto parse(fmt::format_parse_context &ctx) -> decltype(ctx.begin())
    {
        return ctx.end();
    }
    auto format(const QByteArray &arr, fmt::format_context &ctx) const -> decltype(ctx.out())
    {
        const auto str = arr.toStdString();
        return fmt::format_to(ctx.out(), "{}", str);
    }
};

// Create a specialized formatter for QVariant.

template<>
struct fmt::formatter<QVariant>
{
    constexpr auto parse(fmt::format_parse_context &ctx) -> decltype(ctx.begin())
    {
        return ctx.end();
    }
    auto format(const QVariant &var, fmt::format_context &ctx) const -> decltype(ctx.out())
    {
        if (!var.isValid()) {
            return fmt::format_to(ctx.out(), "<invalid>");
        }
        if (var.isNull()) {
            return fmt::format_to(ctx.out(), "<null>");
        }
        const std::string str = var.toString().toStdString();
        return fmt::format_to(ctx.out(), "{}", str);
    }
};

// Use a base class for Qt Enum formatters to subclass.

template<typename T>
struct QtEnumFormatter
{
    static_assert(std::is_enum<T>::value, "T must be an enum type");

    template<typename ParseContext>
    constexpr auto parse(ParseContext &ctx) -> decltype(ctx.begin())
    {
        return ctx.begin();
    }

    template<typename FormatContext>
    auto format(const T &value, FormatContext &ctx) const -> decltype(ctx.out())
    {
        const QMetaEnum metaEnum = QMetaEnum::fromType<T>();
        const char *key = metaEnum.valueToKey(static_cast<int>(value));
        if (key) {
            return fmt::format_to(ctx.out(), "{}", key);
        } else {
            return fmt::format_to(ctx.out(), "{}({})", metaEnum.name(), static_cast<int>(value));
        }
    }
};

// Create a formatter for QnetworkReply::NetworkError.

template<>
struct fmt::formatter<QNetworkReply::NetworkError, char>
    : QtEnumFormatter<QNetworkReply::NetworkError>
{};

// Create a formatter for Qt::CheckState.

template<>
struct fmt::formatter<Qt::CheckState, char> : QtEnumFormatter<Qt::CheckState>
{};
