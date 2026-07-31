// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "search.h"

#include <algorithm>
#include <memory>
#include <type_traits>
#include <utility>

#include "bucket.h"
#include "buyoutmanager.h"
#include "column.h"
#include "filters/filtermatchers.h"
#include "filters/filterspec.h"
#include "items_model.h"
#include "modelprobes.h"
#include "util/fatalerror.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep

namespace {

    // The By-Tab display order: rendered location order with the stable id
    // breaking positional ties, so two tabs colliding on one position
    // mid-refresh keep a deterministic order. Shared by the refilter's
    // bucket sort and the delta path's insertion/reposition (S4) — one
    // definition of display order.
    bool bucketDisplayLess(const Bucket &a, const Bucket &b)
    {
        const ItemLocation &la = a.location();
        const ItemLocation &lb = b.location();
        if (la < lb) {
            return true;
        }
        if (lb < la) {
            return false;
        }
        return la.id() < lb.id();
    }

} // namespace

Search::Search(BuyoutManager &bo_manager,
               const QString &caption,
               const FilterCatalog &catalog,
               const LocationInventory *location_inventory)
    : m_bo_manager(bo_manager)
    , m_location_inventory(location_inventory)
    , m_filter_catalog(catalog)
    , m_model(bo_manager, *this)
    , m_caption(caption)
    , m_filtered(false)
    , m_filtered_item_count(0)
    , m_current_mode(ViewMode::ByTab)
    , m_refresh_reason(RefreshReason::Unknown)
{
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

    m_filter_states.reserve(static_cast<size_t>(m_filter_catalog.size()));
    for (qsizetype index = 0; index < m_filter_catalog.size(); ++index) {
        const auto &spec = m_filter_catalog[index];
        FilterState state = MakeDefaultState(spec);
        const bool matchingState = std::visit(
            [&state](const auto &payload) {
                using Payload = std::decay_t<decltype(payload)>;
                if constexpr (std::is_same_v<Payload, TextPayload>) {
                    return std::holds_alternative<TextState>(state);
                } else if constexpr (std::is_same_v<Payload, ComboPayload>) {
                    return std::holds_alternative<ComboState>(state);
                } else if constexpr (std::is_same_v<Payload, MinMaxPayload>) {
                    return std::holds_alternative<MinMaxState>(state);
                } else if constexpr (std::is_same_v<Payload, ColorsPayload>) {
                    return std::holds_alternative<ColorsState>(state);
                } else if constexpr (std::is_same_v<Payload, BoolPayload>) {
                    return std::holds_alternative<BoolState>(state);
                } else {
                    return std::holds_alternative<ModsState>(state);
                }
            },
            spec.payload);
        Q_ASSERT(matchingState);
        m_filter_states.emplace_back(std::move(state));
    }
    Q_ASSERT(m_filter_states.size() == static_cast<size_t>(m_filter_catalog.size()));
}

Search::~Search() = default;

const FilterState &Search::filterStateAt(qsizetype index) const
{
    return m_filter_states.at(static_cast<size_t>(index));
}

void Search::setFilterState(qsizetype index, FilterState state)
{
    auto &current = m_filter_states.at(static_cast<size_t>(index));
    Q_ASSERT(current.index() == state.index());
    if (current == state) {
        return;
    }
    current = std::move(state);
    m_states_dirty = true;
}

void Search::setExpandedKeys(std::set<LocationInventory::Key> keys)
{
    m_expanded_keys = std::move(keys);
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
    // Look for a bucket that matches the item's location. In ByItem mode
    // the single bucket has a null location and holds every visible item,
    // so it is always searched; a ByTab bucket must match the item's
    // stable display key (M2 D6 — never mutable header text or position).
    const auto &bucket_list = buckets();
    const auto item_key = LocationInventory::KeyFor(item->location());
    const int bucket_count = static_cast<int>(bucket_list.size());
    for (int row = 0; row < bucket_count; ++row) {
        // Check each search bucket against the item's location.
        const auto &bucket = bucket_list[row];
        if ((m_current_mode == ViewMode::ByItem)
            || (LocationInventory::KeyFor(bucket.location()) == item_key)) {
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
    if ((column < 0) || (column >= column_count)) {
        return;
    }
    // D2: only visible order is paid for. The flat By-Item bucket is
    // always materialized (rule 6); a By-Tab bucket is materialized while
    // expanded. A materialized bucket whose flag is valid skips entirely;
    // collapsed buckets are left as the invalidation events flagged them.
    auto &col = *m_columns[column];
    const bool by_item = (m_current_mode == ViewMode::ByItem);
    for (auto &bucket : active_buckets()) {
        if ((by_item || bucket.expanded()) && !bucket.sorted()) {
            bucket.Sort(col, order);
        }
    }
}

void Search::SortBucket(int row, int column, Qt::SortOrder order)
{
    const int column_count = static_cast<int>(m_columns.size());
    auto &bucket_list = active_buckets();
    if ((column < 0) || (column >= column_count) || (row < 0)
        || (row >= static_cast<int>(bucket_list.size()))) {
        return;
    }
    bucket_list[static_cast<size_t>(row)].Sort(*m_columns[column], order);
}

void Search::MarkBucketExpanded(int row)
{
    auto &bucket_list = active_buckets();
    if ((row >= 0) && (row < static_cast<int>(bucket_list.size()))) {
        bucket_list[static_cast<size_t>(row)].SetExpanded(true);
    }
}

void Search::MarkBucketCollapsed(int row)
{
    auto &bucket_list = active_buckets();
    if ((row >= 0) && (row < static_cast<int>(bucket_list.size()))) {
        Bucket &bucket = bucket_list[static_cast<size_t>(row)];
        bucket.SetExpanded(false);
        // Collapse is a view event, not a model event (D2 rule 3): the
        // keys evict, the sorted order and flag persist.
        bucket.EvictKeys();
    }
}

void Search::EvictResidentKeys()
{
    for (auto &bucket : m_bucket_by_tab) {
        bucket.EvictKeys();
    }
    for (auto &bucket : m_bucket_by_item) {
        bucket.EvictKeys();
    }
}

void Search::InvalidateAllOrder()
{
    for (auto &bucket : m_bucket_by_tab) {
        bucket.InvalidateOrder();
    }
    for (auto &bucket : m_bucket_by_item) {
        bucket.InvalidateOrder();
    }
}

std::vector<int> Search::InvalidateBuyoutOrder(const BuyoutChangeSet &changes, int column)
{
    std::vector<int> resort_rows;
    const int column_count = static_cast<int>(m_columns.size());
    if ((column < 0) || (column >= column_count)) {
        return resort_rows;
    }
    const Column &col = *m_columns[column];

    // Resolve the affected buckets through the visible-id index so the
    // scoped pricing pass stays O(delta + affected bucket); an id the
    // index cannot fully represent (duplicated across buckets, or the
    // empty id) forces the every-bucket path, the same fallback shape as
    // the rule-5 repaint. Only item-level buyouts feed Price/Date keys
    // (BuyoutManager::Get is item-keyed; tab prices reach items through
    // propagation's item-level Sets in the same batch), so tab ids alone
    // affect no order.
    bool every_bucket = changes.everything;
    std::set<LocationInventory::Key> affected_tabs;
    if (!every_bucket) {
        for (const QString &id : changes.item_ids) {
            if (visibleIdUnindexed(id)) {
                every_bucket = true;
                affected_tabs.clear();
                break;
            }
            if (const auto item = visibleItemById(id)) {
                affected_tabs.insert(LocationInventory::KeyFor(item->location()));
            }
        }
    }

    // Both view modes' buckets are covered: the inactive mode's flags
    // must not claim validity a later mode switch would trust. Only the
    // active mode's materialized affected buckets are returned — they are
    // the ones the batch reorders now.
    const bool by_item = (m_current_mode == ViewMode::ByItem);
    for (size_t row = 0; row < m_bucket_by_tab.size(); ++row) {
        Bucket &bucket = m_bucket_by_tab[row];
        const bool affected = every_bucket
                              || (affected_tabs.count(LocationInventory::KeyFor(bucket.location()))
                                  > 0);
        if (affected) {
            bucket.RebuildKeyEntries(col, changes.item_ids, every_bucket);
            bucket.InvalidateOrder();
            if (!by_item && bucket.expanded()) {
                resort_rows.push_back(static_cast<int>(row));
            }
        }
    }
    const bool flat_affected = every_bucket || !affected_tabs.empty();
    for (auto &bucket : m_bucket_by_item) {
        if (flat_affected) {
            bucket.RebuildKeyEntries(col, changes.item_ids, every_bucket);
            bucket.InvalidateOrder();
            if (by_item) {
                resort_rows.push_back(0);
            }
        }
    }
    return resort_rows;
}

void Search::FilterItems(const Items &items)
{
    spdlog::debug("FilterItems: reason({})", m_refresh_reason);

    // If we're just changing tabs we don't need to update anything, unless a
    // filter state changed since we last filtered. That happens when a form
    // edit writes through to this search and the debounced refresh lands on a
    // different one: the buckets and caption would otherwise stay stale.
    // The gate also tests items-dirty (M2 D9 rule 1): a streamed delta marks
    // every search, and the flag forces a refilter on next activation.
    if ((m_refresh_reason == RefreshReason::TabChanged) && !m_states_dirty && !m_items_dirty) {
        return;
    }

    if (auto &probes = ModelProbes::instance(); probes.enabled) {
        ++probes.refilters;
    }

    m_model.beginUpdate();

    // Create a temporary vector of only the filters that are
    // active, so we don't have to check every filter against
    // every item.
    std::vector<qsizetype> active_filters;
    active_filters.reserve(m_filter_states.size());
    for (qsizetype index = 0; index < static_cast<qsizetype>(m_filter_states.size()); ++index) {
        if (IsActive(m_filter_states.at(static_cast<size_t>(index)))) {
            active_filters.push_back(index);
        }
    }
    active_filters.shrink_to_fit();

    // Reset everything before starting to filter items. The visible
    // indexes are rebuilt from scratch below — the whole-collection
    // rebuild the index-rebuild probe counts (post-M3, deltas maintain
    // them incrementally and only this path increments it).
    if (auto &probes = ModelProbes::instance(); probes.enabled) {
        ++probes.index_rebuilds;
    }
    m_items.clear();
    // "Filtered" means ANY filter is active (S4 review round 2), not the
    // old "at least one item was rejected" snapshot: the membership
    // decisions hanging off this flag (hidden empty buckets, default
    // expansion) must be stable across deltas, and the snapshot could be
    // flipped by one delta — a whole-view change no bucket-scoped
    // operation can express. This definition flips only at filter edits,
    // which are full refilters by construction (D6).
    m_filtered = !active_filters.empty();
    m_filtered_item_count = 0;
    m_visible_sources.clear();
    m_visible_sources_by_tab.clear();
    m_visible_by_id.clear();
    m_unindexed_visible_ids.clear();
    m_duplicate_visible_ids.clear();
    m_empty_id_visible_count = 0;

    // A single bucket with null location is used to view all items at once.
    m_bucket_by_item.clear();
    m_bucket_by_item.emplace_back(ItemLocation());

    // Temporarily store items-by-tabs in a map keyed by the STABLE display
    // identity (M2 D6/R5-1): ItemLocation orders stash locations by
    // position, so keying on the location itself can split a moved tab into
    // old- and new-position buckets, file fresh items under a stale header,
    // or merge two different tabs whose positions collide mid-refresh. Each
    // bucket renders the freshest metadata seen for its key.
    std::map<LocationInventory::Key, Bucket> bucketed_tabs;

    // Try to minimize the number of times we have to loop over each item,
    // because some players have hundreds of thousands or millions of items.
    for (const auto &item : items) {
        // Start by assuming there is a match and run through evey
        // filter until we find that one that will filter out the
        // current item.
        bool matches = true;
        for (const qsizetype index : active_filters) {
            const auto &state = m_filter_states.at(static_cast<size_t>(index));
            if (!MatchesFilter(*item, m_filter_catalog[index], state)) {
                // Now that we know this item will be filtered out,
                // we don't need to check any more filters.
                matches = false;
                break;
            }
        }
        if (matches) {
            // This item passed through all the filters, so we can
            // add it to the list of items and total count.
            m_items.push_back(item);

            // Add this item to the "By Item" bucket.
            m_bucket_by_item.front().AddItem(item);

            // Add this item to the associated "By Tab" bucket.
            const ItemLocation &location = item->location();
            const auto key = LocationInventory::KeyFor(location);
            auto bucket_it = bucketed_tabs.find(key);
            if (bucket_it == bucketed_tabs.end()) {
                bucket_it = bucketed_tabs.emplace(key, Bucket(canonicalLocation(location))).first;
            }
            bucket_it->second.AddItem(item);

            // Record the fetch source for the D9 intersection tests.
            const FetchSourceKey source_key = FetchSourceKey::ForLocation(location);
            m_visible_sources.insert(source_key);
            m_visible_sources_by_tab[key].insert(source_key);

            // Record the stable identity for R6-3 reselection and the
            // count; the shared helper keeps refilter and delta
            // bookkeeping identical (S4).
            IndexInsertVisible(item);
        }
    }

    // We need to add empty tabs here as there are no items to force their addition
    // But only do so if no filters are active as we want to hide empty tabs when
    // filtering. The published tab list resolves through the canonical
    // inventory too, and tabs known only to the inventory (discovered
    // mid-refresh by an empty delta, R6-1) are included until the final
    // snapshot publishes them.
    if (!m_filtered) {
        for (auto &location : m_bo_manager.GetStashTabLocations()) {
            const auto key = LocationInventory::KeyFor(location);
            if (!bucketed_tabs.count(key)) {
                bucketed_tabs.emplace(key, Bucket(canonicalLocation(location)));
            }
        }
        if (m_location_inventory) {
            for (const auto &[key, location] : m_location_inventory->entries()) {
                if (!bucketed_tabs.count(key)) {
                    bucketed_tabs.emplace(key, Bucket(location));
                }
            }
        }
    }

    // Move the "By Tab" buckets into their final location, ordered by their
    // rendered locations — the map above is keyed for identity, not display
    // order. Stable ids break positional ties so two tabs colliding on one
    // position mid-refresh keep a deterministic order.
    m_bucket_by_tab.clear();
    m_bucket_by_tab.reserve(bucketed_tabs.size());
    for (auto &element : bucketed_tabs) {
        m_bucket_by_tab.emplace_back(std::move(element.second));
    }
    std::sort(m_bucket_by_tab.begin(), m_bucket_by_tab.end(), bucketDisplayLess);

    // Fresh buckets get fresh identities (S4): the reset invalidates every
    // persistent index, so serial continuity across a refilter would buy
    // nothing.
    AssignSerials();
    RebuildRowLookups();
    m_items_stale = false;
    m_flat_bucket_stale = false;

    // A filtered By-Tab search is default-expanded (D2 rule 5): mark the
    // fresh buckets materialized now, so the post-refilter sort pass
    // establishes every visible bucket's order eagerly in one pass and the
    // restore's expansion signals find valid flags. Unfiltered fresh
    // buckets start collapsed; RestoreViewExpansion's expand signals sort
    // exactly the restored ones.
    if ((m_current_mode == ViewMode::ByTab) && m_filtered) {
        for (auto &bucket : m_bucket_by_tab) {
            bucket.SetExpanded(true);
        }
    }

    // Let the model know that current sort order has been invalidated
    m_model.SetSorted(false);
    m_model.endUpdate();

    m_states_dirty = false;
    m_items_dirty = false; // a successful refilter clears its own flag (D9 rule 3)
}

const Items &Search::items() const
{
    if (m_items_stale) {
        // Lazily reconciled from the maintained By-Tab buckets: the delta
        // path never pays an O(collection) pass, and only user-initiated
        // boundaries (mode switch) and tests read this.
        m_items.clear();
        for (const auto &bucket : m_bucket_by_tab) {
            m_items.insert(m_items.end(), bucket.items().begin(), bucket.items().end());
        }
        m_items_stale = false;
    }
    return m_items;
}

int Search::rowForSerial(std::uint64_t serial) const
{
    const auto it = m_row_by_serial.find(serial);
    return (it != m_row_by_serial.end()) ? it->second : -1;
}

int Search::rowForKey(const LocationInventory::Key &key) const
{
    if (m_current_mode == ViewMode::ByItem) {
        return (!m_bucket_by_item.empty()
                && (LocationInventory::KeyFor(m_bucket_by_item.front().location()) == key))
                   ? 0
                   : -1;
    }
    return FindBucketRow(key);
}

void Search::AssignSerials()
{
    for (auto &bucket : m_bucket_by_tab) {
        bucket.SetSerial(m_next_bucket_serial++);
    }
    for (auto &bucket : m_bucket_by_item) {
        bucket.SetSerial(m_next_bucket_serial++);
    }
}

void Search::RebuildRowLookups()
{
    // Serial map: the ACTIVE mode's rows (the model's child-index
    // identity). Key map: the By-Tab display buckets (the delta path's
    // stable-key lookup) — always By-Tab, because deltas only apply
    // there. Structural ops pay O(tabs) here by nature of row positions.
    m_row_by_serial.clear();
    const auto &bucket_list = buckets();
    for (size_t row = 0; row < bucket_list.size(); ++row) {
        m_row_by_serial[bucket_list[row].serial()] = static_cast<int>(row);
    }
    m_row_by_key.clear();
    for (size_t row = 0; row < m_bucket_by_tab.size(); ++row) {
        m_row_by_key[LocationInventory::KeyFor(m_bucket_by_tab[row].location())] = static_cast<int>(
            row);
    }
}

int Search::FindBucketRow(const LocationInventory::Key &key) const
{
    const auto it = m_row_by_key.find(key);
    return (it != m_row_by_key.end()) ? it->second : -1;
}

void Search::IndexInsertVisible(const std::shared_ptr<Item> &item)
{
    m_filtered_item_count += item->count();
    const QString id = item->id();
    if (id.isEmpty()) {
        // Id-less items cannot be identity-tracked; the unindexed mark
        // sends consumers down the every-occurrence path.
        ++m_empty_id_visible_count;
        m_unindexed_visible_ids.insert(id);
        return;
    }
    const auto [it, inserted] = m_visible_by_id.emplace(id, item);
    if (!inserted) {
        // Mid-refresh divergence: one id visible twice. Track every
        // occurrence so a removal restores the survivor exactly.
        auto &occurrences = m_duplicate_visible_ids[id];
        if (occurrences.empty()) {
            occurrences.push_back(it->second);
        }
        occurrences.push_back(item);
        m_unindexed_visible_ids.insert(id);
    }
}

void Search::IndexRemoveVisible(const std::shared_ptr<Item> &item)
{
    m_filtered_item_count -= item->count();
    const QString id = item->id();
    if (id.isEmpty()) {
        if ((m_empty_id_visible_count > 0) && (--m_empty_id_visible_count == 0)) {
            m_unindexed_visible_ids.erase(id);
        }
        return;
    }
    const auto dup = m_duplicate_visible_ids.find(id);
    if (dup != m_duplicate_visible_ids.end()) {
        auto &occurrences = dup->second;
        const auto occurrence = std::find(occurrences.begin(), occurrences.end(), item);
        if (occurrence != occurrences.end()) {
            occurrences.erase(occurrence);
        }
        if (occurrences.size() == 1) {
            // Unique again: the survivor is the freshly-refiltered answer.
            m_visible_by_id[id] = occurrences.front();
            m_duplicate_visible_ids.erase(dup);
            m_unindexed_visible_ids.erase(id);
        } else if (!occurrences.empty()) {
            m_visible_by_id[id] = occurrences.front();
        } else {
            m_visible_by_id.erase(id);
            m_duplicate_visible_ids.erase(dup);
            m_unindexed_visible_ids.erase(id);
        }
    } else {
        const auto it = m_visible_by_id.find(id);
        if ((it != m_visible_by_id.end()) && (it->second == item)) {
            m_visible_by_id.erase(it);
        }
    }
}

int Search::ApplyBucketMetadata(int bucket_row, const ItemLocation &canonical)
{
    Bucket &bucket = m_bucket_by_tab[static_cast<size_t>(bucket_row)];
    const ItemLocation old_location = bucket.location();
    const bool rendered_changed = (old_location.GetHeader() != canonical.GetHeader())
                                  || (old_location.getR() != canonical.getR())
                                  || (old_location.getG() != canonical.getG())
                                  || (old_location.getB() != canonical.getB());
    bucket.SetLocation(canonical);

    // Reposition when display ordering changed (D3): the vector is sorted
    // by display order except possibly this bucket, so its destination is
    // found by walking outward from its current row.
    int target = bucket_row;
    while ((target > 0)
           && bucketDisplayLess(m_bucket_by_tab[static_cast<size_t>(bucket_row)],
                                m_bucket_by_tab[static_cast<size_t>(target - 1)])) {
        --target;
    }
    if (target == bucket_row) {
        while ((target + 1 < static_cast<int>(m_bucket_by_tab.size()))
               && bucketDisplayLess(m_bucket_by_tab[static_cast<size_t>(target + 1)],
                                    m_bucket_by_tab[static_cast<size_t>(bucket_row)])) {
            ++target;
        }
    }
    if ((target != bucket_row) && (m_current_mode == ViewMode::ByTab)
        && m_model.BeginMoveBucketRow(bucket_row, target)) {
        auto first = m_bucket_by_tab.begin();
        if (target < bucket_row) {
            std::rotate(first + target, first + bucket_row, first + bucket_row + 1);
        } else {
            std::rotate(first + bucket_row, first + bucket_row + 1, first + target + 1);
        }
        RebuildRowLookups();
        m_model.EndMoveBucketRow();
        bucket_row = target;
    }
    if (rendered_changed && (m_current_mode == ViewMode::ByTab)) {
        m_model.EmitBucketMetadataChanged(bucket_row);
    }
    return bucket_row;
}

int Search::InsertBucketRow(const ItemLocation &canonical, const Items &accepted)
{
    Bucket bucket(canonical);
    bucket.SetSerial(m_next_bucket_serial++);
    bucket.AddItems(accepted);

    int row = 0;
    while ((row < static_cast<int>(m_bucket_by_tab.size()))
           && bucketDisplayLess(m_bucket_by_tab[static_cast<size_t>(row)], bucket)) {
        ++row;
    }
    m_model.BeginInsertBucketRow(row);
    m_bucket_by_tab.insert(m_bucket_by_tab.begin() + row, std::move(bucket));
    RebuildRowLookups();
    m_model.EndInsertBucketRow();
    for (const auto &item : accepted) {
        IndexInsertVisible(item);
    }
    return row;
}

void Search::RemoveBucketRow(int bucket_row)
{
    m_model.BeginRemoveBucketRow(bucket_row);
    m_bucket_by_tab.erase(m_bucket_by_tab.begin() + bucket_row);
    RebuildRowLookups();
    m_model.EndRemoveBucketRow();
}

template<typename Predicate>
bool Search::RemoveBucketRows(int bucket_row, Predicate &&predicate)
{
    Bucket &bucket = m_bucket_by_tab[static_cast<size_t>(bucket_row)];
    const auto &bucket_items = bucket.items();
    // Contiguous runs, removed back-to-front so earlier runs' rows stay
    // valid — O(runs) model operations (D3).
    struct Run
    {
        int first;
        int last;
    };
    std::vector<Run> runs;
    for (int row = 0; row < static_cast<int>(bucket_items.size()); ++row) {
        if (predicate(*bucket_items[static_cast<size_t>(row)])) {
            if (!runs.empty() && (runs.back().last == row - 1)) {
                runs.back().last = row;
            } else {
                runs.push_back({row, row});
            }
        }
    }
    for (auto run = runs.rbegin(); run != runs.rend(); ++run) {
        m_model.BeginRemoveItemRows(bucket_row, run->first, run->last);
        for (int row = run->first; row <= run->last; ++row) {
            IndexRemoveVisible(bucket_items[static_cast<size_t>(row)]);
        }
        bucket.RemoveRows(run->first, run->last - run->first + 1);
        m_model.EndRemoveItemRows();
    }
    return !runs.empty();
}

void Search::InsertArrivals(int bucket_row, const Items &accepted)
{
    Bucket &bucket = m_bucket_by_tab[static_cast<size_t>(bucket_row)];
    const int column = m_model.GetSortColumn();
    const bool sortable = (column >= 0) && (column < static_cast<int>(m_columns.size()));

    if (bucket.expanded() && bucket.sorted() && sortable) {
        // R2-2: the visible-bucket sorted merge. Sorting the arrivals
        // alone cannot establish the bucket's global order when sibling
        // sources' rows interleave under the sort; merging them into the
        // retained rows' order does.
        const Column &col = *m_columns[column];
        bucket.EnsureResidentKeys(col); // key-consuming op hydrates first (R3-1)
        const Qt::SortOrder order = m_model.GetSortOrder();
        auto &probes = ModelProbes::instance();
        const auto less = [&probes, order](const ItemSortKey &lhs, const ItemSortKey &rhs) {
            if (probes.enabled) {
                ++probes.keyed_compares;
            }
            return (order == Qt::AscendingOrder) ? (lhs < rhs) : (rhs < lhs);
        };
        if (probes.enabled) {
            // The merge is this bucket's order-refresh event: fresh keyed
            // order as part of application (staleOrderNeverSurvivesDelta).
            ++probes.bucket_sorts;
            ++probes.bucket_sorts_by_location[LocationInventory::KeyFor(bucket.location())];
        }

        std::vector<std::pair<ItemSortKey, std::shared_ptr<Item>>> arrivals;
        arrivals.reserve(accepted.size());
        for (const auto &item : accepted) {
            arrivals.emplace_back(col.key(*item), item);
        }
        std::sort(arrivals.begin(), arrivals.end(), [&less](const auto &lhs, const auto &rhs) {
            return less(lhs.first, rhs.first);
        });

        // Insertion positions against the retained order, computed before
        // any insertion mutates the key vector; nondecreasing because the
        // arrivals are sorted.
        const auto &retained = bucket.residentKeys();
        std::vector<int> positions;
        positions.reserve(arrivals.size());
        for (const auto &arrival : arrivals) {
            positions.push_back(static_cast<int>(
                std::upper_bound(retained.begin(), retained.end(), arrival.first, less)
                - retained.begin()));
        }

        int offset = 0;
        size_t start = 0;
        while (start < arrivals.size()) {
            size_t end = start + 1;
            while ((end < arrivals.size()) && (positions[end] == positions[start])) {
                ++end;
            }
            Items run_items;
            std::vector<ItemSortKey> run_keys;
            run_items.reserve(end - start);
            run_keys.reserve(end - start);
            for (size_t n = start; n < end; ++n) {
                run_keys.push_back(std::move(arrivals[n].first));
                run_items.push_back(std::move(arrivals[n].second));
            }
            const int first = positions[start] + offset;
            const int last = first + static_cast<int>(end - start) - 1;
            m_model.BeginInsertItemRows(bucket_row, first, last);
            bucket.InsertRows(first, run_items, &run_keys);
            m_model.EndInsertItemRows();
            for (const auto &item : run_items) {
                IndexInsertVisible(item);
            }
            offset += static_cast<int>(end - start);
            start = end;
        }
    } else {
        // Collapsed (or unsortable): arrival-ordered append; the order
        // defers to expansion (D2, collapsedInvalidBucketResortsOnReexpand).
        const int first = static_cast<int>(bucket.items().size());
        const int last = first + static_cast<int>(accepted.size()) - 1;
        m_model.BeginInsertItemRows(bucket_row, first, last);
        bucket.InsertRows(first, accepted, nullptr);
        m_model.EndInsertItemRows();
        for (const auto &item : accepted) {
            IndexInsertVisible(item);
        }
        bucket.InvalidateOrder();
    }
}

Search::DeltaApplication Search::ApplyTabDelta(const ItemLocation &location, const Items &items)
{
    DeltaApplication result;
    if (m_current_mode != ViewMode::ByTab) {
        return result; // the fallback seam owns other modes (S4, deleted in S5)
    }

    const FetchSourceKey source = FetchSourceKey::ForLocation(location);
    const auto delta_key = LocationInventory::KeyFor(location);
    const ItemLocation &canonical = canonicalLocation(location);

    // Filter the arrivals once. O(delta).
    Items accepted;
    accepted.reserve(items.size());
    for (const auto &item : items) {
        if (MatchesActiveFilters(*item)) {
            accepted.push_back(item);
        }
    }

    int bucket_row = FindBucketRow(delta_key);
    if (bucket_row < 0) {
        // No bucket owns this stable key. One appears when arrivals pass
        // the filters, or metadata-only in an unfiltered search (R1-4's
        // new-empty-tab clause); a filtered search with nothing visible
        // has adjudicated "no visible change".
        if (accepted.empty() && m_filtered) {
            result.processed = true;
            return result;
        }
        result.inserted_bucket_row = InsertBucketRow(canonical, accepted);
        if (!accepted.empty()) {
            m_visible_sources.insert(source);
            m_visible_sources_by_tab[delta_key].insert(source);
            m_items_stale = true;
            m_flat_bucket_stale = true;
            result.rows_changed = true;
        }
        result.processed = true;
        return result;
    }

    // Metadata half first (R1-4): the anchor renders now, item
    // intersection notwithstanding; the content ops below then target the
    // bucket's settled row.
    bucket_row = ApplyBucketMetadata(bucket_row, canonical);

    // Content half: a source-scoped replace (R1-1) — remove exactly the
    // rows fetched from this delta's source, then insert the arrivals.
    const bool removed = RemoveBucketRows(bucket_row, [&source](const Item &item) {
        return FetchSourceKey::ForLocation(item.location()) == source;
    });
    if (!accepted.empty()) {
        InsertArrivals(bucket_row, accepted);
    }

    // Source-index maintenance at the delta's own grain: a fetch source
    // belongs to exactly one stable tab, so replacement settles it.
    auto by_tab = m_visible_sources_by_tab.find(delta_key);
    if (accepted.empty()) {
        m_visible_sources.erase(source);
        if (by_tab != m_visible_sources_by_tab.end()) {
            by_tab->second.erase(source);
            if (by_tab->second.empty()) {
                m_visible_sources_by_tab.erase(by_tab);
            }
        }
    } else {
        m_visible_sources.insert(source);
        m_visible_sources_by_tab[delta_key].insert(source);
    }

    if (removed || !accepted.empty()) {
        m_items_stale = true;
        m_flat_bucket_stale = true;
        result.rows_changed = true;
    }

    // A filtered search hides empty buckets (S4 review round 1): a bucket
    // this delta emptied leaves the view, converging to the
    // freshly-refiltered state. Unfiltered searches keep the empty row
    // (emptyDeltaEmptiesBucketWithoutRemovingIt).
    if (m_filtered && m_bucket_by_tab[static_cast<size_t>(bucket_row)].items().empty()) {
        RemoveBucketRow(bucket_row);
        result.processed = true;
        return result;
    }

    // Defensive: a visible bucket must never keep stale order past a
    // delta (staleOrderNeverSurvivesDelta). The merge path preserved
    // sortedness; the append path only runs on collapsed buckets — this
    // covers the expanded-but-invalid corner.
    Bucket &bucket = m_bucket_by_tab[static_cast<size_t>(bucket_row)];
    if (result.rows_changed && bucket.expanded() && !bucket.sorted()) {
        m_model.ResortBucket(bucket_row);
    }

    result.processed = true;
    return result;
}

Search::DeltaApplication Search::ApplyChildReconciliation(
    const ItemLocation &parent, const std::vector<FetchSourceKey> &expected)
{
    DeltaApplication result;
    if (m_current_mode != ViewMode::ByTab) {
        return result; // the fallback seam owns other modes (S4, deleted in S5)
    }

    const auto parent_key = LocationInventory::KeyFor(parent);
    int bucket_row = FindBucketRow(parent_key);
    if (bucket_row < 0) {
        result.processed = true; // nothing visible under the parent
        return result;
    }
    bucket_row = ApplyBucketMetadata(bucket_row, canonicalLocation(parent));

    // The erase becomes row removals scoped to the parent's bucket (D3):
    // already source-predicate-shaped — keys outside the expected set.
    const std::set<FetchSourceKey> allowed(expected.begin(), expected.end());
    std::set<FetchSourceKey> erased_sources;
    const bool removed = RemoveBucketRows(bucket_row, [&allowed, &erased_sources](const Item &item) {
        const auto key = FetchSourceKey::ForLocation(item.location());
        if (allowed.count(key) == 0) {
            erased_sources.insert(key);
            return true;
        }
        return false;
    });
    if (removed) {
        const auto by_tab = m_visible_sources_by_tab.find(parent_key);
        for (const auto &key : erased_sources) {
            m_visible_sources.erase(key);
            if (by_tab != m_visible_sources_by_tab.end()) {
                by_tab->second.erase(key);
            }
        }
        if ((by_tab != m_visible_sources_by_tab.end()) && by_tab->second.empty()) {
            m_visible_sources_by_tab.erase(by_tab);
        }
        m_items_stale = true;
        m_flat_bucket_stale = true;
        result.rows_changed = true;
        // Same filtered-empty convergence as the content delta (S4
        // review round 1).
        if (m_filtered && m_bucket_by_tab[static_cast<size_t>(bucket_row)].items().empty()) {
            RemoveBucketRow(bucket_row);
        }
    }
    result.processed = true;
    return result;
}

bool Search::MatchesActiveFilters(const Item &item) const
{
    for (qsizetype index = 0; index < static_cast<qsizetype>(m_filter_states.size()); ++index) {
        const auto &state = m_filter_states.at(static_cast<size_t>(index));
        if (IsActive(state) && !MatchesFilter(item, m_filter_catalog[index], state)) {
            return false;
        }
    }
    return true;
}

bool Search::HasVisibleGhostUnder(const ItemLocation &parent,
                                  const std::vector<FetchSourceKey> &expected) const
{
    const auto it = m_visible_sources_by_tab.find(LocationInventory::KeyFor(parent));
    if (it == m_visible_sources_by_tab.end()) {
        return false;
    }
    const std::set<FetchSourceKey> allowed(expected.begin(), expected.end());
    for (const auto &key : it->second) {
        if (allowed.count(key) == 0) {
            return true;
        }
    }
    return false;
}

const ItemLocation &Search::canonicalLocation(const ItemLocation &embedded) const
{
    return m_location_inventory ? m_location_inventory->Canonical(embedded) : embedded;
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
    if (mode == m_current_mode) {
        return;
    }
    // An items-dirty search flips QUIETLY — no reset, no rebuild, no
    // sort (S4 review round 2): its whole collection is stale (the
    // fallback seam skipped application), so a reset here would announce
    // nothing useful and force a second reset moments later. The only
    // sanctioned caller of a dirty flip is MainWindow's mode-switch
    // handler, which refilters immediately — FilterItems' own reset then
    // announces the flip and the fresh content as ONE structural change.
    // Nothing may read the model between the flip and that refilter.
    const bool announce = !m_items_dirty;
    if (announce) {
        m_model.beginUpdate();
    }
    // Leaving a mode dematerializes its buckets: keys evict, orders and
    // flags persist (D1/R2-3 — a view event, like collapse).
    for (auto &bucket : active_buckets()) {
        bucket.EvictKeys();
    }
    m_current_mode = mode;
    if ((mode == ViewMode::ByItem) && announce) {
        // The delta path maintains the By-Tab buckets only (S4); a flat
        // bucket gone stale rebuilds here from the maintained collection
        // — the mode switch is one of D6's honest full-rebuild
        // boundaries.
        if (m_flat_bucket_stale && !m_bucket_by_item.empty()) {
            Bucket &flat = m_bucket_by_item.front();
            Bucket rebuilt{ItemLocation()};
            rebuilt.SetSerial(flat.serial());
            rebuilt.AddItems(items());
            flat = std::move(rebuilt);
            m_flat_bucket_stale = false;
        }
        // The flat bucket is always materialized (D2 rule 6): establish
        // its order now if invalid — the reset leaves nothing else to
        // sort it before it paints.
        const int column = m_model.GetSortColumn();
        if (!m_bucket_by_item.empty() && !m_bucket_by_item.front().sorted() && (column >= 0)
            && (column < static_cast<int>(m_columns.size()))) {
            m_bucket_by_item.front().Sort(*m_columns[column], m_model.GetSortOrder());
        }
    }
    RebuildRowLookups();
    if (announce) {
        // Arriving By-Tab buckets start collapsed after the reset;
        // expansion restore sorts the ones whose flags are invalid (D2
        // rule 2).
        m_model.SetSorted(true);
        m_model.endUpdate();
    }
}
