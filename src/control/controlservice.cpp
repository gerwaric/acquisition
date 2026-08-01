// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/controlservice.h"

#include <QJsonArray>
#include <QJsonDocument>
#include <QUuid>

#include <map>
#include <optional>

#include "buyoutmanager.h"
#include "control/viewprojection.h"
#include "itemsmanager.h"
#include "itemsmanagerworker.h"

namespace control {

namespace {

    constexpr int DEFAULT_PAGE_SIZE = 50;
    constexpr int MAXIMUM_PAGE_SIZE = 100;

    struct ItemQuery
    {
        int limit{DEFAULT_PAGE_SIZE};
        qsizetype offset{0};
        QString tab_id;
        std::optional<ItemLocationType> kind;
    };

    QString KindName(std::optional<ItemLocationType> kind)
    {
        if (!kind) {
            return {};
        }
        return *kind == ItemLocationType::STASH ? "stash" : "character";
    }

    std::optional<ItemLocationType> ParseKind(const QString &kind)
    {
        if (kind == "stash") {
            return ItemLocationType::STASH;
        }
        if (kind == "character") {
            return ItemLocationType::CHARACTER;
        }
        return std::nullopt;
    }

    QString EncodeCursor(const QString &instance_id, quint64 revision, const ItemQuery &query)
    {
        const QJsonObject object{{"instance_id", instance_id},
                                 {"revision", QString::number(revision)},
                                 {"offset", QString::number(query.offset)},
                                 {"limit", query.limit},
                                 {"tab_id", query.tab_id},
                                 {"kind", KindName(query.kind)}};
        return QString::fromLatin1(
            QJsonDocument(object)
                .toJson(QJsonDocument::Compact)
                .toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals));
    }

    std::expected<ItemQuery, ProtocolError> DecodeCursor(const QString &encoded,
                                                         const QString &instance_id,
                                                         quint64 revision)
    {
        const QByteArray payload = QByteArray::fromBase64(encoded.toLatin1(),
                                                          QByteArray::Base64UrlEncoding);
        auto decoded = DecodeObject(payload);
        if (!decoded) {
            return std::unexpected(ProtocolError{"invalid_cursor", "the cursor is malformed"});
        }
        const QJsonObject &object = *decoded;
        if (object.value("instance_id").toString() != instance_id
            || object.value("revision").toString() != QString::number(revision)) {
            return std::unexpected(
                ProtocolError{"revision_changed", "inventory changed; restart pagination"});
        }

        bool offset_ok = false;
        const qlonglong offset = object.value("offset").toString().toLongLong(&offset_ok);
        const int limit = object.value("limit").toInt(0);
        if (!offset_ok || offset < 0 || limit < 1 || limit > MAXIMUM_PAGE_SIZE) {
            return std::unexpected(ProtocolError{"invalid_cursor", "the cursor is malformed"});
        }

        ItemQuery query;
        query.offset = qsizetype(offset);
        query.limit = limit;
        query.tab_id = object.value("tab_id").toString();
        const QString kind = object.value("kind").toString();
        if (!kind.isEmpty()) {
            query.kind = ParseKind(kind);
            if (!query.kind) {
                return std::unexpected(
                    ProtocolError{"invalid_cursor", "the cursor location kind is invalid"});
            }
        }
        if (query.tab_id.isEmpty() != !query.kind.has_value()) {
            return std::unexpected(
                ProtocolError{"invalid_cursor", "the cursor tab and kind must be paired"});
        }
        return query;
    }

    std::expected<ItemQuery, ProtocolError> ParseItemQuery(const QJsonObject &params,
                                                           const QString &instance_id,
                                                           quint64 revision)
    {
        if (params.contains("cursor")) {
            if (!params.value("cursor").isString()
                || params.value("cursor").toString().isEmpty()) {
                return std::unexpected(
                    ProtocolError{"invalid_cursor", "cursor must be a non-empty string"});
            }
            if (params.contains("limit") || params.contains("tab_id") || params.contains("kind")) {
                return std::unexpected(ProtocolError{
                    "invalid_request",
                    "cursor cannot be combined with limit or location filters"});
            }
            return DecodeCursor(params.value("cursor").toString(), instance_id, revision);
        }

        ItemQuery query;
        if (params.contains("limit")) {
            if (!params.value("limit").isDouble()) {
                return std::unexpected(
                    ProtocolError{"invalid_request", "limit must be an integer"});
            }
            query.limit = params.value("limit").toInt(0);
            if (double(query.limit) != params.value("limit").toDouble() || query.limit < 1
                || query.limit > MAXIMUM_PAGE_SIZE) {
                return std::unexpected(ProtocolError{
                    "invalid_request",
                    QString("limit must be between 1 and %1").arg(MAXIMUM_PAGE_SIZE)});
            }
        }

        if (params.contains("tab_id")) {
            if (!params.value("tab_id").isString()
                || params.value("tab_id").toString().isEmpty()) {
                return std::unexpected(
                    ProtocolError{"invalid_request", "tab_id must be a non-empty string"});
            }
            query.tab_id = params.value("tab_id").toString();
        }
        QString kind;
        if (params.contains("kind")) {
            if (!params.value("kind").isString()
                || params.value("kind").toString().isEmpty()) {
                return std::unexpected(
                    ProtocolError{"invalid_request", "kind must be stash or character"});
            }
            kind = params.value("kind").toString();
        }
        if (!kind.isEmpty()) {
            query.kind = ParseKind(kind);
            if (!query.kind) {
                return std::unexpected(
                    ProtocolError{"invalid_request", "kind must be stash or character"});
            }
        }
        if (query.tab_id.isEmpty() != !query.kind.has_value()) {
            return std::unexpected(
                ProtocolError{"invalid_request", "tab_id and kind must be provided together"});
        }
        return query;
    }

    bool Matches(const Item &item, const ItemQuery &query, const LocationInventory &inventory)
    {
        if (query.tab_id.isEmpty()) {
            return true;
        }
        const ItemLocation &canonical = inventory.Canonical(item.location());
        return canonical.id() == query.tab_id && canonical.type() == *query.kind;
    }

} // namespace

ControlService::ControlService(QString application_version, QObject *parent)
    : QObject(parent)
    , m_application_version(std::move(application_version))
    , m_instance_id(QUuid::createUuid().toString(QUuid::WithoutBraces))
{}

void ControlService::SetNeedsLogin()
{
    if (!m_items_manager) {
        m_state = ServiceState::NeedsLogin;
    }
}

void ControlService::AttachSession(ItemsManager &items_manager,
                                   ItemsManagerWorker *worker,
                                   BuyoutManager &buyout_manager,
                                   const QString &account,
                                   const QString &league)
{
    m_items_manager = &items_manager;
    m_worker = worker;
    m_buyout_manager = &buyout_manager;
    m_account = account;
    m_league = league;
    m_state = ServiceState::LoadingCache;

    connect(&items_manager, &ItemsManager::ItemsRefreshed, this, [this](bool initial_refresh) {
        ++m_inventory_revision;
        if (initial_refresh) {
            m_state = ServiceState::Ready;
        }
    });
    connect(&items_manager, &ItemsManager::TabRefreshed, this, [this] { ++m_inventory_revision; });
    connect(&items_manager,
            &ItemsManager::ChildrenReconciled,
            this,
            [this] { ++m_inventory_revision; });
    connect(&buyout_manager,
            &BuyoutManager::BuyoutsChanged,
            this,
            [this] { ++m_inventory_revision; });
}

QJsonObject ControlService::Handle(const Request &request)
{
    if (request.command == "status") {
        return Success(request.request_id, Status());
    }
    if (request.command == "tabs") {
        return Tabs(request.request_id);
    }
    if (request.command == "items") {
        return Items(request.request_id, request.params);
    }
    if (request.command == "item") {
        return Item(request.request_id, request.params);
    }
    return Error(request.request_id,
                 "unknown_command",
                 QString("unknown control command: %1").arg(request.command));
}

QJsonObject ControlService::Status() const
{
    QJsonObject result{{"application_version", m_application_version},
                       {"instance_id", m_instance_id},
                       {"service_state", StateName(m_state)},
                       {"inventory_revision", QString::number(m_inventory_revision)}};

    if (m_items_manager) {
        result.insert("account", m_account);
        result.insert("league", m_league);
        result.insert("item_count", int(m_items_manager->items().size()));
        result.insert("location_count", int(m_items_manager->locationInventory().entries().size()));
    }

    if (m_worker) {
        switch (m_worker->updateReadiness()) {
        case ItemsManagerWorker::UpdateReadiness::Initializing:
            result.insert("refresh_state", "initializing");
            break;
        case ItemsManagerWorker::UpdateReadiness::Ready:
            result.insert("refresh_state", "idle");
            break;
        case ItemsManagerWorker::UpdateReadiness::Busy:
            result.insert("refresh_state", "updating");
            break;
        }
    } else {
        result.insert("refresh_state", "unavailable");
    }

    return result;
}

QJsonObject ControlService::Tabs(const QString &request_id) const
{
    if (m_state != ServiceState::Ready) {
        return NotReady(request_id);
    }

    std::map<LocationInventory::Key, qsizetype> counts;
    for (const auto &item : m_items_manager->items()) {
        ++counts[LocationInventory::KeyFor(item->location())];
    }

    QJsonArray tabs;
    for (const auto &[key, location] : m_items_manager->locationInventory().entries()) {
        tabs.append(ProjectTab(location, *m_buyout_manager, counts[key]));
    }
    return Success(request_id,
                   QJsonObject{{"instance_id", m_instance_id},
                               {"inventory_revision", QString::number(m_inventory_revision)},
                               {"tabs", tabs}});
}

QJsonObject ControlService::Items(const QString &request_id, const QJsonObject &params) const
{
    if (m_state != ServiceState::Ready) {
        return NotReady(request_id);
    }
    auto query = ParseItemQuery(params, m_instance_id, m_inventory_revision);
    if (!query) {
        return Error(request_id, query.error().code, query.error().message);
    }

    QJsonArray page;
    std::optional<qsizetype> next_offset;
    const ::Items &published = m_items_manager->items();
    if (query->offset > qsizetype(published.size())) {
        return Error(request_id, "invalid_cursor", "the cursor offset is outside the result set");
    }
    const auto &inventory = m_items_manager->locationInventory();
    for (qsizetype index = query->offset; index < qsizetype(published.size()); ++index) {
        const auto &item = published[size_t(index)];
        if (!Matches(*item, *query, inventory)) {
            continue;
        }
        if (page.size() == query->limit) {
            next_offset = index;
            break;
        }
        const ItemLocation &canonical = inventory.Canonical(item->location());
        page.append(ProjectItem(*item, canonical, m_buyout_manager->Get(*item)));
    }

    QJsonObject result{{"instance_id", m_instance_id},
                       {"inventory_revision", QString::number(m_inventory_revision)},
                       {"items", page}};
    if (query->tab_id.isEmpty()) {
        result.insert("total", QString::number(published.size()));
    }
    if (next_offset) {
        ItemQuery next = *query;
        next.offset = *next_offset;
        result.insert("next_cursor", EncodeCursor(m_instance_id, m_inventory_revision, next));
    } else {
        result.insert("next_cursor", QJsonValue::Null);
    }
    return Success(request_id, result);
}

QJsonObject ControlService::Item(const QString &request_id, const QJsonObject &params) const
{
    if (m_state != ServiceState::Ready) {
        return NotReady(request_id);
    }
    const QJsonValue id = params.value("id");
    if (!id.isString() || id.toString().isEmpty()) {
        return Error(request_id, "invalid_request", "id must be a non-empty string");
    }

    for (const auto &item : m_items_manager->items()) {
        if (item->id() == id.toString()) {
            const ItemLocation &canonical = m_items_manager->locationInventory().Canonical(
                item->location());
            return Success(request_id,
                           QJsonObject{{"instance_id", m_instance_id},
                                       {"inventory_revision",
                                        QString::number(m_inventory_revision)},
                                       {"item",
                                        ProjectItem(*item,
                                                    canonical,
                                                    m_buyout_manager->Get(*item))}});
        }
    }
    return Error(request_id, "item_not_found", "no published item has that id");
}

QJsonObject ControlService::NotReady(const QString &request_id) const
{
    return Error(request_id,
                 "not_ready",
                 QString("inventory viewing is unavailable while the service is %1")
                     .arg(StateName(m_state)));
}

QString ControlService::StateName(ServiceState state)
{
    switch (state) {
    case ServiceState::Starting:
        return "starting";
    case ServiceState::NeedsLogin:
        return "needs_login";
    case ServiceState::LoadingCache:
        return "loading_cache";
    case ServiceState::Ready:
        return "ready";
    }
    return "starting";
}

} // namespace control
