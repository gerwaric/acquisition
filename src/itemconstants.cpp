// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

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
