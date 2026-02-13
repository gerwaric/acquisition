// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QDir>
#include <QObject>
#include <QString>

class LoginDialog;
class MainWindow;

namespace app {
    class UserSettings;
    class CoreServices;
    class SessionServices;
} // namespace app

class Application : public QObject
{
    Q_OBJECT
public:
    explicit Application(const QDir &dataDir);
    ~Application();

public slots:
    void startNewSession();

private:
    void loadTheme(const QString &theme);
    void createMainWindow();
    void saveDataOnNewVersion();

    QDir m_data_dir;

    std::unique_ptr<app::UserSettings> m_settings;
    std::unique_ptr<app::CoreServices> m_core;
    std::unique_ptr<app::SessionServices> m_session;
    std::unique_ptr<LoginDialog> m_login_dialog;
    std::unique_ptr<MainWindow> m_main_window;
};
