// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QAnyStringView>
#include <QByteArray>
#include <QObject>
#include <QString>

struct OAuthToken;

class KeychainReply : public QObject
{
    Q_OBJECT
signals:
    void saved(const QString &key);
    void loaded(const QString &key, const QByteArray &data);
    void removed(const QString &key);
    void failed(const QString &key, const QString &error);
};

class KeychainStore : public QObject
{
    Q_OBJECT
public:
    explicit KeychainStore();

    void clear();

    KeychainReply *save(const QString &key, const QByteArray &data);
    KeychainReply *load(const QString &key);
    KeychainReply *remove(const QString &key);

    /*
    void saveSessionId(QAnyStringView account, QByteArrayView poesessid);
    void loadSessionId(QAnyStringView account);
    void removeSessionId(QAnyStringView account);

    void saveOAuthToken(QAnyStringView account, const OAuthToken &token);
    void loadOAuthToken(QAnyStringView account);
    void removeOAuthToken(QAnyStringView account);
    */

signals:
    void sessionIdLoaded(const QString &account, const QByteArray &poesessid);
    void sessionIdError(const QString &account, const QString &error);

    void oauthTokenLoaded(const QString &account, const OAuthToken &token);
    void oauthTokenError(const QString &account, const QString &error);

    void contained(const QString &key, bool value);
    void containFailed(const QString &key, const QString &error);

    void saved(const QString &key);
    void saveFailed(const QString &key, const QString &error);

    void loaded(const QString &key, const QByteArray &data);
    void loadFailed(const QString &key, const QString &error);

    void removed(const QString &key);
    void removeFailed(const QString &key, const QString &error);

private:
};
