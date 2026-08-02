// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Auro

#include "control/controlservice.h"

#include <QCryptographicHash>
#include <QDateTime>
#include <QJsonArray>
#include <QJsonDocument>
#include <QMessageAuthenticationCode>
#include <QMetaObject>
#include <QUuid>

#include <algorithm>
#include <exception>
#include <limits>
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
    constexpr qsizetype MAXIMUM_FILTER_LENGTH = 1024;
    constexpr qsizetype MAXIMUM_CURSOR_PAYLOAD = 32 * 1024;
    constexpr qsizetype MAXIMUM_PAGE_SCAN = 10'000;
    // Request ids are capped at 128 characters by DecodeRequest; 64 KiB also
    // covers the fixed response envelope and authenticated continuation cursor.
    constexpr qsizetype RESPONSE_CONTENT_BUDGET = MAX_RESPONSE_BYTES - 64 * 1024;

    QString ErrorKindName(RateLimit::FetchError::Kind kind)
    {
        switch (kind) {
        case RateLimit::FetchError::Kind::Network:
            return "network";
        case RateLimit::FetchError::Kind::Http:
            return "http";
        case RateLimit::FetchError::Kind::Parse:
            return "parse";
        case RateLimit::FetchError::Kind::Protocol:
            return "protocol";
        case RateLimit::FetchError::Kind::RateLimited:
            return "rate_limited";
        case RateLimit::FetchError::Kind::Internal:
            return "internal";
        case RateLimit::FetchError::Kind::Canceled:
            return "canceled";
        }
        return "internal";
    }

    QJsonObject ProjectError(const RateLimit::FetchError &error)
    {
        return QJsonObject{{"kind", ErrorKindName(error.kind)},
                           {"message", error.message},
                           {"http_status", error.http_status},
                           {"attempts", error.attempts}};
    }

    QJsonObject ProjectOutcome(const RefreshOutcome &outcome)
    {
        if (const auto *completed = std::get_if<CompletedRefresh>(&outcome)) {
            std::vector<SkippedSource> skipped_sources = completed->skipped;
            std::sort(skipped_sources.begin(), skipped_sources.end(), [](const auto &left, const auto &right) {
                return left.source < right.source;
            });
            QJsonArray skipped;
            for (const auto &source : skipped_sources) {
                skipped.append(QJsonObject{
                    {"source",
                     QJsonObject{{"kind",
                                  source.source.type == ItemLocationType::STASH ? "stash"
                                                                               : "character"},
                                 {"fetch_source_id", source.source.fetch_id}}},
                    {"error", ProjectError(source.error)}});
            }
            return QJsonObject{{"kind", "completed"},
                               {"clean", skipped_sources.empty()},
                               {"skipped", skipped}};
        }
        const auto &failed = std::get<FailedRefresh>(outcome);
        return QJsonObject{{"kind", "failed"}, {"error", ProjectError(failed.error)}};
    }

    struct ItemQuery
    {
        int limit{DEFAULT_PAGE_SIZE};
        std::optional<FetchSourceKey> source;
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

    std::optional<QString> SignCursor(const QJsonObject &object, const QByteArray &key)
    {
        if (!JsonSizeWithinLimit(object, MAXIMUM_CURSOR_PAYLOAD)) {
            return std::nullopt;
        }
        const QByteArray payload = QJsonDocument(object).toJson(QJsonDocument::Compact);
        const QByteArray encoded = payload.toBase64(QByteArray::Base64UrlEncoding
                                                    | QByteArray::OmitTrailingEquals);
        const QByteArray signature = QMessageAuthenticationCode::hash(
                                         payload,
                                         key,
                                         QCryptographicHash::Sha256)
                                         .toHex();
        return QString::fromLatin1(encoded + '.' + signature);
    }

    std::expected<QJsonObject, ProtocolError> VerifyCursor(const QString &encoded,
                                                           const QByteArray &key)
    {
        const QByteArray token = encoded.toLatin1();
        const qsizetype separator = token.indexOf('.');
        if (separator <= 0 || separator != token.lastIndexOf('.')) {
            return std::unexpected(ProtocolError{"invalid_cursor", "the cursor is malformed"});
        }
        const QByteArray payload = QByteArray::fromBase64(token.first(separator),
                                                          QByteArray::Base64UrlEncoding);
        const QByteArray actual = token.sliced(separator + 1);
        const QByteArray expected = QMessageAuthenticationCode::hash(
                                        payload,
                                        key,
                                        QCryptographicHash::Sha256)
                                        .toHex();
        if (actual != expected) {
            return std::unexpected(ProtocolError{"invalid_cursor", "the cursor is malformed"});
        }
        auto decoded = DecodeObject(payload);
        if (!decoded) {
            return std::unexpected(ProtocolError{"invalid_cursor", "the cursor is malformed"});
        }
        return *decoded;
    }

    std::optional<QString> EncodeCursor(const QString &instance_id,
                         quint64 revision,
                         const ItemQuery &query,
                         const QByteArray &key)
    {
        Q_ASSERT(query.source);
        return SignCursor(QJsonObject{{"type", "items"},
                                      {"instance_id", instance_id},
                                      {"revision", QString::number(revision)},
                                      {"source_kind", KindName(query.source->type)},
                                      {"source_id", query.source->fetch_id},
                                      {"offset", QString::number(query.offset)},
                                      {"limit", query.limit},
                                      {"tab_id", query.tab_id},
                                      {"kind", KindName(query.kind)}},
                          key);
    }

    std::expected<ItemQuery, ProtocolError> DecodeCursor(const QString &encoded,
                                                         const QString &instance_id,
                                                         quint64 revision,
                                                         const QByteArray &key)
    {
        auto decoded = VerifyCursor(encoded, key);
        if (!decoded) {
            return std::unexpected(decoded.error());
        }
        const QJsonObject &object = *decoded;
        if (object.value("type").toString() != "items") {
            return std::unexpected(ProtocolError{"invalid_cursor", "the cursor is malformed"});
        }
        if (object.value("instance_id").toString() != instance_id
            || object.value("revision").toString() != QString::number(revision)) {
            return std::unexpected(
                ProtocolError{"revision_changed", "inventory changed; restart pagination"});
        }

        bool offset_ok = false;
        const qlonglong offset = object.value("offset").toString().toLongLong(&offset_ok);
        const int limit = object.value("limit").toInt(0);
        const QString source_id = object.value("source_id").toString();
        const auto source_kind = ParseKind(object.value("source_kind").toString());
        if (!offset_ok || offset < 0
            || qulonglong(offset) > qulonglong(std::numeric_limits<qsizetype>::max())
            || limit < 1 || limit > MAXIMUM_PAGE_SIZE || source_id.isEmpty()
            || !source_kind) {
            return std::unexpected(ProtocolError{"invalid_cursor", "the cursor is malformed"});
        }

        ItemQuery query;
        query.source = FetchSourceKey{*source_kind, source_id};
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
        if (query.tab_id.size() > MAXIMUM_FILTER_LENGTH) {
            return std::unexpected(
                ProtocolError{"invalid_cursor", "the cursor tab filter is too long"});
        }
        if (query.tab_id.isEmpty() != !query.kind.has_value()) {
            return std::unexpected(
                ProtocolError{"invalid_cursor", "the cursor tab and kind must be paired"});
        }
        return query;
    }

    std::expected<ItemQuery, ProtocolError> ParseItemQuery(const QJsonObject &params,
                                                           const QString &instance_id,
                                                           quint64 revision,
                                                           const QByteArray &key)
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
            return DecodeCursor(params.value("cursor").toString(), instance_id, revision, key);
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
            if (query.tab_id.size() > MAXIMUM_FILTER_LENGTH) {
                return std::unexpected(ProtocolError{
                    "invalid_request",
                    QString("tab_id must not exceed %1 characters")
                        .arg(MAXIMUM_FILTER_LENGTH)});
            }
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
    , m_cursor_key(QUuid::createUuid().toRfc4122())
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
    connect(&items_manager,
            &ItemsManager::StatusUpdate,
            this,
            [this](ProgramState, const QString &status) {
                if (m_active_refresh_id.isEmpty()) {
                    return;
                }
                const auto job = std::find_if(m_refresh_jobs.begin(),
                                              m_refresh_jobs.end(),
                                              [this](const RefreshJob &candidate) {
                                                  return candidate.id == m_active_refresh_id;
                                              });
                if (job != m_refresh_jobs.end() && job->state == "running") {
                    job->progress = status;
                }
            });
    connect(&items_manager,
            &ItemsManager::RefreshFinished,
            this,
            &ControlService::OnRefreshFinished);
    connect(&items_manager, &QObject::destroyed, this, &ControlService::OnSessionDestroyed);
}

void ControlService::ConfigureRefresh(std::function<RefreshReadiness()> readiness,
                                      std::function<void()> start)
{
    m_refresh_readiness = std::move(readiness);
    m_start_refresh = std::move(start);
}

QJsonObject ControlService::Handle(const Request &request)
{
    if (request.command == "status") {
        return Success(request.request_id, Status());
    }
    if (request.command == "tabs") {
        return Tabs(request.request_id, request.params);
    }
    if (request.command == "items") {
        return Items(request.request_id, request.params);
    }
    if (request.command == "item") {
        return Item(request.request_id, request.params);
    }
    if (request.command == "refresh.start") {
        return StartRefresh(request.request_id);
    }
    if (request.command == "refresh.status") {
        return RefreshStatus(request.request_id, request.params);
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
        result.insert("item_count", int(m_items_manager->itemCount()));
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

    if (m_active_refresh_id.isEmpty()) {
        result.insert("active_refresh_id", QJsonValue::Null);
    } else {
        result.insert("active_refresh_id", m_active_refresh_id);
    }
    const auto latest_terminal = std::find_if(m_refresh_jobs.rbegin(),
                                              m_refresh_jobs.rend(),
                                              [](const RefreshJob &job) {
                                                  return job.state == "completed"
                                                         || job.state == "failed";
                                              });
    if (latest_terminal == m_refresh_jobs.rend()) {
        result.insert("latest_refresh", QJsonValue::Null);
    } else {
        result.insert("latest_refresh", RefreshJobJson(*latest_terminal));
    }

    return result;
}

QJsonObject ControlService::Tabs(const QString &request_id, const QJsonObject &params) const
{
    if (m_state != ServiceState::Ready) {
        return NotReady(request_id);
    }

    int limit = DEFAULT_PAGE_SIZE;
    std::optional<LocationInventory::Key> start;
    if (params.contains("cursor")) {
        if (!params.value("cursor").isString() || params.value("cursor").toString().isEmpty()
            || params.contains("limit")) {
            return Error(request_id,
                         "invalid_request",
                         "tab cursor must be a non-empty string and cannot be combined with limit");
        }
        auto cursor = VerifyCursor(params.value("cursor").toString(), m_cursor_key);
        if (!cursor) {
            return Error(request_id, cursor.error().code, cursor.error().message);
        }
        const auto kind = ParseKind(cursor->value("source_kind").toString());
        const QString id = cursor->value("source_id").toString();
        limit = cursor->value("limit").toInt(0);
        if (cursor->value("type").toString() != "tabs") {
            return Error(request_id, "invalid_cursor", "the cursor is malformed");
        }
        if (cursor->value("instance_id").toString() != m_instance_id
            || cursor->value("revision").toString() != QString::number(m_inventory_revision)) {
            return Error(request_id, "revision_changed", "inventory changed; restart pagination");
        }
        if (!kind || id.isEmpty() || limit < 1 || limit > MAXIMUM_PAGE_SIZE) {
            return Error(request_id, "invalid_cursor", "the cursor is malformed");
        }
        start = LocationInventory::Key{*kind, id};
    } else if (params.contains("limit")) {
        if (!params.value("limit").isDouble()) {
            return Error(request_id, "invalid_request", "limit must be an integer");
        }
        limit = params.value("limit").toInt(0);
        if (double(limit) != params.value("limit").toDouble() || limit < 1
            || limit > MAXIMUM_PAGE_SIZE) {
            return Error(request_id,
                         "invalid_request",
                         QString("limit must be between 1 and %1").arg(MAXIMUM_PAGE_SIZE));
        }
    }

    const auto &counts = m_items_manager->itemCountsByLocation();
    const auto &entries = m_items_manager->locationInventory().entries();
    auto entry = start ? entries.find(*start) : entries.begin();
    if (entry == entries.end() && start) {
        return Error(request_id, "invalid_cursor", "the cursor location is not published");
    }

    QJsonArray tabs;
    qsizetype tabs_bytes = 2;
    std::optional<LocationInventory::Key> next;
    for (; entry != entries.end(); ++entry) {
        if (tabs.size() == limit) {
            next = entry->first;
            break;
        }
        const auto count = counts.find(entry->first);
        const QJsonObject projected = ProjectTab(entry->second,
                                                  *m_buyout_manager,
                                                  count == counts.end() ? 0 : count->second);
        const auto projected_bytes = JsonSizeWithinLimit(projected, RESPONSE_CONTENT_BUDGET);
        if (!projected_bytes
            || tabs_bytes + *projected_bytes + (tabs.isEmpty() ? 0 : 1)
                   > RESPONSE_CONTENT_BUDGET) {
            if (tabs.isEmpty()) {
                return Error(request_id,
                             "tab_too_large",
                             "one location exceeds the control response limit");
            }
            next = entry->first;
            break;
        }
        tabs_bytes += *projected_bytes + (tabs.isEmpty() ? 0 : 1);
        tabs.append(projected);
    }

    QJsonObject result{{"instance_id", m_instance_id},
                       {"inventory_revision", QString::number(m_inventory_revision)},
                       {"total", QString::number(entries.size())},
                       {"tabs", tabs}};
    if (next) {
        const auto cursor = SignCursor(
            QJsonObject{{"type", "tabs"},
                        {"instance_id", m_instance_id},
                        {"revision", QString::number(m_inventory_revision)},
                        {"source_kind", KindName(next->first)},
                        {"source_id", next->second},
                        {"limit", limit}},
            m_cursor_key);
        if (!cursor) {
            return Error(request_id,
                         "cursor_too_large",
                         "the next location identity exceeds the cursor limit");
        }
        result.insert("next_cursor", *cursor);
    } else {
        result.insert("next_cursor", QJsonValue::Null);
    }
    return Success(request_id, result);
}

QJsonObject ControlService::Items(const QString &request_id, const QJsonObject &params) const
{
    if (m_state != ServiceState::Ready) {
        return NotReady(request_id);
    }
    auto query = ParseItemQuery(params, m_instance_id, m_inventory_revision, m_cursor_key);
    if (!query) {
        return Error(request_id, query.error().code, query.error().message);
    }

    QJsonArray page;
    qsizetype page_bytes = 2;
    std::optional<FetchSourceKey> next_source;
    qsizetype next_offset = 0;
    const qsizetype published_size = qsizetype(m_items_manager->itemCount());
    const auto &buckets = m_items_manager->itemBuckets();
    auto bucket = query->source ? buckets.find(*query->source) : buckets.begin();
    if (bucket == buckets.end() && query->source) {
        return Error(request_id, "invalid_cursor", "the cursor source is not published");
    }

    const auto &inventory = m_items_manager->locationInventory();
    qsizetype examined = 0;
    bool stopped = false;
    for (; bucket != buckets.end(); ++bucket) {
        const qsizetype bucket_size = qsizetype(bucket->second.size());
        const qsizetype first = query->source && bucket->first == *query->source ? query->offset : 0;
        if (first > bucket_size) {
            return Error(request_id,
                         "invalid_cursor",
                         "the cursor offset is outside its source");
        }
        for (qsizetype index = first; index < bucket_size; ++index) {
            if (examined == MAXIMUM_PAGE_SCAN || page.size() == query->limit) {
                next_source = bucket->first;
                next_offset = index;
                stopped = true;
                break;
            }
            const auto &item = bucket->second[size_t(index)];
            if (Matches(*item, *query, inventory)) {
                const ItemLocation &canonical = inventory.Canonical(item->location());
                const qsizetype separator_bytes = page.isEmpty() ? 0 : 1;
                const qsizetype remaining_bytes
                    = RESPONSE_CONTENT_BUDGET - page_bytes - separator_bytes;
                const auto projected = ProjectItem(*item,
                                                   canonical,
                                                   m_buyout_manager->Get(*item),
                                                   remaining_bytes);
                if (!projected) {
                    if (page.isEmpty()) {
                        return Error(request_id,
                                     "item_too_large",
                                     QString("item %1 exceeds the control response limit")
                                         .arg(item->id()));
                    }
                    next_source = bucket->first;
                    next_offset = index;
                    stopped = true;
                    break;
                }
                page_bytes += projected->bytes + separator_bytes;
                page.append(projected->value);
            }
            ++examined;
        }
        if (stopped) {
            break;
        }
        query->source.reset();
    }

    QJsonObject result{{"instance_id", m_instance_id},
                       {"inventory_revision", QString::number(m_inventory_revision)},
                       {"items", page}};
    if (query->tab_id.isEmpty()) {
        result.insert("total", QString::number(published_size));
    }
    if (next_source) {
        ItemQuery next = *query;
        next.source = *next_source;
        next.offset = next_offset;
        const auto cursor = EncodeCursor(m_instance_id,
                                         m_inventory_revision,
                                         next,
                                         m_cursor_key);
        if (!cursor) {
            return Error(request_id,
                         "cursor_too_large",
                         "the next source identity exceeds the cursor limit");
        }
        result.insert("next_cursor", *cursor);
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

    const auto item = m_items_manager->findItemById(id.toString());
    if (!item) {
        return Error(request_id, "item_not_found", "no published item has that id");
    }
    const ItemLocation &canonical = m_items_manager->locationInventory().Canonical(
        item->location());
    // RESPONSE_CONTENT_BUDGET already excludes 64 KiB for the Success
    // envelope, instance/revision fields, and the bounded request id.
    const auto projected = ProjectItem(*item,
                                       canonical,
                                       m_buyout_manager->Get(*item),
                                       RESPONSE_CONTENT_BUDGET);
    if (!projected) {
        return Error(request_id,
                     "item_too_large",
                     QString("item %1 exceeds the control response limit").arg(item->id()));
    }
    return Success(request_id,
                   QJsonObject{{"instance_id", m_instance_id},
                               {"inventory_revision", QString::number(m_inventory_revision)},
                               {"item", projected->value}});
}

QJsonObject ControlService::StartRefresh(const QString &request_id)
{
    const auto previous = std::find_if(m_refresh_start_responses.begin(),
                                       m_refresh_start_responses.end(),
                                       [&request_id](const auto &entry) {
                                           return entry.first == request_id;
                                       });
    if (previous != m_refresh_start_responses.end()) {
        return previous->second;
    }
    const auto remember = [this, &request_id](QJsonObject response) {
        m_refresh_start_responses.emplace_back(request_id, response);
        while (qsizetype(m_refresh_start_responses.size()) > MAXIMUM_REFRESH_HISTORY) {
            m_refresh_start_responses.pop_front();
        }
        return response;
    };

    if (m_state != ServiceState::Ready) {
        return remember(Error(request_id,
                              "not_ready",
                              QString("refresh control is unavailable while the service is %1")
                                  .arg(StateName(m_state))));
    }
    if (!m_refresh_readiness || !m_start_refresh) {
        return remember(
            Error(request_id, "refresh_unavailable", "refresh control is not configured"));
    }
    const auto existing = std::find_if(m_refresh_jobs.begin(),
                                       m_refresh_jobs.end(),
                                       [&request_id](const RefreshJob &job) {
                                           return job.id == request_id;
                                       });
    if (existing != m_refresh_jobs.end()) {
        return remember(Success(request_id,
                                QJsonObject{{"accepted", true},
                                            {"state", existing->state},
                                            {"operation_id", existing->id}}));
    }
    if (!m_active_refresh_id.isEmpty()) {
        return remember(Success(request_id,
                                QJsonObject{{"accepted", false},
                                            {"state", "busy"},
                                            {"active_refresh_id", m_active_refresh_id}}));
    }

    switch (m_refresh_readiness()) {
    case RefreshReadiness::Initializing:
        return remember(
            Error(request_id, "not_ready", "inventory initialization is still running"));
    case RefreshReadiness::Busy:
        return remember(
            Success(request_id, QJsonObject{{"accepted", false}, {"state", "busy"}}));
    case RefreshReadiness::Ready:
        break;
    }

    RefreshJob job;
    // The request id is the idempotency key. A client can retry an ambiguous
    // start with the same envelope and recover this operation without starting
    // a second refresh.
    job.id = request_id;
    job.state = "queued";
    job.started_at = QDateTime::currentDateTimeUtc().toString(Qt::ISODateWithMs);
    m_active_refresh_id = job.id;
    m_refresh_jobs.push_back(job);
    const QString operation_id = job.id;

    QMetaObject::invokeMethod(
        this,
        [this, operation_id] {
            if (m_active_refresh_id != operation_id) {
                return;
            }
            const auto job = std::find_if(m_refresh_jobs.begin(),
                                          m_refresh_jobs.end(),
                                          [&operation_id](const RefreshJob &candidate) {
                                              return candidate.id == operation_id;
                                          });
            if (job == m_refresh_jobs.end()) {
                return;
            }
            if (m_state != ServiceState::Ready || !m_refresh_readiness || !m_start_refresh) {
                job->state = "failed";
                job->finished_at = QDateTime::currentDateTimeUtc().toString(Qt::ISODateWithMs);
                job->outcome = QJsonObject{
                    {"kind", "not_started"},
                    {"error",
                     QJsonObject{{"kind", "interrupted"},
                                 {"message", "the application session ended before dispatch"}}}};
                m_active_refresh_id.clear();
                TrimRefreshHistory();
                return;
            }
            if (m_refresh_readiness() != RefreshReadiness::Ready) {
                job->state = "failed";
                job->finished_at = QDateTime::currentDateTimeUtc().toString(Qt::ISODateWithMs);
                job->outcome = QJsonObject{
                    {"kind", "not_started"},
                    {"error",
                     QJsonObject{{"kind", "busy"},
                                 {"message", "another refresh started before dispatch"}}}};
                m_active_refresh_id.clear();
                TrimRefreshHistory();
                return;
            }
            job->state = "running";
            try {
                m_start_refresh();
            } catch (const std::exception &error) {
                job->state = "failed";
                job->finished_at = QDateTime::currentDateTimeUtc().toString(Qt::ISODateWithMs);
                job->outcome = QJsonObject{
                    {"kind", "failed"},
                    {"error", QJsonObject{{"kind", "internal"}, {"message", error.what()}}}};
                m_active_refresh_id.clear();
                TrimRefreshHistory();
            } catch (...) {
                job->state = "failed";
                job->finished_at = QDateTime::currentDateTimeUtc().toString(Qt::ISODateWithMs);
                job->outcome = QJsonObject{
                    {"kind", "failed"},
                    {"error",
                     QJsonObject{{"kind", "internal"},
                                 {"message", "refresh dispatch failed unexpectedly"}}}};
                m_active_refresh_id.clear();
                TrimRefreshHistory();
            }
        },
        Qt::QueuedConnection);

    return remember(Success(request_id,
                            QJsonObject{{"accepted", true},
                                        {"state", "queued"},
                                        {"operation_id", operation_id}}));
}

QJsonObject ControlService::RefreshStatus(const QString &request_id,
                                          const QJsonObject &params) const
{
    const QJsonValue id = params.value("operation_id");
    if (!id.isString() || id.toString().isEmpty()) {
        return Error(request_id,
                     "invalid_request",
                     "operation_id must be a non-empty string");
    }
    const auto job = std::find_if(m_refresh_jobs.begin(),
                                  m_refresh_jobs.end(),
                                  [&id](const RefreshJob &candidate) {
                                      return candidate.id == id.toString();
                                  });
    if (job == m_refresh_jobs.end()) {
        return Error(request_id, "operation_not_found", "the refresh operation is not retained");
    }
    return Success(request_id, QJsonObject{{"operation", RefreshJobJson(*job)}});
}

void ControlService::OnRefreshFinished(const RefreshOutcome &outcome)
{
    if (m_active_refresh_id.isEmpty()) {
        return;
    }
    const auto job = std::find_if(m_refresh_jobs.begin(),
                                  m_refresh_jobs.end(),
                                  [this](const RefreshJob &candidate) {
                                      return candidate.id == m_active_refresh_id;
                                  });
    if (job == m_refresh_jobs.end()) {
        m_active_refresh_id.clear();
        return;
    }
    // Readiness stays Busy throughout the worker's terminal fan-out, so a new
    // operation cannot normally be admitted under an older terminal signal.
    // Requiring running state also makes that correlation structural if signal
    // delivery or admission changes later.
    if (job->state != "running") {
        return;
    }

    job->outcome = ProjectOutcome(outcome);
    job->state = std::holds_alternative<CompletedRefresh>(outcome) ? "completed" : "failed";
    job->finished_at = QDateTime::currentDateTimeUtc().toString(Qt::ISODateWithMs);
    m_active_refresh_id.clear();
    TrimRefreshHistory();
}

void ControlService::OnSessionDestroyed()
{
    if (!m_active_refresh_id.isEmpty()) {
        const auto job = std::find_if(m_refresh_jobs.begin(),
                                      m_refresh_jobs.end(),
                                      [this](const RefreshJob &candidate) {
                                          return candidate.id == m_active_refresh_id;
                                      });
        if (job != m_refresh_jobs.end()) {
            job->state = "failed";
            job->finished_at = QDateTime::currentDateTimeUtc().toString(Qt::ISODateWithMs);
            job->outcome = QJsonObject{
                {"kind", "failed"},
                {"error",
                 QJsonObject{{"kind", "interrupted"},
                             {"message", "the application session ended"}}}};
        }
    }
    m_active_refresh_id.clear();
    m_items_manager.clear();
    m_worker.clear();
    m_buyout_manager.clear();
    m_refresh_readiness = {};
    m_start_refresh = {};
    m_state = ServiceState::NeedsLogin;
    TrimRefreshHistory();
}

void ControlService::TrimRefreshHistory()
{
    while (qsizetype(m_refresh_jobs.size()) > MAXIMUM_REFRESH_HISTORY) {
        m_refresh_jobs.pop_front();
    }
}

QJsonObject ControlService::RefreshJobJson(const RefreshJob &job)
{
    QJsonObject result{{"operation_id", job.id},
                       {"state", job.state},
                       {"progress", job.progress},
                       {"started_at", job.started_at}};
    if (job.finished_at.isEmpty()) {
        result.insert("finished_at", QJsonValue::Null);
    } else {
        result.insert("finished_at", job.finished_at);
    }
    if (job.outcome) {
        result.insert("outcome", *job.outcome);
    } else {
        result.insert("outcome", QJsonValue::Null);
    }
    return result;
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
