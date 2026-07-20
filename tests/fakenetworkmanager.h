// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QPointer>

#include <vector>

#include "fakesender.h"
#include "util/networkmanager.h"

// An offline NetworkManager for hub-level tests: createRequest is the one
// funnel Qt routes head()/get() through, so overriding it intercepts the
// RateLimiter's HEAD probes and the pumps' sends alike. Every request is
// recorded and answered with an unfinished InFlightReply for the test to
// complete — real replies are never synchronously finished, and the hub's
// setup path relies on that (a probe must suspend before its outcome).
//
// No Q_OBJECT: no new signals, slots, or properties (same reasoning as
// fakenetwork.h — keeps this header out of the build's moc set).
class FakeNetworkManager : public NetworkManager
{
public:
    struct Sent
    {
        QNetworkAccessManager::Operation op;
        QNetworkRequest request;
        QPointer<InFlightReply> reply; // nulls once its owner releases it
    };

    int count() const { return static_cast<int>(m_sent.size()); }
    const Sent &sent(size_t i) const { return m_sent.at(i); }

    int headCount() const
    {
        int heads = 0;
        for (const auto &sent : m_sent) {
            if (sent.op == QNetworkAccessManager::HeadOperation) {
                ++heads;
            }
        }
        return heads;
    }

protected:
    QNetworkReply *createRequest(QNetworkAccessManager::Operation op,
                                 const QNetworkRequest &request,
                                 QIODevice *) override
    {
        auto *reply = new InFlightReply(request, this);
        m_sent.push_back({op, request, reply});
        return reply;
    }

private:
    std::vector<Sent> m_sent;
};
