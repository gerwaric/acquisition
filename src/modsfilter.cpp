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

#include <ui/mainwindow.h>

#include "modlist.h"

SelectedMod::SelectedMod(
    const QString &name, double min, double max, bool min_filled, bool max_filled)
    : m_data(name, min, max, min_filled, max_filled)
    , m_mod_select(&mod_list_model(), name)
    , m_delete_button("X")
{
    m_mod_select.setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);

    if (min_filled) {
        m_min_text.setText(QString::number(min));
    }
    if (max_filled) {
        m_max_text.setText(QString::number(max));
    }

    // Connect signals for the mod fields.
    QObject::connect(&m_mod_select,
                     &QComboBox::currentIndexChanged,
                     this,
                     &SelectedMod::OnModChanged);
    connect(&m_min_text, &QLineEdit::textEdited, this, &SelectedMod::OnMinChanged);
    connect(&m_max_text, &QLineEdit::textEdited, this, &SelectedMod::OnMaxChanged);
    connect(&m_delete_button, &QPushButton::clicked, this, &SelectedMod::OnModDeleted);
}

void SelectedMod::OnModChanged()
{
    m_data.mod = m_mod_select.currentText();
    emit ModChanged(*this);
}

void SelectedMod::OnMinChanged()
{
    m_data.min = m_min_text.text().toDouble();
    m_data.min_filled = !m_min_text.text().isEmpty();
    emit ModChanged(*this);
}

void SelectedMod::OnMaxChanged()
{
    m_data.max = m_max_text.text().toDouble();
    m_data.max_filled = !m_max_text.text().isEmpty();
    emit ModChanged(*this);
}

void SelectedMod::OnModDeleted()
{
    emit ModDeleted(*this);
}

void SelectedMod::AddToLayout(QGridLayout *layout)
{
    const int row = layout->rowCount();
    layout->addWidget(&m_mod_select, row, 0, 1, ModsFilter::LayoutColumn::kColumnCount);
    layout->addWidget(&m_min_text, row + 1, ModsFilter::LayoutColumn::kMinField);
    layout->addWidget(&m_max_text, row + 1, ModsFilter::LayoutColumn::kMaxField);
    layout->addWidget(&m_delete_button, row + 1, ModsFilter::LayoutColumn::kDeleteButton);
}

void SelectedMod::RemoveFromLayout(QGridLayout *layout)
{
    layout->removeWidget(&m_mod_select);
    layout->removeWidget(&m_min_text);
    layout->removeWidget(&m_max_text);
    layout->removeWidget(&m_delete_button);
}

ModsFilter::ModsFilter(QLayout *parent)
    : m_add_button("Add mod")
    , m_signal_handler(*this)
{
    // Create a widget to hold all of the search mods.
    QWidget *widget = new QWidget;
    widget->setContentsMargins(0, 0, 0, 0);
    widget->setLayout(&m_layout);
    widget->hide();
    parent->addWidget(widget);

    // Setup the 'Add mod' button.
    parent->addWidget(&m_add_button);
    QObject::connect(&m_add_button,
                     &QPushButton::clicked,
                     &m_signal_handler,
                     &ModsFilterSignalHandler::OnAddButtonClicked);

    // Make sure the main window knows when the search form has changed.
    MainWindow *main_window = qobject_cast<MainWindow *>(parent->parentWidget()->window());
    QObject::connect(&m_signal_handler,
                     &ModsFilterSignalHandler::SearchFormChanged,
                     main_window,
                     &MainWindow::OnDelayedSearchFormChange);
}

void ModsFilter::FromForm(FilterData *data)
{
    data->mod_data.clear();
    for (auto &mod : m_mods) {
        data->mod_data.push_back(ModFilterData(mod->data()));
    }
    m_active = !m_mods.empty();
}

void ModsFilter::ToForm(FilterData *data)
{
    ResetForm();

    // Add search mods from the filter data.
    for (auto &mod_data : data->mod_data) {
        auto mod = std::make_unique<SelectedMod>(mod_data.mod,
                                                 mod_data.min,
                                                 mod_data.max,
                                                 mod_data.min_filled,
                                                 mod_data.max_filled);
        QObject::connect(mod.get(),
                         &SelectedMod::ModChanged,
                         &m_signal_handler,
                         &ModsFilterSignalHandler::OnModChanged);
        QObject::connect(mod.get(),
                         &SelectedMod::ModDeleted,
                         &m_signal_handler,
                         &ModsFilterSignalHandler::OnModDeleted);

        m_mods.push_back(std::move(mod));
        m_mods.back()->AddToLayout(&m_layout);
    }
}

void ModsFilter::ResetForm()
{
    while (auto item = m_layout.takeAt(0)) {
    }
    m_mods.clear();
    m_active = false;
}

bool ModsFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    for (auto &mod : data->mod_data) {
        if (mod.mod.isEmpty()) {
            continue;
        }
        const ModTable &mod_table = item->mod_table();
        if (!mod_table.count(mod.mod)) {
            return false;
        }
        double value = mod_table.at(mod.mod);
        if (mod.min_filled && value < mod.min) {
            return false;
        }
        if (mod.max_filled && value > mod.max) {
            return false;
        }
    }
    return true;
}

void ModsFilter::AddNewMod()
{
    // Create the mod, connect signals, and add it to the UI.
    auto mod = std::make_unique<SelectedMod>("", 0, 0, false, false);
    QObject::connect(mod.get(),
                     &SelectedMod::ModChanged,
                     &m_signal_handler,
                     &ModsFilterSignalHandler::OnModChanged);
    QObject::connect(mod.get(),
                     &SelectedMod::ModDeleted,
                     &m_signal_handler,
                     &ModsFilterSignalHandler::OnModDeleted);
    mod->AddToLayout(&m_layout);
    m_mods.push_back(std::move(mod));

    // The parent might be hidden if there were no mod searches.
    if (m_layout.parentWidget()->isHidden()) {
        m_layout.parentWidget()->show();
    }

    m_active = true;
}

void ModsFilter::DeleteMod(SelectedMod &mod)
{
    mod.RemoveFromLayout(&m_layout);
    for (auto it = m_mods.begin(); it < m_mods.end(); ++it) {
        if (&mod == it->get()) {
            m_mods.erase(it);
            // Hide the entire layout if there are no mod searches.
            if (m_mods.empty()) {
                m_layout.parentWidget()->hide();
                m_active = false;
            }
            return;
        }
    }
}

void ModsFilterSignalHandler::OnAddButtonClicked()
{
    m_parent.AddNewMod();
}

void ModsFilterSignalHandler::OnModChanged()
{
    emit SearchFormChanged();
}

void ModsFilterSignalHandler::OnModDeleted(SelectedMod &mod)
{
    m_parent.DeleteMod(mod);
    emit SearchFormChanged();
}
