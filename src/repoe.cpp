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

constexpr const char* ITEM_CLASSES_URL = REPOE_DATA("/item_classes.json");
constexpr const char* BASE_ITEMS_URL = REPOE_DATA("/base_items.json");

// Modifiers from this list of files will be loaded in order from first to last.
constexpr const char* STAT_TRANSLATION_URLS[] = {
    REPOE_DATA("/stat_translations.json"),
    REPOE_DATA("/stat_translations/necropolis.json")
};

bool RePoE::initialized_ = false;

RePoE::RePoE(QNetworkAccessManager& network_manager)
    : network_manager_(network_manager)
{
    QLOG_TRACE() << "RePoE::RePoE() entered";
}

void RePoE::Init() {

    QLOG_TRACE() << "RePoE::Init() entered";
    if (initialized_) {
        QLOG_INFO() << "RePoE is already initialized.";
        return;
    }

    QLOG_INFO() << "Initializing RePoE";
    emit StatusUpdate(ProgramState::Initializing, "Waiting for RePoE item classes.");

    QLOG_TRACE() << "RePoE: sending item classes request:" << ITEM_CLASSES_URL;
    QNetworkRequest request = QNetworkRequest(QUrl(ITEM_CLASSES_URL));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* reply = network_manager_.get(request);
    connect(reply, &QNetworkReply::finished, this, &RePoE::OnItemClassesReceived);
}

void RePoE::OnItemClassesReceived() {

    QLOG_TRACE() << "RePoE::OnItemClassesReceived() entered";
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

    QLOG_TRACE() << "RePoE: sending base items request:" << BASE_ITEMS_URL;
    QNetworkRequest request = QNetworkRequest(QUrl(BASE_ITEMS_URL));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* next_reply = network_manager_.get(request);
    connect(next_reply, &QNetworkReply::finished, this, &RePoE::OnBaseItemsReceived);
}

void RePoE::OnBaseItemsReceived() {

    QLOG_TRACE() << "RePoE::OnBaseItemsReceived() entered";
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

    QLOG_TRACE() << "RePoE::GetStatTranslations() entered";
    static QStringList urls = GetTranslationUrls();
    if (urls.isEmpty()) {
        QLOG_INFO() << "RePoE data received.";
        initialized_ = true;
        InitModList();
        emit finished();
        return;
    };

    QLOG_TRACE() << "RePoE: getting next stat translation";
    const QString next = urls.front();
    urls.pop_front();

    QLOG_TRACE() << "RePoE: requesting stat translation:" << next;
    QNetworkRequest request = QNetworkRequest(QUrl(next));
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    QNetworkReply* reply = network_manager_.get(request);
    connect(reply, &QNetworkReply::finished, this, &RePoE::OnStatTranslationReceived);
}

QStringList RePoE::GetTranslationUrls() {
    QLOG_TRACE() << "RePoE::GetTranslationUrls() entered";
    QStringList list;
    for (const auto& url : STAT_TRANSLATION_URLS) {
        QLOG_TRACE() << "RePoE: adding stat translation:" << url;
        list.append(QString(url));
    };
    return list;
}


void RePoE::OnStatTranslationReceived() {

    QLOG_TRACE() << "RePoE::OnStatTranslationReceived() entered";
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
