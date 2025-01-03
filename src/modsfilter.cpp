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

    You should have received a copy of the GNU General Public Licensex
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "modsfilter.h"

#include <QApplication>
#include <QComboBox>
#include <QLineEdit>
#include <QListView>
#include <QObject>
#include <QPushButton>

#include "ui/mainwindow.h"

#include "modlist.h"

SelectedMod::SelectedMod(const std::string& name, double min, double max, bool min_filled, bool max_filled)
    : data_(name, min, max, min_filled, max_filled)
    , mod_select_(&mod_list_model())
    , delete_button_("X")
{
    mod_select_.setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);

    if (min_filled) {
        min_text_.setText(QString::number(min));
    };
    if (max_filled) {
        max_text_.setText(QString::number(max));
    };

    // Connect signals for the mod fields.
    QObject::connect(&mod_select_, &QComboBox::currentIndexChanged, this, &SelectedMod::OnModChanged);
    connect(&min_text_, &QLineEdit::textEdited, this, &SelectedMod::OnMinChanged);
    connect(&max_text_, &QLineEdit::textEdited, this, &SelectedMod::OnMaxChanged);
    connect(&delete_button_, &QPushButton::clicked, this, &SelectedMod::OnModDeleted);
}

void SelectedMod::OnModChanged() {
    data_.mod = mod_select_.currentText().toStdString();
    emit ModChanged(*this);
}

void SelectedMod::OnMinChanged() {
    data_.min = min_text_.text().toDouble();
    data_.min_filled = !min_text_.text().isEmpty();
    emit ModChanged(*this);
}

void SelectedMod::OnMaxChanged() {
    data_.max = max_text_.text().toDouble();
    data_.max_filled = !max_text_.text().isEmpty();
    emit ModChanged(*this);
}

void SelectedMod::OnModDeleted() {
    emit ModDeleted(*this);
}

void SelectedMod::AddToLayout(QGridLayout* layout) {
    const int row = layout->rowCount();
    layout->addWidget(&mod_select_, row, 0, 1, ModsFilter::LayoutColumn::kColumnCount);
    layout->addWidget(&min_text_, row + 1, ModsFilter::LayoutColumn::kMinField);
    layout->addWidget(&max_text_, row + 1, ModsFilter::LayoutColumn::kMaxField);
    layout->addWidget(&delete_button_, row + 1, ModsFilter::LayoutColumn::kDeleteButton);
}

void SelectedMod::RemoveFromLayout(QGridLayout* layout) {
    layout->removeWidget(&mod_select_);
    layout->removeWidget(&min_text_);
    layout->removeWidget(&max_text_);
    layout->removeWidget(&delete_button_);
}

ModsFilter::ModsFilter(QLayout* parent) :
    add_button_("Add mod"),
    signal_handler_(*this)
{
    // Create a widget to hold all of the search mods.
    QWidget* widget = new QWidget;
    widget->setContentsMargins(0, 0, 0, 0);
    widget->setLayout(&layout_);
    widget->hide();
    parent->addWidget(widget);

    // Setup the 'Add mod' button.
    parent->addWidget(&add_button_);
    QObject::connect(&add_button_, &QPushButton::clicked, &signal_handler_, &ModsFilterSignalHandler::OnAddButtonClicked);

    // Make sure the main window knows when the search form has changed.
    MainWindow* main_window = qobject_cast<MainWindow*>(parent->parentWidget()->window());
    QObject::connect(
        &signal_handler_, &ModsFilterSignalHandler::SearchFormChanged,
        main_window, &MainWindow::OnDelayedSearchFormChange);
}

void ModsFilter::FromForm(FilterData* data) {
    data->mod_data.clear();
    for (auto& mod : mods_) {
        data->mod_data.push_back(mod->data());
    };
    active_ = !mods_.empty();
}

void ModsFilter::ToForm(FilterData* data) {
    ResetForm();

    // Add search mods from the filter data.
    for (auto& mod : data->mod_data) {
        mods_.push_back(std::make_unique<SelectedMod>(mod.mod, mod.min, mod.max, mod.min_filled, mod.max_filled));
        mods_.back()->AddToLayout(&layout_);
    };
}

void ModsFilter::ResetForm() {
    while (auto item = layout_.takeAt(0)) {};
    mods_.clear();
    active_ = false;
}

bool ModsFilter::Matches(const std::shared_ptr<Item>& item, FilterData* data) {
    for (auto& mod : data->mod_data) {
        if (mod.mod.empty()) {
            continue;
        };
        const ModTable& mod_table = item->mod_table();
        if (!mod_table.count(mod.mod)) {
            return false;
        };
        double value = mod_table.at(mod.mod);
        if (mod.min_filled && value < mod.min) {
            return false;
        };
        if (mod.max_filled && value > mod.max) {
            return false;
        };
    }
    return true;
}

void ModsFilter::AddNewMod() {

    // Create the mod, connect signals, and add it to the UI.
    auto mod = std::make_unique<SelectedMod>("", 0, 0, false, false);
    QObject::connect(mod.get(), &SelectedMod::ModChanged, &signal_handler_, &ModsFilterSignalHandler::OnModChanged);
    QObject::connect(mod.get(), &SelectedMod::ModDeleted, &signal_handler_, &ModsFilterSignalHandler::OnModDeleted);
    mod->AddToLayout(&layout_);
    mods_.push_back(std::move(mod));

    // The parent might be hidden if there were no mod searches.
    if (layout_.parentWidget()->isHidden()) {
        layout_.parentWidget()->show();
    };

    active_ = true;
}

void ModsFilter::DeleteMod(SelectedMod& mod) {
    mod.RemoveFromLayout(&layout_);
    for (auto it = mods_.begin(); it < mods_.end(); ++it) {
        if (&mod == it->get()) {
            mods_.erase(it);
            // Hide the entire layout if there are no mod searches.
            if (mods_.empty()) {
                layout_.parentWidget()->hide();
                active_ = false;
            };
            return;
        };
    };
}

void ModsFilterSignalHandler::OnAddButtonClicked() {
    parent_.AddNewMod();
}

void ModsFilterSignalHandler::OnModChanged() {
    emit SearchFormChanged();
}

void ModsFilterSignalHandler::OnModDeleted(SelectedMod& mod) {
    parent_.DeleteMod(mod);
    emit SearchFormChanged();
}
