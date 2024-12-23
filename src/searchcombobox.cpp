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

#include "searchcombobox.h"

#include <QAbstractItemModel>
#include <QAbstractItemView>
#include <QLineEdit>

// Set the width of the popup view to the width of the completer options
void SearchComboCompleter::complete(const QRect& rect) {
    if (popup() == nullptr) {
        return;
    };
    const int rows = completionModel()->rowCount();
    const int width = (rows > 0) ? popup()->sizeHintForColumn(0) : 0;
    popup()->setMinimumWidth(width);
    QCompleter::complete(rect);
}

// Return a custom shortened dalay for the combox box hover tooltip
int SearchComboStyle::styleHint(StyleHint hint, const QStyleOption* option, const QWidget* widget, QStyleHintReturn* returnData) const {
    if (hint == QStyle::SH_ToolTip_WakeUpDelay) {
        return TOOLTIP_DELAY_MSEC;
    } else {
        return QProxyStyle::styleHint(hint, option, widget, returnData);
    };
}

SearchComboBox::SearchComboBox(QAbstractItemModel* model, QWidget* parent) :
    QComboBox(parent),
    completer_(model, this)
{
    setEditable(true);
    setModel(model);
    setCompleter(nullptr);
    setInsertPolicy(QComboBox::NoInsert);
    setStyle(new SearchComboStyle(style()));

    view()->setSizePolicy(QSizePolicy::Minimum, QSizePolicy::Fixed);

    completer_.setCompletionMode(QCompleter::PopupCompletion);
    completer_.setFilterMode(Qt::MatchContains);
    completer_.setCaseSensitivity(Qt::CaseInsensitive);
    completer_.setModelSorting(QCompleter::CaseInsensitivelySortedModel);
    completer_.setWidget(this);

    connect(this, &QComboBox::editTextChanged,
        this, &SearchComboBox::OnTextEdited);

    connect(&edit_timer_, &QTimer::timeout,
        this, &SearchComboBox::OnEditTimeout);

    connect(&completer_, QOverload<const QString&>::of(&QCompleter::activated),
        this, &SearchComboBox::OnCompleterActivated);
}

void SearchComboBox::OnTextEdited() {
    edit_timer_.start(350);
}

void SearchComboBox::OnEditTimeout() {
    edit_timer_.stop();
    const QString& text = lineEdit()->text();
    if (text.isEmpty()) {
        return;
    };
    completer_.setCompletionPrefix(text);
    if (completer_.setCurrentRow(1)) {
        // Trigger completion if there are 2 or more results.
        completer_.complete();
    } else if (completer_.setCurrentRow(0)) {
        // If there is only one compleition result, check to see
        // if it's just a partial completion.
        if (text != completer_.currentCompletion()) {
            completer_.complete();
        } else {
            setCurrentText(text);
        };
    };
}

void SearchComboBox::OnCompleterActivated(const QString& text) {
    setCurrentText(text);
    setToolTip(text);
}
