#pragma once

#include <QNetworkRequest>
#include <QObject>

#include <string>
#include <unordered_map>
#include <vector>

#include "json_struct/json_struct.h"
#include "QsLog.h"

#include "ratelimit.h"

class QString;

namespace RePoE {

	struct ItemClass {
		std::string name;
		JS_OBJ(name);
	};

	struct BaseType {
		std::string item_class;
		std::string name;
		JS_OBJ(item_class, name);
	};

	struct Translation {
		std::vector<std::string> format;
		std::string string;
		JS_OBJ(format, string);
	};

	struct StatTranslation {
		std::vector<Translation> English;
		JS_OBJ(English);
	};

	using ItemClasses = std::unordered_map<std::string, ItemClass>;
	using BaseTypes = std::unordered_map<std::string, BaseType>;
	using StatTranslations = std::vector<StatTranslation>;

	class Updater : public QObject {
		Q_OBJECT

	public:
		Updater(QObject* parent) : QObject(parent) {};

	public slots:
		void RequestItemClasses();
		void RequestBaseTypes();
		void RequestStatTranslations();
	
	signals:
		void GetRequest(const QString& endpoint, QNetworkRequest network_request, RateLimit::Callback request_callback);
		void ItemClassesUpdate(const RePoE::ItemClasses& item_classes);
		void BaseTypesUpdate(const RePoE::BaseTypes& base_types);
		void StatTranslationsUpdate(const RePoE::StatTranslations& translations);
	};
}