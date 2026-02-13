// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "buyoutmanager.h"

#include <QRegularExpression>
#include <QVariant>

#include "app/usersettings.h"
#include "datastore/buyoutstore.h"
#include "datastore/sessionstore.h"
#include "item.h"
#include "itemlocation.h"
#include "util/glaze_qt.h" // IWYU pragma: keep
#include "util/json_utils.h"
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

const std::unordered_map<QString, BuyoutType> BuyoutManager::m_string_to_buyout_type = {
    {"~gb/o", Buyout::BUYOUT_TYPE_BUYOUT},
    {"~b/o", Buyout::BUYOUT_TYPE_BUYOUT},
    {"~c/o", Buyout::BUYOUT_TYPE_CURRENT_OFFER},
    {"~price", Buyout::BUYOUT_TYPE_FIXED},
};

BuyoutManager::BuyoutManager(app::UserSettings &settings, SessionStore &data, BuyoutStore &repo)
    : m_settings(settings)
    , m_data(data)
    , m_repo(repo)
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

    if (buyout.IsNull()) {
        m_repo.removeItemBuyout(item);
        return;
    }

    const auto &it = m_buyouts.find(item.id());
    if (it == m_buyouts.end()) {
        // The item hash is not present.
        m_buyouts[item.id()] = buyout;
        emit SetItemBuyout(buyout, item);
    } else if (buyout != it->second) {
        // The item hash is present and the buyout has changed.
        it->second = buyout;
        emit SetItemBuyout(buyout, item);
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

Buyout BuyoutManager::GetTab(const ItemLocation &location) const
{
    const QString tab = location.id();
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

void BuyoutManager::SetTab(const ItemLocation &location, const Buyout &buyout)
{
    if (buyout.type == Buyout::BUYOUT_TYPE_CURRENT_OFFER) {
        spdlog::warn(
            "BuyoutManager: tried to set an obsolete 'current offer' tab buyout for {}: {}",
            location.id(),
            buyout.AsText());
    }

    if (buyout.IsNull()) {
        m_repo.removeLocationBuyout(location);
        return;
    }

    const auto &it = m_tab_buyouts.find(location.id());
    if (it == m_tab_buyouts.end()) {
        m_tab_buyouts[location.id()] = buyout;
        emit SetLocationBuyout(buyout, location);
    } else if (buyout != it->second) {
        it->second = buyout;
        emit SetLocationBuyout(buyout, location);
    }
}

void BuyoutManager::SetRefreshChecked(const ItemLocation &loc, bool value)
{
    const auto it = m_refresh_checked.find(loc.id());
    if (it != m_refresh_checked.end()) {
        // The location is already in the map.
        if (value != it->second) {
            if (value) {
                it->second = true;
            } else {
                m_refresh_checked.erase(loc.id());
            }
            m_save_needed = true;
        }
    } else {
        // The location is not in the map.
        if (value) {
            m_refresh_checked[loc.id()] = value;
            m_save_needed = true;
        }
    }
}

bool BuyoutManager::GetRefreshChecked(const ItemLocation &loc) const
{
    if (GetRefreshLocked(loc)) {
        return true;
    }
    const auto it = m_refresh_checked.find(loc.id());
    return (it == m_refresh_checked.end()) ? false : it->second;
}

bool BuyoutManager::GetRefreshLocked(const ItemLocation &loc) const
{
    return m_refresh_locked.count(loc.id());
}

void BuyoutManager::SetRefreshLocked(const ItemLocation &loc)
{
    m_refresh_locked.emplace(loc.id());
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

QString BuyoutManager::Serialize(const std::unordered_map<QString, Buyout> &buyouts)
{
    std::unordered_map<QString, SerializedBuyout> output;

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

void BuyoutManager::Deserialize(const QString &data, std::unordered_map<QString, Buyout> &buyouts)
{
    buyouts.clear();

    // if data is empty (on first use) we shouldn't make user panic by showing ERROR messages
    if (data.isEmpty()) {
        return;
    }

    const QByteArray bytes{data.toUtf8()};
    const std::string_view sv{bytes.constData(), size_t(bytes.size())};
    const auto result = glz::read_json<std::unordered_map<QString, SerializedBuyout>>(sv);
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

QString BuyoutManager::Serialize(const std::unordered_map<QString, bool> &obj)
{
    const auto result = glz::write_json(obj);
    if (!result) {
        const auto msg = glz::format_error(result.error());
        spdlog::error("Error serializing boolean buyout map: {}", msg);
        return QString();
    }
    return QString::fromStdString(*result);
}

void BuyoutManager::Deserialize(const QString &data, std::unordered_map<QString, bool> &obj)
{
    obj.clear();

    // if data is empty (on first use) we shouldn't make user panic by showing ERROR messages
    if (data.isEmpty()) {
        return;
    }

    const QByteArray bytes{data.toUtf8()};
    const std::string_view sv{bytes.constData(), size_t(bytes.size())};
    const auto result = glz::read_json<std::unordered_map<QString, bool>>(sv);
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
    if (m_save_needed) {
        m_data.refreshChecked(write_json(m_refresh_checked));
        m_save_needed = false;
    }
}

void BuyoutManager::Load()
{
    m_buyouts = m_repo.getItemBuyouts();
    m_tab_buyouts = m_repo.getLocationBuyouts();

    const auto json = m_data.refreshChecked();
    const auto result = read_json<std::unordered_map<QString, bool>>(json);
    m_refresh_checked = result.value_or({});
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
