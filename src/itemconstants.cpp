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

#include "itemconstants.h"

const std::map<QString, position> &POS_MAP()
{
    static const std::map<QString, position> map = {{"MainInventory", {0, 7}},
                                                    {"BodyArmour", {5, 2}},
                                                    {"Weapon", {2, 0}},
                                                    {"Weapon2", {2, 0}},
                                                    {"Offhand", {8, 0}},
                                                    {"Offhand2", {8, 0}},
                                                    {"Boots", {7, 4}},
                                                    {"Ring", {4, 3}},
                                                    {"Ring2", {7, 3}},
                                                    {"Amulet", {7, 2}},
                                                    {"Gloves", {3, 4}},
                                                    {"Belt", {5, 5}},
                                                    {"Helm", {5, 0}},
                                                    {"Flask", {3.5, 6}}};
    return map;
};
