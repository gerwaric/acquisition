// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "ui/searchform.h"

#include <QAbstractItemModel>
#include <QCheckBox>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QStringListModel>
#include <QVBoxLayout>
#include <QWidget>

#include <array>
#include <optional>

#include "itemcategories.h"
#include "modsfilter.h"
#include "search.h"
#include "ui/flowlayout.h"
#include "util/util.h"

namespace {

    class BoolFilterForm final : public FilterFormAdapter
    {
    public:
        BoolFilterForm(QLayout *parent, const QString &caption, const FilterCallbacks &callbacks)
        {
            auto *group = new QWidget;
            auto *layout = new QHBoxLayout;
            layout->setContentsMargins(0, 0, 0, 0);
            auto *label = new QLabel(caption);
            label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
            m_checkbox = new QCheckBox;
            layout->addWidget(label);
            layout->addWidget(m_checkbox);
            group->setLayout(layout);
            parent->addWidget(group);
            label->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_BOOL_LABEL));

            QObject::connect(m_checkbox,
                             &QCheckBox::clicked,
                             callbacks.receiver,
                             callbacks.onChanged);
        }

        void saveTo(FilterState &state) const override
        {
            auto *boolState = std::get_if<BoolState>(&state);
            Q_ASSERT(boolState);
            if (boolState) {
                boolState->checked = m_checkbox->isChecked();
            }
        }

        void loadFrom(const FilterState &state) override
        {
            const auto *boolState = std::get_if<BoolState>(&state);
            Q_ASSERT(boolState);
            if (boolState) {
                m_checkbox->setChecked(boolState->checked);
            }
        }

        void reset() override { m_checkbox->setChecked(false); }

    private:
        QCheckBox *m_checkbox = nullptr;
    };

    class MinMaxFilterForm final : public FilterFormAdapter
    {
    public:
        MinMaxFilterForm(QLayout *parent, const QString &caption, const FilterCallbacks &callbacks)
        {
            auto *group = new QWidget;
            auto *layout = new QHBoxLayout;
            layout->setContentsMargins(0, 0, 0, 0);
            auto *label = new QLabel(caption);
            label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
            m_textboxMin = new QLineEdit;
            m_textboxMax = new QLineEdit;
            layout->addWidget(label);
            layout->addWidget(m_textboxMin);
            layout->addWidget(m_textboxMax);
            group->setLayout(layout);
            parent->addWidget(group);
            m_textboxMin->setPlaceholderText("min");
            m_textboxMax->setPlaceholderText("max");
            m_textboxMin->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_MIN_MAX));
            m_textboxMax->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_MIN_MAX));
            label->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_LABEL));

            QObject::connect(m_textboxMin,
                             &QLineEdit::textEdited,
                             callbacks.receiver,
                             callbacks.onChangedDelayed);
            QObject::connect(m_textboxMax,
                             &QLineEdit::textEdited,
                             callbacks.receiver,
                             callbacks.onChangedDelayed);
        }

        void saveTo(FilterState &state) const override
        {
            auto *minMaxState = std::get_if<MinMaxState>(&state);
            Q_ASSERT(minMaxState);
            if (minMaxState) {
                minMaxState->min = parse(m_textboxMin);
                minMaxState->max = parse(m_textboxMax);
            }
        }

        void loadFrom(const FilterState &state) override
        {
            const auto *minMaxState = std::get_if<MinMaxState>(&state);
            Q_ASSERT(minMaxState);
            if (minMaxState) {
                m_textboxMin->setText(
                    minMaxState->min.has_value() ? QString::number(*minMaxState->min) : QString{});
                m_textboxMax->setText(
                    minMaxState->max.has_value() ? QString::number(*minMaxState->max) : QString{});
            }
        }

        void reset() override
        {
            m_textboxMin->setText("");
            m_textboxMax->setText("");
        }

    private:
        static std::optional<double> parse(const QLineEdit *textbox)
        {
            if (textbox->text().isEmpty()) {
                return std::nullopt;
            }
            return textbox->text().toDouble();
        }

        QLineEdit *m_textboxMin = nullptr;
        QLineEdit *m_textboxMax = nullptr;
    };

} // namespace

SearchForm::SearchForm(QVBoxLayout &layout,
                       const FilterCatalog &catalog,
                       const FilterCallbacks &callbacks)
    : m_layout(layout)
    , m_catalog(catalog)
{
    auto categoryModel = std::make_unique<QStringListModel>(GetItemCategories());
    auto rarityModel = std::make_unique<QStringListModel>(RaritySearchFilter::RARITY_LIST);
    auto *const categoryModelPtr = categoryModel.get();
    auto *const rarityModelPtr = rarityModel.get();
    m_models.push_back(std::move(categoryModel));
    m_models.push_back(std::move(rarityModel));

    m_slots.reserve(static_cast<size_t>(m_catalog.size()));
    m_legacyFilters.resize(static_cast<size_t>(m_catalog.size()));
    const auto addLegacy = [this]<typename FilterType, typename... Args>(qsizetype index,
                                                                         Args &&...args) {
        auto filter = std::make_unique<FilterType>(std::forward<Args>(args)...);
        m_legacyFilters.at(static_cast<size_t>(index)) = filter.get();
        m_slots.emplace_back(std::move(filter));
    };
    const auto addBoolean = [this](QLayout *parent,
                                   qsizetype index,
                                   const QString &caption,
                                   const FilterCallbacks &formCallbacks) {
        Q_ASSERT(!m_legacyFilters.at(static_cast<size_t>(index)));
        m_slots.emplace_back(std::make_unique<BoolFilterForm>(parent, caption, formCallbacks));
    };
    const auto addMinMax = [this](QLayout *parent,
                                  qsizetype index,
                                  const QString &caption,
                                  const FilterCallbacks &formCallbacks) {
        Q_ASSERT(!m_legacyFilters.at(static_cast<size_t>(index)));
        m_slots.emplace_back(std::make_unique<MinMaxFilterForm>(parent, caption, formCallbacks));
    };

    Q_ASSERT(m_catalog.size() >= 4);
    constexpr std::array topKinds{LegacyFilterKind::Tab,
                                  LegacyFilterKind::Name,
                                  LegacyFilterKind::Category,
                                  LegacyFilterKind::Rarity};
    for (qsizetype index = 0; index < static_cast<qsizetype>(topKinds.size()); ++index) {
        const auto *legacy = std::get_if<LegacyPayload>(&m_catalog[index].payload);
        Q_ASSERT(legacy && legacy->kind == topKinds[static_cast<size_t>(index)]);
    }
    addLegacy.template operator()<TabSearchFilter>(0, &m_layout, callbacks);
    addLegacy.template operator()<NameSearchFilter>(1, &m_layout, callbacks);
    addLegacy.template operator()<CategorySearchFilter>(2, &m_layout, categoryModelPtr, callbacks);
    addLegacy.template operator()<RaritySearchFilter>(3, &m_layout, rarityModelPtr, callbacks);

    auto *offenseLayout = new FlowLayout;
    auto *defenseLayout = new FlowLayout;
    auto *socketsLayout = new FlowLayout;
    auto *requirementsLayout = new FlowLayout;
    auto *miscLayout = new FlowLayout;
    auto *miscFlagsLayout = new FlowLayout;
    auto *miscFlags2Layout = new FlowLayout;
    auto *modsLayout = new QVBoxLayout;

    addSearchGroup(offenseLayout, "Offense");
    addSearchGroup(defenseLayout, "Defense");
    addSearchGroup(socketsLayout, "Sockets");
    addSearchGroup(requirementsLayout, "Requirements");
    addSearchGroup(miscLayout, "Misc");
    addSearchGroup(miscFlagsLayout);
    addSearchGroup(miscFlags2Layout);
    addSearchGroup(modsLayout, "Mods");

    for (qsizetype index = 4; index < m_catalog.size(); ++index) {
        const auto &spec = m_catalog[index];
        if (std::holds_alternative<BoolPayload>(spec.payload)) {
            Q_ASSERT(spec.refreshMode == RefreshMode::Immediate);
            switch (spec.group) {
            case FilterGroup::MiscFlags:
                addBoolean(miscFlagsLayout, index, spec.caption, callbacks);
                break;
            case FilterGroup::MiscFlags2:
                addBoolean(miscFlags2Layout, index, spec.caption, callbacks);
                break;
            default:
                Q_ASSERT(false);
                break;
            }
            continue;
        }
        if (std::holds_alternative<MinMaxPayload>(spec.payload)) {
            Q_ASSERT(spec.refreshMode == RefreshMode::Debounced);
            switch (spec.group) {
            case FilterGroup::Offense:
                addMinMax(offenseLayout, index, spec.caption, callbacks);
                break;
            case FilterGroup::Defense:
                addMinMax(defenseLayout, index, spec.caption, callbacks);
                break;
            case FilterGroup::Sockets:
                addMinMax(socketsLayout, index, spec.caption, callbacks);
                break;
            case FilterGroup::Requirements:
                addMinMax(requirementsLayout, index, spec.caption, callbacks);
                break;
            case FilterGroup::Misc:
                addMinMax(miscLayout, index, spec.caption, callbacks);
                break;
            default:
                Q_ASSERT(false);
                break;
            }
            continue;
        }

        const auto *legacy = std::get_if<LegacyPayload>(&spec.payload);
        Q_ASSERT(legacy);
        if (!legacy) {
            continue;
        }

        using Kind = LegacyFilterKind;
        switch (legacy->kind) {
        case Kind::Tab:
            addLegacy.template operator()<TabSearchFilter>(index, &m_layout, callbacks);
            break;
        case Kind::Name:
            addLegacy.template operator()<NameSearchFilter>(index, &m_layout, callbacks);
            break;
        case Kind::Category:
            addLegacy.template operator()<CategorySearchFilter>(index,
                                                                &m_layout,
                                                                categoryModelPtr,
                                                                callbacks);
            break;
        case Kind::Rarity:
            addLegacy.template operator()<RaritySearchFilter>(index,
                                                              &m_layout,
                                                              rarityModelPtr,
                                                              callbacks);
            break;
        case Kind::SocketColors:
            addLegacy.template operator()<SocketsColorsFilter>(index, socketsLayout, callbacks);
            break;
        case Kind::LinkColors:
            addLegacy.template operator()<LinksColorsFilter>(index, socketsLayout, callbacks);
            break;
        case Kind::Mods:
            addLegacy.template operator()<ModsFilter>(index, modsLayout, callbacks);
            break;
        }
    }

    Q_ASSERT(m_slots.size() == static_cast<size_t>(m_catalog.size()));
    Q_ASSERT(m_legacyFilters.size() == static_cast<size_t>(m_catalog.size()));
    for (qsizetype index = 0; index < m_catalog.size(); ++index) {
        const bool isLegacy = std::holds_alternative<LegacyPayload>(m_catalog[index].payload);
        Q_ASSERT((m_legacyFilters.at(static_cast<size_t>(index)) != nullptr) == isLegacy);
        Q_ASSERT(
            std::holds_alternative<std::unique_ptr<Filter>>(m_slots.at(static_cast<size_t>(index)))
            == isLegacy);
    }
}

SearchForm::~SearchForm() = default;

void SearchForm::addSearchGroup(QLayout *layout, const QString &name)
{
    if (!name.isEmpty()) {
        m_layout.addWidget(new QLabel("<h3>" + name + "</h3>"));
    }
    layout->setContentsMargins(0, 0, 0, 0);
    auto *layoutContainer = new QWidget;
    layoutContainer->setLayout(layout);
    m_layout.addWidget(layoutContainer);
}

void SearchForm::saveTo(Search &search)
{
    Q_ASSERT(search.filterSlotCount() == m_catalog.size());
    Q_ASSERT(m_slots.size() == static_cast<size_t>(m_catalog.size()));
    for (qsizetype index = 0; index < m_catalog.size(); ++index) {
        const auto &slot = m_slots.at(static_cast<size_t>(index));
        if (const auto *legacy = std::get_if<std::unique_ptr<Filter>>(&slot)) {
            Q_ASSERT(*legacy);
            Q_ASSERT(std::holds_alternative<LegacyPayload>(m_catalog[index].payload));
            auto &data = search.legacyFilterDataAt(index);
            Q_ASSERT(data.filter() == legacy->get());
            data.FromForm();
        } else {
            const auto *adapter = std::get_if<std::unique_ptr<FilterFormAdapter>>(&slot);
            Q_ASSERT(adapter && *adapter);
            Q_ASSERT(!std::holds_alternative<LegacyPayload>(m_catalog[index].payload));
            (*adapter)->saveTo(search.filterStateAt(index));
        }
    }
}

void SearchForm::loadFrom(Search &search)
{
    Q_ASSERT(search.filterSlotCount() == m_catalog.size());
    Q_ASSERT(m_slots.size() == static_cast<size_t>(m_catalog.size()));
    for (qsizetype index = 0; index < m_catalog.size(); ++index) {
        auto &slot = m_slots.at(static_cast<size_t>(index));
        if (auto *legacy = std::get_if<std::unique_ptr<Filter>>(&slot)) {
            Q_ASSERT(*legacy);
            Q_ASSERT(std::holds_alternative<LegacyPayload>(m_catalog[index].payload));
            auto &data = search.legacyFilterDataAt(index);
            Q_ASSERT(data.filter() == legacy->get());
            data.ToForm();
        } else {
            auto *adapter = std::get_if<std::unique_ptr<FilterFormAdapter>>(&slot);
            Q_ASSERT(adapter && *adapter);
            Q_ASSERT(!std::holds_alternative<LegacyPayload>(m_catalog[index].payload));
            (*adapter)->loadFrom(search.filterStateAt(index));
        }
    }
}

void SearchForm::reset()
{
    for (qsizetype index = 0; index < m_catalog.size(); ++index) {
        auto &slot = m_slots.at(static_cast<size_t>(index));
        if (auto *legacy = std::get_if<std::unique_ptr<Filter>>(&slot)) {
            Q_ASSERT(*legacy);
            Q_ASSERT(std::holds_alternative<LegacyPayload>(m_catalog[index].payload));
            (*legacy)->ResetForm();
        } else {
            auto *adapter = std::get_if<std::unique_ptr<FilterFormAdapter>>(&slot);
            Q_ASSERT(adapter && *adapter);
            Q_ASSERT(!std::holds_alternative<LegacyPayload>(m_catalog[index].payload));
            (*adapter)->reset();
        }
    }
}
