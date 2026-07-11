// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "search.h"

#include <memory>

#include "bucket.h"
#include "buyoutmanager.h"
#include "column.h"
#include "filters.h"
#include "filters/filterspec.h"
#include "items_model.h"
#include "util/fatalerror.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

Search::Search(BuyoutManager &bo_manager,
               const QString &caption,
               const FilterCatalog &catalog,
               const std::vector<Filter *> &filters)
    : m_bo_manager(bo_manager)
    , m_filter_catalog(catalog)
    , m_model(bo_manager, *this)
    , m_caption(caption)
    , m_filtered(false)
    , m_filtered_item_count(0)
    , m_current_mode(ViewMode::ByTab)
    , m_refresh_reason(RefreshReason::Unknown)
{
    Q_ASSERT(filters.size() == static_cast<size_t>(m_filter_catalog.size()));

    using move_only = std::unique_ptr<Column>;
    move_only init[] = {std::make_unique<NameColumn>(),
                        std::make_unique<PriceColumn>(bo_manager),
                        std::make_unique<DateColumn>(bo_manager),
                        std::make_unique<PropertyColumn>("Q", "Quality"),
                        std::make_unique<PropertyColumn>("Stack", "Stack Size"),
                        std::make_unique<CorruptedColumn>(),
                        std::make_unique<CraftedColumn>(),
                        std::make_unique<EnchantedColumn>(),
                        std::make_unique<InfluncedColumn>(),
                        std::make_unique<PropertyColumn>("PD", "Physical Damage"),
                        std::make_unique<ElementalDamageColumn>(0),
                        std::make_unique<ElementalDamageColumn>(1),
                        std::make_unique<ElementalDamageColumn>(2),
                        std::make_unique<ChaosDamageColumn>(),
                        std::make_unique<PropertyColumn>("APS", "Attacks per Second"),
                        std::make_unique<DPSColumn>(),
                        std::make_unique<pDPSColumn>(),
                        std::make_unique<eDPSColumn>(),
                        std::make_unique<cDPSColumn>(),
                        std::make_unique<PropertyColumn>("Crit", "Critical Strike Chance"),
                        std::make_unique<PropertyColumn>("Ar", "Armour"),
                        std::make_unique<PropertyColumn>("Ev", "Evasion Rating"),
                        std::make_unique<PropertyColumn>("ES", "Energy Shield"),
                        std::make_unique<PropertyColumn>("B", "Chance to Block"),
                        std::make_unique<PropertyColumn>("Lvl", "Level"),
                        std::make_unique<ItemlevelColumn>()};
    m_columns = std::vector<move_only>(std::make_move_iterator(std::begin(init)),
                                       std::make_move_iterator(std::end(init)));

    for (qsizetype index = 0; index < m_filter_catalog.size(); ++index) {
        const auto &spec = m_filter_catalog[index];
        if (std::holds_alternative<LegacyPayload>(spec.payload)) {
            auto *filter = filters.at(static_cast<size_t>(index));
            Q_ASSERT(filter);
            m_filter_slots.emplace_back(filter->CreateData());
        } else {
            m_filter_slots.emplace_back(MakeDefaultState(spec));
        }
    }
}

void Search::FromForm()
{
    for (auto &slot : m_filter_slots) {
        auto &filter = std::get<std::unique_ptr<FilterData>>(slot);
        filter->FromForm();
    }
}

void Search::ToForm()
{
    for (auto &slot : m_filter_slots) {
        auto &filter = std::get<std::unique_ptr<FilterData>>(slot);
        filter->ToForm();
    }
}

void Search::ResetForm()
{
    for (auto &slot : m_filter_slots) {
        auto &filter = std::get<std::unique_ptr<FilterData>>(slot);
        filter->filter()->ResetForm();
    }
}

void Search::setExpandedHeaders(std::set<QString> headers)
{
    m_expanded_property = std::move(headers);
}

const std::vector<Bucket> &Search::buckets() const
{
    switch (m_current_mode) {
    case ViewMode::ByTab:
        return m_bucket_by_tab;
        break;
    case ViewMode::ByItem:
        return m_bucket_by_item;
        break;
    default:
        spdlog::error("Invalid view mode: {}", m_current_mode);
        return m_bucket_by_item;
        break;
    }
}

std::vector<Bucket> &Search::active_buckets()
{
    switch (m_current_mode) {
    case ViewMode::ByTab:
        return m_bucket_by_tab;
        break;
    case ViewMode::ByItem:
        return m_bucket_by_item;
        break;
    default:
        spdlog::error("Invalid view mode: {}", static_cast<int>(m_current_mode));
        return m_bucket_by_item;
        break;
    }
}

bool Search::has_bucket(int row) const
{
    return (row >= 0) && (row < static_cast<int>(buckets().size()));
}

const Bucket &Search::bucket(int row) const
{
    const auto &bucket_list = buckets();
    const int bucket_count = static_cast<int>(bucket_list.size());
    if ((row < 0) || (row >= bucket_count)) {
        const int mode = static_cast<std::underlying_type_t<Search::ViewMode>>(m_current_mode);
        const QString message
            = QString("Bucket row out of bounds: %1 bucket size: %2 mode: %3. Program will abort.")
                  .arg(QString::number(row), QString::number(bucket_count), QString::number(mode));
        FatalError(message);
    }
    return bucket_list[row];
}

const QModelIndex Search::index(const std::shared_ptr<Item> &item) const
{
    if (!item) {
        // Return an invalid index because there is no current item.
        return QModelIndex();
    }
    // Look for a bucket that matches the item's location.
    const auto &bucket_list = buckets();
    const auto &location_id = item->location().id();
    const int bucket_count = static_cast<int>(bucket_list.size());
    for (int row = 0; row < bucket_count; ++row) {
        // Check each search bucket against the item's location.
        const auto &bucket = bucket_list[row];
        const auto &bucket_id = bucket.location().id();
        if (location_id == bucket_id) {
            // Check each item in the bucket.
            const QModelIndex parent = m_model.index(row);
            const auto &items = bucket.items();
            const int item_count = static_cast<int>(items.size());
            for (int n = 0; n < item_count; ++n) {
                const auto &model_item = items[n];
                if (item == model_item) {
                    // Found the index of a match.
                    return m_model.index(n, 0, parent);
                }
            }
        }
    }
    // If we get here, that means the previously selected item is no
    // longer part of the current view.
    return QModelIndex();
}

void Search::Sort(int column, Qt::SortOrder order)
{
    const int column_count = static_cast<int>(m_columns.size());
    if ((column >= 0) && (column < column_count)) {
        auto &col = *m_columns[column];
        for (auto &bucket : active_buckets()) {
            bucket.Sort(col, order);
        }
    }
}

void Search::FilterItems(const Items &items)
{
    spdlog::debug("FilterItems: reason({})", m_refresh_reason);

    // If we're just changing tabs we don't need to update anything
    if (m_refresh_reason == RefreshReason::TabChanged) {
        return;
    }

    m_model.beginUpdate();

    // Create a temporary vector of only the filters that are
    // active, so we don't have to check every filter against
    // every item.
    std::vector<FilterData *> active_filters;
    active_filters.reserve(m_filter_slots.size());
    for (auto &slot : m_filter_slots) {
        auto &filter = std::get<std::unique_ptr<FilterData>>(slot);
        if (filter->filter()->IsActive()) {
            active_filters.push_back(filter.get());
        }
    }
    active_filters.shrink_to_fit();

    // Reset everything before starting to filter items.
    m_items.clear();
    m_filtered = false;
    m_filtered_item_count = 0;

    // A single bucket with null location is used to view all items at once.
    m_bucket_by_item.clear();
    m_bucket_by_item.emplace_back(ItemLocation());

    // Temporarily store items-by-tabs in a map.
    std::map<ItemLocation, Bucket> bucketed_tabs;

    // Try to minimize the number of times we have to loop over each item,
    // because some players have hundreds of thousands or millions of items.
    for (const auto &item : items) {
        // Start by assuming there is a match and run through evey
        // filter until we find that one that will filter out the
        // current item.
        bool matches = true;
        for (const auto &filter : active_filters) {
            if (!filter->Matches(item)) {
                // Now that we know this item will be filtered out,
                // we don't need to check any more filters.
                matches = false;
                m_filtered = true;
                break;
            }
        }
        if (matches) {
            // This item passed through all the filters, so we can
            // add it to the list of items and total count.
            m_items.push_back(item);
            m_filtered_item_count += item->count();

            // Add this item to the "By Item" bucket.
            m_bucket_by_item.front().AddItem(item);

            // Add this item to the associagted "By Tab" bucket.
            const ItemLocation location = item->location();
            if (!bucketed_tabs.count(location)) {
                bucketed_tabs[location] = Bucket(location);
            }
            bucketed_tabs[location].AddItem(item);
        }
    }

    // We need to add empty tabs here as there are no items to force their addition
    // But only do so if no filters are active as we want to hide empty tabs when
    // filtering
    if (!m_filtered) {
        for (auto &location : m_bo_manager.GetStashTabLocations()) {
            if (!bucketed_tabs.count(location)) {
                bucketed_tabs[location] = Bucket(location);
            }
        }
    }

    // Move the "By Tab" buckets into their final location.
    m_bucket_by_tab.clear();
    m_bucket_by_tab.reserve(bucketed_tabs.size());
    for (auto &element : bucketed_tabs) {
        m_bucket_by_tab.emplace_back(std::move(element.second));
    }

    // Let the model know that current sort order has been invalidated
    m_model.SetSorted(false);
    m_model.endUpdate();
}

void Search::RenameCaption(const QString &newName)
{
    m_caption = newName;
}

QString Search::GetCaption() const
{
    return QString("%1 [%2]").arg(m_caption).arg(m_filtered_item_count);
}

ItemLocation Search::GetTabLocation(const QModelIndex &index) const
{
    if (!index.isValid()) {
        return ItemLocation();
    }
    if (index.internalId() > 0) {
        // If index represents an item, get location from item as view may be on 'item' view
        // where bucket location doesn't match items location
        const int bucket_row = index.parent().row();
        if (has_bucket(bucket_row)) {
            const Bucket &b = bucket(bucket_row);
            const int item_row = index.row();
            if (b.has_item(item_row)) {
                return b.item(item_row)->location();
            } else {
                spdlog::warn("GetTabLocation(): parent bucket {} does not have {} items",
                             bucket_row,
                             item_row);
            }
        } else {
            spdlog::warn("GetTabLocation(): parent bucket {} does not exist", bucket_row);
        }
    } else {
        // Otherwise index represents a tab already, get location from there
        const int bucket_row = index.row();
        if (has_bucket(bucket_row)) {
            return bucket(bucket_row).location();
        } else {
            spdlog::warn("GetTabLocation(): bucket {} does not exist", bucket_row);
        }
    }
    return ItemLocation();
}

void Search::SetViewMode(ViewMode mode)
{
    if (mode != m_current_mode) {
        m_model.beginUpdate();
        m_current_mode = mode;
        Sort(m_model.GetSortColumn(), m_model.GetSortOrder());
        m_model.SetSorted(true);
        m_model.endUpdate();
    }
}

void Search::Activate(const Items &items)
{
    FromForm();
    FilterItems(items);
}
