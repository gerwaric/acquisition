/*
    Copyright 2014 Ilya Zhuravlev

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

#include "filters.h"

#include <QComboBox>
#include <QCompleter>
#include <QGridLayout>
#include <QLineEdit>
#include <QObject>
#include <QPushButton>
#include <QTimer>

#include <vector>

#include "searchcombobox.h"

class SelectedMod : public QObject {
    Q_OBJECT
public:
    SelectedMod(const std::string& name, double min, double max, bool min_selected, bool max_selected);
    void AddToLayout(QGridLayout* layout);
    void RemoveFromLayout(QGridLayout* layout);
    const ModFilterData& data() const { return data_; }
signals:
    void ModChanged(SelectedMod& mod);
    void ModDeleted(SelectedMod& mod);
private slots:
    void OnModChanged();
    void OnMinChanged();
    void OnMaxChanged();
    void OnModDeleted();
private:
    ModFilterData data_;
    SearchComboBox mod_select_;
    QLineEdit min_text_, max_text_;
    QPushButton delete_button_;
};

class ModsFilter;

class ModsFilterSignalHandler : public QObject {
    Q_OBJECT
public:
    ModsFilterSignalHandler(ModsFilter& parent) : parent_(parent) {}
signals:
    void SearchFormChanged();
public slots:
    void OnAddButtonClicked();
    void OnModChanged();
    void OnModDeleted(SelectedMod& mod);
private:
    ModsFilter& parent_;
};

class ModsFilter : public Filter {
    friend class ModsFilterSignalHandler;
public:
    enum LayoutColumn {
        kMinField,
        kMaxField,
        kDeleteButton,
        kColumnCount
    };
    explicit ModsFilter(QLayout* parent);
    void FromForm(FilterData* data);
    void ToForm(FilterData* data);
    void ResetForm();
    bool Matches(const std::shared_ptr<Item>& item, FilterData* data);
private:
    void AddNewMod();
    void UpdateMod();
    void DeleteMod(SelectedMod& mod);

    QGridLayout layout_;
    std::vector<std::unique_ptr<SelectedMod>> mods_;
    QPushButton add_button_;
    ModsFilterSignalHandler signal_handler_;
};
