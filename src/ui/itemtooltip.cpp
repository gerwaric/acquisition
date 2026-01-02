// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2015 Ilya Zhuravlev

#include "ui/itemtooltip.h"

#include <QImage>
#include <QPainter>
#include <QString>

#include <array>
#include <vector>

#include "ui_mainwindow.h"

#include "item.h"
#include "itemconstants.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

constexpr int LINKH_HEIGHT = 16;
constexpr int LINKH_WIDTH = 38;
constexpr int LINKV_HEIGHT = LINKH_WIDTH;
constexpr int LINKV_WIDTH = LINKH_HEIGHT;

constexpr int HEADER_SINGLELINE_WIDTH = 29;
constexpr int HEADER_SINGLELINE_HEIGHT = 34;
constexpr int HEADER_DOUBLELINE_WIDTH = 44;
constexpr int HEADER_DOUBLELINE_HEIGHT = 54;

constexpr QSize HEADER_SINGLELINE_SIZE(HEADER_SINGLELINE_WIDTH, HEADER_SINGLELINE_HEIGHT);
constexpr QSize HEADER_DOUBLELINE_SIZE(HEADER_DOUBLELINE_WIDTH, HEADER_DOUBLELINE_HEIGHT);
constexpr QSize HEADER_OVERLAY_SIZE(27, 27);

class IMAGES
{
private:
    IMAGES() = default;

public:
    static const IMAGES &instance()
    {
        static IMAGES images;
        return images;
    };
    const QImage link_h{":/sockets/linkH.png"};
    const QImage link_v{":/sockets/linkV.png"};
    const QImage elder_1x1{":/backgrounds/ElderBackground_1x1.png"};
    const QImage elder_1x3{":/backgrounds/ElderBackground_1x3.png"};
    const QImage elder_1x4{":/backgrounds/ElderBackground_1x4.png"};
    const QImage elder_2x1{":/backgrounds/ElderBackground_2x1.png"};
    const QImage elder_2x2{":/backgrounds/ElderBackground_2x2.png"};
    const QImage elder_2x3{":/backgrounds/ElderBackground_2x3.png"};
    const QImage elder_2x4{":/backgrounds/ElderBackground_2x4.png"};
    const QImage shaper_1x1{":/backgrounds/ShaperBackground_1x1.png"};
    const QImage shaper_1x3{":/backgrounds/ShaperBackground_1x3.png"};
    const QImage shaper_1x4{":/backgrounds/ShaperBackground_1x4.png"};
    const QImage shaper_2x1{":/backgrounds/ShaperBackground_2x1.png"};
    const QImage shaper_2x2{":/backgrounds/ShaperBackground_2x2.png"};
    const QImage shaper_2x3{":/backgrounds/ShaperBackground_2x3.png"};
    const QImage shaper_2x4{":/backgrounds/ShaperBackground_2x4.png"};
    const QImage shaper_icon{":/tooltip/ShaperItemSymbol.png"};
    const QImage elder_icon{":/tooltip/ElderItemSymbol.png"};
    const QImage crusader_icon{":/tooltip/Crusader-item-symbol.png"};
    const QImage hunter_icon{":/tooltip/Hunter-item-symbol.png"};
    const QImage redeemer_icon{":/tooltip/Redeemer-item-symbol.png"};
    const QImage warlord_icon{":/tooltip/Warlord-item-symbol.png"};
    const QImage synthesised_icon{":/tooltip/Synthesised-item-symbol.png"};
    const QImage fractured_icon{":/tooltip/Fractured-item-symbol.png"};
    const QImage searing_exarch_icon{":/tooltip/Searing-exarch-item-symbol.png"};
    const QImage eater_of_worlds_icon{":/tooltip/Eater-of-worlds-item-symbol.png"};
};

/*
    PoE colors:
    Default: 0
    Augmented: 1
    Unmet: 2
    PhysicalDamage: 3
    FireDamage: 4
    ColdDamage: 5
    LightningDamage: 6
    ChaosDamage: 7
    MagicItem: 8
    RareItem: 9
    UniqueItem: 10
*/

static std::array kPoEColors
    = {"#fff", "#88f", "#d20000", "#fff", "#960000", "#366492", "gold", "#d02090"};

static QString ColorPropertyValue(const ItemPropertyValue &value)
{
    size_t type = value.type;
    if (type >= kPoEColors.size()) {
        type = 0;
    }
    const QString color = kPoEColors[type];
    return "<font color='" + color + "'>" + value.str + "</font>";
}

static QString FormatProperty(const ItemProperty &prop)
{
    if (prop.display_mode == 3) {
        QString format(prop.name);
        for (auto &value : prop.values) {
            format = format.arg(ColorPropertyValue(value));
        }
        return format;
    }
    QString text = prop.name;
    if (prop.values.size()) {
        if (prop.name.size() > 0) {
            text += ": ";
        }
        bool first = true;
        for (auto &value : prop.values) {
            if (!first) {
                text += ", ";
            }
            first = false;
            text += ColorPropertyValue(value);
        }
    }
    return text;
}

static QString GenerateProperties(const Item &item)
{
    QString text;
    bool first = true;
    for (auto &property : item.text_properties()) {
        if (!first) {
            text += "<br>";
        }
        first = false;
        text += FormatProperty(property);
    }

    return text;
}

static QString GenerateRequirements(const Item &item)
{
    QString text;
    bool first = true;
    // Talisman level is not really a requirement but it lives in the requirements section
    if (item.talisman_tier()) {
        text += "Talisman Tier: " + std::to_string(item.talisman_tier()) + "<br>";
    }
    for (auto &requirement : item.text_requirements()) {
        text += first ? "Requires " : ", ";
        first = false;
        text += requirement.name + " " + ColorPropertyValue(requirement.value);
    }
    return text;
}

static QString getTextMods(const Item &item, const QString &modType, const char *modColor)
{
    const auto &item_mods = item.text_mods();
    const auto it = item_mods.find(modType);
    if (it == item_mods.end()) {
        return QString();
    }
    const auto modvec = it->second;
    if (modvec.empty()) {
        return QString();
    }
    const auto mods = QStringList{modvec.begin(), modvec.end()};
    return QString("<font color='%1'>%2</font>").arg(modColor, mods.join("<br>"));
}

static std::vector<QString> GenerateMods(const Item &item)
{
    // Create colored strings for each mod set.
    const auto enchantMods = getTextMods(item, "enchantMods", "#b4b4ff");
    const auto implicitMods = getTextMods(item, "implicitMods", "#88f");
    const auto fracturedMods = getTextMods(item, "fracturedMods", "#a29162");
    const auto explicitMods = getTextMods(item, "explicitMods", "#88f");
    const auto craftedMods = getTextMods(item, "craftedMods", "#b4b4ff");

    // There are no spacers between fractured, implicit, and crafted mods.
    QStringList main_section;
    if (!fracturedMods.isEmpty()) {
        main_section.push_back(fracturedMods);
    }
    if (!explicitMods.isEmpty()) {
        main_section.push_back(explicitMods);
    }
    if (!craftedMods.isEmpty()) {
        main_section.push_back(craftedMods);
    }

    // There are spacers between enchants, implicits, and the main section.
    std::vector<QString> sections;
    if (!enchantMods.isEmpty()) {
        sections.push_back(enchantMods);
    }
    if (!implicitMods.isEmpty()) {
        sections.push_back(implicitMods);
    }
    if (!main_section.isEmpty()) {
        sections.push_back(main_section.join("<br>"));
    }
    return sections;
}

static QString GenerateItemInfo(const Item &item, const QString &key, bool fancy)
{
    std::vector<QString> sections;

    QString properties_text = GenerateProperties(item);
    if (properties_text.size() > 0) {
        sections.push_back(properties_text);
    }

    QString requirements_text = GenerateRequirements(item);
    if (requirements_text.size() > 0) {
        sections.push_back(requirements_text);
    }

    std::vector<QString> mods = GenerateMods(item);
    sections.insert(sections.end(), mods.begin(), mods.end());

    QString unmet;
    if (!item.identified()) {
        unmet += "Unidentified";
    }
    if (item.corrupted()) {
        unmet += (unmet.isEmpty() ? "" : "<br>") + QString("Corrupted");
    }
    if (!unmet.isEmpty()) {
        sections.emplace_back(ColorPropertyValue(ItemPropertyValue{unmet, 2}));
    }

    QString text;
    bool first = true;
    for (auto &s : sections) {
        if (!first) {
            text += "<br>";
            if (fancy) {
                text += "<img src=':/tooltip/Separator" + key + ".png'><br>";
            } else {
                text += "<hr>";
            }
        }
        first = false;
        text += s;
    }
    if (!fancy) {
        text = ColorPropertyValue(ItemPropertyValue{item.PrettyName(), 0}) + "<hr>" + text;
    }
    return "<center>" + text + "</center>";
}

static std::array FrameToKey = {"White", "Magic", "Rare", "Unique", "Gem", "Currency"};

static std::array FrameToColor = {"#c8c8c8", "#88f", "#ff7", "#af6025", "#1ba29b", "#aa9e82"};

static void UpdateMinimap(const Item &item, Ui::MainWindow *ui)
{
    QPixmap pixmap(MINIMAP_SIZE, MINIMAP_SIZE);
    pixmap.fill(QColor("transparent"));

    QPainter painter(&pixmap);
    painter.setBrush(QBrush(QColor(0x0c, 0x0b, 0x0b)));
    painter.drawRect(0, 0, MINIMAP_SIZE, MINIMAP_SIZE);
    const ItemLocation &location = item.location();
    painter.setBrush(QBrush(location.socketed() ? Qt::blue : Qt::red));
    QRectF rect = location.GetRect();
    painter.drawRect(rect);

    ui->minimapLabel->setPixmap(pixmap);
}

void GenerateItemHeaderSide(QLabel *itemHeader,
                            bool leftNotRight,
                            QString header_path_prefix,
                            bool singleline,
                            Item::INFLUENCE_TYPES base)
{
    QImage header(header_path_prefix + (leftNotRight ? "Left.png" : "Right.png"));
    QSize header_size = singleline ? HEADER_SINGLELINE_SIZE : HEADER_DOUBLELINE_SIZE;
    header = header.scaled(header_size);

    QPixmap header_pixmap(header_size);
    header_pixmap.fill(Qt::transparent);
    QPainter header_painter(&header_pixmap);
    header_painter.drawImage(0, 0, header);

    QImage overlay_image;

    const auto &images = IMAGES::instance();

    if (base != Item::NONE) {
        switch (base) {
        case Item::ELDER:
            overlay_image = images.elder_icon;
            break;
        case Item::SHAPER:
            overlay_image = images.shaper_icon;
            break;
        case Item::HUNTER:
            overlay_image = images.hunter_icon;
            break;
        case Item::WARLORD:
            overlay_image = images.warlord_icon;
            break;
        case Item::CRUSADER:
            overlay_image = images.crusader_icon;
            break;
        case Item::REDEEMER:
            overlay_image = images.redeemer_icon;
            break;
        case Item::SYNTHESISED:
            overlay_image = images.synthesised_icon;
            break;
        case Item::FRACTURED:
            overlay_image = images.fractured_icon;
            break;
        case Item::SEARING_EXARCH:
            overlay_image = images.searing_exarch_icon;
            break;
        case Item::EATER_OF_WORLDS:
            overlay_image = images.eater_of_worlds_icon;
            break;
        case Item::NONE:
            break;
        }

        overlay_image = overlay_image.scaled(HEADER_OVERLAY_SIZE);
        int overlay_x = singleline ? (leftNotRight ? 2 : 1) : (leftNotRight ? 2 : 15);
        int overlay_y = (int) (0.5 * (header.height() - overlay_image.height()));
        header_painter.drawImage(overlay_x, overlay_y, overlay_image);
    }

    itemHeader->setFixedSize(header_size);
    itemHeader->setPixmap(header_pixmap);
}

void UpdateItemTooltip(const Item &item, Ui::MainWindow *ui)
{
    size_t frame = item.frameType();
    if (frame >= FrameToKey.size()) {
        frame = 0;
    }
    QString key = FrameToKey[frame];

    ui->propertiesLabel->setText(GenerateItemInfo(item, key, true));
    ui->itemTextTooltip->setText(GenerateItemInfo(item, key, false));
    UpdateMinimap(item, ui);

    bool singleline = item.name().isEmpty();
    if (singleline) {
        ui->itemNameFirstLine->hide();
        ui->itemNameSecondLine->setAlignment(Qt::AlignCenter);
        ui->itemNameContainerWidget->setFixedSize(16777215, HEADER_SINGLELINE_HEIGHT);
    } else {
        ui->itemNameFirstLine->show();
        ui->itemNameFirstLine->setAlignment(Qt::AlignBottom | Qt::AlignHCenter);
        ui->itemNameSecondLine->setAlignment(Qt::AlignTop | Qt::AlignHCenter);
        ui->itemNameContainerWidget->setFixedSize(16777215, HEADER_DOUBLELINE_HEIGHT);
    }
    QString suffix = (singleline
                      && (frame == FrameType::FRAME_TYPE_RARE
                          || frame == FrameType::FRAME_TYPE_UNIQUE))
                         ? "SingleLine"
                         : "";
    QString header_path_prefix = ":/tooltip/ItemsHeader" + key + suffix;

    GenerateItemHeaderSide(ui->itemHeaderLeft,
                           true,
                           header_path_prefix,
                           singleline,
                           item.influenceLeft());
    GenerateItemHeaderSide(ui->itemHeaderRight,
                           false,
                           header_path_prefix,
                           singleline,
                           item.influenceRight());

    ui->itemNameContainerWidget->setStyleSheet(
        ("border-radius: 0px; border: 0px; border-image: url(" + header_path_prefix
         + "Middle.png);"));

    ui->itemNameFirstLine->setText(item.name());
    ui->itemNameSecondLine->setText(item.typeLine());

    const QString color = FrameToColor[frame];
    const QString css
        = "border-image: none; background-color: transparent; font-size: 20px; color: " + color;
    ui->itemNameFirstLine->setStyleSheet(css);
    ui->itemNameSecondLine->setStyleSheet(css);
}

QPixmap GenerateItemSockets(const int width,
                            const int height,
                            const std::vector<ItemSocket> &sockets)
{
    QPixmap pixmap(width * PIXELS_PER_SLOT,
                   height
                       * PIXELS_PER_SLOT); // This will ensure we have enough room to draw the slots
    pixmap.fill(Qt::transparent);
    QPainter painter(&pixmap);

    int socket_rows = 0;
    int socket_columns = 0;
    ItemSocket prev = {255, '-'};
    int i = 0;

    if (sockets.size() == 0) {
        // Do nothing
    } else if (sockets.size() == 1) {
        auto &socket = sockets.front();
        QImage socket_image(":/sockets/" + QString(socket.attr) + ".png");
        painter.drawImage(0, PIXELS_PER_SLOT * i, socket_image);
        socket_rows = 1;
        socket_columns = 1;
    } else {
        for (auto &socket : sockets) {
            const auto &images = IMAGES::instance();
            const auto &link_h = images.link_h;
            const auto &link_v = images.link_v;
            bool link = socket.group == prev.group;
            QImage socket_image(":/sockets/" + QString(socket.attr) + ".png");
            if (width == 1) {
                painter.drawImage(0, PIXELS_PER_SLOT * i, socket_image);
                if (link) {
                    painter.drawImage(16, PIXELS_PER_SLOT * i - 19, link_v);
                }
                socket_columns = 1;
                socket_rows = i + 1;
            } else /* w == 2 */ {
                int row = i / 2;
                int column = i % 2;
                if (row % 2 == 1)
                    column = 1 - column;
                socket_columns = qMax(column + 1, socket_columns);
                socket_rows = qMax(row + 1, socket_rows);
                painter.drawImage(PIXELS_PER_SLOT * column, PIXELS_PER_SLOT * row, socket_image);
                if (link) {
                    switch (i) {
                    case 1:
                    case 3:
                    case 5:
                        // horizontal link
                        painter.drawImage(PIXELS_PER_SLOT - LINKH_WIDTH / 2,
                                          row * PIXELS_PER_SLOT + PIXELS_PER_SLOT / 2
                                              - LINKH_HEIGHT / 2,
                                          link_h);
                        break;
                    case 2:
                        painter.drawImage(PIXELS_PER_SLOT * 1.5 - LINKV_WIDTH / 2,
                                          row * PIXELS_PER_SLOT - LINKV_HEIGHT / 2,
                                          link_v);
                        break;
                    case 4:
                        painter.drawImage(PIXELS_PER_SLOT / 2 - LINKV_WIDTH / 2,
                                          row * PIXELS_PER_SLOT - LINKV_HEIGHT / 2,
                                          link_v);
                        break;
                    default:
                        spdlog::error("No idea how to draw link for socket {}", i);
                        break;
                    }
                }
            }

            prev = socket;
            ++i;
        }
    }

    return pixmap.copy(0, 0, PIXELS_PER_SLOT * socket_columns, PIXELS_PER_SLOT * socket_rows);
}

QPixmap GenerateItemIcon(const Item &item, const QImage &image)
{
    const int height = item.h();
    const int width = item.w();

    QPixmap layered(image.width(), image.height());
    layered.fill(Qt::transparent);
    QPainter layered_painter(&layered);

    if (item.hasInfluence(Item::SHAPER) || item.hasInfluence(Item::ELDER)) {
        // Assumes width <= 2
        const auto &images = IMAGES::instance();
        const QImage *background_image = nullptr;
        if (item.hasInfluence(Item::ELDER)) {
            switch (height) {
            case 1:
                background_image = width == 1 ? &images.shaper_1x1 : &images.shaper_2x1;
                break;
            case 2:
                background_image = width == 1 ? nullptr : &images.shaper_2x2;
                break;
            case 3:
                background_image = width == 1 ? &images.shaper_1x3 : &images.shaper_2x3;
                break;
            case 4:
                background_image = width == 1 ? &images.shaper_1x4 : &images.shaper_2x4;
                break;
            default:
                break;
            }
        } else { // Elder
            switch (height) {
            case 1:
                background_image = width == 1 ? &images.elder_1x1 : &images.elder_2x1;
                break;
            case 2:
                background_image = width == 1 ? nullptr : &images.elder_2x2;
                break;
            case 3:
                background_image = width == 1 ? &images.elder_1x3 : &images.elder_2x3;
                break;
            case 4:
                background_image = width == 1 ? &images.elder_1x4 : &images.elder_2x4;
                break;
            default:
                break;
            }
        }
        if (background_image) {
            layered_painter.drawImage(0, 0, *background_image);
        } else {
            spdlog::error("Problem drawing background for {}", item.PrettyName());
        }
    }

    layered_painter.drawImage(0, 0, image);

    if (item.text_sockets().size() > 0) {
        QPixmap sockets = GenerateItemSockets(width, height, item.text_sockets());

        layered_painter.drawPixmap((int) (0.5 * (image.width() - sockets.width())),
                                   (int) (0.5 * (image.height() - sockets.height())),
                                   sockets); // Center sockets on overall image
    }

    return layered;
}
