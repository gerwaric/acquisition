// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QNetworkReply>
#include <QObject>
#include <QUrlQuery>

#include <expected>
#include <map>
#include <memory>
#include <vector>

#include "buyout.h"
#include "itemlocation.h"
#include "poe/types/website/webstashtab.h"
#include "ratelimit/fetcherror.h"
#include "util/programstate.h"

class QSettings;

class Application;
class BuyoutManager;
class DataStore;
class ItemsManager;
class NetworkManager;
class PoeApiClient;

class Shop : public QObject
{
    Q_OBJECT
public:
    explicit Shop(QSettings &settings,
                  NetworkManager &network_manager,
                  PoeApiClient &api,
                  DataStore &datastore,
                  ItemsManager &items_manager,
                  BuyoutManager &buyout_manager);
    void SetThread(const QStringList &threads);
    void SetAutoUpdate(bool update);
    void SetShopTemplate(const QString &shop_template);
    void CopyToClipboard();
    void ExpireShopData();
    void SubmitShopToForum(bool force = false);
    bool auto_update() const { return m_auto_update; }

    QStringList threads() const { return m_threads; }
    QStringList shop_data() const { return m_shop_data; }
    QString shop_template() const { return m_shop_template; }
    // True when the preview/clipboard cache was rendered from an input
    // revision older than the current one (items-pipeline M2, D8). Deltas
    // deliberately do not advance the input revision (R4-2), so this is
    // snapshot-granular, not delta-granular.
    bool shop_data_outdated() const { return m_cache_revision < m_input_revision; }

public slots:
    void OnEditPageFinished();
    void OnShopSubmitted(QUrlQuery query, QNetworkReply *reply);

signals:
    void StashesIndexed();
    void StatusUpdate(ProgramState state, const QString &status);
    void UserWarning(const QString &message);

private:
    // One forum submission job (items-pipeline M2, D8/R2-1): its input is
    // captured BY VALUE at request time — the postable items' identity,
    // location, and buyout fields plus every other output-affecting input
    // (template, realm/league, target thread list) — and is immutable from
    // then on. The job also owns all of its transport state (stash index,
    // rendered data and hash, request counter), so nothing it reads can be
    // mutated by streamed deltas, local edits, or a later job. Value
    // capture, not retained shared_ptrs: a successful FinishUpdate rebases
    // the shared Item objects in place, so a pointer capture would mutate
    // under the submission.
    struct ShopJob
    {
        struct CapturedItem
        {
            QString id;
            QString pretty_name;
            ItemLocation location;
            Buyout buyout;
        };
        // Immutable capture.
        std::vector<CapturedItem> items;
        QString shop_template;
        QString realm;
        QString league;
        QStringList threads;
        bool force{false};
        quint64 input_revision{0};
        // Job-local transport state.
        std::map<QString, unsigned int> tab_index;
        QStringList shop_data;
        QString shop_hash;
        qsizetype requests_completed{0};
    };

    std::unique_ptr<ShopJob> CaptureJob(bool force) const;
    // Every submission renders from its capture (M2 D8): correctness never
    // depends on preview-cache freshness.
    void RenderJob(ShopJob &job) const;
    void PublishPreviewCache(const ShopJob &job);

    void UpdateStashIndex();
    void OnStashIndexReceived(
        const std::expected<poe::WebStashListWrapper, RateLimit::FetchError> &result);
    void OnStashIndexUpdated();

    void SubmitSingleShop();
    void SubmitNextShop(const QString &title, const QString &hash);
    QString ShopEditUrl(qsizetype idx);
    static QString SpoilerBuyout(const Buyout &bo);

    QSettings &m_settings;
    NetworkManager &m_network_manager;
    PoeApiClient &m_api;
    DataStore &m_datastore;
    ItemsManager &m_items_manager;
    BuyoutManager &m_buyout_manager;

    QStringList m_threads;
    QString m_shop_template;
    bool m_auto_update;

    // The single active submission job (M2 D8): at most one. Stage 8 of the
    // M2 sequence adds the waiting automatic capture and its drop/drain
    // transitions; until then a busy shop refuses, as before.
    std::unique_ptr<ShopJob> m_active_job;

    // Preview/clipboard cache, published only by rendered jobs that are not
    // older than the cache already here. Monotonic revisions replace the old
    // single outdated flag: ExpireShopData() advances the input revision;
    // rendering job N can mark only N's revision clean — never a newer one.
    QStringList m_shop_data;
    quint64 m_input_revision{1};
    quint64 m_cache_revision{0};

    static const QRegularExpression error_regex;
    static const QRegularExpression ratelimit_regex;
};
