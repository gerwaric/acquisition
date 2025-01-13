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

#include <QCompleter>
#include <QComboBox>
#include <QProxyStyle>
#include <QTimer>

class QAbstractItemModel;

class SearchComboCompleter : public QCompleter {
    Q_OBJECT
public:
    using QCompleter::QCompleter;
public slots:
    void complete(const QRect& rect = QRect());
};

class SearchComboStyle : public QProxyStyle {
public:
    using QProxyStyle::QProxyStyle;
    int styleHint(StyleHint hint, const QStyleOption* option = nullptr, const QWidget* widget = nullptr, QStyleHintReturn* returnData = nullptr) const override;
private:
    const int TOOLTIP_DELAY_MSEC = 50;
};

class SearchComboBox : public QComboBox {
    Q_OBJECT
public:
    SearchComboBox(QAbstractItemModel* model, QWidget* parent = nullptr);
private slots:
    void OnTextEdited();
    void OnEditTimeout();
    void OnCompleterActivated(const QString& text);
private:
    SearchComboCompleter m_completer;
    QTimer m_edit_timer;
};
