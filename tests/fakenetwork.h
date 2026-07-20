#pragma once

#include <QNetworkReply>
#include <QNetworkRequest>
#include <QPointer>

#include <cstring>
#include <vector>

#include "ratelimit/ratelimitedreply.h"
#include "ratelimit/ratelimiter.h"

// An offline stand-in for a completed QNetworkReply. The body is served
// through readData() so QNetworkReply::readAll() works as usual.
//
// No Q_OBJECT: neither class adds signals, slots, or properties, and
// omitting it lets this header stay out of the build's moc set.
class FakeNetworkReply : public QNetworkReply
{
public:
    explicit FakeNetworkReply(const QNetworkRequest &request,
                              const QByteArray &body,
                              QNetworkReply::NetworkError error = QNetworkReply::NoError,
                              QObject *parent = nullptr,
                              const QList<QNetworkReply::RawHeaderPair> &headers = {},
                              int http_status = 0)
        : QNetworkReply(parent)
        , m_body(body)
    {
        setRequest(request);
        setUrl(request.url());
        setOpenMode(QIODevice::ReadOnly);
        setFinished(true);
        if (error != QNetworkReply::NoError) {
            setError(error, "fake network error");
        }
        for (const auto &header : headers) {
            setRawHeader(header.first, header.second);
        }
        if (http_status != 0) {
            setAttribute(QNetworkRequest::HttpStatusCodeAttribute, http_status);
        }
    }

    void abort() override {}
    bool isSequential() const override { return true; }
    qint64 bytesAvailable() const override
    {
        return (m_body.size() - m_offset) + QNetworkReply::bytesAvailable();
    }

protected:
    qint64 readData(char *data, qint64 maxSize) override
    {
        const qint64 n = std::min<qint64>(maxSize, m_body.size() - m_offset);
        if (n <= 0) {
            return -1;
        }
        std::memcpy(data, m_body.constData() + m_offset, size_t(n));
        m_offset += n;
        return n;
    }

private:
    QByteArray m_body;
    qint64 m_offset{0};
};

// A RateLimiter whose Submit() records requests instead of touching the
// network. Tests complete them with deliver(), synchronously and in any
// order. A request may be delivered more than once as long as the event
// loop has not run in between (the receiver only deleteLater()s the reply
// objects) — re-delivery is how a stale reply from a superseded update is
// simulated (F28).
class FakeRateLimiter : public RateLimiter
{
public:
    struct PendingRequest
    {
        QString endpoint;
        QNetworkRequest request;
        QPointer<RateLimitedReply> reply;
    };

    explicit FakeRateLimiter(NetworkManager &network_manager)
        : RateLimiter(network_manager)
    {}

    RateLimitedReply *Submit(const QString &endpoint, QNetworkRequest network_request) override
    {
        auto *reply = new RateLimitedReply();
        m_requests.push_back({endpoint, network_request, reply});
        return reply;
    }

    size_t requestCount() const { return m_requests.size(); }
    const PendingRequest &request(size_t i) const { return m_requests.at(i); }

    void deliver(size_t i,
                 const QByteArray &body,
                 QNetworkReply::NetworkError error = QNetworkReply::NoError)
    {
        PendingRequest &pending = m_requests.at(i);
        if (!pending.reply) {
            qFatal("FakeRateLimiter::deliver: request %zu has no reply to complete", i);
        }
        auto *network_reply = new FakeNetworkReply(pending.request, body, error);
        emit pending.reply->complete(network_reply);
    }

private:
    std::vector<PendingRequest> m_requests;
};
