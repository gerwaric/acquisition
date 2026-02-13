// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <QByteArray>
#include <QByteArrayView>
#include <QMetaEnum>
#include <QNetworkReply>
#include <QString>
#include <QStringView>
#include <QVariant>

#include <spdlog/spdlog.h>

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
struct fmt::formatter<QString> : fmt::formatter<std::string_view>
{
    auto format(const QString &s, fmt::format_context &ctx) const -> fmt::format_context::iterator
    {
        const QByteArray utf8 = s.toUtf8();
        const std::string_view sv{utf8.constData(), static_cast<size_t>(utf8.size())};
        return fmt::formatter<std::string_view>::format(sv, ctx);
    }
};

template<>
struct fmt::formatter<QStringView> : fmt::formatter<std::string_view>
{
    auto format(QStringView s, fmt::format_context &ctx) const -> fmt::format_context::iterator
    {
        const QByteArray utf8 = s.toUtf8();
        const std::string_view sv{utf8.constData(), static_cast<size_t>(utf8.size())};
        return fmt::formatter<std::string_view>::format(sv, ctx);
    }
};

template<>
struct fmt::formatter<QLatin1StringView> : fmt::formatter<std::string_view>
{
    auto format(QStringView s, fmt::format_context &ctx) const -> fmt::format_context::iterator
    {
        const QByteArray utf8 = s.toUtf8();
        const std::string_view sv{utf8.constData(), static_cast<size_t>(s.size())};
        return fmt::formatter<std::string_view>::format(sv, ctx);
    }
};

// Create a specialized formatter for QByteArray.

template<>
struct fmt::formatter<QByteArray> : fmt::formatter<std::string_view>
{
    auto format(const QByteArray &ba, fmt::format_context &ctx) const
        -> fmt::format_context::iterator
    {
        const char *data = reinterpret_cast<const char *>(ba.data());
        const size_t n = static_cast<size_t>(ba.size());
        const std::string_view sv{data ? data : "", n};
        return fmt::formatter<std::string_view>::format(sv, ctx);
    }
};

template<>
struct fmt::formatter<QByteArrayView> : fmt::formatter<std::string_view>
{
    auto format(QByteArrayView bav, fmt::format_context &ctx) const -> fmt::format_context::iterator
    {
        // QByteArrayView exposes data/size; its data may be null for empty.
        const char *data = reinterpret_cast<const char *>(bav.data());
        const size_t n = static_cast<size_t>(bav.size());
        const std::string_view sv{data ? data : "", n};
        return fmt::formatter<std::string_view>::format(sv, ctx);
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
