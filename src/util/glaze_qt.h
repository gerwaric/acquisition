// Copyright (C) 2025 Tom Holz.
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <glaze/glaze.hpp>

#include "util/spdlog_qt.h"

#include <QByteArray>
#include <QDateTime>
#include <QString>

#include <map>
#include <string>
#include <unordered_map>

// This is a helper define to avoid Qt Creator's warnings that this header is unused.
constexpr bool ACQUISITION_USE_GLAZE = true;

// This file adds support to glaze for:
//
//  - QString
//  - QByteArray
//  - QDateTime
//  - std::map<QString,T>
//  - std::map<QByteArray,T>
//  - std::unordered_map<QString,T>
//  - std::unordered_map<QByteArray,T>
//
// WARNING: this only works with the two-argument forms of map and unordered_map.

namespace {

    template<typename T>
    constexpr bool is_qbytearray = std::is_same_v<T, QByteArray>;

    template<typename T>
    constexpr bool is_qstring = std::is_same_v<T, QString>;

    template<typename Key>
    constexpr bool is_qt_key = is_qbytearray<Key> || is_qstring<Key>;

    template<template<typename, typename> class Map, typename Key, typename T>
    constexpr bool is_map = std::is_same_v<Map<Key, T>, std::map<Key, T>>;

    template<template<typename, typename> class Map, typename Key, typename T>
    constexpr bool is_unordered_map = std::is_same_v<Map<Key, T>, std::unordered_map<Key, T>>;

    template<template<typename, typename> class Map, typename Key, typename T>
    constexpr bool is_supported_map = is_map<Map, Key, T> || is_unordered_map<Map, Key, T>;

    template<template<typename, typename> class Map, typename Key, typename T>
    constexpr bool is_supported_map_with_qt_key = is_qt_key<Key> && is_supported_map<Map, Key, T>;

} // namespace

namespace glz {

    // ----- QString -----

    template<>
    struct to<JSON, QString>
    {
        template<auto Opts>
        static inline void op(const QString &value, auto &&...args) noexcept
        {
            const QByteArray utf8{value.toUtf8()};
            const std::string str{utf8.toStdString()};
            glz::serialize<JSON>::op<Opts>(str, args...);
        }
    };

    template<>
    struct from<JSON, QString>
    {
        template<auto Opts>
        static inline void op(QString &value, auto &&...args) noexcept
        {
            std::string str;
            glz::parse<JSON>::op<Opts>(str, args...);
            value = QString::fromUtf8(str);
        }
    };

    // ----- QByteArray -----

    template<>
    struct to<JSON, QByteArray>
    {
        template<auto Opts>
        static inline void op(const QByteArray &value, auto &&...args) noexcept
        {
            const std::string_view str(value.constData(), value.size());
            glz::serialize<JSON>::op<Opts>(str, args...);
        }
    };

    template<>
    struct from<JSON, QByteArray>
    {
        template<auto Opts>
        static inline void op(QByteArray &value, auto &&...args) noexcept
        {
            std::string str;
            glz::parse<JSON>::op<Opts>(str, args...);
            value = QByteArray::fromStdString(str);
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
            const std::string str(utf8.constData(), utf8.size());
            serialize<JSON>::op<Opts>(str, std::forward<decltype(args)>(args)...);
        }
    };

    template<>
    struct from<JSON, QDateTime>
    {
        template<auto Opts>
        static inline void op(QDateTime &dt, auto &&...args) noexcept
        {
            std::string str;
            parse<JSON>::op<Opts>(str, std::forward<decltype(args)>(args)...);
            dt = QDateTime::fromString(QString::fromStdString(str), Qt::RFC2822Date);
        }
    };

    // ----- maps with QString and QByteArray keys -----

    template<template<typename, typename> class Map, typename Key, typename T>
        requires is_supported_map_with_qt_key<Map, Key, T>
    struct to<JSON, Map<Key, T>>
    {
        template<auto Opts>
        static void op(const Map<Key, T> &map, auto &&...args) noexcept
        {
            Map<std::string, T> std_map;
            for (const auto &[k, v] : map) {
                if constexpr (is_qbytearray<Key>) {
                    const std::string s(k.constData(), k.size());
                    std_map.emplace(s, std::move(v));
                } else if constexpr (is_qstring<Key>) {
                    const QByteArray utf8 = k.toUtf8();
                    const std::string s(utf8.constData(), utf8.size());
                    std_map.emplace(s, std::move(v));
                }
            }
            glz::serialize<JSON>::op<Opts>(std_map, std::forward<decltype(args)>(args)...);
        }
    };

    template<template<typename, typename> class Map, typename Key, typename T>
        requires is_supported_map_with_qt_key<Map, Key, T>
    struct from<JSON, Map<Key, T>>
    {
        template<auto Opts>
        static void op(Map<Key, T> &map, auto &&...args) noexcept
        {
            Map<std::string, T> std_map;
            glz::parse<JSON>::op<Opts>(std_map, std::forward<decltype(args)>(args)...);
            map.clear();
            for (auto &[k, v] : std_map) {
                if constexpr (is_qbytearray<Key>) {
                    map.emplace(QByteArray::fromStdString(k), std::move(v));
                } else if constexpr (is_qstring<Key>) {
                    map.emplace(QString::fromUtf8(k), std::move(v));
                }
            }
        }
    };

} // namespace glz
