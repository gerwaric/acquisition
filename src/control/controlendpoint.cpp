// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/controlendpoint.h"

#include <QCryptographicHash>
#include <QFileInfo>
#include <QStandardPaths>

namespace control {

QDir DefaultDataDirectory()
{
    const QDir generic(QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation));
    return QDir(generic.filePath("acquisition"));
}

QString CanonicalDataDirectory(const QDir &directory)
{
    const QFileInfo info(directory.absolutePath());
    const QString canonical = info.canonicalFilePath();
    return canonical.isEmpty() ? QDir::cleanPath(info.absoluteFilePath()) : canonical;
}

QString EndpointName(const QDir &directory)
{
    const QByteArray identity = QDir::homePath().toUtf8() + '\0'
                                + CanonicalDataDirectory(directory).toUtf8();
    const QByteArray digest = QCryptographicHash::hash(identity, QCryptographicHash::Sha256).toHex();
#ifdef Q_OS_WIN
    return "acquisition-control-" + QString::fromLatin1(digest.first(24));
#else
    const QString runtime = QStandardPaths::writableLocation(QStandardPaths::RuntimeLocation);
    if (runtime.isEmpty()) {
        return {};
    }
    return QDir(runtime).filePath("acqctl-" + QString::fromLatin1(digest.first(16)));
#endif
}

QString EndpointLockPath(const QDir &directory)
{
#ifdef Q_OS_WIN
    const QString cache = QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation);
    const QString endpoint = EndpointName(directory);
    if (cache.isEmpty() || endpoint.isEmpty()) {
        return {};
    }
    return QDir(cache).filePath("acquisition/" + QFileInfo(endpoint).fileName() + ".lock");
#else
    const QString runtime = QStandardPaths::writableLocation(QStandardPaths::RuntimeLocation);
    const QString endpoint = EndpointName(directory);
    if (runtime.isEmpty() || endpoint.isEmpty()) {
        return {};
    }
    return QDir(runtime).filePath(QFileInfo(endpoint).fileName() + ".lock");
#endif
}

} // namespace control
