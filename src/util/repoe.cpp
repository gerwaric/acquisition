/*
    Copyright (C) 2014-2024 Acquisition Contributors

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "repoe.h"

#include <QDir>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QString>

#include <QsLog/QsLog.h>

#include "ui/mainwindow.h"

#include "itemcategories.h"
#include "modlist.h"
#include "network_info.h"

constexpr const char* REPOE_URL = "https://raw.githubusercontent.com/repoe-fork/repoe-fork.github.io/master/RePoE/data/";

constexpr std::array REPOE_FILES = {
    "item_classes.json",
    "base_items.json"
};

constexpr std::array STAT_TRANSLATIONS = {
    "stat_translations.json",
    "stat_translations/necropolis.json"
};

RePoE::RePoE(QNetworkAccessManager& network_manager)
    : m_network_manager(network_manager)
    , m_initialized(false)
{
    QLOG_TRACE() << "RePoE::RePoE() entered";
}

void RePoE::Init(const QString& data_dir) {

    QLOG_INFO() << "Initializing RePoE";
    if (m_initialized) {
        QLOG_INFO() << "RePoE is already initialized.";
        return;
    };

    m_data_dir = data_dir;

    emit StatusUpdate(ProgramState::Initializing, "Waiting for RePoE version.");

    const QString url = QString(REPOE_URL) + "/version.txt";

    // We start be requesting the current version from Github to see if we need to update.
    QLOG_DEBUG() << "RePoE: requesting version.txt";
    QNetworkRequest request = QNetworkRequest(QUrl(url));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* reply = m_network_manager.get(request);
    connect(reply, &QNetworkReply::finished, this, &RePoE::OnVersionReceived);
}

void RePoE::OnVersionReceived() {

    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    if (reply->error()) {
        const auto error = reply->error();
        if ((error < 200) || (error > 299)) {
            QLOG_ERROR() << "RePoE: error requesting version.txt:" << error << reply->errorString();
            reply->deleteLater();
            return;
        };
    };
    QLOG_DEBUG() << "RePoE: received version.txt";

    const QByteArray data = reply->readAll();
    reply->deleteLater();

    QDir repoe_dir(m_data_dir);
    if (!repoe_dir.exists("repoe")) {
        repoe_dir.mkdir("repoe");
    };
    if (!repoe_dir.exists("repoe/stat_translations")) {
        repoe_dir.mkdir("repoe/stat_translations");
    };

    bool update = false;
    for (const auto& filename : REPOE_FILES) {
        update |= !repoe_dir.exists("repoe/" + QString(filename));
    };
    for (const auto& filename : STAT_TRANSLATIONS) {
        update |= !repoe_dir.exists("repoe/" + QString(filename));
    };

    const QString version_path = repoe_dir.absoluteFilePath("repoe/version.txt");
    QFile version_file(version_path);
    if (version_file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QTextStream in(&version_file);
        const QString remote_version = QString::fromUtf8(data);
        const QString local_version = in.readAll();
        version_file.close();
        QLOG_DEBUG() << "RePoE: local version is" << local_version;
        QLOG_DEBUG() << "RePoE: remote version is" << remote_version;
        if (local_version != remote_version) {
            update = true;
        };
    } else {
        update = true;
    };

    if (update) {
        if (!version_file.open(QIODevice::WriteOnly)) {
            QLOG_ERROR() << "RePoE: error opening version.txt for writing:" << version_file.errorString();
            return;
        };
        if (version_file.write(data) < 0) {
            QLOG_ERROR() << "RePoE: error writing version.txt:" << version_file.errorString();
            version_file.close();
            return;
        };
        version_file.close();
        BeginUpdate();
    } else {
        QLOG_INFO() << "RePoE: an update is not needed";
        FinishUpdate();
    }
}

void RePoE::BeginUpdate() {

    QLOG_INFO() << "RePoE: beginning update";
    emit StatusUpdate(ProgramState::Initializing, "Waiting for RePoE item classes.");

    m_needed_files.clear();
    m_needed_files.reserve(REPOE_FILES.size() + STAT_TRANSLATIONS.size());
    for (const auto& filename : REPOE_FILES) {
        m_needed_files.emplace_back(filename);
    };
    for (const auto& filename : STAT_TRANSLATIONS) {
        m_needed_files.emplace_back(filename);
    };
    RequestNextFile();
}

void RePoE::RequestNextFile() {

    if (m_needed_files.empty()) {

        FinishUpdate();

    } else {

        const QString& filename = m_needed_files.back();
        emit StatusUpdate(ProgramState::Initializing, "Waiting for RePoE file: " + filename);

        QLOG_DEBUG() << "RePoE: requesting" << filename;
        const QString url = QString(REPOE_URL) + "/" + filename;
        QNetworkRequest request = QNetworkRequest(QUrl(url));
        request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
        QNetworkReply* reply = m_network_manager.get(request);
        connect(reply, &QNetworkReply::finished, this, &RePoE::OnFileReceived);

    };
}

void RePoE::OnFileReceived() {

    const QString filename = m_needed_files.back();
    m_needed_files.pop_back();

    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    const auto error = reply->error();
    if (error != QNetworkReply::NetworkError::NoError) {
        if ((error < 200) || (error > 299)) {
            QLOG_ERROR() << "RePoE: network error:" << reply->errorString();
            QLOG_ERROR() << "RePoE: failed to download" << filename;
            reply->deleteLater();
            return;
        };
    };

    const QString savefile = m_data_dir + "/repoe/" + filename;
    const QByteArray data = reply->readAll();
    reply->deleteLater();

    QFile file(savefile);
    if (!file.open(QIODevice::WriteOnly)) {
        QLOG_ERROR() << "RePoE: error opening file for writing:" << file.errorString();
        return;
    };
    if (file.write(data) < 0) {
        QLOG_ERROR() << "RePoE: error writing data:" << file.errorString();
        file.close();
        return;
    };
    file.close();

    RequestNextFile();
}

void RePoE::FinishUpdate() {

    emit StatusUpdate(ProgramState::Initializing, "RePoE updating item classes, base types, and mods");

    InitItemClasses(ReadFile("item_classes.json"));
    InitItemBaseTypes(ReadFile("base_items.json"));

    InitStatTranslations();
    for (const auto& filename : STAT_TRANSLATIONS) {
        AddStatTranslations(ReadFile(QString(filename)));
    };
    InitModList();

    QLOG_INFO() << "RePoE: update finished";
    m_initialized = true;
    emit finished();
}

QByteArray RePoE::ReadFile(const QString& filename) {
    const QString filepath = m_data_dir + "/repoe/" + filename;
    QFile file(filepath);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QLOG_ERROR() << "RePoE: cannot open file for reading:" << filepath;
        return QByteArray();
    } else {
        const QByteArray data = file.readAll();
        return data;
    };
}
