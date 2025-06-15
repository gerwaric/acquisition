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
#include <QDateTime>
#include <QString>

#include <map>
#include <string>
#include <unordered_map>

#include <glaze/glaze.hpp>

namespace glz {

    // ----- QByteArray -----

    template <>
    struct to<JSON, QByteArray> {
        template <auto Opts>
        static inline void op(const QByteArray& value, auto&&... args) noexcept {
            const std::string str = value.toStdString();
            serialize<JSON>::op<Opts>(str, args...);
        }
    };

    template <>
    struct from<JSON, QByteArray> {
        template <auto Opts>
        static inline void op(QByteArray& value, auto&&... args) noexcept {
            std::string str;
            parse<JSON>::op<Opts>(str, args...);
            value = QByteArray::fromStdString(str);
        }
    };

    // QByteArray -- std::map

    template <typename T>
    struct to<JSON, std::map<QByteArray, T>> {
        template <auto Opts>
        static void op(const std::map<QByteArray, T>& map, auto&&... args) noexcept {
            std::map<std::string, T> temp;
            for (const auto& [k, v] : map) {
                temp.emplace(k.toStdString(), v);
            }
            serialize<JSON>::op<Opts>(temp, std::forward<decltype(args)>(args)...);
        }
    };

    template <typename T>
    struct from<JSON, std::map<QByteArray, T>> {
        template <auto Opts>
        static void op(std::map<QByteArray, T>& map, auto&&... args) noexcept {
            std::map<std::string, T> temp;
            parse<JSON>::op<Opts>(temp, std::forward<decltype(args)>(args)...);
            map.clear();
            for (auto& [k, v] : temp) {
                map.emplace(QByteArray::fromStdString(k), std::move(v));
            }
        }
    };

    // QByteArray -- std::unordered_map

    template <typename T>
    struct to<JSON, std::unordered_map<QByteArray, T>> {
        template <auto Opts>
        static void op(const std::unordered_map<QByteArray, T>& map, auto&&... args) noexcept {
            std::unordered_map<std::string, T> temp;
            for (const auto& [k, v] : map) {
                temp.emplace(k.toStdString(), v);
            }
            serialize<JSON>::op<Opts>(temp, std::forward<decltype(args)>(args)...);
        }
    };

    template <typename T>
    struct from<JSON, std::unordered_map<QByteArray, T>> {
        template <auto Opts>
        static void op(std::unordered_map<QByteArray, T>& map, auto&&... args) noexcept {
            std::unordered_map<std::string, T> temp;
            parse<JSON>::op<Opts>(temp, std::forward<decltype(args)>(args)...);
            map.clear();
            for (auto& [k, v] : temp) {
                map.emplace(QByteArray::fromStdString(k), std::move(v));
            }
        }
    };

    // ----- QString -----

    template <>
    struct to<JSON, QString> {
        template <auto Opts>
        static inline void op(const QString& value, auto&&... args) noexcept {
            const std::string str = value.toUtf8().toStdString();
            serialize<JSON>::op<Opts>(str, args...);
        }
    };

    template <>
    struct from<JSON, QString> {
        template <auto Opts>
        static inline void op(QString& value, auto&&... args) noexcept {
            std::string str;
            parse<JSON>::op<Opts>(str, args...);
            value = QString::fromUtf8(str);
        }
    };

    // std::map<QString, T>

    template <typename T>
    struct to<JSON, std::map<QString, T>> {
        template <auto Opts>
        static void op(const std::map<QString, T>& map, auto&&... args) noexcept {
            std::map<std::string, T> temp;
            for (const auto& [k, v] : map) {
                temp.emplace(k.toUtf8().toStdString(), v);
            };
            serialize<JSON>::op<Opts>(temp, std::forward<decltype(args)>(args)...);
        };
    };

    template <typename T>
    struct from<JSON, std::map<QString, T>> {
        template <auto Opts>
        static void op(std::map<QString, T>& map, auto&&... args) noexcept {
            std::map<std::string, T> temp;
            parse<JSON>::op<Opts>(temp, std::forward<decltype(args)>(args)...);
            map.clear();
            for (auto& [k, v] : temp) {
                map.emplace(QString::fromUtf8(k), std::move(v));
            };
        };
    };

    // std::unordered_map<QString, T>

    template <typename T>
    struct to<JSON, std::unordered_map<QString, T>> {
        template <auto Opts>
        static void op(const std::unordered_map<QString, T>& map, auto&&... args) noexcept {
            std::unordered_map<std::string, T> temp;
            for (const auto& [k, v] : map) {
                temp.emplace(k.toUtf8().toStdString(), v);
            }
            serialize<JSON>::op<Opts>(temp, std::forward<decltype(args)>(args)...);
        }
    };

    template <typename T>
    struct from<JSON, std::unordered_map<QString, T>> {
        template <auto Opts>
        static void op(std::unordered_map<QString, T>& map, auto&&... args) noexcept {
            std::unordered_map<std::string, T> temp;
            parse<JSON>::op<Opts>(temp, std::forward<decltype(args)>(args)...);
            map.clear();
            for (auto& [k, v] : temp) {
                map.emplace(QString::fromUtf8(k), std::move(v));
            }
        }
    };

    // ----- QDateTime -----

    template <>
    struct to<JSON, QDateTime> {
        template <auto Opts>
        static inline void op(const QDateTime& dt, auto&&... args) noexcept {
            const std::string str = dt.toString(Qt::RFC2822Date).toStdString();
            serialize<JSON>::op<Opts>(str, std::forward<decltype(args)>(args)...);
        }
    };

    template <>
    struct from<JSON, QDateTime> {
        template <auto Opts>
        static inline void op(QDateTime& dt, auto&&... args) noexcept {
            std::string str;
            parse<JSON>::op<Opts>(str, std::forward<decltype(args)>(args)...);
            dt = QDateTime::fromString(QString::fromStdString(str), Qt::RFC2822Date);
        }
    };

}

