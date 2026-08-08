// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/controlendpoint.h"

#include <QCryptographicHash>
#include <QFile>
#include <QFileInfo>
#include <QStandardPaths>

#ifndef Q_OS_WIN
#include <cerrno>
#include <pwd.h>
#include <sys/stat.h>
#include <sys/un.h>
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

    bool IsMissingOrPrivateDirectory(const QString &path)
    {
        const QByteArray encoded = QFile::encodeName(path);
        struct stat status {};
        if (::lstat(encoded.constData(), &status) == 0) {
            return S_ISDIR(status.st_mode) && status.st_uid == getuid()
                   && (status.st_mode & S_IWUSR) && (status.st_mode & S_IXUSR)
                   && !(status.st_mode & (S_IWGRP | S_IWOTH));
        }
        return errno == ENOENT;
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

    bool EndpointPathFits(const QString &root)
    {
        const QString longest_endpoint = QDir(root).filePath("socket-0000000000000000");
        return QFile::encodeName(longest_endpoint).size()
               < qsizetype(sizeof(sockaddr_un::sun_path));
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

    QString HomeControlRoot(const QString &home)
    {
        if (!IsPrivateUserDirectory(home)) {
            return {};
        }
        const QString legacy_root = QDir(home).filePath(".acquisition-control");
        if (EndpointPathFits(legacy_root) && IsMissingOrPrivateDirectory(legacy_root)) {
            return legacy_root;
        }
        const QString short_root = QDir(home).filePath(".acq-control");
        return EndpointPathFits(short_root) && IsMissingOrPrivateDirectory(short_root)
                   ? short_root
                   : QString{};
    }

#endif

} // namespace

#ifndef Q_OS_WIN
namespace detail {

QString SelectUnixControlRoot(const QString &runtime, const QString &home)
{
    const QString runtime_root = QDir(runtime).filePath("acquisition-control");
    if (IsPrivateUserDirectory(runtime) && EndpointPathFits(runtime_root)
        && IsMissingOrPrivateDirectory(runtime_root)) {
        return runtime_root;
    }
    return HomeControlRoot(home);
}

} // namespace detail
#endif

QDir DefaultDataDirectory()
{
    // writableLocation(GenericDataLocation) is the first user-specific entry
    // (LocalAppData on Windows), not the later system-wide search locations.
    const QDir generic(QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation));
    return QDir(generic.filePath("acquisition"));
}

QString CanonicalDataDirectory(const QDir &directory)
{
    QString candidate = QDir::cleanPath(QFileInfo(directory.absolutePath()).absoluteFilePath());
    QStringList missing_components;
    while (true) {
        const QFileInfo info(candidate);
        const QString canonical = info.canonicalFilePath();
        if (!canonical.isEmpty()) {
            QString resolved = canonical;
            for (const QString &component : missing_components) {
                resolved = QDir(resolved).filePath(component);
            }
            return QDir::cleanPath(resolved);
        }

        const QString parent = info.dir().absolutePath();
        if (parent == candidate) {
            return QDir::cleanPath(directory.absolutePath());
        }
        missing_components.prepend(info.fileName());
        candidate = parent;
    }
}

QStringList EndpointNames(const QDir &directory)
{
    const QByteArray identity = CanonicalDataDirectory(directory).toUtf8();
    const QByteArray digest = QCryptographicHash::hash(identity, QCryptographicHash::Sha256).toHex();
#ifdef Q_OS_WIN
    return {"acquisition-control-" + QString::fromLatin1(digest.first(24))};
#else
    QStringList roots;
    const QString runtime = SystemRuntimeRoot();
    const QString runtime_root = QDir(runtime).filePath("acquisition-control");
    if (IsPrivateUserDirectory(runtime) && EndpointPathFits(runtime_root)
        && IsMissingOrPrivateDirectory(runtime_root)) {
        roots.push_back(runtime_root);
    }
    const QString home_root = HomeControlRoot(PasswdHome());
    if (!home_root.isEmpty() && !roots.contains(home_root)) {
        roots.push_back(home_root);
    }

    QStringList endpoints;
    for (const QString &root : roots) {
        endpoints.push_back(
            QDir(root).filePath("socket-" + QString::fromLatin1(digest.first(16))));
    }
    return endpoints;
#endif
}

QString EndpointName(const QDir &directory)
{
    const QStringList endpoints = EndpointNames(directory);
    return endpoints.isEmpty() ? QString{} : endpoints.front();
}

QString EndpointLockPath(const QDir &directory)
{
#ifdef Q_OS_WIN
    return QDir(CanonicalDataDirectory(directory)).filePath(".acquisition-control/owner.lock");
#else
    const QStringList endpoints = EndpointNames(directory);
    return endpoints.isEmpty() ? QString{} : endpoints.back() + ".lock";
#endif
}

} // namespace control
