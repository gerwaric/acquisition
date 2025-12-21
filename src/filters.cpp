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

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "filters.h"

#include <QAbstractItemView>
#include <QCheckBox>
#include <QComboBox>
#include <QCompleter>
#include <QGroupBox>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QListView>

#include <memory>

#include <ui/mainwindow.h>
#include <ui/searchcombobox.h>
#include <util/util.h>

#include "buyoutmanager.h"
#include "itemconstants.h"

const QString CategorySearchFilter::k_Default = "<any>";
const QString RaritySearchFilter::k_Default = "<any>";
const QStringList RaritySearchFilter::RARITY_LIST{"<any>",
                                                  "Normal",
                                                  "Magic",
                                                  "Rare",
                                                  "Unique",
                                                  "Unique (Relic)"};

std::unique_ptr<FilterData> Filter::CreateData()
{
    return std::make_unique<FilterData>(this);
}

FilterData::FilterData(Filter *filter)
    : text_query("")
    , min(0.0)
    , max(0.0)
    , min_filled(false)
    , max_filled(false)
    , r(0)
    , g(0)
    , b(0)
    , r_filled(false)
    , g_filled(false)
    , b_filled(false)
    , checked(false)
    , m_filter(filter)
{}

bool FilterData::Matches(const std::shared_ptr<Item> &item)
{
    return m_filter->Matches(item, this);
}

void FilterData::FromForm()
{
    m_filter->FromForm(this);
}

void FilterData::ToForm()
{
    m_filter->ToForm(this);
}

NameSearchFilter::NameSearchFilter(QLayout *parent)
{
    Initialize(parent);
}

void NameSearchFilter::FromForm(FilterData *data)
{
    data->text_query = m_textbox->text().toUtf8().constData();
    m_active = !data->text_query.isEmpty();
}

void NameSearchFilter::ToForm(FilterData *data)
{
    m_textbox->setText(data->text_query);
}

void NameSearchFilter::ResetForm()
{
    m_textbox->setText("");
    m_active = false;
}

bool NameSearchFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    const QString query = data->text_query.toLower();
    const QString name = item->PrettyName().toLower();
    return name.contains(query);
}

void NameSearchFilter::Initialize(QLayout *parent)
{
    MainWindow *main_window = qobject_cast<MainWindow *>(parent->parentWidget()->window());
    QWidget *group = new QWidget;
    QHBoxLayout *layout = new QHBoxLayout;
    layout->setContentsMargins(0, 0, 0, 0);
    QLabel *label = new QLabel("Name");
    label->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_LABEL));
    label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    m_textbox = new QLineEdit;
    layout->addWidget(label);
    layout->addWidget(m_textbox);
    group->setLayout(layout);
    parent->addWidget(group);
    QObject::connect(m_textbox,
                     &QLineEdit::textEdited,
                     main_window,
                     &MainWindow::OnDelayedSearchFormChange);
}

CategorySearchFilter::CategorySearchFilter(QLayout *parent, QAbstractListModel *model)
    : m_model(model)
{
    Initialize(parent);
}

void CategorySearchFilter::FromForm(FilterData *data)
{
    QString current_text = m_combobox->currentText().toLower();
    data->text_query = (current_text == k_Default) ? "" : current_text;
    m_active = !data->text_query.isEmpty();
}

void CategorySearchFilter::ToForm(FilterData *data)
{
    auto index = m_combobox->findText(data->text_query, Qt::MatchFixedString);
    m_combobox->setCurrentIndex(std::max(0, index));
}

void CategorySearchFilter::ResetForm()
{
    m_combobox->setCurrentText(k_Default);
    m_active = false;
}

bool CategorySearchFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    return item->category().contains(data->text_query);
}

void CategorySearchFilter::Initialize(QLayout *parent)
{
    MainWindow *main_window = qobject_cast<MainWindow *>(parent->parentWidget()->window());
    QWidget *group = new QWidget;
    QHBoxLayout *layout = new QHBoxLayout;
    layout->setContentsMargins(0, 0, 0, 0);
    QLabel *label = new QLabel("Type");
    label->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_LABEL));
    label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    m_combobox = new SearchComboBox(m_model, "", group);
    layout->addWidget(label);
    layout->addWidget(m_combobox);
    group->setLayout(layout);
    parent->addWidget(group);
    QObject::connect(m_combobox,
                     &QComboBox::currentIndexChanged,
                     main_window,
                     &MainWindow::OnDelayedSearchFormChange);
}

RaritySearchFilter::RaritySearchFilter(QLayout *parent, QAbstractListModel *model)
    : m_model(model)
{
    Initialize(parent);
}

void RaritySearchFilter::FromForm(FilterData *data)
{
    QString current_text = m_combobox->currentText();
    data->text_query = (current_text == k_Default) ? "" : current_text;
    m_active = !data->text_query.isEmpty();
}

void RaritySearchFilter::ToForm(FilterData *data)
{
    auto index = m_combobox->findText(data->text_query, Qt::MatchFixedString);
    m_combobox->setCurrentIndex(std::max(0, index));
}

void RaritySearchFilter::ResetForm()
{
    m_combobox->setCurrentText(k_Default);
    m_active = false;
}

bool RaritySearchFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    if (data->text_query.isEmpty()) {
        return true;
    }
    switch (item->frameType()) {
    case FrameType::FRAME_TYPE_NORMAL:
        return (data->text_query == "Normal");
    case FrameType::FRAME_TYPE_MAGIC:
        return (data->text_query == "Magic");
    case FrameType::FRAME_TYPE_RARE:
        return (data->text_query == "Rare");
    case FrameType::FRAME_TYPE_UNIQUE:
        return (data->text_query == "Unique");
    case FrameType::FRAME_TYPE_RELIC:
        return (data->text_query == "Unique (Relic)");
    default:
        return false;
    }
}

void RaritySearchFilter::Initialize(QLayout *parent)
{
    MainWindow *main_window = qobject_cast<MainWindow *>(parent->parentWidget()->window());
    QWidget *group = new QWidget;
    QHBoxLayout *layout = new QHBoxLayout;
    layout->setContentsMargins(0, 0, 0, 0);
    QLabel *label = new QLabel("Rarity");
    label->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_LABEL));
    label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    m_combobox = new QComboBox;
    m_combobox->setModel(m_model);
    m_combobox->setEditable(false);
    m_combobox->setInsertPolicy(QComboBox::NoInsert);
    layout->addWidget(label);
    layout->addWidget(m_combobox);
    group->setLayout(layout);
    parent->addWidget(group);
    QObject::connect(m_combobox,
                     &QComboBox::currentIndexChanged,
                     main_window,
                     &MainWindow::OnDelayedSearchFormChange);
}

MinMaxFilter::MinMaxFilter(QLayout *parent, QString property)
    : m_property(property)
    , m_caption(property)
{
    Initialize(parent);
}

MinMaxFilter::MinMaxFilter(QLayout *parent, QString property, QString caption)
    : m_property(property)
    , m_caption(caption)
{
    Initialize(parent);
}

void MinMaxFilter::Initialize(QLayout *parent)
{
    MainWindow *main_window = qobject_cast<MainWindow *>(parent->parentWidget()->window());
    QWidget *group = new QWidget;
    QHBoxLayout *layout = new QHBoxLayout;
    layout->setContentsMargins(0, 0, 0, 0);
    QLabel *label = new QLabel(m_caption);
    label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    m_textbox_min = new QLineEdit;
    m_textbox_max = new QLineEdit;
    layout->addWidget(label);
    layout->addWidget(m_textbox_min);
    layout->addWidget(m_textbox_max);
    group->setLayout(layout);
    parent->addWidget(group);
    m_textbox_min->setPlaceholderText("min");
    m_textbox_max->setPlaceholderText("max");
    m_textbox_min->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_MIN_MAX));
    m_textbox_max->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_MIN_MAX));
    label->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_LABEL));
    QObject::connect(m_textbox_min,
                     &QLineEdit::textEdited,
                     main_window,
                     &MainWindow::OnDelayedSearchFormChange);
    QObject::connect(m_textbox_max,
                     &QLineEdit::textEdited,
                     main_window,
                     &MainWindow::OnDelayedSearchFormChange);
}

void MinMaxFilter::FromForm(FilterData *data)
{
    data->min_filled = m_textbox_min->text().size() > 0;
    data->min = m_textbox_min->text().toDouble();
    data->max_filled = m_textbox_max->text().size() > 0;
    data->max = m_textbox_max->text().toDouble();
    m_active = data->min_filled || data->max_filled;
}

void MinMaxFilter::ToForm(FilterData *data)
{
    if (data->min_filled) {
        m_textbox_min->setText(QString::number(data->min));
    } else {
        m_textbox_min->setText("");
    }
    if (data->max_filled) {
        m_textbox_max->setText(QString::number(data->max));
    } else {
        m_textbox_max->setText("");
    }
}

void MinMaxFilter::ResetForm()
{
    m_textbox_min->setText("");
    m_textbox_max->setText("");
    m_active = false;
}

bool MinMaxFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    if (IsValuePresent(item)) {
        double value = GetValue(item);
        if (data->min_filled && data->min > value) {
            return false;
        }
        if (data->max_filled && data->max < value) {
            return false;
        }
        return true;
    } else {
        return !data->max_filled && !data->min_filled;
    }
}

bool SimplePropertyFilter::IsValuePresent(const std::shared_ptr<Item> &item)
{
    return item->properties().count(m_property);
}

double SimplePropertyFilter::GetValue(const std::shared_ptr<Item> &item)
{
    return item->properties().at(m_property).toDouble();
}

double DefaultPropertyFilter::GetValue(const std::shared_ptr<Item> &item)
{
    if (!item->properties().count(m_property)) {
        return m_default_value;
    }
    return SimplePropertyFilter::GetValue(item);
}

double RequiredStatFilter::GetValue(const std::shared_ptr<Item> &item)
{
    auto &requirements = item->requirements();
    if (requirements.count(m_property)) {
        return requirements.at(m_property);
    }
    return 0;
}

ItemMethodFilter::ItemMethodFilter(QLayout *parent,
                                   std::function<double(Item *)> func,
                                   QString caption)
    : MinMaxFilter(parent, caption, caption)
    , m_func(func)
{}

double ItemMethodFilter::GetValue(const std::shared_ptr<Item> &item)
{
    return m_func(&*item);
}

double SocketsFilter::GetValue(const std::shared_ptr<Item> &item)
{
    return item->sockets_cnt();
}

double LinksFilter::GetValue(const std::shared_ptr<Item> &item)
{
    return item->links_cnt();
}

SocketsColorsFilter::SocketsColorsFilter(QLayout *parent)
{
    Initialize(parent, "Colors");
}

// TODO(xyz): ugh, a lot of copypasta below, perhaps this could be done
// in a nice way?
void SocketsColorsFilter::Initialize(QLayout *parent, const char *caption)
{
    MainWindow *main_window = qobject_cast<MainWindow *>(parent->parentWidget()->window());
    QWidget *group = new QWidget;
    QHBoxLayout *layout = new QHBoxLayout;
    layout->setContentsMargins(0, 0, 0, 0);
    QLabel *label = new QLabel(caption);
    label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    m_textbox_r = new QLineEdit;
    m_textbox_r->setPlaceholderText("R");
    m_textbox_g = new QLineEdit;
    m_textbox_g->setPlaceholderText("G");
    m_textbox_b = new QLineEdit;
    m_textbox_b->setPlaceholderText("B");
    layout->addWidget(label);
    layout->addWidget(m_textbox_r);
    layout->addWidget(m_textbox_g);
    layout->addWidget(m_textbox_b);
    group->setLayout(layout);
    parent->addWidget(group);
    m_textbox_r->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_RGB));
    m_textbox_g->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_RGB));
    m_textbox_b->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_RGB));
    label->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_LABEL));
    QObject::connect(m_textbox_r,
                     &QLineEdit::textEdited,
                     main_window,
                     &MainWindow::OnSearchFormChange);
    QObject::connect(m_textbox_g,
                     &QLineEdit::textEdited,
                     main_window,
                     &MainWindow::OnSearchFormChange);
    QObject::connect(m_textbox_b,
                     &QLineEdit::textEdited,
                     main_window,
                     &MainWindow::OnSearchFormChange);
}

void SocketsColorsFilter::FromForm(FilterData *data)
{
    data->r_filled = m_textbox_r->text().size() > 0;
    data->g_filled = m_textbox_g->text().size() > 0;
    data->b_filled = m_textbox_b->text().size() > 0;
    data->r = m_textbox_r->text().toInt();
    data->g = m_textbox_g->text().toInt();
    data->b = m_textbox_b->text().toInt();
    m_active = data->r_filled || data->g_filled || data->b_filled;
}

void SocketsColorsFilter::ToForm(FilterData *data)
{
    if (data->r_filled) {
        m_textbox_r->setText(QString::number(data->r));
    }
    if (data->g_filled) {
        m_textbox_g->setText(QString::number(data->g));
    }
    if (data->b_filled) {
        m_textbox_b->setText(QString::number(data->b));
    }
}

void SocketsColorsFilter::ResetForm()
{
    m_textbox_r->setText("");
    m_textbox_g->setText("");
    m_textbox_b->setText("");
    m_active = false;
}

bool SocketsColorsFilter::Check(
    int need_r, int need_g, int need_b, int got_r, int got_g, int got_b, int got_w)
{
    int diff = std::max(0, need_r - got_r) + std::max(0, need_g - got_g)
               + std::max(0, need_b - got_b);
    return diff <= got_w;
}

bool SocketsColorsFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    if (!data->r_filled && !data->g_filled && !data->b_filled) {
        return true;
    }
    const int need_r = data->r_filled ? data->r : 0;
    const int need_g = data->g_filled ? data->g : 0;
    const int need_b = data->b_filled ? data->b : 0;
    const ItemSocketGroup &sockets = item->sockets();
    return Check(need_r, need_g, need_b, sockets.r, sockets.g, sockets.b, sockets.w);
}

LinksColorsFilter::LinksColorsFilter(QLayout *parent)
{
    Initialize(parent, "Linked");
}

bool LinksColorsFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    if (!data->r_filled && !data->g_filled && !data->b_filled) {
        return true;
    }
    const int need_r = data->r_filled ? data->r : 0;
    const int need_g = data->g_filled ? data->g : 0;
    const int need_b = data->b_filled ? data->b : 0;
    for (auto &group : item->socket_groups()) {
        if (Check(need_r, need_g, need_b, group.r, group.g, group.b, group.w)) {
            return true;
        }
    }
    return false;
}

BooleanFilter::BooleanFilter(QLayout *parent, QString property, QString caption)
    : m_property(property)
    , m_caption(caption)
{
    Initialize(parent);
}

void BooleanFilter::Initialize(QLayout *parent)
{
    MainWindow *main_window = qobject_cast<MainWindow *>(parent->parentWidget()->window());
    QWidget *group = new QWidget;
    QHBoxLayout *layout = new QHBoxLayout;
    layout->setContentsMargins(0, 0, 0, 0);
    QLabel *label = new QLabel(m_caption);
    label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    m_checkbox = new QCheckBox;
    layout->addWidget(label);
    layout->addWidget(m_checkbox);
    group->setLayout(layout);
    parent->addWidget(group);
    label->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_BOOL_LABEL));

    QObject::connect(m_checkbox, &QCheckBox::clicked, main_window, &MainWindow::OnSearchFormChange);
}

void BooleanFilter::FromForm(FilterData *data)
{
    data->checked = m_checkbox->isChecked();
    m_active = data->checked;
}

void BooleanFilter::ToForm(FilterData *data)
{
    m_checkbox->setChecked(data->checked);
}

void BooleanFilter::ResetForm()
{
    m_checkbox->setChecked(false);
    m_active = false;
}

bool BooleanFilter::Matches(const std::shared_ptr<Item> & /* item */, FilterData * /* data */)
{
    return true;
}

bool AltartFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    static const QStringList altart = {
        // season 1
        "RedBeak2.png",
        "Wanderlust2.png",
        "Ring2b.png",
        "Goldrim2.png",
        "FaceBreaker2.png",
        "Atzirismirror2.png",
        // season 2
        "KaruiWardAlt.png",
        "ShiverstingAlt.png",
        "QuillRainAlt.png",
        "OnyxAmuletAlt.png",
        "DeathsharpAlt.png",
        "CarnageHeartAlt.png",
        "TabulaRasaAlt.png",
        "andvariusAlt.png",
        "AstramentisAlt.png",
        // season 3
        "BlackheartAlt.png",
        "SinTrekAlt.png",
        "ShavronnesPaceAlt.png",
        "Belt3Alt.png",
        "EyeofChayulaAlt.png",
        "SundanceAlt.png",
        "ReapersPursuitAlt.png",
        "WindscreamAlt.png",
        "RainbowStrideAlt.png",
        "TarynsShiverAlt.png",
        // season 4
        "BrightbeakAlt.png",
        "RubyRingAlt.png",
        "TheSearingTouchAlt.png",
        "CloakofFlameAlt.png",
        "AtzirisFoibleAlt.png",
        "DivinariusAlt.png",
        "HrimnorsResolveAlt.png",
        "CarcassJackAlt.png",
        "TheIgnomonAlt.png",
        "HeatShiverAlt.png",
        // season 5
        "KaomsSignAlt.png",
        "StormcloudAlt.png",
        "FairgravesTricorneAlt.png",
        "MoonstoneRingAlt.png",
        "GiftsfromAboveAlt.png",
        "LeHeupofAllAlt.png",
        "QueensDecreeAlt.png",
        "PerandusSignetAlt.png",
        "AuxiumAlt.png",
        "dGlsbGF0ZUFsdCI7czoy",
        // season 6
        "PerandusBlazonAlt.png",
        "AurumvoraxAlt.png",
        "GoldwyrmAlt.png",
        "AmethystAlt.png",
        "DeathRushAlt.png",
        "RingUnique1.png",
        "MeginordsGirdleAlt.png",
        "SidhebreathAlt.png",
        "MingsHeartAlt.png",
        "VoidBatteryAlt.png",
        // season 7
        "Empty-Socket2.png",
        "PrismaticEclipseAlt.png",
        "ThiefsTorment2.png",
        "Amulet5Unique2.png",
        "FurryheadofstarkonjaAlt.png",
        "Headhunter2.png",
        "Belt6Unique2.png",
        "BlackgleamAlt.png",
        "ThousandribbonsAlt.png",
        "IjtzOjI6InNwIjtkOjAu",
        // season 8
        "TheThreeDragonsAlt.png",
        "ImmortalFleshAlt.png",
        "DreamFragmentsAlt2.png",
        "BereksGripAlt.png",
        "SaffellsFrameAlt.png",
        "BereksRespiteAlt.png",
        "LifesprigAlt.png",
        "PillaroftheCagedGodAlt.png",
        "BereksPassAlt.png",
        "PrismaticRingAlt.png",
        // season 9
        "Fencoil.png",
        "TopazRing.png",
        "Cherufe2.png",
        "cy9CbG9ja0ZsYXNrMiI7",
        "BringerOfRain.png",
        "AgateAmuletUnique2.png",
        // season 10
        "StoneofLazhwarAlt.png",
        "SapphireRingAlt.png",
        "CybilsClawAlt.png",
        "DoedresDamningAlt.png",
        "AlphasHowlAlt.png",
        "dCI7czoyOiJzcCI7ZDow",
        // season 11
        "MalachaisArtificeAlt.png",
        "MokousEmbraceAlt.png",
        "RusticSashAlt2.png",
        "MaligarosVirtuosityAlt.png",
        "BinosKitchenKnifeAlt.png",
        "WarpedTimepieceAlt.png",
        // emberwake season
        "UngilsHarmonyAlt.png",
        "LightningColdTwoStoneRingAlt.png",
        "EdgeOfMadnessAlt.png",
        "RashkaldorsPatienceAlt.png",
        "RathpithGlobeAlt.png",
        "EmberwakeAlt.png",
        // bloodgrip season
        "GoreFrenzyAlt.png",
        "BloodGloves.png",
        "BloodAmuletALT.png",
        "TheBloodThornALT.png",
        "BloodJewel.png",
        "BloodRIng.png",
        // soulthirst season
        "ThePrincessAlt.png",
        "EclipseStaff.png",
        "Perandus.png",
        "SoultakerAlt.png",
        "SoulthirstALT.png",
        "bHQiO3M6Mjoic3AiO2Q6",
        // winterheart season
        "AsphyxiasWrathRaceAlt.png",
        "SapphireRingRaceAlt.png",
        "TheWhisperingIceRaceAlt.png",
        "DyadianDawnRaceAlt.png",
        "CallOfTheBrotherhoodRaceAlt.png",
        "WinterHeart.png",
    };

    if (!data->checked) {
        return true;
    }
    for (auto &needle : altart) {
        if (item->icon().contains(needle)) {
            return true;
        }
    }
    return false;
}

bool PricedFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    return !data->checked || m_bm.Get(*item).IsActive();
}
