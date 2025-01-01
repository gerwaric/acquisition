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

#include <QByteArray>
#include <QString>

#include <memory>
#include <unordered_map>
#include <vector>
#include <QStringList>
#include <QStringListModel>

#include <rapidjson/document.h>

class Item;
typedef std::unordered_map<QString, double> ModTable;


// This generates regular expressions for mods and does other setup, should be called when the app starts, perhaps in main()
// Maybe this is not needed and constexpr could do the trick, but VS doesn't support it right now.
void InitModList();

QStringListModel& mod_list_model();

class ModGenerator {
public:
    virtual ~ModGenerator() {};
    void Generate(const rapidjson::Value& json, ModTable* output);
    virtual void Generate(const QString& json, ModTable* output) = 0;
};

class SumModGenerator : public ModGenerator {
public:
    SumModGenerator(const QString& name, const std::vector<QString>& sum);
    virtual ~SumModGenerator() {};
    virtual void Generate(const QString& json, ModTable* output);
private:
    bool Match(const char* mod, double* output);

    QString m_name;
    std::vector<QString> m_matches;
};

typedef std::shared_ptr<SumModGenerator> SumModGen;

void InitStatTranslations();
void AddStatTranslations(const QByteArray& statTranslations);
void AddModToTable(const QString& mod, ModTable* output);
