// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#pragma once

#include <QDir>
#include <QString>

namespace control {

QDir DefaultDataDirectory();
QString CanonicalDataDirectory(const QDir &directory);
QString EndpointName(const QDir &directory);
QString EndpointLockPath(const QDir &directory);

#ifndef Q_OS_WIN
namespace detail {
    QString SelectUnixControlRoot(const QString &runtime, const QString &home);
}
#endif

} // namespace control
