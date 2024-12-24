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

#include <memory>

class BuyoutManager;
class DataStore;
class ItemsManager;

class TestItemsManager : public QObject {
    Q_OBJECT
public:
    TestItemsManager(
        DataStore& data,
        ItemsManager& items_manager,
        BuyoutManager& buyout_manager);
private slots:
    void cleanup();
    void BuyoutForNewItem();
    void BuyoutPropagation();
    void UserSetBuyoutPropagation();
    void MoveItemNoBoToBo();
    void MoveItemBoToNoBo();
    void MoveItemBoToBo();
    void ItemHashMigration();
private:
    DataStore& data_;
    ItemsManager& items_manager_;
    BuyoutManager& buyout_manager_;
};
