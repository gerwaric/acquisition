// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Tom Holz

#include "searchcombobox.h"

#include <QAbstractItemModel>
#include <QAbstractItemView>
#include <QLineEdit>

const QRegularExpression TokenAndFilterProxy::s_sep = QRegularExpression("\\s+");

// Set the width of the popup view to the width of the completer options
void SearchComboCompleter::complete(const QRect &rect)
{
    if (popup() == nullptr) {
        return;
    };
    const int rows = completionModel()->rowCount();
    const int width = (rows > 0) ? popup()->sizeHintForColumn(0) : 0;
    popup()->setMinimumWidth(width);
    QCompleter::complete(rect);
}

// Return a custom shortened dalay for the combox box hover tooltip
int SearchComboStyle::styleHint(StyleHint hint,
                                const QStyleOption *option,
                                const QWidget *widget,
                                QStyleHintReturn *returnData) const
{
    if (hint == QStyle::SH_ToolTip_WakeUpDelay) {
        return TOOLTIP_DELAY_MSEC;
    } else {
        return QProxyStyle::styleHint(hint, option, widget, returnData);
    }
}

SearchComboBox::SearchComboBox(QAbstractItemModel *model, const QString &value, QWidget *parent)
    : QComboBox(parent)
    , m_completer(model, this)
{
    setEditable(true);
    setModel(model);
    if (!value.isEmpty()) {
        setCurrentText(value);
    }
    setCompleter(nullptr);
    setInsertPolicy(QComboBox::NoInsert);
    setStyle(new SearchComboStyle(style()));

    view()->setSizePolicy(QSizePolicy::Minimum, QSizePolicy::Fixed);

    m_completer.setCompletionMode(QCompleter::PopupCompletion);
    m_completer.setFilterMode(Qt::MatchContains);
    m_completer.setCaseSensitivity(Qt::CaseInsensitive);
    m_completer.setModelSorting(QCompleter::CaseInsensitivelySortedModel);
    m_completer.setWidget(this);

    connect(this, &QComboBox::editTextChanged, this, &SearchComboBox::OnTextEdited);

    connect(&m_edit_timer, &QTimer::timeout, this, &SearchComboBox::OnEditTimeout);

    connect(&m_completer,
            QOverload<const QString &>::of(&QCompleter::activated),
            this,
            &SearchComboBox::OnCompleterActivated);
}

void SearchComboBox::OnTextEdited()
{
    m_edit_timer.start(350);
}

void SearchComboBox::OnEditTimeout()
{
    m_edit_timer.stop();
    const QString &text = lineEdit()->text();
    if (text.isEmpty()) {
        return;
    }
    if (m_skip_completer) {
        setCurrentText(text);
        m_skip_completer = false;
        return;
    }
    m_completer.setCompletionPrefix(text);
    if (m_completer.setCurrentRow(1)) {
        // Trigger completion if there are 2 or more results.
        m_completer.complete();
    } else if (m_completer.setCurrentRow(0)) {
        // If there is only one compleition result, check to see
        // if it's just a partial completion.
        if (text != m_completer.currentCompletion()) {
            m_completer.complete();
        } else {
            setCurrentText(text);
        }
    }
}

void SearchComboBox::OnCompleterActivated(const QString &text)
{
    m_skip_completer = true;
    setCurrentText(text);
    setToolTip(text);
}
