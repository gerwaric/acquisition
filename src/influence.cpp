// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2020 testpushpleaseignore <elnino2k10@gmail.com>

#include "influence.h"

#include <QIcon>
#include <QImage>
#include <QPainter>

QIcon combineInflunceIcons(const QIcon &leftIcon, const QIcon &rightIcon)
{
    const int width = 54;
    const int height = 27;

    QPixmap layered(width, height);
    layered.fill(Qt::transparent);
    QPainter layered_painter(&layered);

    layered_painter.drawPixmap(0, 0, leftIcon.pixmap(27, 27));
    layered_painter.drawPixmap(27, 0, rightIcon.pixmap(27, 27));

    return QIcon(layered);
}
