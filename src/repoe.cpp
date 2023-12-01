#include "repoe.h"

#include <QByteArray>
#include <QNetworkReply>
#include <QString>
#include <QUrl>

#define REPOE_ "https://raw.githubusercontent.com/brather1ng/RePoE/master/RePoE/data"
const char* item_classes_url = REPOE_ "/item_classes.min.json";
const char* base_types_url = REPOE_ "/base_items.min.json";
const char* stat_translations_url = REPOE_ "/stat_translations.min.json";
#undef REPOE_

template<class ResultType>
struct ResultWrapper {
	ResultWrapper(QNetworkReply* reply) {
		const QString type = typeid(ResultType).name();
		if (reply->error()) {
			QLOG_ERROR() << "RePoE: error receiving" << type << ":" << reply->errorString();
			return;
		};
		QLOG_DEBUG() << "RePoE: received reply for" << type;
		const QByteArray data = "{\"result\":" + reply->readAll() + "}";
		JS::ParseContext context(data);
		JS::Error error = context.parseTo(*this);
		if (error != JS::Error::NoError) {
			QLOG_ERROR() << "PePoE: error parsing:" << type << ":" << context.makeErrorString();
			return;
		};
	};
	ResultType result;
	JS_OBJ(result);
};

void RePoE::Updater::RequestItemClasses() {
	QNetworkRequest request = QNetworkRequest(QUrl(QString(item_classes_url)));
	emit GetRequest("<NONE>", request,
		[=](QNetworkReply* reply) {
			const ResultWrapper<RePoE::ItemClasses> wrapper(reply);
			emit ItemClassesUpdate(wrapper.result);
			reply->deleteLater();
		});
}

void RePoE::Updater::RequestBaseTypes() {
	QNetworkRequest request = QNetworkRequest(QUrl(QString(base_types_url)));
	emit GetRequest("<NONE>", request,
		[=](QNetworkReply* reply) {
			const ResultWrapper<RePoE::BaseTypes> wrapper(reply);
			emit BaseTypesUpdate(wrapper.result);
			reply->deleteLater();
		});
}

void RePoE::Updater::RequestStatTranslations() {
	QNetworkRequest request = QNetworkRequest(QUrl(QString(stat_translations_url)));
	emit GetRequest("<NONE>", request,
		[=](QNetworkReply* reply) {
			const ResultWrapper<RePoE::StatTranslations> wrapper(reply);
			emit StatTranslationsUpdate(wrapper.result);
			reply->deleteLater();
		});
}

