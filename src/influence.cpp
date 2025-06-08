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

#include "influence.h"

#include <QIcon>
#include <QImage>
#include <QPainter>

QIcon combineInflunceIcons(const QIcon& leftIcon, const QIcon& rightIcon) {
    const int width = 54;
    const int height = 27;

    QPixmap layered(width, height);
    layered.fill(Qt::transparent);
    QPainter layered_painter(&layered);

    layered_painter.drawPixmap(0, 0, leftIcon.pixmap(27, 27));
    layered_painter.drawPixmap(27, 0, rightIcon.pixmap(27, 27));

    return QIcon(layered);
}
