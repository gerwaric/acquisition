#include "tabcache.h"
#include "QsLog.h"
#include "network_info.h"

#include <QDir>
#include <QDateTime>
#include <QUrl>

// TabCache
//
// This class is a 'work around' for path of exile API not using 'ETag' headers
// and supporting existing cache methods that QNetworkDiskCache would have
// otherwise supported directly.  If they ever fix this we should be able to
// just remove this implementation and use QNetworkDiskCache directly.
//
// Currently the API sends headers that look like this:
//
// DEBUG 2016-01-16T13:04:10.054 "Server" "nginx/1.4.4"
// DEBUG 2016-01-16T13:04:10.054 "Date" "Sat, 16 Jan 2016 19:04:10 GMT"
// DEBUG 2016-01-16T13:04:10.054 "Content-Type" "application/json"
// DEBUG 2016-01-16T13:04:10.054 "Expires" "Thu, 19 Nov 1981 08:52:00 GMT"
// DEBUG 2016-01-16T13:04:10.055 "Cache-Control" "no-store, no-cache, must-revalidate, post-check=0, pre-check=0"
// DEBUG 2016-01-16T13:04:10.055 "Pragma" "no-cache"
// DEBUG 2016-01-16T13:04:10.055 "X-Frame-Options" "SAMEORIGIN"
//
// 'no-cache' - actually allows client to cache requests, but *requests*
// revalidation to use cached data (using ETag or Last Modified headers).
// clients can choose to ignore revalidation request and use possibly
// stale content.
//
// 'must-revalidate' - *requires* revalidation of cached content.  If no
// revalidation method exists don't use cached data.
//
// 'no-store' - *requires* client to not store content to disk.  This is the what
// really disables caching.
//
// So the plan is to basically just to ignore the Cache-Control and Pragma headers.
// This is achieved by overridding QNetworkDiskCache 'prepare' method so it
// effectively doesn't seem them.
//
// A helper method is needed however to create our network requests properly such
// such that the requests always prefer hitting in the cache.
//
// The method is:
//
// Request(url, <flags>);
//
// Where <flags> can be used to force an eviction, causing a reload
//   Request(url, TabCache::Refresh);
//
// If 'Refresh' is not specified we're guaranteed to hit in cache if entry exists
// and fetch otherwise.
//
// So, to minimize overall API requests it will become a matter of minimizing requests
// that ask for a refresh.  From the point of view of the rest of the application it will
// always appear as if we request all the tabs.

TabCache::TabCache(QObject* parent)
	:QNetworkDiskCache(parent)
{
}

QNetworkRequest TabCache::Request(const QUrl &url, Flags flags) {
	QNetworkRequest request{url};
	bool evicted = false;

	if (flags.testFlag(Refresh)) {
		remove(url);
		evicted = true;
	}

	// At this point we've evicted any request that should be refreshed, so we always
	// tell the 'real' request to prefer but not require the entry be in the cache.
	// If it is not in the cache it will be fetched from the network regardless.
    request.setAttribute(QNetworkRequest::CacheLoadControlAttribute, QNetworkRequest::PreferCache);
	request.setAttribute(QNetworkRequest::CacheSaveControlAttribute, QNetworkRequest::PreferCache);
	request.setHeader(QNetworkRequest::KnownHeaders::UserAgentHeader, USER_AGENT);
    request.setRawHeader("Cache-Control", "max-age=3600");
	QLOG_DEBUG() << "Evicted:" << evicted << ":" << url.toDisplayString();

	return request;
}

QIODevice *TabCache::prepare(const QNetworkCacheMetaData &metaData) {
	QNetworkCacheMetaData local{metaData};

	//Default policy based on received HTTP headers is to not save to disk.
	//Override here, and set proper expiration so items are put in cache
	//(setSaveToDisk) and valid when retrieved using metaData call -
	//(setExpirationDate)

	local.setSaveToDisk(true);

	// Need to set some reasonable length of time in which our cache entries
	// will expire.  It's possible we'll want to allow users to customize this.
	local.setExpirationDate(QDateTime::currentDateTime().addDays(kCacheExpireInDays));

	QNetworkCacheMetaData::RawHeaderList headers;

	for (auto &header: local.rawHeaders()) {
		// Modify Cache-Control headers - basically need to drop 'no-store'
		// as we want to store to cache and 'must-revalidate' as we don't have
		// ETag or last modified headers available to attempt re-validation. To
		// be on the safe side though just drop Cache-Control and Pragma headers.

		if (header.first == "Cache-Control") continue;
		if (header.first == "Pragma") continue;

        // GGG has switched to Cloudflare, so we need to remove these headers now.
        if (header.first == "CF-Cache-Status") continue;
        if (header.first == "CF-RAY") continue;

        // Remove rate limiting information from the metadata since it will be stale.
        if (header.first.startsWith("X-Rate-Limit")) continue;

		headers.push_back(header);
	}
	local.setRawHeaders(headers);

	return QNetworkDiskCache::prepare(local);
}

