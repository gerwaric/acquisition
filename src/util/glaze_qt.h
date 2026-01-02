// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#pragma once

#include <map>
#include <string>
#include <string_view>
#include <unordered_map>

#include <QByteArray>
#include <QDateTime>
#include <QString>

#include <glaze/glaze.hpp>

// This is a helper define to avoid Qt Creator's warnings that this header is unused.
[[maybe_unused]] inline constexpr bool ACQUISITION_USE_GLAZE = true;

[[maybe_unused]] inline constexpr glz::opts GLAZE_OPTIONS{.null_terminated = false,
                                                          .error_on_unknown_keys = true,
                                                          .error_on_missing_keys = true};

// This file adds support to glaze for:
//
//  - QString
//  - QByteArray
//  - QDateTime (assuming RFC2822)
//  - std::map<QString, ...>
//  - std::map<QByteArray, ...>
//  - std::unordered_map<QString, ...>
//  - std::unordered_map<QByteArray, ...>

namespace {

    template<typename K>
    constexpr bool qt_key_v = std::is_same_v<K, QString> || std::is_same_v<K, QByteArray>;

} // namespace

namespace glz {

    // ----- QString -----

    template<>
    struct to<JSON, QString>
    {
        template<auto Opts>
        static inline void op(const QString &value, auto &&...args) noexcept
        {
            const QByteArray utf8 = value.toUtf8();
            const std::string_view sv{utf8.constData(), size_t(utf8.size())};
            glz::serialize<JSON>::op<Opts>(sv, std::forward<decltype(args)>(args)...);
        }
    };

    template<>
    struct from<JSON, QString>
    {
        template<auto Opts>
        static inline void op(QString &value, auto &&...args) noexcept
        {
            std::string str;
            glz::parse<JSON>::op<Opts>(str, std::forward<decltype(args)>(args)...);
            value = QString::fromUtf8(str.data(), qsizetype(str.size()));
        }
    };

    // ----- QByteArray -----

    template<>
    struct to<JSON, QByteArray>
    {
        template<auto Opts>
        static inline void op(const QByteArray &value, auto &&...args) noexcept
        {
            const std::string_view str(value.constData(), size_t(value.size()));
            glz::serialize<JSON>::op<Opts>(str, std::forward<decltype(args)>(args)...);
        }
    };

    template<>
    struct from<JSON, QByteArray>
    {
        template<auto Opts>
        static inline void op(QByteArray &value, auto &&...args) noexcept
        {
            std::string str;
            glz::parse<JSON>::op<Opts>(str, std::forward<decltype(args)>(args)...);
            value = QByteArray(str.data(), int(str.size()));
        }
    };

    // ----- QDateTime -----

    template<>
    struct to<JSON, QDateTime>
    {
        template<auto Opts>
        static inline void op(const QDateTime &dt, auto &&...args) noexcept
        {
            const QByteArray utf8 = dt.toString(Qt::RFC2822Date).toUtf8();
            const std::string_view sv(utf8.constData(), size_t(utf8.size()));
            glz::serialize<JSON>::op<Opts>(sv, std::forward<decltype(args)>(args)...);
        }
    };

    template<>
    struct from<JSON, QDateTime>
    {
        template<auto Opts>
        static inline void op(QDateTime &dt, auto &&...args) noexcept
        {
            std::string str;
            glz::parse<JSON>::op<Opts>(str, std::forward<decltype(args)>(args)...);
            dt = QDateTime::fromString(QString::fromUtf8(str.data(), qsizetype(str.size())),
                                       Qt::RFC2822Date);
        }
    };

    // -------------------- std::unordered_map --------------------

    // ---- QString keys ----

    template<typename T, typename Hash, typename KeyEq, typename Alloc>
    struct from<JSON, std::unordered_map<QString, T, Hash, KeyEq, Alloc>>
    {
        template<auto Opts>
        static void op(std::unordered_map<QString, T, Hash, KeyEq, Alloc> &out,
                       auto &&...args) noexcept
        {
            std::unordered_map<std::string, T> tmp;
            glz::parse<JSON>::op<Opts>(tmp, std::forward<decltype(args)>(args)...);

            out.clear();
            out.reserve(tmp.size());
            for (auto &[k, v] : tmp) {
                out.emplace(QString::fromUtf8(k.data(), int(k.size())), std::move(v));
            }
        }
    };

    template<typename T, typename Hash, typename KeyEq, typename Alloc>
    struct to<JSON, std::unordered_map<QString, T, Hash, KeyEq, Alloc>>
    {
        template<auto Opts>
        static void op(const std::unordered_map<QString, T, Hash, KeyEq, Alloc> &in,
                       auto &&...args) noexcept
        {
            std::unordered_map<std::string, T> tmp;
            tmp.reserve(in.size());
            for (const auto &[k, v] : in) {
                const QByteArray utf8 = k.toUtf8();
                tmp.emplace(std::string(utf8.constData(), size_t(utf8.size())), v); // copy v
            }
            glz::serialize<JSON>::op<Opts>(tmp, std::forward<decltype(args)>(args)...);
        }
    };

    // ---- QByteArray keys ----

    template<typename T, typename Hash, typename KeyEq, typename Alloc>
    struct from<JSON, std::unordered_map<QByteArray, T, Hash, KeyEq, Alloc>>
    {
        template<auto Opts>
        static void op(std::unordered_map<QByteArray, T, Hash, KeyEq, Alloc> &out,
                       auto &&...args) noexcept
        {
            std::unordered_map<std::string, T> tmp;
            glz::parse<JSON>::op<Opts>(tmp, std::forward<decltype(args)>(args)...);

            out.clear();
            out.reserve(tmp.size());
            for (auto &[k, v] : tmp) {
                out.emplace(QByteArray(k.data(), int(k.size())), std::move(v));
            }
        }
    };

    template<typename T, typename Hash, typename KeyEq, typename Alloc>
    struct to<JSON, std::unordered_map<QByteArray, T, Hash, KeyEq, Alloc>>
    {
        template<auto Opts>
        static void op(const std::unordered_map<QByteArray, T, Hash, KeyEq, Alloc> &in,
                       auto &&...args) noexcept
        {
            std::unordered_map<std::string, T> tmp;
            tmp.reserve(in.size());
            for (const auto &[k, v] : in) {
                tmp.emplace(std::string(k.constData(), size_t(k.size())), v);
            }
            glz::serialize<JSON>::op<Opts>(tmp, std::forward<decltype(args)>(args)...);
        }
    };

    // -------------------- std::map --------------------

    // QString keys

    template<typename T, typename Compare, typename Alloc>
    struct from<JSON, std::map<QString, T, Compare, Alloc>>
    {
        template<auto Opts>
        static void op(std::map<QString, T, Compare, Alloc> &out, auto &&...args) noexcept
        {
            std::map<std::string, T> tmp;
            glz::parse<JSON>::op<Opts>(tmp, std::forward<decltype(args)>(args)...);

            out.clear();
            for (auto &[k, v] : tmp) {
                out.emplace(QString::fromUtf8(k.data(), int(k.size())), std::move(v));
            }
        }
    };

    template<typename T, typename Compare, typename Alloc>
    struct to<JSON, std::map<QString, T, Compare, Alloc>>
    {
        template<auto Opts>
        static void op(const std::map<QString, T, Compare, Alloc> &in, auto &&...args) noexcept
        {
            std::map<std::string, T> tmp;
            for (const auto &[k, v] : in) {
                const QByteArray utf8 = k.toUtf8();
                tmp.emplace(std::string(utf8.constData(), size_t(utf8.size())), v);
            }
            glz::serialize<JSON>::op<Opts>(tmp, std::forward<decltype(args)>(args)...);
        }
    };

    // ---- QByteArray keys ----

    template<typename T, typename Compare, typename Alloc>
    struct from<JSON, std::map<QByteArray, T, Compare, Alloc>>
    {
        template<auto Opts>
        static void op(std::map<QByteArray, T, Compare, Alloc> &out, auto &&...args) noexcept
        {
            std::map<std::string, T> tmp;
            glz::parse<JSON>::op<Opts>(tmp, std::forward<decltype(args)>(args)...);

            out.clear();
            for (auto &[k, v] : tmp) {
                out.emplace(QByteArray(k.data(), int(k.size())), std::move(v));
            }
        }
    };

    template<typename T, typename Compare, typename Alloc>
    struct to<JSON, std::map<QByteArray, T, Compare, Alloc>>
    {
        template<auto Opts>
        static void op(const std::map<QByteArray, T, Compare, Alloc> &in, auto &&...args) noexcept
        {
            std::map<std::string, T> tmp;
            for (const auto &[k, v] : in) {
                tmp.emplace(std::string(k.constData(), size_t(k.size())), v);
            }
            glz::serialize<JSON>::op<Opts>(tmp, std::forward<decltype(args)>(args)...);
        }
    };

} // namespace glz
