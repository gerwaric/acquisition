#pragma once

#include <QObject>
#include <QString>

bool initializeCrashpad(
    const QString& dataDir,
    const QString& dbName,
    const QString& appName,
    const QString& appVersion);
