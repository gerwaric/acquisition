// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QDir>
#include <QObject>
#include <QPointer>

class KeychainStore;
class NetworkManager;
class UpdateChecker;
class OAuthManager;
struct OAuthToken;
class RePoE;
class ImageCache;

namespace app {

    class UserSettings;

    class CoreServices : public QObject
    {
        Q_OBJECT
    public:
        explicit CoreServices(app::UserSettings &setting);
        ~CoreServices();

        void start();

        KeychainStore &keychain();
        NetworkManager &network_manager();
        UpdateChecker &update_checker();
        OAuthManager &oauth_manager();
        ImageCache &image_cache();
        RePoE &repoe();

    public slots:
        void accessGranted(const OAuthToken &token);
        void setSessionId(const QByteArray &poesessid);

    private:
        void loadSettings();

        app::UserSettings &m_settings;

        const QString m_settings_path;

        std::unique_ptr<KeychainStore> m_keychain;
        std::unique_ptr<NetworkManager> m_network_manager;
        std::unique_ptr<UpdateChecker> m_update_checker;
        std::unique_ptr<OAuthManager> m_oauth_manager;
        std::unique_ptr<ImageCache> m_image_cache;
        std::unique_ptr<RePoE> m_repoe;
    };

} // namespace app
