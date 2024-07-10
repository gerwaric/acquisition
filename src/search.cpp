/*
    Copyright 2014 Ilya Zhuravlev

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

#include "search.h"

#include <memory>
#include <QHeaderView>
#include <QTreeView>

#include "buyoutmanager.h"
#include "bucket.h"
#include "column.h"
#include "filters.h"
#include "items_model.h"
#include "QsLog.h"
#include <QMessageBox>

Search::Search(
    BuyoutManager& bo_manager,
    const std::string& caption,
    const std::vector<std::unique_ptr<Filter>>& filters,
    QTreeView* view)
    :
    bo_manager_(bo_manager),
    view_(*view),
    model_(std::make_unique<ItemsModel>(bo_manager, *this)),
    caption_(caption),
    unfiltered_item_count_(0),
    filtered_item_count_total_(0),
    current_mode_(ViewMode::ByTab),
    refresh_reason_(RefreshReason::Unknown)
{
    using move_only = std::unique_ptr<Column>;
    move_only init[] = {
        std::make_unique<NameColumn>(),
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
        std::make_unique<ItemlevelColumn>()
    };
    columns_ = std::vector<move_only>(
        std::make_move_iterator(std::begin(init)),
        std::make_move_iterator(std::end(init)));

    for (auto& filter : filters) {
        filters_.emplace_back(filter->CreateData());
    };
}

void Search::FromForm() {
    for (auto& filter : filters_) {
        filter->FromForm();
    };
}

void Search::ToForm() {
    for (auto& filter : filters_) {
        filter->ToForm();
    };
}

void Search::ResetForm() {
    for (auto& filter : filters_) {
        filter->filter()->ResetForm();
    };
}

const std::vector<Bucket>& Search::buckets() const {
    switch (current_mode_) {
    case ViewMode::ByTab: return bucket_by_tab_; break;
    case ViewMode::ByItem: return bucket_by_item_; break;
    };
}

std::vector<Bucket>& Search::active_buckets() {
    switch (current_mode_) {
    case ViewMode::ByTab: return bucket_by_tab_; break;
    case ViewMode::ByItem: return bucket_by_item_; break;
    };
}

const Bucket& Search::bucket(int row) const {
    const auto& bucket_list = buckets();
    if ((row < 0) || (row >= bucket_list.size())) {
        const QString message = QString("Bucket row out of bounds: %1 bucket size: %2 mode: %3. Program will abort.").arg(
            QString::number(row),
            QString::number(bucket_list.size()),
            QString::number(static_cast<std::underlying_type_t<Search::ViewMode>>(current_mode_)));
        QLOG_FATAL() << message;
        QMessageBox::critical(nullptr, "Fatal Error", message);
        abort();
    };
    return bucket_list[row];
}

const QModelIndex Search::index(const std::shared_ptr<Item> item) const {
    if (!item) {
        // Return an invalid index because there is no current item.
        return QModelIndex();
    };
    // Look for a bucket that matches the item's location.
    const auto& bucket_list = buckets();
    const auto& location_id = item->location().get_tab_uniq_id();
    for (int row = 0; row < bucket_list.size(); ++row) {
        // Check each search bucket against the item's location.
        const auto& bucket = bucket_list[row];
        const auto& bucket_id = bucket.location().get_tab_uniq_id();
        if (location_id == bucket_id) {
            // Check each item in the bucket.
            const QModelIndex parent = model_->index(row);
            const auto& items = bucket.items();
            for (int n = 0; n < items.size(); ++n) {
                const auto& model_item = items[n];
                if (item == model_item) {
                    // Found the index of a match.
                    return model_->index(n, 0, parent);
                };
            };
        };
    };
    // If we get here, that means the previously selected item is no
    // longer part of the current view.
    return QModelIndex();
}

void Search::Sort(int column, Qt::SortOrder order) {
    if ((column >= 0) && (column < columns_.size())) {
        auto& col = *columns_[column];
        for (auto& bucket : active_buckets()) {
            bucket.Sort(col, order);
        };
    };
}

void Search::FilterItems(const Items& items) {
    // If we're just changing tabs we don't need to update anything
    if (refresh_reason_ == RefreshReason::TabChanged) {
        return;
    };

    QLOG_DEBUG() << "FilterItems: reason(" << refresh_reason_ << ")";
    items_.clear();
    for (const auto& item : items) {
        bool matches = true;
        for (auto& filter : filters_)
            if (!filter->Matches(item)) {
                matches = false;
                break;
            };
        if (matches) {
            items_.push_back(item);
        };
    };

    UpdateItemCounts(items);

    // Single bucket with null location is used to view all items at once
    bucket_by_item_.clear();
    bucket_by_item_.emplace_back(ItemLocation());

    std::map<ItemLocation, Bucket> bucketed_tabs;
    for (const auto& item : items_) {
        ItemLocation location = item->location();
        if (!bucketed_tabs.count(location)) {
            bucketed_tabs.emplace(location, Bucket(location));
        };
        bucketed_tabs[location].AddItem(item);
        bucket_by_item_.front().AddItem(item);
    };

    // We need to add empty tabs here as there are no items to force their addition
    // But only do so if no filters are active as we want to hide empty tabs when
    // filtering
    if (!IsAnyFilterActive()) {
        for (auto& location : bo_manager_.GetStashTabLocations()) {
            if (!bucketed_tabs.count(location)) {
                bucketed_tabs.emplace(location, Bucket(location));
            };
        };
    };

    //buckets_by_tab_.clear();
    for (auto& element : bucketed_tabs) {
        bucket_by_tab_.push_back(std::move(element.second));
    };

    // Let the model know that current sort order has been invalidated
    model_->SetSorted(false);
}

void Search::RenameCaption(const std::string newName) {
    caption_ = newName;
}

QString Search::GetCaption() const {
    return QString("%1 [%2]").arg(caption_.c_str()).arg(GetItemsCount());
}

ItemLocation Search::GetTabLocation(const QModelIndex& index) const {
    if (!index.isValid()) {
        return ItemLocation();
    };

    if (index.internalId() > 0) {
        // If index represents an item, get location from item as view may be on 'item' view
        // where bucket location doesn't match items location
        return bucket(index.parent().row()).item(index.row())->location();
    } else {
        // Otherwise index represents a tab already, get location from there
        return bucket(index.row()).location();
    }
}

void Search::SetViewMode(ViewMode mode)
{
    if (mode != current_mode_) {
        if (mode == ViewMode::ByItem) {
            SaveViewProperties();
        };

        current_mode_ = mode;

        // Force immediate view update
        view_.reset();
        model_->SetSorted(false);
        model_->sort();

        if (mode == ViewMode::ByTab) {
            RestoreViewProperties();
        };
    }
}

size_t Search::GetItemsCount() const {
    return filtered_item_count_total_;
}

void Search::Activate(const Items& items) {
    FromForm();
    FilterItems(items);
    view_.setSortingEnabled(false);
    view_.setModel(model_.get());
    view_.header()->setSortIndicator(model_->GetSortColumn(), model_->GetSortOrder());
    view_.setSortingEnabled(true);
}

void Search::SaveViewProperties() {
    expanded_property_.clear();
    const int rowCount = model_->rowCount();
    for (int row = 0; row < rowCount; ++row) {
        QModelIndex index = model_->index(row, 0, QModelIndex());
        if (index.isValid() && view_.isExpanded(index)) {
            expanded_property_.insert(bucket(row).location().GetHeader());
        };
    };
}

void Search::RestoreViewProperties() {
    if (!expanded_property_.empty()) {
        // There are some rows to expand.
        const int rowCount = model_->rowCount();
        for (int row = 0; row < rowCount; ++row) {
            QModelIndex index = model_->index(row, 0, QModelIndex());
            // Block signals else columns will be resized on every expand which can be super slow.
            view_.blockSignals(true);
            if (expanded_property_.count(bucket(row).location().GetHeader())) {
                view_.expand(index);
            } else {
                view_.collapse(index);
            };
            view_.blockSignals(false);
        }
    } else {
        // Make sure all the rows are collapsed otherwise.
        view_.blockSignals(true);
        view_.collapseAll();
        view_.blockSignals(false);
    };
}

bool Search::IsAnyFilterActive() const {
    return (items_.size() != unfiltered_item_count_);
}

void Search::UpdateItemCounts(const Items& items) {
    unfiltered_item_count_ = items.size();
    filtered_item_count_total_ = 0;
    for (auto& item : items_) {
        filtered_item_count_total_ += item->count();
    };
}

