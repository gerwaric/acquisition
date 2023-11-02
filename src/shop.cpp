/*
	Copyright 2014 Ilya Zhuravlev

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
#include <QTextDocumentFragment>
#include <QTimer>
#include <QUrl>
#include <QUrlQuery>
#include "QsLog.h"

#include "application.h"
#include "buyoutmanager.h"
#include "datastore.h"
#include "itemsmanager.h"
#include "porting.h"
#include "util.h"
#include "mainwindow.h"
#include "replytimeout.h"
#include "network_info.h"

const std::string kPoeEditThread = "https://www.pathofexile.com/forum/edit-thread/";
const std::string kShopTemplateItems = "[items]";
const int kMaxCharactersInPost = 50000;
const int kSpoilerOverhead = 19; // "[spoiler][/spoiler]" length

// Use a regular expression to look for html errors.
const QRegularExpression error_regex(
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

const QRegularExpression ratelimit_regex(
	R"regex(You must wait (\d+) seconds.)regex",
	QRegularExpression::CaseInsensitiveOption);

Shop::Shop(Application& app) :
	app_(app),
	shop_data_outdated_(true),
	submitting_(false)
{
	threads_ = Util::StringSplit(app_.data().Get("shop"), ';');
	auto_update_ = app_.data().GetBool("shop_update", false);
	shop_template_ = app_.data().Get("shop_template");
	if (shop_template_.empty())
		shop_template_ = kShopTemplateItems;
}

void Shop::SetThread(const std::vector<std::string>& threads) {
	if (submitting_)
		return;
	threads_ = threads;
	app_.data().Set("shop", Util::StringJoin(threads, ";"));
	ExpireShopData();
	app_.data().Set("shop_hash", "");
}

void Shop::SetAutoUpdate(bool update) {
	auto_update_ = update;
	app_.data().SetBool("shop_update", update);
}

void Shop::SetShopTemplate(const std::string& shop_template) {
	shop_template_ = shop_template;
	app_.data().Set("shop_template", shop_template);
	ExpireShopData();
}
std::string Shop::SpoilerBuyout(Buyout& bo) {
	std::string out = "";
	out += "[spoiler=\"" + bo.BuyoutTypeAsPrefix();
	if (bo.IsPriced())
		out += " " + QString::number(bo.value).toStdString() + " " + bo.CurrencyAsTag();
	out += "\"]";
	return out;
}

void Shop::Update() {
	if (submitting_) {
		QLOG_WARN() << "Submitting shop right now, the request to update shop data will be ignored";
		return;
	}
	shop_data_outdated_ = false;
	shop_data_.clear();
	std::string data = "";
	std::vector<AugmentedItem> aug_items;
	AugmentedItem tmp = AugmentedItem();
	//Get all buyouts to be able to sort them
	for (auto& item : app_.items_manager().items()) {
		tmp.item = item.get();
		tmp.bo = app_.buyout_manager().Get(*item);

		if (!tmp.bo.IsPostable())
			continue;

		aug_items.push_back(tmp);
	}
	if (aug_items.size() == 0)
		return;
	std::sort(aug_items.begin(), aug_items.end());

	Buyout current_bo = aug_items[0].bo;
	data += SpoilerBuyout(current_bo);
	for (auto& aug : aug_items) {
		if (aug.bo.type != current_bo.type || aug.bo.currency != current_bo.currency || aug.bo.value != current_bo.value) {
			current_bo = aug.bo;
			data += "[/spoiler]";
			data += SpoilerBuyout(current_bo);
		}
		std::string item_string = aug.item->location().GetForumCode(app_.league());
		if (data.size() + item_string.size() + shop_template_.size() + kSpoilerOverhead + QString("[/spoiler]").size() > kMaxCharactersInPost) {
			data += "[/spoiler]";
			shop_data_.push_back(data);
			data = SpoilerBuyout(current_bo);
			data += item_string;
		} else {
			data += item_string;
		}
	}
	if (!data.empty())
		shop_data_.push_back(data);

	for (size_t i = 0; i < shop_data_.size(); ++i)
		shop_data_[i] = Util::StringReplace(shop_template_, kShopTemplateItems, "[spoiler]" + shop_data_[i] + "[/spoiler]");

	shop_hash_ = Util::Md5(Util::StringJoin(shop_data_, ";"));
}

void Shop::ExpireShopData() {
	shop_data_outdated_ = true;
}

void Shop::SubmitShopToForum(bool force) {
	if (submitting_) {
		QLOG_WARN() << "Already submitting your shop.";
		return;
	}
	if (threads_.empty()) {
		QLOG_ERROR() << "Asked to update a shop with no shop ID defined.";
		return;
	}

	if (shop_data_outdated_)
		Update();

	std::string previous_hash = app_.data().Get("shop_hash");
	// Don't update the shop if it hasn't changed
	if (previous_hash == shop_hash_ && !force) {
		QLOG_TRACE() << "Shop hash has not changed. Skipping update.";
		return;
	}

	if (threads_.size() < shop_data_.size()) {
		QLOG_WARN() << "Need" << shop_data_.size() - threads_.size() << "more shops defined to fit all your items.";
	}

	requests_completed_ = 0;
	submitting_ = true;
	SubmitSingleShop();
}

std::string Shop::ShopEditUrl(size_t idx) {
	return kPoeEditThread + threads_[idx];
}

void Shop::SubmitSingleShop() {
	CurrentStatusUpdate status = CurrentStatusUpdate();
	status.state = ProgramState::ShopSubmitting;
	status.progress = requests_completed_;
	status.total = threads_.size();
	if (requests_completed_ == threads_.size()) {
		status.state = ProgramState::ShopCompleted;
		submitting_ = false;
		app_.data().Set("shop_hash", shop_hash_);
	} else {
		// first, get to the edit-thread page to grab CSRF token
		QNetworkRequest request = QNetworkRequest(QUrl(ShopEditUrl(requests_completed_).c_str()));
		request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
		request.setRawHeader("Cache-Control", "max-age=0");
		QNetworkReply* fetched = app_.network_manager().get(request);
		new QReplyTimeout(fetched, kEditThreadTimeout);
		connect(fetched, SIGNAL(finished()), this, SLOT(OnEditPageFinished()));
	}
	emit StatusUpdate(status);
}

void Shop::OnEditPageFinished() {
	QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
	const QByteArray bytes = reply->readAll();
	const std::string hash = Util::GetCsrfToken(bytes, "hash");
	if (hash.empty()) {
		QLOG_ERROR() << "Can't update shop -- cannot extract CSRF token from the page. Check if thread ID is valid."
			<< "If you're using Steam to login make sure you use the same login method (steam or login/password) in Acquisition, Path of Exile website and Path of Exile game client."
			<< "For example, if you created a shop thread while using Steam to log into the website and then logged into Acquisition with login/password it will not work."
			<< "In this case you should either recreate your shop thread or use a correct login method in Acquisition.";
		submitting_ = false;
		return;
	} else {
		QLOG_TRACE() << "CSRF token is" << QString(hash.c_str());
	};

	// now submit our edit

	// holy shit give me some html parser library please
	const std::string page(bytes.constData(), bytes.size());
	std::string title = Util::FindTextBetween(page, "<input type=\"text\" name=\"title\" id=\"title\" onkeypress=\"return&#x20;event.keyCode&#x21;&#x3D;13\" value=\"", "\">");
	if (title.empty()) {
		QLOG_ERROR() << "Can't update shop -- title is empty. Check if thread ID is valid.";
		submitting_ = false;
		reply->deleteLater();
		return;
	};
	
	QTimer::singleShot(500, [=]() { SubmitNextShop(title, hash); });
	reply->deleteLater();
}

void Shop::SubmitNextShop(const std::string title, const std::string hash)
{
	QUrlQuery query;
	query.addQueryItem("title", Util::Decode(title).c_str());
	query.addQueryItem("content", requests_completed_ < shop_data_.size() ? shop_data_[requests_completed_].c_str() : "Empty");
	query.addQueryItem("notify_owner", "0");
	query.addQueryItem("hash", hash.c_str());
	query.addQueryItem("submit", "Submit");

	QByteArray data(query.query().toUtf8());
	QNetworkRequest request((QUrl(ShopEditUrl(requests_completed_).c_str())));
	request.setHeader(QNetworkRequest::ContentTypeHeader, "application/x-www-form-urlencoded");
	request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
	request.setRawHeader("Cache-Control", "max-age=0");

	QNetworkReply* submitted = app_.network_manager().post(request, data);
	new QReplyTimeout(submitted, kEditThreadTimeout);
	connect(submitted, &QNetworkReply::finished,
		[=]() {
			OnShopSubmitted(query, submitted);
			submitted->deleteLater();
		});
}

void Shop::OnShopSubmitted(QUrlQuery query, QNetworkReply* reply) {
	//QNetworkReply* reply = qobject_cast<QNetworkReply*>(QObject::sender());
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
			QLOG_ERROR() << "Error submitting shop thread:" << error_message;
			QLOG_TRACE() << "The html fragment containing the error is" << error_match.captured(0);
			// This error would occur somewhat randomly before a delay was added in OnEditPageFinished.
			// With that delay, this error doesn't seem to happen any more, but we should probably
			// keep checking for it.
			if (error_message.startsWith("Security token has expired.")) {
				if (seconds < 5) {
					seconds = 5;
					QLOG_TRACE() << "Setting" << seconds << "delay.";
				};
			};
			// Look for a rate limiting error here, because there are no headers to check for
			// like the other API calls.
			if (error_message.startsWith("Rate limiting")) {
				const QRegularExpressionMatch ratelimit_match = ratelimit_regex.match(error_message);
				int ratelimit_delay = ratelimit_match.captured(1).toInt();
				if (ratelimit_delay == 0) {
					QLOG_ERROR() << "Error parsing wait time from error message.";
					submitting_ = false;
					return;
				}
				if (seconds < ratelimit_delay) {
					seconds = ratelimit_delay + 1;
					QLOG_TRACE() << "Setting" << seconds << "second delay.";
				};
			};
		};
		if (seconds > 0) {
			// Resubmit if the errors indicate we only have to try again later.
			const int ms = seconds * 1000;
			const std::string title = query.queryItemValue("title").toStdString();
			const std::string hash = Util::GetCsrfToken(bytes, "hash");
			QLOG_WARN() << "Resubmitting shop after" << seconds << "seconds.";
			QTimer::singleShot(ms, [=]() { SubmitNextShop(title, hash); });
			return;
		} else {
			// Quit the update for any other error.
			submitting_ = false;
			return;
		};
	};

	// Keep legacy error-checking in place for now.
	std::string page(bytes.constData(), bytes.size());
	std::string error = Util::FindTextBetween(page, "<ul class=\"errors\"><li>", "</li></ul>");
	if (!error.empty()) {
		QLOG_ERROR() << "(DEPRECATED) Error while submitting shop to forums:" << error.c_str();
		submitting_ = false;
		return;
	}
	// This slightly different error was encountered while debugging an issue with v0.9.9-beta.1.
	// It's possible GGG has updated the forums so the previous error checking is no longer
	// relavent, but that's not certain or documented anywhere, so let's do both.
	std::string input_error = Util::FindTextBetween(page, "class=\"input-error\">", "</div>");
	if (!input_error.empty()) {
		QLOG_ERROR() << "(DEPRECATED) Input error while submitting shop to forums:" << input_error.c_str();
		submitting_ = false;
		return;
	}
	// Let's err on the side of being cautious and look for an error the above code might
	// have missed. Otherwise errors might just silently fall through the cracks.
	for (auto& substr : { "class=\"errors\"", "class=\"input-error\"" }) {
		if (page.find(substr) != std::string::npos) {
			QLOG_ERROR() << "(DEPRECATED) An error was detected but not handled while submitting shop to forums:" << substr;
			QLOG_ERROR() << page.c_str();
			submitting_ = false;
			return;
		};
	}

	++requests_completed_;
	SubmitSingleShop();
}

void Shop::CopyToClipboard() {
	if (shop_data_outdated_)
		Update();

	if (shop_data_.empty())
		return;

	if (shop_data_.size() > 1) {
		QLOG_WARN() << "You have more than one shop, only the first one will be copied.";
	}

	QClipboard* clipboard = QApplication::clipboard();
	clipboard->setText(QString(shop_data_[0].c_str()));
}
