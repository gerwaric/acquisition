// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QByteArray>

#include <optional>

#include "util/glaze_qt.h"  // IWYU pragma: keep
#include "util/spdlog_qt.h" // IWYU pragma: keep

template<typename T>
bool read_json(const QByteArray &json, T &out)
{
    const std::string_view str{json.constData(), size_t(json.size())};
    const glz::error_ctx err = glz::read_json(out, str);
    if (err) {
        const char *type = typeid(T).name();
        const std::string msg = glz::format_error(err, str);
        spdlog::error("Error reading {} from json: {}", type, msg);
        return false;
    }
    return true;
}

template<typename T>
std::optional<T> read_json(const QByteArray &json)
{
    T result;
    if (read_json<T>(json, result)) {
        return result;
    }
    return std::nullopt;
}

template<typename T>
QByteArray write_json(const T &obj)
{
    std::string json;
    const glz::error_ctx err = glz::write_json(obj, json);
    if (err) {
        const char *type = typeid(T).name();
        const std::string msg = glz::format_error(err);
        spdlog::error("Error writing {} to json: {}", type, msg);
        return {};
    }
    return QByteArray::fromStdString(json);
}
