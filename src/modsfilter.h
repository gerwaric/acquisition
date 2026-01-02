// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include "filters.h"

#include <QComboBox>
#include <QCompleter>
#include <QGridLayout>
#include <QLineEdit>
#include <QObject>
#include <QPushButton>
#include <QString>
#include <QTimer>

#include <vector>

#include "ui/searchcombobox.h"

class SelectedMod : public QObject
{
    Q_OBJECT
public:
    SelectedMod(const QString &name, double min, double max, bool min_selected, bool max_selected);
    void AddToLayout(QGridLayout *layout);
    void RemoveFromLayout(QGridLayout *layout);
    const ModFilterData &data() const { return m_data; }
signals:
    void ModChanged(SelectedMod &mod);
    void ModDeleted(SelectedMod &mod);
private slots:
    void OnModChanged();
    void OnMinChanged();
    void OnMaxChanged();
    void OnModDeleted();

private:
    ModFilterData m_data;
    //SearchComboBox m_mod_select;
    QComboBox m_mod_select;
    QLineEdit m_min_text;
    QLineEdit m_max_text;
    QPushButton m_delete_button;
    TokenAndFilterProxy m_proxy;
    QCompleter m_completer;
    QTimer m_timer;

    QString m_pending_text;
};

class ModsFilter;

class ModsFilterSignalHandler : public QObject
{
    Q_OBJECT
public:
    ModsFilterSignalHandler(ModsFilter &parent)
        : m_parent(parent)
    {}
signals:
    void SearchFormChanged();
public slots:
    void OnAddButtonClicked();
    void OnModChanged();
    void OnModDeleted(SelectedMod &mod);

private:
    ModsFilter &m_parent;
};

class ModsFilter : public Filter
{
    friend class ModsFilterSignalHandler;

public:
    enum LayoutColumn { kMinField, kMaxField, kDeleteButton, kColumnCount };
    explicit ModsFilter(QLayout *parent);
    void FromForm(FilterData *data);
    void ToForm(FilterData *data);
    void ResetForm();
    bool Matches(const std::shared_ptr<Item> &item, FilterData *data);

private:
    void AddNewMod();
    void UpdateMod();
    void DeleteMod(SelectedMod &mod);

    QGridLayout m_layout;
    std::vector<std::unique_ptr<SelectedMod>> m_mods;
    QPushButton m_add_button;
    ModsFilterSignalHandler m_signal_handler;
};
