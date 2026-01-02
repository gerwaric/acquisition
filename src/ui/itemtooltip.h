// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2015 Ilya Zhuravlev

#pragma once

#include <QImage>
#include <QPixmap>

namespace Ui {
    class MainWindow;
}

class Item;

void UpdateItemTooltip(const Item &item, Ui::MainWindow *ui);
QPixmap GenerateItemIcon(const Item &item, const QImage &image);
