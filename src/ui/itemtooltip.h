// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2015 Ilya Zhuravlev

#pragma once

#include "ui_mainwindow.h"

#include "item.h"

void UpdateItemTooltip(const Item &item, Ui::MainWindow *ui);
QPixmap GenerateItemIcon(const Item &item, const QImage &image);
