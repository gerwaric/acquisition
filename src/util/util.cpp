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

#include "util.h"

#include <QComboBox>
#include <QCryptographicHash>
#include <QString>
#include <QStringList>
#include <QLineEdit>
#include <QLabel>
#include <QFontMetrics>
#include <QMetaEnum>
#include <QNetworkReply>
#include <QRegularExpression>
#include <QTextDocument>
#include <QUrlQuery>
#include <QPainter>

#include <cmath>

#include <QsLog/QsLog.h>

#include <rapidjson/document.h>
#include <rapidjson/writer.h>
#include <rapidjson/prettywriter.h>

#include "util/rapidjson_util.h"

#include "currency.h"

QsLogging::Level Util::TextToLogLevel(const QString& value) {
    if (0 == value.compare("TRACE", Qt::CaseInsensitive)) { return QsLogging::TraceLevel; };
    if (0 == value.compare("DEBUG", Qt::CaseInsensitive)) { return QsLogging::DebugLevel; };
    if (0 == value.compare("INFO", Qt::CaseInsensitive)) { return QsLogging::InfoLevel; };
    if (0 == value.compare("WARN", Qt::CaseInsensitive)) { return QsLogging::WarnLevel; };
    if (0 == value.compare("ERROR", Qt::CaseInsensitive)) { return QsLogging::ErrorLevel; };
    if (0 == value.compare("FATAL", Qt::CaseInsensitive)) { return QsLogging::FatalLevel; };
    if (0 == value.compare("OFF", Qt::CaseInsensitive)) { return QsLogging::OffLevel; };
    QLOG_ERROR() << "Invalid logging level:" << value << "(defaulting to DEBUG)";
    return QsLogging::DebugLevel;
}

QString Util::LogLevelToText(QsLogging::Level level) {
    switch (level) {
    case QsLogging::Level::TraceLevel: return "TRACE";
    case QsLogging::Level::DebugLevel: return "DEBUG";
    case QsLogging::Level::InfoLevel: return "INFO";
    case QsLogging::Level::WarnLevel: return "WARN";
    case QsLogging::Level::ErrorLevel: return "ERROR";
    case QsLogging::Level::FatalLevel: return "FATAL";
    case QsLogging::Level::OffLevel: return "OFF";
    default:
        QLOG_ERROR() << "Invalid log level:" << QString::number(level);
        return "<INVALID_LEVEL>";
    };
};

QString Util::Md5(const QString& value) {
    const QString hash = QString(QCryptographicHash::hash(value.toStdString().c_str(), QCryptographicHash::Md5).toHex());
    return hash.toUtf8().constData();
}

double Util::AverageDamage(const QString& s) {
    const QStringList parts = s.split("-");
    if (parts.size() < 2) {
        return s.toDouble();
    } else {
        return (parts[0].toDouble() + parts[1].toDouble()) / 2;
    };
}

void Util::PopulateBuyoutTypeComboBox(QComboBox* combobox) {
    combobox->addItems(QStringList({ "[Ignore]", "Buyout", "Fixed price", "Current Offer", "No price", "[Inherit]" }));
    combobox->setCurrentIndex(5);
}

void Util::PopulateBuyoutCurrencyComboBox(QComboBox* combobox) {
    for (auto type : Currency::Types()) {
        combobox->addItem(QString(Currency(type).AsString()));
    };
}

constexpr std::array width_strings = {
    "max#",
    "Map Tier",
    "R##",
    "Defense",
    "Master-crafted"
};

int Util::TextWidth(TextWidthId id) {
    static bool calculated = false;
    static std::vector<int> result;

    if (!calculated) {
        calculated = true;
        result.resize(width_strings.size());
        QLineEdit textbox;
        QFontMetrics fm(textbox.fontMetrics());
        for (size_t i = 0; i < width_strings.size(); ++i) {
            result[i] = fm.horizontalAdvance(width_strings[i]);
        };
    };
    return result[static_cast<int>(id)];
}

void Util::ParseJson(QNetworkReply* reply, rapidjson::Document* doc) {
    QByteArray bytes = reply->readAll();
    doc->Parse(bytes.constData());
}

QString Util::GetCsrfToken(const QByteArray& page, const QString& name) {
    // As of October 2023, the CSRF token can appear in one of two ways:
    //  name="hash" value="..."
    //	or
    //	name="hash" class="input-error" value="..."
    static const QString expr = QString(
        R"regex(
			name="%1"
			\s+
			(?:
				class=".*?"
				\s+
			)?
			value="(.*?)"
		)regex").simplified().arg(name);
    static const QRegularExpression re(expr,
        QRegularExpression::CaseInsensitiveOption |
        QRegularExpression::MultilineOption |
        QRegularExpression::DotMatchesEverythingOption |
        QRegularExpression::ExtendedPatternSyntaxOption);
    const QRegularExpressionMatch match = re.match(page);
    return match.captured(1);
}

QString Util::FindTextBetween(const QString& page, const QString& left, const QString& right) {
    const std::string s = page.toStdString();
    const size_t first = s.find(left.toStdString());
    const size_t last = s.find(right.toStdString(), first);
    if ((first == std::string::npos) || (last == std::string::npos) || (first > last)) {
        return "";
    } else {
        return QString::fromStdString(s.substr(first + left.size(), last - first - left.size()));
    };
}

QString Util::RapidjsonSerialize(const rapidjson::Value& val) {
    rapidjson::StringBuffer buffer;
    rapidjson::Writer<rapidjson::StringBuffer> writer(buffer);
    val.Accept(writer);
    return buffer.GetString();
}

QString Util::RapidjsonPretty(const rapidjson::Value& val) {
    rapidjson::StringBuffer buffer;
    rapidjson::PrettyWriter<rapidjson::StringBuffer> writer(buffer);
    val.Accept(writer);
    return buffer.GetString();
}

void Util::RapidjsonAddString(rapidjson::Value* object, const char* const name, const QString& value, rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>& alloc) {
    const QByteArray bytes = value.toUtf8();
    rapidjson::Value rjson_name(name, rapidjson::SizeType(strlen(name)), alloc);
    rapidjson::Value rjson_val(bytes.constData(), bytes.length(), alloc);
    object->AddMember(rjson_name, rjson_val, alloc);
}

void Util::RapidjsonAddInt64(rapidjson::Value* object, const char* const name, qint64 value, rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>& alloc) {
    rapidjson::Value rjson_name(name, rapidjson::SizeType(strlen(name)), alloc);
    rapidjson::Value rjson_val(static_cast<int64_t>(value));
    object->AddMember(rjson_name, rjson_val, alloc);
}

void Util::GetTabColor(rapidjson::Value& json, int& r, int& g, int& b) {

    r = 0;
    g = 0;
    b = 0;

    if (rapidjson::HasObject(json, "colour")) {

        // Tabs retrieved with the legacy api will have a "colour" field.
        const auto& colour = json["colour"];
        if (rapidjson::HasInt(colour, "r")) { r = colour["r"].GetInt(); };
        if (rapidjson::HasInt(colour, "g")) { g = colour["g"].GetInt(); };
        if (rapidjson::HasInt(colour, "b")) { b = colour["b"].GetInt(); };

    } else if (rapidjson::HasObject(json, "metadata")) {

        // Tabs retrieved with the OAuth api have a "metadata" field that may have a colour.
        const auto& metadata = json["metadata"];
        if (rapidjson::HasString(metadata, "colour")) {

            // The colour field is supposed to be a 6-character string, but it some really old
            // tabs it's only 4 characters or 2 characters, and GGG has confirmed that in these
            // cases the leading values should be treated as zeros.
            const std::string colour = metadata["colour"].GetString();
            switch (colour.length()) {
            case 6:
                r = std::stoul(colour.substr(0, 2), nullptr, 16);
                g = std::stoul(colour.substr(2, 2), nullptr, 16);
                b = std::stoul(colour.substr(4, 2), nullptr, 16);
                break;
            case 4:
                g = std::stoul(colour.substr(0, 2), nullptr, 16);
                b = std::stoul(colour.substr(2, 2), nullptr, 16);
                break;
            case 2:
                b = std::stoul(colour.substr(0, 2), nullptr, 16);
                break;
            default:
                QLOG_DEBUG() << "Could not parse stash tab colour:" << Util::RapidjsonSerialize(json);
                break;
            };
        } else {
            QLOG_DEBUG() << "Stab tab metadata does not have a colour:" << Util::RapidjsonSerialize(json);
        };
    } else {
        QLOG_DEBUG() << "Stash tab does not have a colour:" << Util::RapidjsonSerialize(json);
    };
}

QString Util::StringReplace(const QString& haystack, const QString& needle, const QString& replace) {
    std::string out = haystack.toStdString();
    for (size_t pos = 0; ; pos += replace.length()) {
        pos = out.find(needle.toStdString(), pos);
        if (pos == std::string::npos) {
            break;
        };
        out.erase(pos, needle.length());
        out.insert(pos, replace.toStdString());
    };
    return QString::fromStdString(out);
}

bool Util::MatchMod(const char* const match, const char* const mod, double* output) {
    double result = 0.0;
    auto pmatch = match;
    auto pmod = mod;
    int cnt = 0;

    while (*pmatch && *pmod) {
        if (*pmatch == '#') {
            ++cnt;
            auto prev = pmod;
            while ((*pmod >= '0' && *pmod <= '9') || *pmod == '.')
                ++pmod;
            result += std::strtod(prev, NULL);
            ++pmatch;
        } else if (*pmatch == *pmod) {
            ++pmatch;
            ++pmod;
        } else {
            return false;
        }
    }
    *output = result / cnt;
    return !*pmatch && !*pmod;
}

QString Util::Capitalise(const QString& str) {
    QString capitalized = str;
    if (!capitalized.isEmpty()) {
        capitalized[0] = capitalized[0].toUpper();
    };
    return capitalized;
}

QString Util::TimeAgoInWords(const QDateTime& buyout_time) {
    const QDateTime current_date = QDateTime::currentDateTime();
    const qint64 secs = buyout_time.secsTo(current_date);
    const qint64 days = secs / 60 / 60 / 24;
    const qint64 hours = (secs / 60 / 60) % 24;
    const qint64 minutes = (secs / 60) % 60;

    // YEARS
    if (days > 365) {
        int years = (days / 365);
        if (days % 365 != 0) {
            years++;
        };
        return QString("%1 %2 ago").arg(years).arg(years == 1 ? "year" : "years");
    };

    // MONTHS
    if (days > 30) {
        int months = (days / 365);
        if (days % 30 != 0) {
            months++;
        };
        return QString("%1 %2 ago").arg(months).arg(months == 1 ? "month" : "months");
    };

    // DAYS
    if (days > 0) {
        return QString("%1 %2 ago").arg(days).arg(days == 1 ? "day" : "days");
    };

    // HOURS
    if (hours > 0) {
        return QString("%1 %2 ago").arg(hours).arg(hours == 1 ? "hour" : "hours");
    };

    //MINUTES
    if (minutes > 0) {
        return QString("%1 %2 ago").arg(minutes).arg(minutes == 1 ? "minute" : "minutes");
    };

    // SECONDS
    if (secs > 5) {
        return QString("%1 %2 ago").arg(secs).arg("seconds");
    } else if (secs < 5) {
        return QString("just now");
    };

    return "";
}

QString Util::Decode(const QString& entity) {
    QTextDocument text;
    text.setHtml(entity);
    return text.toPlainText();
}

QUrlQuery Util::EncodeQueryItems(const std::vector<std::pair<QString, QString>>& items) {
    // https://github.com/owncloud/client/issues/9203
    QUrlQuery result;
    for (const auto& item : items) {
        const QString key = QUrl::toPercentEncoding(item.first);
        const QString value = QUrl::toPercentEncoding(item.second);
        result.addQueryItem(key, value);
    };
    return result;
}

QColor Util::recommendedForegroundTextColor(const QColor& backgroundColor) {
    const float R = (float)backgroundColor.red() / 255.0f;
    const float G = (float)backgroundColor.green() / 255.0f;
    const float B = (float)backgroundColor.blue() / 255.0f;

    const float gamma = 2.2f;
    const float L = 0.2126f * pow(R, gamma)
        + 0.7152f * pow(G, gamma)
        + 0.0722f * pow(B, gamma);

    return (L > 0.5f) ? QColor(QColorConstants::Black) : QColor(QColorConstants::White);
}

// Obsolete timezones are allowed by RFC2822, but they aren't parsed by
// QT 6.5.3 so we have to fix them manually.
QByteArray Util::FixTimezone(const QByteArray& rfc2822_date) {
    constexpr const std::array<std::pair<const char*, const char*>, 10> OBSOLETE_ZONES{ {
        {"GMT", "+0000"},
        {"UT" , "+0000"},
        {"EST", "-0005"},
        {"EDT", "-0004"},
        {"CST", "-0006"},
        {"CDT", "-0005"},
        {"MST", "-0007"},
        {"MDT", "-0006"},
        {"PST", "-0008"},
        {"PDT", "-0007"}
    } };
    for (auto& pair : OBSOLETE_ZONES) {
        const QByteArray& zone = pair.first;
        const QByteArray& offset = pair.second;
        if (rfc2822_date.endsWith(zone)) {
            const int k = rfc2822_date.length() - zone.length();
            return rfc2822_date.left(k) + offset;
        };
    };
    return rfc2822_date;
}

QDebug& operator<<(QDebug& os, const RefreshReason::Type obj)
{
    const QMetaObject* meta = &RefreshReason::staticMetaObject;
    os << meta->enumerator(meta->indexOfEnumerator("Type")).key(obj);
    return os;
}

QDebug& operator<<(QDebug& os, const TabSelection::Type obj)
{
    const QMetaObject* meta = &TabSelection::staticMetaObject;
    os << meta->enumerator(meta->indexOfEnumerator("Type")).key(obj);
    return os;
}

QDebug& operator<<(QDebug& os, const QsLogging::Level obj) {
    switch (obj) {
    case QsLogging::Level::TraceLevel: return os << "TRACE";
    case QsLogging::Level::DebugLevel: return os << "DEBUG";
    case QsLogging::Level::InfoLevel: return os << "INFO";
    case QsLogging::Level::WarnLevel: return os << "WARN";
    case QsLogging::Level::ErrorLevel: return os << "ERROR";
    case QsLogging::Level::FatalLevel: return os << "FATAL";
    case QsLogging::Level::OffLevel: return os << "OFF";
    default: return os << "None (log level is invalid)";
    };
}
