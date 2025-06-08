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

#include <QObject>

class QNetworkReply;

// This is the object returned to the end-user of the rate limiter.
// When the underlying network request is finished, the complete
// signal will be issued so that the caller can use a slot to
// process the reply.
class RateLimitedReply : public QObject {
    Q_OBJECT
signals:
    void complete(QNetworkReply* reply);
};
