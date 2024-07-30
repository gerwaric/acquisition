/*
    Copyright 2015 Ilya Zhuravlev

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

#include "filesystem.h"

#include <QStandardPaths>

#include "QsLog.h"

namespace Filesystem {

    QString user_dir;

    void Init() {
        QLOG_TRACE() << "Filesystem::Init() entered";
        SetUserDir(QStandardPaths::writableLocation(QStandardPaths::AppLocalDataLocation));
    }

    void SetUserDir(const QString& dir) {
        QLOG_TRACE() << "Filesystem::SetUserDir() entered";
        QLOG_TRACE() << "Filesystem::SetUserDir() dir =" << dir;
        user_dir = dir;
    }

    QString UserDir() {
        QLOG_TRACE() << "Filesystem::UserDir() entered";
        QLOG_TRACE() << "Filesystem::UserDir() user_dir =" << user_dir;
        return user_dir;
    }

}
