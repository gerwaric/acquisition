// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QObject>
#include <QString>

#include <cstdint>
#include <memory>
#include <optional>
#include <set>
#include <unordered_map>
#include <utility>
#include <vector>

#include "bucket.h"
#include "column.h"
#include "fetchsourcekey.h"
#include "filters/filterstate.h"
#include "item.h"
#include "items_model.h"
#include "locationinventory.h"
#include "util/util.h"

class BuyoutManager;
class FilterCatalog;
class ItemsModel;
class QModelIndex;
struct BuyoutChangeSet;

class Search
{
    Q_GADGET
public:
    enum class ViewMode : int { ByTab = 0, ByItem = 1 };
    Q_ENUM(ViewMode)

    // The inventory resolves bucket metadata to the freshest location seen
    // per stable display key (M2 D6); null (tests without a pipeline) keeps
    // each item's embedded location and the published tab list alone.
    Search(BuyoutManager &bo,
           const QString &caption,
           const FilterCatalog &catalog,
           const LocationInventory *location_inventory = nullptr);
    ~Search();
    void FilterItems(const Items &items);
    const QString &caption() const { return m_caption; }
    // The visible filtered collection. After S4 the delta path maintains
    // the buckets and indexes incrementally and this flat vector is
    // rebuilt lazily from the By-Tab buckets on first read — no delta
    // pays an O(collection) pass to keep it fresh.
    const Items &items() const;
    const std::vector<std::unique_ptr<Column>> &columns() const { return m_columns; }
    ItemsModel &model() { return m_model; }
    // Expansion is keyed by the stable (type, id) display key (M2 R6-3):
    // header text mutates on rename, so a header-keyed save would orphan
    // the expansion state exactly when a delta renames the expanded tab.
    const std::set<LocationInventory::Key> &expandedKeys() const { return m_expanded_keys; }
    void setExpandedKeys(std::set<LocationInventory::Key> keys);

    // The scroll state captured immediately before the last reset of this
    // search's model (M2 R6-3): a top-row anchor plus the raw scrollbar
    // value as the fallback when the anchored row no longer exists.
    struct ScrollAnchor
    {
        // The top visible row's bucket, and the item's stable id when that
        // row was an item (empty when it was the bucket header itself).
        std::optional<LocationInventory::Key> bucket_key;
        QString item_id;
        int scrollbar_value{0};
    };
    const ScrollAnchor &scrollAnchor() const { return m_scroll_anchor; }
    void setScrollAnchor(ScrollAnchor anchor) { m_scroll_anchor = std::move(anchor); }

    // Global stable-identity lookup over the visible filtered result (M2
    // R6-3): index-backed, rebuilt by every refilter, so reselection never
    // scans the model and survives an item moving to another tab
    // mid-refresh. Returns null for unknown or empty ids.
    std::shared_ptr<Item> visibleItemById(const QString &id) const
    {
        const auto it = m_visible_by_id.find(id);
        return (it != m_visible_by_id.end()) ? it->second : nullptr;
    }

    // True when the id index cannot represent every visible occurrence of
    // this id (M3 S2): a duplicated id (the index keeps the first
    // occurrence only — R6-3 reselection wants exactly one item) or the
    // empty id shared by id-less items. The buyout repaint falls back to
    // scanning every bucket for such ids; rebuilt by every refilter.
    bool visibleIdUnindexed(const QString &id) const
    {
        return m_unindexed_visible_ids.count(id) > 0;
    }
    const std::shared_ptr<Item> &currentItem() const { return m_current_item; }
    void setCurrentItem(std::shared_ptr<Item> item) { m_current_item = std::move(item); }
    const std::optional<ItemLocation> &currentBucket() const { return m_current_bucket; }
    void setCurrentBucket(std::optional<ItemLocation> bucket)
    {
        m_current_bucket = std::move(bucket);
    }
    bool defaultExpanded() const { return m_filtered || (m_current_mode == ViewMode::ByItem); }
    const std::vector<Bucket> &buckets() const;
    void RenameCaption(const QString &newName);
    QString GetCaption() const;
    ItemLocation GetTabLocation(const QModelIndex &index) const;
    void SetViewMode(ViewMode mode);
    ViewMode GetViewMode() const { return m_current_mode; }
    bool has_bucket(int row) const;
    const Bucket &bucket(int row) const;
    const QModelIndex index(const std::shared_ptr<Item> &item) const;
    void SetRefreshReason(RefreshReason reason) { m_refresh_reason = reason; }

    // Sorts the materialized buckets whose flags are invalid (D2): the
    // By-Item flat bucket always, expanded By-Tab buckets by their marks.
    // Collapsed buckets are untouched — the invalidation events cleared
    // their flags, and their sort defers to expansion.
    void Sort(int column, Qt::SortOrder order);

    // Sort-on-expand's per-bucket entry (D2 rule 2), called under the
    // model's layout-change protocol by ItemsModel.
    void SortBucket(int row, int column, Qt::SortOrder order);

    // The view's materialization marks (D1/D2), driven by the tree's
    // expand/collapse signals through ItemsModel. Collapse evicts the
    // bucket's keys; its order and flag persist (D2 rule 3).
    void MarkBucketExpanded(int row);
    void MarkBucketCollapsed(int row);

    // R2-4 / D1 rule 2: evicts every resident key vector, both view modes.
    // Orders and flags persist. Runs on deactivation and on a sort-column
    // switch.
    void EvictResidentKeys();

    // D1 rules 2-3: clears every bucket's sorted flag, both view modes —
    // any (column, order) change invalidates every order at once.
    void InvalidateAllOrder();

    // The buyout key effect (D1 rule 4, R3-2): rebuilds the affected
    // items' entries in whichever key vectors are resident and clears the
    // affected buckets' sorted flags, both view modes — so the batch's
    // re-sort never runs on stale resident keys. Returns the active-mode
    // rows of the affected MATERIALIZED buckets — exactly the set the
    // batch must reorder now; an empty return means the batch needs no
    // layout operation at all (collapsed buckets defer to expansion).
    // `column` is the active sort column.
    std::vector<int> InvalidateBuyoutOrder(const BuyoutChangeSet &changes, int column);

    const FilterCatalog &catalog() const { return m_filter_catalog; }
    qsizetype filterStateCount() const { return static_cast<qsizetype>(m_filter_states.size()); }
    const FilterState &filterStateAt(qsizetype index) const;

    // The only way to change a filter state. Marks the search dirty when the
    // new state differs, so a state changed while this search is in the
    // background still forces a refilter when it is next shown.
    void setFilterState(qsizetype index, FilterState state);

    // D9 rule 1 (items-pipeline M2): every delta marks every search
    // items-dirty; the flag is cleared only by this search's own successful
    // refilter and is consumed by the same refilter-on-next-activation gate
    // as m_states_dirty.
    bool itemsDirty() const { return m_items_dirty; }
    void setItemsDirty(bool dirty) { m_items_dirty = dirty; }

    // D9 intersection, match half: does this item pass the current active
    // filter set?
    bool MatchesActiveFilters(const Item &item) const;

    // D9 intersection, removal half: was anything in the visible filtered
    // result fetched from this source? Rebuilt by every refilter.
    bool HasVisibleSource(const FetchSourceKey &key) const
    {
        return m_visible_sources.count(key) > 0;
    }

    // D9 intersection, aggregate-reconciliation form (R5-2/R6-2): does any
    // visible item under the parent's stable display key carry a fetch
    // source outside the expected set?
    bool HasVisibleGhostUnder(const ItemLocation &parent,
                              const std::vector<FetchSourceKey> &expected) const;

    // The result of one delta application (M3 S4, D3). `processed` is the
    // R1-7 adjudication: true when the delta was applied or correctly
    // adjudicated "no visible change" — the search stays clean; false when
    // application was skipped for any reason — the caller marks the search
    // dirty (fail-safe direction).
    struct DeltaApplication
    {
        bool processed{false};
        bool rows_changed{false};
        // A bucket inserted by this delta (new tab, or first filter-passing
        // items for a hidden one); the caller default-expands it in the
        // view when the search is default-expanded.
        int inserted_bucket_row{-1};
    };

    // Applies a TabRefreshed delta as bucket-scoped operations (D3):
    // metadata first (R1-4 — rename/move/color render now, item
    // intersection notwithstanding), then a source-scoped row replacement
    // within the display bucket — arrivals merged into the retained sorted
    // order when the bucket is visible (R2-2), appended arrival-ordered
    // when collapsed. Visible indexes are maintained incrementally.
    // By-Tab mode only: any other mode returns unprocessed and the caller
    // falls back to the throttled seam.
    DeltaApplication ApplyTabDelta(const ItemLocation &location, const Items &items);

    // Applies a ChildrenReconciled aggregate as row removals scoped to the
    // parent's bucket (D3): visible items under the parent's stable key
    // whose fetch source is outside the expected set leave via contiguous
    // removeRows batches.
    DeltaApplication ApplyChildReconciliation(const ItemLocation &parent,
                                              const std::vector<FetchSourceKey> &expected);

    // The model's child-index identity (M3 S4): child indexes carry the
    // bucket's stable serial, so top-level insert/remove/move operations
    // never invalidate a child's parent mapping. -1 for unknown serials.
    int rowForSerial(std::uint64_t serial) const;

private:
    std::vector<Bucket> &active_buckets();
    const ItemLocation &canonicalLocation(const ItemLocation &embedded) const;

    // S4 delta internals. The index helpers keep m_visible_by_id, the
    // duplicate-occurrence side table, the unindexed set, and the filtered
    // item count answering as if freshly refiltered, in O(1) per item.
    void IndexInsertVisible(const std::shared_ptr<Item> &item);
    void IndexRemoveVisible(const std::shared_ptr<Item> &item);
    void AssignSerials();
    void RebuildSerialRows();
    int FindBucketRow(const LocationInventory::Key &key) const;
    // Renders a delta's fresh location anchor on an existing bucket:
    // dataChanged for header text/color, beginMoveRows when the display
    // position changes. Returns the bucket's (possibly moved) row.
    int ApplyBucketMetadata(int bucket_row, const ItemLocation &canonical);
    // Inserts a new display bucket at its display position; returns its row.
    int InsertBucketRow(const ItemLocation &canonical, const Items &accepted);
    // Removes the bucket rows matching `predicate` as contiguous
    // removeRows runs, unwinding the visible indexes per removed item.
    // Returns true when anything was removed.
    template<typename Predicate>
    bool RemoveBucketRows(int bucket_row, Predicate &&predicate);
    // Inserts filter-accepted arrivals: sorted merge into a visible sorted
    // bucket (R2-2), arrival-ordered append otherwise.
    void InsertArrivals(int bucket_row, const Items &accepted);

    BuyoutManager &m_bo_manager;
    const LocationInventory *m_location_inventory{nullptr};

    // Catalog and filter states are index-aligned. MainWindow owns the catalog
    // and outlives every Search.
    const FilterCatalog &m_filter_catalog;
    std::vector<FilterState> m_filter_states;

    // True when a filter state changed since the last time this search
    // actually filtered. A tab change alone does not need a refilter; a tab
    // change after a state change does.
    bool m_states_dirty{false};

    // True when a streamed delta changed the underlying items since this
    // search last filtered (D9 rule 1).
    bool m_items_dirty{false};

    // Fetch sources of the visible filtered result, flat and grouped by
    // stable display key, rebuilt by every refilter (D9 intersection).
    std::set<FetchSourceKey> m_visible_sources;
    std::map<LocationInventory::Key, std::set<FetchSourceKey>> m_visible_sources_by_tab;

    // The visible result by stable item id (R6-3 reselection), rebuilt by
    // every refilter and maintained per delta (S4). For a duplicated id
    // the entry is the first occurrence — the invariant is
    // m_visible_by_id[id] == m_duplicate_visible_ids[id].front().
    std::unordered_map<QString, std::shared_ptr<Item>> m_visible_by_id;

    // Ids the index above cannot fully represent: duplicated ids and the
    // empty id (see visibleIdUnindexed).
    std::set<QString> m_unindexed_visible_ids;

    // Every visible occurrence of a currently-duplicated non-empty id
    // (mid-refresh divergence can show one id in two tabs). Kept so a
    // delta removing one occurrence restores the survivor to the id index
    // exactly — the indexes answer as if freshly refiltered
    // (deltaUpdatesVisibleIndexesIncrementally). Empty except mid-refresh.
    std::unordered_map<QString, Items> m_duplicate_visible_ids;
    // Visible items sharing the empty id; the unindexed set carries the
    // empty id while this is nonzero.
    size_t m_empty_id_visible_count{0};

    std::vector<std::unique_ptr<Column>> m_columns;

    ItemsModel m_model;
    std::vector<Bucket> m_bucket_by_tab;
    std::vector<Bucket> m_bucket_by_item;

    // Child-index identity (S4): the active mode's bucket row per stable
    // bucket serial, rebuilt on refilter/mode switch and maintained by
    // the structural delta operations.
    std::unordered_map<std::uint64_t, int> m_row_by_serial;
    std::uint64_t m_next_bucket_serial{1};

    // S4 staleness marks: the delta path maintains the By-Tab buckets and
    // indexes; the flat views rebuild lazily — m_items on first read, the
    // By-Item flat bucket at the next mode switch.
    mutable bool m_items_stale{false};
    bool m_flat_bucket_stale{false};

    QString m_caption;
    mutable Items m_items;
    bool m_filtered;
    size_t m_filtered_item_count;
    std::set<LocationInventory::Key> m_expanded_keys;
    ScrollAnchor m_scroll_anchor;
    std::shared_ptr<Item> m_current_item;
    std::optional<ItemLocation> m_current_bucket;
    ViewMode m_current_mode;
    RefreshReason m_refresh_reason;
};

template<>
struct fmt::formatter<Search::ViewMode, char> : QtEnumFormatter<Search::ViewMode>
{};
