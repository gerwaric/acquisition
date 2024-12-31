/*
    Copyright 2015 Guillaume DUPUY <glorf@glorf.fr>

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

#include <QCheckBox>
#include <QDialog>
#include <QLabel>
#include <QObject>
#include <QWidget>

#include "buyoutmanager.h"

class QDoubleSpinBox;
class QSettings;
class QVBoxLayout;

class CurrencyManager;
class DataStore;
class ItemsManager;

struct CurrencyRatio {
    Currency curr1;
    Currency curr2;
    double value1;
    double value2;
    CurrencyRatio() :
        curr1(CURRENCY_NONE),
        curr2(CURRENCY_NONE),
        value1(0),
        value2(0)
    {}
    CurrencyRatio(Currency c1, Currency c2, double v1, double v2) :
        curr1(c1),
        curr2(c2),
        value1(v1),
        value2(v2)
    {}
};

struct CurrencyItem {
    int count;
    Currency currency;
    std::string name;
    CurrencyRatio exalt;
    CurrencyRatio chaos;
    CurrencyItem(int co, Currency curr, double chaos_ratio, double exalt_ratio) {
        count = co;
        currency = curr;
        name = curr.AsString();
        chaos = CurrencyRatio(currency, CURRENCY_CHAOS_ORB, chaos_ratio, 1);
        exalt = CurrencyRatio(currency, CURRENCY_EXALTED_ORB, exalt_ratio, 1);
    }

};
struct CurrencyLabels {
    QLabel* name;
    QLabel* count;
    QLabel* chaos_ratio;
    QLabel* chaos_value;
    QLabel* exalt_ratio;
    QLabel* exalt_value;
    QLabel* exalt_total;
    QLabel* chaos_total;
    QLabel* wisdom_total;
    CurrencyLabels() {
        name = new QLabel("Name");
        count = new QLabel("Count");
        chaos_ratio = new QLabel("Amount a chaos Orb can buy");
        chaos_value = new QLabel("Value in Chaos Orb");
        exalt_ratio = new QLabel("Amount an Exalted Orb can buy");
        exalt_value = new QLabel("Value in Exalted Orb");
        exalt_total = new QLabel("Total Exalted Orbs");
        chaos_total = new QLabel("Total Chaos Orbs");
        wisdom_total = new QLabel("Total Scrolls of Wisdom");
    }

};
class CurrencyDialog;
class CurrencyWidget : public QWidget
{
    Q_OBJECT
public slots:
    void Update();
    void UpdateVisual(bool show_chaos, bool show_exalt);
public:
    CurrencyWidget(std::shared_ptr<CurrencyItem> currency);
    bool IsNone() const { return m_currency->currency.type == CURRENCY_NONE; }
    //Visual stuff
    QLabel* name;
    QLabel* count;
    QDoubleSpinBox* chaos_ratio;
    QDoubleSpinBox* chaos_value;
    QDoubleSpinBox* exalt_ratio;
    QDoubleSpinBox* exalt_value;

private:
    //Data
    std::shared_ptr<CurrencyItem> m_currency;
};

// For now we just serialize/deserialize 'value' inside CurrencyManager
// Later we might need more logic if GGG adds more currency types and we want to be backwards compatible
struct CurrencyUpdate {
    long long timestamp{ 0 };
    std::string value;
};

constexpr std::array<const char*, 5> CurrencyForWisdom({ {
    "Scroll of Wisdom",
    "Portal Scroll",
    "Armourer's Scrap",
    "Blacksmith's Whetstone",
    "Orb of Transmutation"
    } });

constexpr std::array<int, 5> CurrencyWisdomValue({ {
    1,
    1,
    2,
    4,
    4
    } });

class CurrencyDialog : public QDialog
{
    Q_OBJECT
public:
    CurrencyDialog(CurrencyManager& manager, bool show_chaos, bool show_exalt);
    bool ShowChaos() const { return m_show_chaos->isChecked(); }
    bool ShowExalt() const { return m_show_exalt->isChecked(); }

public slots:
    void Update();
    void UpdateVisual();
    void UpdateVisibility(bool show_chaos, bool show_exalt);
    void UpdateTotalValue();
private:
    CurrencyManager& m_currency_manager;
    std::vector<CurrencyWidget*> m_currencies_widgets;
    CurrencyLabels* m_headers;
    QVBoxLayout* m_layout;
    QLabel* m_total_exalt_value;
    QLabel* m_total_chaos_value;
    QLabel* m_total_wisdom_value;
    QCheckBox* m_show_chaos;
    QCheckBox* m_show_exalt;
    QFrame* m_separator;
    QVBoxLayout* GenerateLayout(bool show_chaos, bool show_exalt);
    void UpdateTotalWisdomValue();
};


class CurrencyManager : public QObject
{
    Q_OBJECT
public:
    explicit CurrencyManager(
        QSettings& settings,
        DataStore& datastore,
        ItemsManager& items_manager);
    ~CurrencyManager();
    void ClearCurrency();
    // Called in itemmanagerworker::ParseItem
    void ParseSingleItem(const Item& item);
    //void UpdateBaseValue(int ind, double value);
    const std::vector<std::shared_ptr<CurrencyItem>>& currencies() const { return m_currencies; }
    double TotalExaltedValue();
    double TotalChaosValue();
    int TotalWisdomValue();
    void Update();

public slots:
    void DisplayCurrency();
    void ExportCurrency();
    void SaveCurrencyValue();

private:
    QSettings& m_settings;
    DataStore& m_data;
    ItemsManager& m_items_manager;

    std::vector<std::shared_ptr<CurrencyItem>> m_currencies;
    // We only need the "count" of a CurrencyItem so int will be enough
    std::vector<int> m_wisdoms;
    std::shared_ptr<CurrencyDialog> m_dialog;
    // Used only the first time we launch the app
    void FirstInitCurrency();
    //Migrate from old storage (csv-like serializing) to new one (using json)
    void MigrateCurrency();
    void InitCurrency();
    void SaveCurrencyItems();
    std::string Serialize(const std::vector<std::shared_ptr<CurrencyItem>>& currencies);
    void Deserialize(const std::string& data, std::vector<std::shared_ptr<CurrencyItem>>* currencies);
    void Save();
};
