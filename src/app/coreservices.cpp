// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "app/coreservices.h"

#include <QApplication>

#include <qtkeychain/keychain.h>

#include "app/usersettings.h"
#include "datastore/keychainstore.h"
#include "imagecache.h"
#include "repoe/repoe.h"
#include "util/deref.h"
#include "util/json_utils.h"
#include "util/networkmanager.h"
#include "util/oauthmanager.h"
#include "util/spdlog_qt.h" // IWYU pragma: keep
#include "util/updatechecker.h"

app::CoreServices::CoreServices(app::UserSettings &settings)
    : m_settings(settings)
{
    spdlog::debug("CoreServices: setting up core services.");

    auto dir = m_settings.userDir();
    const QString image_cache_dir = dir.filePath("image_cache");
    const QString repoe_cache_dir = dir.filePath("repoe_cache");

    m_keychain = std::make_unique<KeychainStore>();
    m_network_manager = std::make_unique<NetworkManager>();
    m_update_checker = std::make_unique<UpdateChecker>(*m_network_manager, settings);
    m_oauth_manager = std::make_unique<OAuthManager>(*m_network_manager);
    m_image_cache = std::make_unique<ImageCache>(*m_network_manager, image_cache_dir);
    m_repoe = std::make_unique<RePoE>(*m_network_manager, repoe_cache_dir);

    connect(m_network_manager.get(),
            &NetworkManager::sessionIdChanged,
            this,
            &app::CoreServices::setSessionId);

    connect(m_oauth_manager.get(),
            &OAuthManager::grantAccess,
            this,
            &app::CoreServices::accessGranted);

    connect(m_update_checker.get(),
            &UpdateChecker::UpdateAvailable,
            m_update_checker.get(),
            &UpdateChecker::AskUserToUpdate);
}

app::CoreServices::~CoreServices() {};

void app::CoreServices::start()
{
    spdlog::debug("CoreServices: starting core services.");

    // TODO: TBD: SaveDataOnNewVersion();

    // Load an oauth token if it exists.
    auto oauth_reply = m_keychain->load("oauth_token");
    connect(oauth_reply,
            &KeychainReply::loaded,
            this,
            [this](const QString &key, const QByteArray &data) {
                spdlog::info("CoreServices: oauth token loaded from keychain: '{}'", key);
                const auto token = read_json<OAuthToken>(data);
                if (token) {
                    m_oauth_manager->setToken(*token);
                }
            });

    // Load a session cookie if it exists.
    auto posessid_reply = m_keychain->load("poesessid");
    connect(posessid_reply,
            &KeychainReply::loaded,
            this,
            [this](const QString &key, const QByteArray &data) {
                spdlog::info("CoreServices: poesessid loaded from Keychain: '{}'", key);
                m_network_manager->setPoesessid(data);
            });

    // Start the process of fetching RePoE data.
    spdlog::debug("CoreServices: initializing RePoE");
    m_repoe->start();

    // Start the initial check for updates.
    spdlog::debug("CoreServices: checking for application updates");
    m_update_checker->setLastSkippedUpdates(m_settings.lastSkippedRelease(),
                                            m_settings.lastSkippedPreRelease());
    m_update_checker->CheckForUpdates();
}

KeychainStore &app::CoreServices::keychain()
{
    return deref(m_keychain, "CoreServices::keychain()");
}

NetworkManager &app::CoreServices::network_manager()
{
    return deref(m_network_manager, "CoreServices::network_manager()");
}

UpdateChecker &app::CoreServices::update_checker()
{
    return deref(m_update_checker, "CoreServices::update_checker()");
}

OAuthManager &app::CoreServices::oauth_manager()
{
    return deref(m_oauth_manager, "CoreServices::oauth_manager()");
}

RePoE &app::CoreServices::repoe()
{
    return deref(m_repoe, "CoreServices::repoe()");
}

ImageCache &app::CoreServices::image_cache()
{
    return deref(m_image_cache, "CoreServices::image_cache()");
}

void app::CoreServices::setSessionId(const QByteArray &poesessid)
{
    const auto key = "poesessid/" + m_settings.username();
    m_keychain->save(key, poesessid);
}

void app::CoreServices::accessGranted(const OAuthToken &token)
{
    // Update the network manager.
    m_network_manager->setBearerToken(token.access_token);

    // Save the oauth_token securely.
    const auto key = "oauth_token/" + token.username;
    const auto data = write_json(token);
    m_keychain->save(key, data);
}
