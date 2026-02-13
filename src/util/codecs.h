// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QSettings>
#include <QVariant>

#include <semver/semver.hpp>
#include <spdlog/common.h>

template<typename T>
struct VariantCodec
{
    static T decode(const QVariant &v) { return v.template value<T>(); }

    static QVariant encode(const T &v) { return QVariant::fromValue(v); }
};

template<>
struct VariantCodec<semver::version>
{
    static semver::version decode(const QVariant &v)
    {
        return semver::version::parse(v.toByteArray().toStdString(), true);
    }

    static QVariant encode(const semver::version &version)
    {
        return QString::fromStdString(version.str());
    }
};

template<>
struct VariantCodec<spdlog::level::level_enum>
{
    static spdlog::level::level_enum decode(const QVariant &v)
    {
        return spdlog::level::from_str(v.toByteArray().toStdString());
    }

    static QVariant encode(const spdlog::level::level_enum &level)
    {
        return QString::fromStdString(spdlog::level::to_short_c_str(level));
    }
};
