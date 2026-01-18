// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "buyoutmanager.h"

#include <QRegularExpression>
#include <QVariant>

#include "application.h"
#include "datastore/datastore.h"
#include "itemlocation.h"
#include "util/glaze_qt.h"  // IWYU pragma: keep
#include "util/spdlog_qt.h" // IWYU pragma: keep

struct SerializedBuyout
{
    QString currency;
    bool inherited;
    quint64 last_update;
    QString source;
    QString type;
    double value;
};

const std::map<QString, BuyoutType> BuyoutManager::m_string_to_buyout_type = {
    {"~gb/o", Buyout::BUYOUT_TYPE_BUYOUT},
    {"~b/o", Buyout::BUYOUT_TYPE_BUYOUT},
    {"~c/o", Buyout::BUYOUT_TYPE_CURRENT_OFFER},
    {"~price", Buyout::BUYOUT_TYPE_FIXED},
};

BuyoutManager::BuyoutManager(DataStore &data)
    : m_data(data)
    , m_save_needed(false)
{
    Load();
}

BuyoutManager::~BuyoutManager()
{
    Save();
}

void BuyoutManager::Set(const Item &item, const Buyout &buyout)
{
    if (buyout.type == Buyout::BUYOUT_TYPE_CURRENT_OFFER) {
        spdlog::warn("BuyoutManager: tried to set an obsolete 'current offer' buyout for {}: {}",
                     item.PrettyName(),
                     buyout.AsText());
    }
    const auto &it = m_buyouts.find(item.id());
    if (it != m_buyouts.end()) {
        // The item hash is present, so check to see if the buyout has changed before saving
        if (buyout != it->second) {
            m_save_needed = true;
            it->second = buyout;
        }
    } else {
        // The item hash is not present, so we need to save buyouts
        m_save_needed = true;
        m_buyouts[item.id()] = buyout;
    }
}

Buyout BuyoutManager::Get(const Item &item) const
{
    const auto &it = m_buyouts.find(item.id());
    if (it != m_buyouts.end()) {
        Buyout buyout = it->second;
        if (buyout.type == Buyout::BUYOUT_TYPE_CURRENT_OFFER) {
            spdlog::warn("BuyoutManager: detected an obsolete 'current offer' buyout for {}: {}",
                         item.PrettyName(),
                         buyout.AsText());
        }
        return buyout;
    }
    return Buyout();
}

Buyout BuyoutManager::GetTab(const QString &tab) const
{
    const auto &it = m_tab_buyouts.find(tab);
    if (it != m_tab_buyouts.end()) {
        Buyout buyout = it->second;
        if (buyout.type == Buyout::BUYOUT_TYPE_CURRENT_OFFER) {
            spdlog::warn(
                "BuyoutManager: detected an obsolete 'current offer' tab buyout for {}: {}",
                tab,
                buyout.AsText());
        }
        return buyout;
    }
    return Buyout();
}

void BuyoutManager::SetTab(const QString &tab, const Buyout &buyout)
{
    if (buyout.type == Buyout::BUYOUT_TYPE_CURRENT_OFFER) {
        spdlog::warn(
            "BuyoutManager: tried to set an obsolete 'current offer' tab buyout for {}: {}",
            tab,
            buyout.AsText());
    }
    const auto &it = m_tab_buyouts.lower_bound(tab);
    if (it != m_tab_buyouts.end() && !(m_tab_buyouts.key_comp()(tab, it->first))) {
        // Entry exists - we don't want to update if buyout is equal to existing
        if (buyout != it->second) {
            m_save_needed = true;
            it->second = buyout;
        }
    } else {
        m_save_needed = true;
        m_tab_buyouts[tab] = buyout;
    }
}

void BuyoutManager::CompressTabBuyouts()
{
    // When tabs are renamed we end up with stale tab buyouts that aren't deleted.
    // This function is to remove buyouts associated with tab names that don't
    // currently exist.
    std::set<QString> tmp;
    for (const auto &loc : m_tabs) {
        tmp.emplace(loc.GetUniqueHash());
    }

    for (auto it = m_tab_buyouts.begin(), ite = m_tab_buyouts.end(); it != ite;) {
        if (tmp.count(it->first) == 0) {
            m_save_needed = true;
            it = m_tab_buyouts.erase(it);
        } else {
            ++it;
        }
    }
}

void BuyoutManager::CompressItemBuyouts(const Items &items)
{
    // When items are moved between tabs or deleted their buyouts entries remain
    // This function looks at buyouts and makes sure there is an associated item
    // that exists
    std::set<QString> tmp;
    for (const auto &item_sp : items) {
        const Item &item = *item_sp;
        tmp.insert(item.id());
    }

    for (auto it = m_buyouts.cbegin(); it != m_buyouts.cend();) {
        if (tmp.count(it->first) == 0) {
            m_buyouts.erase(it++);
        } else {
            ++it;
        }
    }
}

void BuyoutManager::SetRefreshChecked(const ItemLocation &loc, bool value)
{
    m_save_needed = true;
    m_refresh_checked[loc.GetUniqueHash()] = value;
}

bool BuyoutManager::GetRefreshChecked(const ItemLocation &loc) const
{
    auto it = m_refresh_checked.find(loc.GetUniqueHash());
    bool refresh_checked = (it != m_refresh_checked.end()) ? it->second : true;
    return (refresh_checked || GetRefreshLocked(loc));
}

bool BuyoutManager::GetRefreshLocked(const ItemLocation &loc) const
{
    return m_refresh_locked.count(loc.GetUniqueHash());
}

void BuyoutManager::SetRefreshLocked(const ItemLocation &loc)
{
    m_refresh_locked.emplace(loc.GetUniqueHash());
}

void BuyoutManager::ClearRefreshLocks()
{
    m_refresh_locked.clear();
}

void BuyoutManager::Clear()
{
    m_save_needed = true;
    m_buyouts.clear();
    m_tab_buyouts.clear();
    m_refresh_locked.clear();
    m_refresh_checked.clear();
    m_tabs.clear();
}

QString BuyoutManager::Serialize(const std::map<QString, Buyout> &buyouts)
{
    std::map<QString, SerializedBuyout> output;

    for (const auto &[key, buyout] : buyouts) {
        const quint64 last_update = buyout.last_update.isNull()
                                        ? QDateTime::currentSecsSinceEpoch()
                                        : buyout.last_update.toSecsSinceEpoch();

        output[key] = SerializedBuyout{.currency = buyout.CurrencyAsTag(),
                                       .inherited = buyout.inherited,
                                       .last_update = last_update,
                                       .source = buyout.BuyoutSourceAsTag(),
                                       .type = buyout.BuyoutTypeAsTag(),
                                       .value = buyout.value};
    }

    const auto result = glz::write_json(output);
    if (!result) {
        const auto msg = glz::format_error(result.error());
        spdlog::error("Error serializing buyouts: {}", msg);
        return QString();
    }
    return QString::fromStdString(*result);
}

void BuyoutManager::Deserialize(const QString &data, std::map<QString, Buyout> &buyouts)
{
    buyouts.clear();

    // if data is empty (on first use) we shouldn't make user panic by showing ERROR messages
    if (data.isEmpty()) {
        return;
    }

    const QByteArray bytes{data.toUtf8()};
    const std::string_view sv{bytes.constData(), size_t(bytes.size())};
    const auto result = glz::read_json<std::map<QString, SerializedBuyout>>(sv);
    if (!result) {
        const auto msg = glz::format_error(result.error());
        spdlog::error("Error deserializing buyouts: {}", msg);
        return;
    }

    for (const auto &[name, obj] : *result) {
        Buyout bo;
        bo.currency = Currency::FromTag(obj.currency);
        bo.type = Buyout::TagAsBuyoutType(obj.type);
        bo.value = obj.value;
        bo.last_update = QDateTime::fromSecsSinceEpoch(obj.last_update);
        bo.source = Buyout::TagAsBuyoutSource(obj.source);
        bo.inherited = obj.inherited;

        if (bo.type == Buyout::BUYOUT_TYPE_CURRENT_OFFER) {
            spdlog::warn(
                "BuyoutManager::Deserialize() obsolete 'current offer' buyout detected: {}", name);
        }

        buyouts[name] = bo;
    }
}

QString BuyoutManager::Serialize(const std::map<QString, bool> &obj)
{
    const auto result = glz::write_json(obj);
    if (!result) {
        const auto msg = glz::format_error(result.error());
        spdlog::error("Error serializing boolean buyout map: {}", msg);
        return QString();
    }
    return QString::fromStdString(*result);
}

void BuyoutManager::Deserialize(const QString &data, std::map<QString, bool> &obj)
{
    obj.clear();

    // if data is empty (on first use) we shouldn't make user panic by showing ERROR messages
    if (data.isEmpty()) {
        return;
    }

    const QByteArray bytes{data.toUtf8()};
    const std::string_view sv{bytes.constData(), size_t(bytes.size())};
    const auto result = glz::read_json<std::map<QString, bool>>(sv);
    if (!result) {
        const auto msg = glz::format_error(result.error());
        spdlog::error("Error deserializing boolean buyout map: {}", msg);
        return;
    }

    for (const auto &[key, value] : *result) {
        obj[key] = value;
    }
}

void BuyoutManager::Save()
{
    if (!m_save_needed) {
        return;
    }
    m_save_needed = false;
    m_data.Set("buyouts", Serialize(m_buyouts));
    m_data.Set("tab_buyouts", Serialize(m_tab_buyouts));
    m_data.Set("refresh_checked_state", Serialize(m_refresh_checked));
}

void BuyoutManager::Load()
{
    Deserialize(m_data.Get("buyouts"), m_buyouts);
    Deserialize(m_data.Get("tab_buyouts"), m_tab_buyouts);
    Deserialize(m_data.Get("refresh_checked_state"), m_refresh_checked);
}
void BuyoutManager::SetStashTabLocations(const std::vector<ItemLocation> &tabs)
{
    m_tabs = tabs;
}

const std::vector<ItemLocation> &BuyoutManager::GetStashTabLocations() const
{
    return m_tabs;
}

BuyoutType BuyoutManager::StringToBuyoutType(QString bo_str) const
{
    const auto &it = m_string_to_buyout_type.find(bo_str);
    if (it != m_string_to_buyout_type.end()) {
        return it->second;
    }
    return Buyout::BUYOUT_TYPE_INHERIT;
}

Buyout BuyoutManager::StringToBuyout(QString format)
{
    // Parse format string and initialize buyout object, if string does not match any known format
    // then the buyout object will not be valid (IsValid will return false).
    static const QRegularExpression exp("(~\\S+)\\s+(\\d+\\.?\\d*)\\s+(\\w+)");

    Buyout tmp;
    // regex_search allows for stuff before ~ and after currency type.  We only want to honor the formats
    // that POE trade also accept so this may need to change if it's too generous
    QRegularExpressionMatch m = exp.match(format);
    if (m.hasMatch()) {
        tmp.type = StringToBuyoutType(m.captured(1));
        tmp.value = m.captured(2).toDouble();
        tmp.currency = Currency::FromString(m.captured(3));
        tmp.source = Buyout::BUYOUT_SOURCE_GAME;
        tmp.last_update = QDateTime::currentDateTime();
    }
    return tmp;
}

void BuyoutManager::MigrateItem(const QString &old_hash, const QString &new_hash)
{
    const auto old_it = m_buyouts.find(old_hash);

    // Return if there is nothing to migrate.
    if (old_it == m_buyouts.end()) {
        return;
    }

    const auto new_it = m_buyouts.find(new_hash);

    if ((new_it == m_buyouts.end()) || (new_it->second.source != Buyout::BUYOUT_SOURCE_MANUAL)) {
        const auto buyout = old_it->second;
        m_buyouts[new_hash] = buyout;
        m_buyouts.erase(old_it);
        m_save_needed = true;
    }
}

void BuyoutManager::ImportBuyouts(const QString &filename)
{
    spdlog::info("Importing buyouts from {}", filename);
}
