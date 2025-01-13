/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#include "item.h"

class TestItem : public QObject {
    Q_OBJECT
private slots:
    void testBasicParsing();

    void testDivCardCategory();
    void testBeltCategory();
    void testEssenceCategory();
    void testVaalGemCategory();
    void testSupportGemCategory();
    void testBowCategory();
    void testClawCategory();
    void testFragmentCategory();
    void testMapCategory();
    void testUniqueMapCategory();
    void testBreachstoneCategory();

    void testBeltPOB();
    void testBowPOB();
    void testClawPOB();

private:
    static Item parseItem(const char* json);
    static QString getCategory(const char* json);
    static QString getPOB(const char* json);
};
