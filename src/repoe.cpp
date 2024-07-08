#include "repoe.h"

#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QString>

#include "QsLog.h"

#include "itemcategories.h"
#include "mainwindow.h"
#include "modlist.h"
#include "network_info.h"

#define REPOE_DATA(x) ("https://raw.githubusercontent.com/lvlvllvlvllvlvl/RePoE/master/RePoE/data" x)

constexpr const char* ITEM_CLASSES_URL = REPOE_DATA("/item_classes.min.json");
constexpr const char* BASE_ITEMS_URL = REPOE_DATA("/base_items.min.json");

// Modifiers from this list of files will be loaded in order from first to last.
constexpr const char* STAT_TRANSLATION_URLS[] = {
    REPOE_DATA("/stat_translations.min.json"),
    REPOE_DATA("/stat_translations/necropolis.min.json")
};

bool RePoE::initialized_ = false;

RePoE::RePoE(QObject* parent, QNetworkAccessManager& network_manager) :
    QObject(parent),
    network_manager_(network_manager)
{}

void RePoE::Init() {

    if (initialized_) {
        QLOG_INFO() << "RePoE is already initialized.";
        return;
    }

    QLOG_INFO() << "Initializing RePoE";
    emit StatusUpdate(ProgramState::Initializing, "Waiting for RePoE item classes.");

    QNetworkRequest request = QNetworkRequest(QUrl(ITEM_CLASSES_URL));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* reply = network_manager_.get(request);
    connect(reply, &QNetworkReply::finished, this, &RePoE::OnItemClassesReceived);
}

void RePoE::OnItemClassesReceived() {
    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    if (reply->error()) {
        QLOG_ERROR() << "Error fetching RePoE item classes:" << reply->url().toDisplayString()
            << "due to error:" << reply->errorString() << "The type dropdown will remain empty.";
    } else {
        QLOG_DEBUG() << "Received RePoE item classes";
        const QByteArray bytes = reply->readAll();
        InitItemClasses(bytes);
    };
    reply->deleteLater();

    emit StatusUpdate(ProgramState::Initializing, "Waiting for RePoE item base types.");

    QNetworkRequest request = QNetworkRequest(QUrl(BASE_ITEMS_URL));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* next_reply = network_manager_.get(request);
    connect(next_reply, &QNetworkReply::finished, this, &RePoE::OnBaseItemsReceived);
}

void RePoE::OnBaseItemsReceived() {

    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    if (reply->error()) {
        QLOG_ERROR() << "Error fetching RePoE base items:" << reply->url().toDisplayString()
            << "due to error:" << reply->errorString() << "The type dropdown will remain empty.";
    } else {
        QLOG_DEBUG() << "Received RePoE base items";
        const QByteArray bytes = reply->readAll();
        InitItemBaseTypes(bytes);
    };
    reply->deleteLater();

    emit StatusUpdate(ProgramState::Initializing, "RePoE data received; updating mod list.");

    InitStatTranslations();
    GetStatTranslations();
}

void RePoE::GetStatTranslations() {

    static QStringList urls = GetTranslationUrls();
    if (urls.isEmpty()) {
        QLOG_INFO() << "RePoE data received.";
        initialized_ = true;
        InitModList();
        emit finished();
        return;
    };

    const QString next = urls.front();
    urls.pop_front();

    QNetworkRequest request = QNetworkRequest(QUrl(next));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* reply = network_manager_.get(request);
    connect(reply, &QNetworkReply::finished, this, &RePoE::OnStatTranslationReceived);
}

QStringList RePoE::GetTranslationUrls() {
    QStringList list;
    for (const auto& url : STAT_TRANSLATION_URLS) {
        list.append(QString(url));
    };
    return list;
}


void RePoE::OnStatTranslationReceived() {

    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    if (reply->error()) {
        QLOG_ERROR() << "Couldn't fetch RePoE Stat Translations: " << reply->url().toDisplayString()
            << " due to error: " << reply->errorString() << " Aborting update.";
    } else {
        QLOG_INFO() << "Stat translations received:" << reply->request().url().toString();
        const QByteArray bytes = reply->readAll();
        AddStatTranslations(bytes);
    };
    reply->deleteLater();
    GetStatTranslations();
}
