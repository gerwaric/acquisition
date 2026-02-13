// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "datastore/keychainstore.h"

#include <qtkeychain/keychain.h>

#include "util/spdlog_qt.h" // IWYU pragma: keep

constexpr const char *SERVICE = "acquisition";

KeychainStore::KeychainStore() {}

void KeychainStore::clear()
{
    spdlog::warn("KeychainStore: clear() is not implemented yet.");
}

KeychainReply *KeychainStore::save(const QString &key, const QByteArray &data)
{
    spdlog::debug("KeychainStore: saving '{}'", key);

    KeychainReply *reply = new KeychainReply();

    auto *job = new QKeychain::WritePasswordJob(SERVICE, this);
    job->setKey(key);
    job->setBinaryData(data);
    connect(job, &QKeychain::Job::finished, this, [job, reply] {
        const auto &key = job->key();
        if (job->error()) {
            const auto &errorString = job->errorString();
            spdlog::error("KeychainStore: failed to save '{}': '{}'", key, errorString);
            emit reply->failed(key, errorString);
        } else {
            spdlog::debug("KeychainStore: saved '{}'", key);
            emit reply->saved(key);
        }
        job->deleteLater();
    });
    job->start();

    return reply;
}

KeychainReply *KeychainStore::load(const QString &key)
{
    spdlog::debug("KeychainStore: loading '{}'", key);

    KeychainReply *reply = new KeychainReply();

    auto *job = new QKeychain::ReadPasswordJob(SERVICE, this);
    job->setKey(key);
    connect(job, &QKeychain::Job::finished, this, [job, reply] {
        const auto &key = job->key();
        if (job->error()) {
            const auto &errorString = job->errorString();
            spdlog::error("KeychainStore: failed to load '{}': '{}'", key, errorString);
            emit reply->failed(key, errorString);
        } else {
            spdlog::debug("KeychainStore: loaded '{}'", key);
            emit reply->loaded(key, job->binaryData());
        }
        job->deleteLater();
    });
    job->start();

    return reply;
}

KeychainReply *KeychainStore::remove(const QString &key)
{
    spdlog::debug("KeychainStore: removing '{}'", key);

    KeychainReply *reply = new KeychainReply();

    auto *job = new QKeychain::DeletePasswordJob(SERVICE, this);
    job->setKey(key);
    connect(job, &QKeychain::Job::finished, this, [job, reply] {
        const auto &key = job->key();
        if (job->error()) {
            const auto &errorString = job->errorString();
            spdlog::error("KeychainStore: failed to remove '{}': '{}'", key, errorString);
            emit reply->failed(key, errorString);
        } else {
            spdlog::debug("KeychainStore: removed '{}', key");
            emit reply->removed(key);
        }
        job->deleteLater();
    });
    job->start();

    return reply;
}
