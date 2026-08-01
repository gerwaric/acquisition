// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/controlendpoint.h"

#include <QCryptographicHash>
#include <QFile>
#include <QFileInfo>
#include <QStandardPaths>

#ifndef Q_OS_WIN
#include <pwd.h>
#include <sys/stat.h>
#include <unistd.h>
#endif

namespace control {

namespace {

#ifndef Q_OS_WIN
    bool IsPrivateUserDirectory(const QString &path)
    {
        const QByteArray encoded = QFile::encodeName(path);
        struct stat status {};
        return ::lstat(encoded.constData(), &status) == 0 && S_ISDIR(status.st_mode)
               && status.st_uid == getuid() && (status.st_mode & S_IWUSR)
               && (status.st_mode & S_IXUSR) && !(status.st_mode & (S_IWGRP | S_IWOTH));
    }

    QString PasswdHome()
    {
        const long configured_size = sysconf(_SC_GETPW_R_SIZE_MAX);
        QByteArray buffer(int(configured_size > 0 ? configured_size : 16 * 1024), '\0');
        struct passwd entry {};
        struct passwd *result = nullptr;
        if (getpwuid_r(getuid(), &entry, buffer.data(), size_t(buffer.size()), &result) != 0
            || !result || !entry.pw_dir) {
            return {};
        }
        return QString::fromLocal8Bit(entry.pw_dir);
    }

    QString SystemRuntimeRoot()
    {
#ifdef Q_OS_MACOS
        const size_t size = confstr(_CS_DARWIN_USER_TEMP_DIR, nullptr, 0);
        if (size > 0) {
            QByteArray buffer(qsizetype(size), '\0');
            if (confstr(_CS_DARWIN_USER_TEMP_DIR, buffer.data(), size) > 0) {
                return QString::fromLocal8Bit(buffer.constData());
            }
        }
        return {};
#elif defined(Q_OS_LINUX)
        return QString("/run/user/%1").arg(qulonglong(getuid()));
#else
        return {};
#endif
    }

    QString UserControlRoot()
    {
        return detail::SelectUnixControlRoot(SystemRuntimeRoot(), PasswdHome());
    }
#endif

} // namespace

#ifndef Q_OS_WIN
namespace detail {

QString SelectUnixControlRoot(const QString &runtime, const QString &home)
{
    if (IsPrivateUserDirectory(runtime)) {
        return QDir(runtime).filePath("acquisition-control");
    }
    return IsPrivateUserDirectory(home) ? QDir(home).filePath(".acquisition-control")
                                        : QString{};
}

} // namespace detail
#endif

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
    const QByteArray identity = CanonicalDataDirectory(directory).toUtf8();
    const QByteArray digest = QCryptographicHash::hash(identity, QCryptographicHash::Sha256).toHex();
#ifdef Q_OS_WIN
    return "acquisition-control-" + QString::fromLatin1(digest.first(24));
#else
    const QString root = UserControlRoot();
    return root.isEmpty()
               ? QString{}
               : QDir(root).filePath("socket-" + QString::fromLatin1(digest.first(16)));
#endif
}

QString EndpointLockPath(const QDir &directory)
{
#ifdef Q_OS_WIN
    return QDir(CanonicalDataDirectory(directory)).filePath(".acquisition-control/owner.lock");
#else
    const QString endpoint_name = EndpointName(directory);
    if (endpoint_name.isEmpty()) {
        return {};
    }
    const QFileInfo endpoint(endpoint_name);
    return QDir(endpoint.absolutePath()).filePath(endpoint.fileName() + ".lock");
#endif
}

} // namespace control
