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

} // namespace control
