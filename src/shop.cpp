/*
    Copyright (C) 2014-2025 Acquisition Contributors

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

#include "shop.h"

#include <QApplication>
#include <QClipboard>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QRegularExpression>
#include <QSettings>
#include <QTextDocumentFragment>
#include <QTimer>
#include <QUrl>
#include <QUrlQuery>

#include <rapidjson/document.h>

#include <datastore/datastore.h>
#include <ratelimit/ratelimitedreply.h>
#include <ratelimit/ratelimiter.h>
#include <ui/mainwindow.h>
#include <util/util.h>
#include <util/spdlog_qt.h>

#include "application.h"
#include "buyoutmanager.h"
#include "itemsmanager.h"
#include "network_info.h"
#include "replytimeout.h"

constexpr const char* kPoeEditThread = "https://www.pathofexile.com/forum/edit-thread/";
constexpr const char* kShopTemplateItems = "[items]";
constexpr int kMaxCharactersInPost = 50000;
constexpr int kSpoilerOverhead = 19; // "[spoiler][/spoiler]" length

// Use a regular expression to look for html errors.
const QRegularExpression Shop::error_regex(
    R"regex(
		# Start the match looking for any class attribute that indicates an error
		class="(?:input-error|errors)"

		# Skip over as much as possible while looking for an <li> start tag that
		# should be the start of the error message.
		.*?

		# Match the list item element and capture it's contents, because this is
		# expected to be the error message.
		<li>(.*?)</li>
	)regex",
    QRegularExpression::CaseInsensitiveOption |
    QRegularExpression::MultilineOption |
    QRegularExpression::DotMatchesEverythingOption |
    QRegularExpression::ExtendedPatternSyntaxOption);

const QRegularExpression Shop::ratelimit_regex(
    R"regex(You must wait (\d+) seconds.)regex",
    QRegularExpression::CaseInsensitiveOption);

Shop::Shop(
    QSettings& settings,
    QNetworkAccessManager& network_manager,
    RateLimiter& rate_limiter,
    DataStore& datastore,
    ItemsManager& items_manager,
    BuyoutManager& buyout_manager)
    : m_settings(settings)
    , m_network_manager(network_manager)
    , m_rate_limiter(rate_limiter)
    , m_datastore(datastore)
    , m_items_manager(items_manager)
    , m_buyout_manager(buyout_manager)
    , m_initialized(false)
    , m_shop_data_outdated(true)
    , m_submitting(false)
    , m_indexing(false)
    , m_requests_completed(0)
{
    spdlog::debug("Shop: initializing");
    m_threads = m_datastore.Get("shop").split(";");
    m_auto_update = m_settings.value("shop_autoupdate").toBool();
    m_shop_template = m_datastore.Get("shop_template");
    if (m_shop_template.isEmpty()) {
        m_shop_template = kShopTemplateItems;
    };
    if (!m_settings.value("session_id").toString().isEmpty()) {
        UpdateStashIndex();
    };
}

void Shop::SetThread(const QStringList& threads) {
    if (m_submitting) {
        spdlog::warn("Shop: cannot set shop threads: shop is still updating");
        return;
    };
    spdlog::debug("Shop: setting {} thread(s) to {}", threads.size(), threads.join(";"));
    m_threads = threads;
    m_datastore.Set("shop", threads.join(";"));
    ExpireShopData();
    m_datastore.Set("shop_hash", "");
}

void Shop::SetAutoUpdate(bool update) {
    spdlog::debug("Shop: setting autoupdate to {}", update);
    m_auto_update = update;
    m_settings.setValue("shop_autoupdate", update);
}

void Shop::SetShopTemplate(const QString& shop_template) {
    spdlog::debug("Shop: setting template to {}", shop_template);
    m_shop_template = shop_template;
    m_datastore.Set("shop_template", shop_template);
    ExpireShopData();
}

QString Shop::SpoilerBuyout(Buyout& bo) {
    spdlog::trace("Shop: spoiler buyout called");
    QString out = "";
    out += "[spoiler=\"" + bo.BuyoutTypeAsPrefix();
    if (bo.IsPriced()) {
        out += " " + QString::number(bo.value) + " " + bo.CurrencyAsTag();
    };
    out += "\"]";
    return out;
}

void Shop::UpdateStashIndex() {

    spdlog::debug("Shop: updating the stash index");
    m_indexing = true;
    m_tab_index.clear();

    const QString kStashItemsUrl = "https://www.pathofexile.com/character-window/get-stash-items";
    const QString account = m_settings.value("account").toString();
    const QString realm = m_settings.value("realm").toString();
    const QString league = m_settings.value("league").toString();

    QUrlQuery query;
    query.addQueryItem("accountName", account);
    query.addQueryItem("realm", realm);
    query.addQueryItem("league", league);
    query.addQueryItem("tabs", "1");
    query.addQueryItem("tabIndex", "0");

    QUrl url(kStashItemsUrl);
    url.setQuery(query);
    QNetworkRequest request(url);

    RateLimitedReply* reply = m_rate_limiter.Submit(kStashItemsUrl, request);
    connect(reply, &RateLimitedReply::complete, this, &Shop::OnStashTabIndexReceived);
}

void Shop::OnStashTabIndexReceived(QNetworkReply* reply) {

    if (reply->error() != QNetworkReply::NetworkError::NoError) {
        const int status = reply->error();
        if ((status >= 200) && (status <= 299)) {
            spdlog::debug("Shop: http reply status {}", status);
        } else {
            spdlog::error("Shop: network error indexing stashes: {} {}", status, reply->errorString());
            m_indexing = false;
            return;
        };
    };

    spdlog::debug("Shop: stash tab list received");
    rapidjson::Document doc;
    QByteArray bytes = reply->readAll();
    doc.Parse(bytes.constData());
    reply->deleteLater();

    if (!doc.IsObject()) {
        spdlog::error("Shop: can't even fetch first legacy tab. Failed to update items.");
        m_submitting = false;
        return;
    };

    if (doc.HasMember("error")) {
        spdlog::error("Shop: aborting legacy update since first fetch failed due to 'error': {}", Util::RapidjsonSerialize(doc["error"]));
        m_submitting = false;
        return;
    };

    if (!HasArray(doc, "tabs") || doc["tabs"].Size() == 0) {
        spdlog::error("Shop: there are no legacy tabs, this should not happen, bailing out.");
        m_submitting = false;
        return;
    };

    auto& tabs = doc["tabs"];
    spdlog::debug("Shop: received legacy tabs list, there are {} tabs", tabs.Size());

    for (const auto& tab : tabs) {
        const int index = tab["i"].GetInt();
        const QString uid = QString::fromStdString(tab["id"].GetString()).first(10);
        m_tab_index[uid] = index;
    };
    m_initialized = true;
    m_indexing = false;
    ExpireShopData();
    emit StashesIndexed();
}

void Shop::Update() {
    if (!m_initialized) {
        spdlog::debug("Shop: cannot update shop data yet because the shop is being initialized");
        return;
    };
    if (m_submitting) {
        spdlog::warn("Shop: skipping shop data update becuase the shop is being submitted");
        return;
    };
    if (m_indexing) {
        spdlog::warn("Shop: skipping shop data update because the shop is being indexed");
        return;
    };
    if (m_tab_index.empty()) {
        spdlog::warn("Shop: skipping shop data update because the stash tab index is empty");
        return;
    };
    spdlog::debug("Shop: updating shop data");

    m_shop_data_outdated = false;
    m_shop_data.clear();
    QString data = "";
    std::vector<AugmentedItem> aug_items;
    AugmentedItem tmp = AugmentedItem();
    //Get all buyouts to be able to sort them
    for (auto& item : m_items_manager.items()) {
        tmp.item = item.get();
        tmp.bo = m_buyout_manager.Get(*item);

        if (!tmp.bo.IsPostable()) {
            continue;
        };

        aug_items.push_back(tmp);
    }
    if (aug_items.size() == 0) {
        return;
    };
    std::sort(aug_items.begin(), aug_items.end());

    const QString realm = m_settings.value("realm").toString();
    const QString league = m_settings.value("league").toString();

    Buyout current_bo = aug_items[0].bo;
    data += SpoilerBuyout(current_bo);
    for (auto& aug : aug_items) {
        if (aug.bo.type != current_bo.type || aug.bo.currency != current_bo.currency || aug.bo.value != current_bo.value) {
            current_bo = aug.bo;
            data += "[/spoiler]";
            data += SpoilerBuyout(current_bo);
        };
        const ItemLocation loc = aug.item->location();
        QString item_string;
        if (loc.get_type() == ItemLocationType::CHARACTER) {
            item_string = loc.GetForumCode(realm, league, 0);
        } else {
            const QString uid = loc.get_tab_uniq_id();
            const auto it = m_tab_index.find(uid);
            if (it == m_tab_index.end()) {
                spdlog::error("Shop: cannot determine tab index for {} in {}", aug.item->PrettyName(), loc.GetHeader());
                continue;
            };
            const unsigned int ind = it->second;
            item_string = loc.GetForumCode(realm, league, ind);
        };
        const size_t n = data.size()
            + item_string.size()
            + m_shop_template.size()
            + kSpoilerOverhead
            + QStringLiteral("[/spoiler]").size();
        if (n > kMaxCharactersInPost) {
            data += "[/spoiler]";
            m_shop_data.push_back(data);
            data = SpoilerBuyout(current_bo);
            data += item_string;
        } else {
            data += item_string;
        };
    }
    if (!data.isEmpty()) {
        m_shop_data.push_back(data);
    };

    for (int i = 0; i < m_shop_data.size(); ++i) {
        m_shop_data[i] = Util::StringReplace(m_shop_template, kShopTemplateItems, "[spoiler]" + m_shop_data[i] + "[/spoiler]");
    };
    m_shop_hash = Util::Md5(m_shop_data.join(";"));
}

void Shop::ExpireShopData() {
    spdlog::trace("Shop: expiring shop data");
    m_shop_data_outdated = true;
}

void Shop::SubmitShopToForum(bool force) {
    spdlog::debug("Shop: submitting shop(s) to forums");
    if (m_submitting) {
        spdlog::warn("Shop: cannot submit two shops at once");
        return;
    };
    if (m_indexing) {
        spdlog::warn("Shop: cannot submit shop because tabs are still being indexed");
        return;
    };
    if (m_tab_index.empty()) {
        spdlog::error("Shop: cannot submit shop because the tab index is empty");
        return;
    };
    if (m_threads.empty()) {
        spdlog::error("Shop: asked to update a shop with no shop ID defined.");
        QMessageBox::warning(nullptr,
            "Acquisition Shop Manager",
            "No forum threads have been set."
            "\n\n"
            "Use the Shop --> 'Forum shop thread...' menu item.");
        return;
    };
    const QString session_id = m_settings.value("session_id").toString();
    if (session_id.isEmpty()) {
        spdlog::error("Shop: cannot submit a shop because the POESESSID is not set");
        QMessageBox::warning(nullptr,
            "Acquisition Shop Manager",
            "Cannot update forum shop threads because POESESSID has not been set."
            "\n\n"
            "Use the Settings --> POESESSID --> 'show or edit session cookie' menu item.");
        return;
    };

    if (m_shop_data_outdated) {
        Update();
    };

    spdlog::debug("Shop: updating {} forum shop threads", m_threads.size());
    QString previous_hash = m_datastore.Get("shop_hash");
    // Don't update the shop if it hasn't changed
    if (previous_hash == m_shop_hash && !force) {
        spdlog::trace("Shop: skipping update because the shop hash has not changed");
        return;
    };

    if (m_threads.size() < m_shop_data.size()) {
        spdlog::warn("Shop: need {} more shops defined to fit all your items.", m_shop_data.size() - m_threads.size());
    };

    m_requests_completed = 0;
    m_submitting = true;
    SubmitSingleShop();
}

QString Shop::ShopEditUrl(int idx) {
    if (idx >= m_threads.size()) {
        spdlog::error("Shop: cannot create edit url for thread # {}", idx);
        return "";
    };
    const QString url = kPoeEditThread + m_threads[idx];
    spdlog::debug("Shop: shop edit url # {} is {}", idx, url);
    return url;
}

void Shop::SubmitSingleShop() {
    spdlog::debug("Shop: submitting a single shop.");
    if (m_requests_completed > m_threads.size()) {
        spdlog::error("Shop: shop thread # {} does not exist", m_requests_completed);
    } else if (m_requests_completed == m_threads.size()) {
        spdlog::debug("Shop: updated {} threads", m_threads.size());
        emit StatusUpdate(ProgramState::Ready, "Shop threads updated");
        m_submitting = false;
        m_datastore.Set("shop_hash", m_shop_hash);
    } else {
        spdlog::debug("Shop: updating shop thread # {}: {}", (m_requests_completed + 1),  m_threads[m_requests_completed]);
        emit StatusUpdate(ProgramState::Ready,
            QString("Sending your shops to the forum, %1/%2")
            .arg(m_requests_completed)
            .arg(m_threads.size()));
        // first, get to the edit-thread page to grab CSRF token
        QNetworkRequest request = QNetworkRequest(QUrl(ShopEditUrl(m_requests_completed)));
        request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
        request.setRawHeader("Cache-Control", "max-age=0");
        request.setTransferTimeout(kEditThreadTimeout);
        QNetworkReply* fetched = m_network_manager.get(request);
        connect(fetched, &QNetworkReply::finished, this, &Shop::OnEditPageFinished);
    };
}

void Shop::OnEditPageFinished() {
    spdlog::trace("Shop: edit page finished");
    QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
    const QByteArray bytes = reply->readAll();
    const QString hash = Util::GetCsrfToken(bytes, "hash");
    if (hash.isEmpty()) {
        if (bytes.contains("Login Required")) {
            spdlog::error("Cannot update shop: the POESESSID is missing or invalid.");
        } else if (bytes.contains("Permission Denied")) {
            spdlog::error("Cannot update shop: the POESESSID may be invalid or associated with another account.");
        } else {
            spdlog::error("Cannot update shop: unable to extract CSRF token from the page. The thread ID may be invalid.");
        };
        m_submitting = false;
        return;
    };
    spdlog::trace("CSRF token found.");

    // now submit our edit
    // holy shit give me some html parser library please
    const QString page(bytes);
    QString title = Util::FindTextBetween(page, "<input type=\"text\" name=\"title\" id=\"title\" onkeypress=\"return&#x20;event.keyCode&#x21;&#x3D;13\" value=\"", "\">");
    if (title.isEmpty()) {
        spdlog::error("Cannot update shop: title is empty. Check if thread ID is valid.");
        m_submitting = false;
        reply->deleteLater();
        return;
    };

    QTimer::singleShot(500, this, [=]() { SubmitNextShop(title, hash); });
    reply->deleteLater();
}

void Shop::SubmitNextShop(const QString& title, const QString& hash)
{
    spdlog::debug("Shop: submitting the next shop.");

    const QString content = m_requests_completed < m_shop_data.size() ? m_shop_data[m_requests_completed] : "Empty";

    QUrlQuery query;
    query.addQueryItem("title", Util::Decode(title));
    query.addQueryItem("content", content);
    query.addQueryItem("notify_owner", "0");
    query.addQueryItem("hash", hash);
    query.addQueryItem("submit", "Submit");

    QByteArray data(query.query().toUtf8());
    QNetworkRequest request((QUrl(ShopEditUrl(m_requests_completed))));
    request.setHeader(QNetworkRequest::ContentTypeHeader, "application/x-www-form-urlencoded");
    request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    request.setRawHeader("Cache-Control", "max-age=0");
    request.setTransferTimeout(kEditThreadTimeout);

    QNetworkReply* submitted = m_network_manager.post(request, data);
    connect(submitted, &QNetworkReply::finished, this,
        [=]() {
            OnShopSubmitted(query, submitted);
            submitted->deleteLater();
        });
}

void Shop::OnShopSubmitted(QUrlQuery query, QNetworkReply* reply) {
    spdlog::debug("Shop: shop submission reply received.");
    const QByteArray bytes = reply->readAll();

    // Errors can show up in a couple different places. So far, the easiest way to identify
    // them seems to be to look for an html tag with the "class" attribute set to
    // "input-error" or "errors".
    // 
    // After this class attribute, there seems to always be an error message enclosed in
    // an list item tag, with varying differents between the class attribute and that tag.
    QRegularExpressionMatchIterator i = error_regex.globalMatch(bytes);
    if (i.hasNext()) {
        // Process one or more errors (I've never seen more than one in practice).
        int seconds = 0;
        while (i.hasNext()) {
            // We only know the error message if the list item element was found.
            const QRegularExpressionMatch error_match = i.next();
            const QString error_message = (error_match.lastCapturedIndex() > 0)
                ? QTextDocumentFragment::fromHtml(error_match.captured(1)).toPlainText()
                : "(Failed to parse the error message)";
            spdlog::error("Shop: error submitting shop thread: {}", error_message);
            if (error_message.startsWith("Failed to find item.", Qt::CaseInsensitive)) {
                spdlog::error("Shop: the stash index may be out of date. (Try Shop->\"Update stash index\")");
            } else if (error_message.startsWith("Security token has expired.")) {
                // This error would occur somewhat randomly before a delay was added in OnEditPageFinished.
                // With that delay, this error doesn't seem to happen any more, but we should probably
                // keep checking for it.
                if (seconds < 5) {
                    seconds = 5;
                    spdlog::trace("Shop: setting {} second delay.", seconds);
                };
            } else if (error_message.startsWith("Rate limiting", Qt::CaseInsensitive)) {
                // Look for a rate limiting error here, because there are no headers to check for
                // like the other API calls.
                const QRegularExpressionMatch ratelimit_match = ratelimit_regex.match(error_message);
                int ratelimit_delay = ratelimit_match.captured(1).toInt();
                if (ratelimit_delay == 0) {
                    spdlog::error("Shop: error parsing wait time from error message.");
                    m_submitting = false;
                    return;
                };
                if (seconds < ratelimit_delay) {
                    seconds = ratelimit_delay + 1;
                    spdlog::trace("Shop: setting {} second delay.", seconds);
                };
            } else {
                spdlog::error("Shop: unknown error fragment: {}", error_match.captured(0));
                spdlog::debug("Shop: the query was: {}", query.toString());
            };
        };
        if (seconds > 0) {
            // Resubmit if the errors indicate we only have to try again later.
            const int ms = seconds * 1000;
            const QString title = query.queryItemValue("title");
            const QString hash = Util::GetCsrfToken(bytes, "hash");
            spdlog::warn("Shop: resubmitting shop after {} seconds.", seconds);
            QTimer::singleShot(ms, this, [=]() { SubmitNextShop(title, hash); });
            return;
        } else {
            // Quit the update for any other error.
            m_submitting = false;
            return;
        };
    };

    // Keep legacy error-checking in place for now.
    QString page(bytes);
    QString error = Util::FindTextBetween(page, "<ul class=\"errors\"><li>", "</li></ul>");
    if (!error.isEmpty()) {
        spdlog::error("Shop: (DEPRECATED) Error while submitting shop to forums: {}", error);
        m_submitting = false;
        return;
    };
    // This slightly different error was encountered while debugging an issue with v0.9.9-beta.1.
    // It's possible GGG has updated the forums so the previous error checking is no longer
    // relavent, but that's not certain or documented anywhere, so let's do both.
    QString input_error = Util::FindTextBetween(page, "class=\"input-error\">", "</div>");
    if (!input_error.isEmpty()) {
        spdlog::error("Shop: (DEPRECATED) Input error while submitting shop to forums: {}", input_error);
        m_submitting = false;
        return;
    };
    // Let's err on the side of being cautious and look for an error the above code might
    // have missed. Otherwise errors might just silently fall through the cracks.
    for (auto& substr : { "class=\"errors\"", "class=\"input-error\"" }) {
        if (page.contains(substr)) {
            spdlog::error("Shop: (DEPRECATED) An error was detected but not handled while submitting shop to forums: {}", substr);
            spdlog::error(page);
            m_submitting = false;
            return;
        };
    };

    ++m_requests_completed;
    SubmitSingleShop();
}

void Shop::CopyToClipboard() {
    if (m_shop_data_outdated) {
        spdlog::warn("Shop: copying outdated shop data!");
    };
    if (m_shop_data.empty()) {
        spdlog::warn("Shop: there is nothing to copy to the clipboard");
        return;
    };
    if (m_shop_data.size() > 1) {
        spdlog::warn("Shop: you have multiple shops; only the first will be copied to the clipboard");
    };
    spdlog::trace("Shop: copying shop data to clipboard for thread: {}", m_shop_data[0]);
    QClipboard* clipboard = QApplication::clipboard();
    clipboard->setText(m_shop_data[0]);
}
