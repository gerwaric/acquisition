// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

// M1-M2 measurement harness (items-pipeline M2, D10; recorded as a post-M2
// follow-up): measures the batch-submit `QueueUpdated` burst against
// status-widget frame time. The production shape being reproduced: the
// worker's batch submit queues one request per tab in a single loop turn;
// every Enqueue on the draining pump emits `QueueUpdated` synchronously,
// which fans through the hub (`RateLimiter::OnQueueUpdated` re-emit) into
// `RateLimitDialog::OnQueueUpdate` (direct connection, same thread). The
// dialog is constructed unconditionally at startup
// (`MainWindow::InitializeRateLimitDialog`), so the handler runs whether or
// not the dialog is visible. Before the D10 coalesce that handler was a
// top-level row scan plus `setText` per emission (the measured miss); with
// the coalesce it parks the latest value per policy and a single-shot flush
// timer applies the batch after the burst's loop turn ends.
//
// Not a test: run by hand, offscreen, in a Release build:
//
//   ./m1m2_benchmark
//   ./m1m2_benchmark --entries 8000
//
// The measured unit is the wall time of the whole submission loop — that is
// exactly how long the UI thread is blocked, since the burst happens in one
// loop turn — under three receiver configurations: dialog shown (user
// watching the status window), dialog hidden (the production default), and
// no dialog (hub cost alone, for attribution). A post-burst settle pass is
// timed separately: Qt coalesces the 2,000 repaints into the next frame, so
// that pass carries whatever repaint cost the burst deferred.
//
// The stack below the signal is the real one: a real RateLimiter over the
// offline FakeNetworkManager, the endpoint established through the real
// HEAD probe path, the real RateLimitManager pump suspended in its pacing
// sleep on the FakeScheduler (never advanced during the burst) so every
// enqueue takes the draining branch and emits. Excluded, deliberately: the
// worker's per-future continuation attach (worker-side cost, covered by
// M2-M2) and any real send (the gate never grants during the burst).

#include <QApplication>
#include <QCommandLineParser>
#include <QDateTime>
#include <QElapsedTimer>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QTreeWidget>

#include <algorithm>
#include <chrono>
#include <cstdio>
#include <memory>
#include <vector>

#include <spdlog/spdlog.h>

#include "fakenetworkmanager.h"
#include "fakescheduler.h"
#include "ratelimit/ratelimitdialog.h"
#include "ratelimit/ratelimiter.h"

namespace {

    using namespace std::chrono_literals;

    // The stash-fetch policy GGG actually serves at the 2,000-tab scale.
    constexpr const char *kPolicyName = "backend-item-request-limit";
    constexpr const char *kEndpoint = "stash-fetch";

    QByteArray rfcDateNow()
    {
        return QDateTime::currentDateTimeUtc().toString(Qt::RFC2822Date).toUtf8();
    }

    // Synthetic headers for a single-rule ("Ip") policy — the shape GGG
    // actually serves (see the captured example in tst_networkcapture.cpp).
    QList<QNetworkReply::RawHeaderPair> policyHeaders(const QByteArray &limit,
                                                      const QByteArray &state)
    {
        return {
            {"X-Rate-Limit-Policy", kPolicyName},
            {"X-Rate-Limit-Rules", "Ip"},
            {"X-Rate-Limit-Ip", limit},
            {"X-Rate-Limit-Ip-State", state},
            {"Date", rfcDateNow()},
        };
    }

    QNetworkRequest request(int i)
    {
        return QNetworkRequest(QUrl("https://api.example.test/stash/" + QString::number(i)));
    }

    void drainEvents()
    {
        for (int i = 0; i < 20; ++i) {
            QCoreApplication::processEvents();
            QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        }
    }

    void settle(FakeScheduler &scheduler)
    {
        for (int i = 0; i < 10; ++i) {
            drainEvents();
            scheduler.AdvanceBy(0ms);
        }
        drainEvents();
    }

    // The network outlives the limiter (declared first).
    struct Rig
    {
        FakeScheduler scheduler;
        FakeNetworkManager network;
        RateLimiter limiter{network, &scheduler};
    };

    enum class DialogMode { None, Hidden, Shown };

    const char *modeName(DialogMode mode)
    {
        switch (mode) {
        case DialogMode::None:
            return "no dialog (baseline)";
        case DialogMode::Hidden:
            return "dialog hidden (default)";
        case DialogMode::Shown:
            return "dialog shown";
        }
        return "?";
    }

    struct RunResult
    {
        double burst_ms = 0.0;
        double settle_ms = 0.0;
    };

    // One full cycle: fresh rig, establish the endpoint through the real
    // HEAD path, run the timed burst, then the timed post-burst settle.
    // When `verify_emissions` is set the burst is additionally observed by
    // a counting connection — an extra receiver perturbs every emission, so
    // verification runs are never the timed ones.
    RunResult runOnce(int entries, DialogMode mode, bool verify_emissions)
    {
        Rig rig;
        std::unique_ptr<RateLimitDialog> dialog;
        if (mode != DialogMode::None) {
            dialog = std::make_unique<RateLimitDialog>(nullptr, &rig.limiter);
            if (mode == DialogMode::Shown) {
                dialog->show();
            }
        }

        // Establish: the first submission fires the HEAD; finishing it
        // installs the policy (PolicyUpdate populates the dialog's row) and
        // forwards the entry into the pump, whose drain suspends in the
        // pacing sleep on the fake scheduler — draining, nothing sent.
        auto first = rig.limiter.SubmitFuture(kEndpoint, request(0));
        settle(rig.scheduler);
        if (rig.network.count() != 1
            || rig.network.sent(0).op != QNetworkAccessManager::HeadOperation) {
            qFatal("establishment did not fire exactly one HEAD probe");
        }
        rig.network.sent(0).reply->finish(policyHeaders("45:60:120", "0:60:0"), 200);
        drainEvents();

        int observed = 0;
        QMetaObject::Connection observer;
        if (verify_emissions) {
            observer = QObject::connect(&rig.limiter,
                                        &RateLimiter::QueueUpdate,
                                        [&observed](const QString &, int) { ++observed; });
        }

        QElapsedTimer timer;
        timer.start();
        for (int i = 1; i <= entries; ++i) {
            (void) rig.limiter.SubmitFuture(kEndpoint, request(i));
        }
        const qint64 burst_ns = timer.nsecsElapsed();

        timer.restart();
        drainEvents();
        const qint64 settle_ns = timer.nsecsElapsed();

        if (rig.network.count() != 1) {
            qFatal("the burst must not send anything (gate never granted)");
        }
        if (verify_emissions) {
            QObject::disconnect(observer);
            if (observed != entries) {
                qFatal("expected %d QueueUpdate emissions, observed %d", entries, observed);
            }
        }
        if (dialog) {
            // End-to-end delivery check with zero perturbation of the timed
            // path: the row's queue cell must converge to the final queue
            // depth (the drain holds entry 0; all burst entries are
            // waiting). With the D10 coalesce in place the value lands on
            // the flush timer, so this polls briefly instead of asserting
            // synchronously.
            auto *tree = dialog->findChild<QTreeWidget *>();
            if (!tree || tree->topLevelItemCount() != 1) {
                qFatal("the dialog does not show exactly one policy row");
            }
            const QString expected = QString::number(entries);
            QElapsedTimer deadline;
            deadline.start();
            while (tree->topLevelItem(0)->text(1) != expected) {
                if (deadline.elapsed() > 2000) {
                    qFatal("queue cell reads '%s', expected %s",
                           qPrintable(tree->topLevelItem(0)->text(1)),
                           qPrintable(expected));
                }
                QCoreApplication::processEvents(QEventLoop::WaitForMoreEvents, 25);
            }
        }

        return {burst_ns / 1e6, settle_ns / 1e6};
    }

    double median(std::vector<double> values)
    {
        std::sort(values.begin(), values.end());
        return values[values.size() / 2];
    }

} // namespace

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    QCommandLineParser parser;
    parser.addHelpOption();
    parser.addOption({"entries", "Burst size (default 2000).", "n", "2000"});
    parser.addOption({"reps", "Timed repetitions per configuration (default 7).", "n", "7"});
    parser.process(app);
    const int entries = parser.value("entries").toInt();
    const int reps = parser.value("reps").toInt();

    // Production default: traces in the hot path must be level-gated out,
    // not formatted.
    spdlog::set_level(spdlog::level::info);

    std::printf("M1-M2: %d-entry QueueUpdated burst, %d timed reps per configuration\n",
                entries,
                reps);
    std::printf("(one verification rep per configuration precedes the timed ones)\n\n");
    std::printf("%-26s %12s %12s %12s %14s\n",
                "configuration",
                "burst ms",
                "min ms",
                "max ms",
                "settle ms");

    double shown_median = 0.0;
    double baseline_median = 0.0;
    for (DialogMode mode : {DialogMode::None, DialogMode::Hidden, DialogMode::Shown}) {
        (void) runOnce(entries, mode, true);
        std::vector<double> burst;
        std::vector<double> settle;
        for (int r = 0; r < reps; ++r) {
            const RunResult result = runOnce(entries, mode, false);
            burst.push_back(result.burst_ms);
            settle.push_back(result.settle_ms);
        }
        const double med = median(burst);
        std::printf("%-26s %12.3f %12.3f %12.3f %14.3f\n",
                    modeName(mode),
                    med,
                    *std::min_element(burst.begin(), burst.end()),
                    *std::max_element(burst.begin(), burst.end()),
                    median(settle));
        if (mode == DialogMode::Shown) {
            shown_median = med;
        } else if (mode == DialogMode::None) {
            baseline_median = med;
        }
    }

    const double frame_ms = 1000.0 / 60.0;
    std::printf("\nper-emission (shown): %.2f us; status-widget marginal cost of the "
                "burst: %.3f ms vs one 60 Hz frame %.1f ms\n",
                shown_median * 1e3 / entries,
                shown_median - baseline_median,
                frame_ms);
    return 0;
}
