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

#include "currencymanager.h"

#include <QDateTime>
#include <QDoubleSpinBox>
#include <QFile>
#include <QFileDialog>
#include <QSettings>
#include <QVBoxLayout>

#include <rapidjson/document.h>
#include <rapidjson/error/en.h>

#include <datastore/datastore.h>
#include <util/spdlog_qt.h>
#include <util/util.h>

#include "buyoutmanager.h"
#include "itemsmanager.h"
#include "item.h"


CurrencyManager::CurrencyManager(
    QSettings& settings,
    DataStore& datastore,
    ItemsManager& items_manager)
    : QObject()
    , m_settings(settings)
    , m_data(datastore)
    , m_items_manager(items_manager)
{
    if (m_data.Get("currency_items", "").isEmpty()) {
        FirstInitCurrency();
        if (!m_data.Get("currency_base", "").isEmpty()) {
            MigrateCurrency();
            spdlog::warn("Found old currency values, migrated them to the new system");
        };
    } else {
        InitCurrency();
    };
    const bool show_chaos = m_settings.value("show_chaos", false).toBool();
    const bool show_exalt = m_settings.value("show_exalt", false).toBool();
    m_dialog = std::make_shared<CurrencyDialog>(*this, show_chaos, show_exalt);
}

CurrencyManager::~CurrencyManager() {
    Save();
}

void CurrencyManager::Save() {
    SaveCurrencyItems();
    SaveCurrencyValue();
    m_settings.setValue("show_chaos", m_dialog->ShowChaos());
    m_settings.setValue("show_exalt", m_dialog->ShowExalt());
}

void CurrencyManager::Update() {
    ClearCurrency();
    for (auto& item : m_items_manager.items()) {
        ParseSingleItem(*item);
    };
    SaveCurrencyValue();
    m_dialog->Update();
}

const double EPS = 1e-6;
double CurrencyManager::TotalExaltedValue() {
    double out = 0;
    for (const auto& currency : m_currencies) {
        if (fabs(currency->exalt.value1) > EPS) {
            out += currency->count / currency->exalt.value1;
        };
    };
    return out;
}

double CurrencyManager::TotalChaosValue() {
    double out = 0;
    for (const auto& currency : m_currencies) {
        if (fabs(currency->chaos.value1) > EPS) {
            out += currency->count / currency->chaos.value1;
        };
    };
    return out;
}

int CurrencyManager::TotalWisdomValue() {
    int out = 0;
    for (unsigned int i = 0; i < m_wisdoms.size(); i++) {
        out += m_wisdoms[i] * CurrencyWisdomValue[i];
    };
    return out;
}

void CurrencyManager::ClearCurrency() {
    for (auto& currency : m_currencies) {
        currency->count = 0;
    };
    for (auto& wisdom : m_wisdoms) {
        wisdom = 0;
    };
}
void CurrencyManager::InitCurrency() {
    for (auto type : Currency::Types()) {
        m_currencies.push_back(std::make_shared<CurrencyItem>(0, Currency(type), 1, 1));
    };
    Deserialize(m_data.Get("currency_items"), &m_currencies);
    for (unsigned int i = 0; i < CurrencyWisdomValue.size(); i++) {
        m_wisdoms.push_back(0);
    };
}

void CurrencyManager::FirstInitCurrency() {
    QString value = "";
    //Dummy items + dummy currency_last_value
    //TODO : can i get the size of the Currency enum ??
    for (auto type : Currency::Types()) {
        m_currencies.push_back(std::make_shared<CurrencyItem>(0, Currency(type), 1, 1));
        value += "0;";
    };
    value.chop(1); // Remove the last ";"
    m_data.Set("currency_items", Serialize(m_currencies));
    m_data.Set("currency_last_value", value);
    m_settings.setValue("show_chaos", true);
    m_settings.setValue("show_exalt", true);
}

void CurrencyManager::MigrateCurrency() {
    QStringList list = m_data.Get("currency_base").split(';');
    for (unsigned int i = 0; i < list.size(); i++) {
        // We can't use the toDouble function from QT, because it encodes a double with a ","
        // Might be related to localisation issue, but anyway it's safer this way
        m_currencies[i]->exalt.value1 = list[i].toDouble();
    };
    //Set to empty so we won't trigger it next time !
    m_data.Set("currency_base", "");
}

QString CurrencyManager::Serialize(const std::vector<std::shared_ptr<CurrencyItem>>& currencies) {
    rapidjson::Document doc;
    doc.SetObject();
    auto& alloc = doc.GetAllocator();
    for (auto& curr : currencies) {
        rapidjson::Value item(rapidjson::kObjectType);
        item.AddMember("count", curr->count, alloc);
        item.AddMember("chaos_ratio", curr->chaos.value1, alloc);
        item.AddMember("exalt_ratio", curr->exalt.value1, alloc);
        Util::RapidjsonAddString(&item, "currency", curr->currency.AsTag(), alloc);
        rapidjson::Value name(curr->name.toStdString().c_str(), alloc);
        doc.AddMember(name, item, alloc);
    };
    return Util::RapidjsonSerialize(doc);
}

void CurrencyManager::Deserialize(const QString& string_data, std::vector<std::shared_ptr<CurrencyItem> >* currencies) {
    //Maybe clear something would be good
    if (string_data.isEmpty()) {
        return;
    };
    rapidjson::Document doc;
    if (doc.Parse(string_data.toStdString().c_str()).HasParseError()) {
        spdlog::error("Error while parsing currency ratios.");
        spdlog::error(rapidjson::GetParseError_En(doc.GetParseError()));
        return;
    };
    if (!doc.IsObject()) {
        return;
    };
    for (auto itr = doc.MemberBegin(); itr != doc.MemberEnd(); ++itr) {
        auto& object = itr->value;
        Currency curr = Currency::FromTag(object["currency"].GetString());
        for (auto& item : *currencies) {
            if (item->currency == curr) {
                item = std::make_shared<CurrencyItem>(
                    object["count"].GetDouble(), curr,
                    object["chaos_ratio"].GetDouble(),
                    object["exalt_ratio"].GetDouble());
            };
        };
        //currencies->push_back(item);
    };
}

void CurrencyManager::SaveCurrencyItems() {
    m_data.Set("currency_items", Serialize(m_currencies));
}

void CurrencyManager::SaveCurrencyValue() {
    QString value = "";
    // Useless to save if every count is 0.
    bool empty = true;
    value = QString::number(TotalExaltedValue());
    for (auto& currency : m_currencies) {
        if (currency->name != "") {
            value += ";" + std::to_string(currency->count);
        };
        if (currency->count != 0) {
            empty = false;
        };
    };
    QString old_value = m_data.Get("currency_last_value", "");
    if (value != old_value && !empty) {
        CurrencyUpdate update = CurrencyUpdate();
        update.timestamp = QDateTime::currentSecsSinceEpoch();
        update.value = value;
        m_data.InsertCurrencyUpdate(update);
        m_data.Set("currency_last_value", value);
    };
}

void CurrencyManager::ExportCurrency() {
    QString header_csv = "Date,Total value";
    for (auto& item : m_currencies) {
        auto& label = item->currency.AsString();
        if (label != "") {
            header_csv += "," + label;
        };
    };
    std::vector<CurrencyUpdate> result = m_data.GetAllCurrency();

    QString fileName = QFileDialog::getSaveFileName(nullptr, tr("Save Export file"),
        QDir::toNativeSeparators(QDir::homePath() + "/" + "acquisition_export_currency.csv"));
    if (fileName.isEmpty()) {
        return;
    };
    QFile file(QDir::toNativeSeparators(fileName));
    if (!file.open(QFile::WriteOnly | QFile::Text)) {
        spdlog::warn("CurrencyManager::ExportCurrency : couldn't open CSV export file ");
        return;
    };
    QTextStream out(&file);
    out << header_csv << "\n";
    for (auto& update : result) {
        const QDateTime timestamp = QDateTime::fromSecsSinceEpoch(update.timestamp).toLocalTime();
        const QString value = update.value;
        out << timestamp.toString("yyyy-MM-dd hh:mm") << ",";
        out << value.split(";").join(",") << "\n";
    };
}

void CurrencyManager::ParseSingleItem(const Item& item) {
    for (unsigned int i = 0; i < m_currencies.size(); i++) {
        if (item.PrettyName() == m_currencies[i]->name) {
            m_currencies[i]->count += item.count();
        };
    };

    for (unsigned int i = 0; i < m_wisdoms.size(); i++) {
        if (item.PrettyName() == CurrencyForWisdom[i]) {
            m_wisdoms[i] += item.count();
        };
    };
}

void CurrencyManager::DisplayCurrency() {

    m_dialog->show();
}


CurrencyWidget::CurrencyWidget(std::shared_ptr<CurrencyItem> currency)
    : QWidget()
    , m_currency(currency)
{
    name = new QLabel("");
    count = new QLabel("");
    chaos_ratio = new QDoubleSpinBox;
    chaos_ratio->setMaximum(100000);
    chaos_ratio->setValue(currency->chaos.value1);
    chaos_value = new QDoubleSpinBox;
    chaos_value->setMaximum(100000);
    chaos_value->setEnabled(false);
    exalt_ratio = new QDoubleSpinBox;
    exalt_ratio->setMaximum(100000);
    exalt_ratio->setValue(currency->exalt.value1);
    exalt_value = new QDoubleSpinBox;
    exalt_value->setMaximum(100000);
    exalt_value->setEnabled(false);
    Update();
    connect(chaos_ratio, &QDoubleSpinBox::valueChanged, this, &CurrencyWidget::Update);
    connect(exalt_ratio, &QDoubleSpinBox::valueChanged, this, &CurrencyWidget::Update);
}

void CurrencyWidget::UpdateVisual(bool show_chaos, bool show_exalt) {

    chaos_value->setVisible(show_chaos);
    chaos_ratio->setVisible(show_chaos);
    exalt_value->setVisible(show_exalt);
    exalt_ratio->setVisible(show_exalt);
}

void CurrencyWidget::Update() {
    auto& chaos = m_currency->chaos;
    auto& exalt = m_currency->exalt;
    chaos.value1 = chaos_ratio->value();
    exalt.value1 = exalt_ratio->value();
    name->setText(m_currency->name);
    count->setText(QString::number(m_currency->count));
    if (fabs(chaos.value1) > EPS) {
        chaos_value->setValue(m_currency->count / chaos.value1);
    };
    if (fabs(exalt.value1) > EPS) {
        exalt_value->setValue(m_currency->count / exalt.value1);
    };
}

CurrencyDialog::CurrencyDialog(CurrencyManager& manager, bool show_chaos, bool show_exalt) : m_currency_manager(manager) {
    m_headers = new CurrencyLabels;
    for (auto& curr : m_currency_manager.currencies()) {
        CurrencyWidget* tmp = new CurrencyWidget(curr);
        // To keep every vector the same size, we DO create spinboxes for the empty currency, just don't display them
        if (curr->currency == Currency::CURRENCY_NONE) {
            continue;
        };
        connect(tmp->exalt_ratio, &QDoubleSpinBox::valueChanged, this, &CurrencyDialog::UpdateTotalValue);
        connect(tmp->chaos_ratio, &QDoubleSpinBox::valueChanged, this, &CurrencyDialog::UpdateTotalValue);
        m_currencies_widgets.push_back(tmp);
    };
    m_separator = new QFrame;
    m_separator->setFrameShape(QFrame::Shape::HLine);
    m_total_exalt_value = new QLabel("");
    m_show_exalt = new QCheckBox("show exalt ratio");
    m_show_exalt->setChecked(show_exalt);
    connect(m_show_exalt, &QCheckBox::checkStateChanged, this, &CurrencyDialog::UpdateVisual);

    m_total_chaos_value = new QLabel("");
    m_show_chaos = new QCheckBox("show chaos ratio");
    m_show_chaos->setChecked(show_chaos);
    connect(m_show_chaos, &QCheckBox::checkStateChanged, this, &CurrencyDialog::UpdateVisual);
    m_total_wisdom_value = new QLabel("");
    m_layout = new QVBoxLayout;
    Update();
    UpdateVisual();
#if defined(Q_OS_LINUX)
    setWindowFlags(Qt::WindowCloseButtonHint);
#endif
}

void CurrencyDialog::Update() {
    for (auto widget : m_currencies_widgets) {
        widget->Update();
    };
    UpdateTotalValue();
    UpdateTotalWisdomValue();
}

void CurrencyDialog::UpdateVisual() {
    //Destroy old layout (because you can't replace a layout, that would be too fun :/)
    delete m_layout;
    m_layout = GenerateLayout(ShowChaos(), ShowExalt());
    setLayout(m_layout);
    UpdateVisibility(ShowChaos(), ShowExalt());
    adjustSize();
}

void CurrencyDialog::UpdateVisibility(bool show_chaos, bool show_exalt) {
    m_headers->chaos_value->setVisible(show_chaos);
    m_headers->chaos_ratio->setVisible(show_chaos);
    m_headers->exalt_value->setVisible(show_exalt);
    m_headers->exalt_ratio->setVisible(show_exalt);

    for (auto& item : m_currencies_widgets) {
        item->UpdateVisual(show_chaos, show_exalt);
    };

    m_headers->chaos_total->setVisible(show_chaos);
    m_total_chaos_value->setVisible(show_chaos);
    m_headers->exalt_total->setVisible(show_exalt);
    m_total_exalt_value->setVisible(show_exalt);
}

QVBoxLayout* CurrencyDialog::GenerateLayout(bool show_chaos, bool show_exalt) {
    //Header
    QGridLayout* grid = new QGridLayout;
    grid->addWidget(m_headers->name, 0, 0);
    grid->addWidget(m_headers->count, 0, 1);
    int col = 2;

    //Qt is shit, if we don't hide widget that aren't in the grid, it still display them like shit
    //Also, if we don't show widget in the grid, if they weren't in it before, it doesn't display them
    if (show_chaos) {
        grid->addWidget(m_headers->chaos_value, 0, col);
        grid->addWidget(m_headers->chaos_ratio, 0, col + 1);
        col += 2;
    };
    if (show_exalt) {
        grid->addWidget(m_headers->exalt_value, 0, col);
        grid->addWidget(m_headers->exalt_ratio, 0, col + 1);
    };

    //Main part
    for (auto& item : m_currencies_widgets) {
        int curr_row = grid->rowCount() + 1;
        // To keep every vector the same size, we DO create spinboxes for the empty currency, just don't display them
        if (item->IsNone()) {
            continue;
        };
        grid->addWidget(item->name, curr_row, 0);
        grid->addWidget(item->count, curr_row, 1, 1, -1);
        col = 2;

        if (show_chaos) {
            grid->addWidget(item->chaos_value, curr_row, col);
            grid->addWidget(item->chaos_ratio, curr_row, col + 1);
            col += 2;
        };
        if (show_exalt) {
            grid->addWidget(item->exalt_value, curr_row, col);
            grid->addWidget(item->exalt_ratio, curr_row, col + 1);
        };
    };

    //Bottom header
    QGridLayout* bottom = new QGridLayout;
    //Used to display in the first column the checkbox if we don't display chaos/exalt
    col = 0;

    if (show_chaos) {
        bottom->addWidget(m_headers->chaos_total, 0, 0);
        bottom->addWidget(m_total_chaos_value, 0, 1);
        col = 2;
    };
    bottom->addWidget(m_show_chaos, 0, col);
    col = 0;
    if (show_exalt) {
        bottom->addWidget(m_headers->exalt_total, 1, 0);
        bottom->addWidget(m_total_exalt_value, 1, 1);
        col = 2;
    };
    bottom->addWidget(m_show_exalt, 1, col);
    bottom->addWidget(m_headers->wisdom_total, 2, 0);
    bottom->addWidget(m_total_wisdom_value, 2, 1);
    QVBoxLayout* layout = new QVBoxLayout;
    layout->addLayout(grid);
    layout->addWidget(m_separator);
    layout->addLayout(bottom);
    return layout;
}

void CurrencyDialog::UpdateTotalValue() {
    m_total_exalt_value->setText(QString::number(m_currency_manager.TotalExaltedValue()));
    m_total_chaos_value->setText(QString::number(m_currency_manager.TotalChaosValue()));
}

void CurrencyDialog::UpdateTotalWisdomValue() {
    m_total_wisdom_value->setText(QString::number(m_currency_manager.TotalWisdomValue()));
}
