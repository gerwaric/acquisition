// Phase-0 batch-scale measurement (network-redesign spec, phasing step 0,
// "still open from this step").
//
// Question: batch submission at the 2,000-tab scale the items-pipeline doc
// names — peak memory and abort cost for the full promise/future/frame/token
// population. Measure first; bounded flow control is speculative machinery
// until numbers demand it.
//
// The program builds the spec's production topology per entry (D1/D2/D6/D7):
//
//   - a pump queue entry in a std::deque: QNetworkRequest (realistic URL,
//     auth + UA headers, 10s transfer timeout), endpoint label, submission
//     timestamp, the QPromise, and the update's stop_token;
//   - the facade chain: `parent.future().then(parse)` returning the CHILD
//     future; no parent future is retained after chaining (D2);
//   - one eager per-fetch worker coroutine per entry, suspended on
//     `co_await qCoro(child).takeResult()` (D6/S1-7), post-await stop check
//     first (IR2), handle stored in a worker-owned container (R4-3);
//   - one std::stop_source per update; every entry carries its token (D2).
//
// Scenario measured: the maximal standing population — every entry still
// queued (none dispatched; mid-update at most the gate cap of 2 is ever
// in-flight, negligible) — then a full abort:
//
//   request_stop() → drain loop pops each entry, sees the stopped token at
//   the dequeue checkpoint, completes it Canceled (the synchronous burst,
//   which also runs the facade's pass-through parse continuations on the
//   completing stack) → the event loop delivers the queued QFutureWatcher
//   resumptions (workers resume, see stop, return silently) → the deferred
//   sweep destroys the completed handles (D6).
//
// Not measured here: the stop-interruptible pacing-sleep wakeup that starts
// a real abort (~22ms, one per policy pump — S1-5), and payload memory
// (canceled entries never carry payloads; success-path payloads are an
// items-pipeline concern, not machinery cost).
//
// CHECK lines are internal validity (exit code = failures); the numbers are
// the deliverable. Build with -DCMAKE_BUILD_TYPE=Release for quotable
// numbers; the tool prints its build flavor.

#include <QCoroFuture>
#include <QCoroTask>

#include <QCoreApplication>
#include <QDateTime>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QFuture>
#include <QNetworkRequest>
#include <QPromise>
#include <QTimer>
#include <QUrl>

#include <cstdio>
#include <cstdlib>
#include <deque>
#include <expected>
#include <memory>
#include <stop_token>
#include <vector>

#ifdef __APPLE__
#include <mach/mach.h>
#include <malloc/malloc.h>
#endif

static int g_checkFailures = 0;

static void check(bool ok, const char *what)
{
    std::printf("  %s CHECK   %s\n", ok ? "[ok]" : "[!!]", what);
    if (!ok) {
        ++g_checkFailures;
    }
}

// ---------------------------------------------------------------------------
// Memory instrumentation (macOS): allocator bytes in use across all malloc
// zones (the precise machinery cost) and the task's physical footprint (what
// the OS actually charges the process).
// ---------------------------------------------------------------------------

struct MemSnapshot
{
    long long heapInUse = 0;
    long long footprint = 0;
};

static MemSnapshot memNow()
{
    MemSnapshot m;
#ifdef __APPLE__
    malloc_statistics_t stats{};
    malloc_zone_statistics(nullptr, &stats);
    m.heapInUse = static_cast<long long>(stats.size_in_use);
    task_vm_info_data_t vm{};
    mach_msg_type_number_t count = TASK_VM_INFO_COUNT;
    if (task_info(mach_task_self(), TASK_VM_INFO, reinterpret_cast<task_info_t>(&vm), &count)
        == KERN_SUCCESS) {
        m.footprint = static_cast<long long>(vm.phys_footprint);
    }
#endif
    return m;
}

static double asMB(long long bytes)
{
    return static_cast<double>(bytes) / (1024.0 * 1024.0);
}

// ---------------------------------------------------------------------------
// The spec's boundary types, shaped like D1/D7 (representative, not exact).
// ---------------------------------------------------------------------------

enum class ErrorKind { Network, Http, Parse, Protocol, RateLimited, Internal, Canceled };

struct FetchError
{
    ErrorKind kind = ErrorKind::Internal;
    int status = 0;
    int attempts = 0;
};

using RawOutcome = std::expected<QByteArray, FetchError>;

struct ParsedTab
{
    QString id;
    QString name;
    QString type;
    std::vector<int> items;
};

using ParsedOutcome = std::expected<ParsedTab, FetchError>;

// A pump queue entry (D1): request, endpoint label, capture timestamp,
// promise, and the update's stop token.
struct Entry
{
    QNetworkRequest request;
    QString endpoint;
    QDateTime submitted;
    QPromise<RawOutcome> promise;
    std::stop_token token;
};

struct BatchStats
{
    int expected = 0;
    int finished = 0;     // coroutines that ran to co_return (measurement bookkeeping)
    int canceled = 0;     // outcomes observed as Canceled
    int staleResumes = 0; // post-await stop check fired (abort: all of them)
    int parsesRan = 0;    // parse continuations that did real work (abort: none)
};

struct Update
{
    std::stop_source stop;
    std::deque<Entry> queue;            // the pump's deque (D3)
    std::vector<QCoro::Task<>> handles; // worker-owned handle container (R4-3)
    BatchStats stats;
};

// The per-fetch worker coroutine (D6): one await site, post-await stop check
// before touching anything, then completion counting.
static QCoro::Task<> fetchOne(BatchStats *s, QFuture<ParsedOutcome> child, std::stop_token tok)
{
    ParsedOutcome r = co_await qCoro(child).takeResult();
    if (tok.stop_requested()) { // post-await identity invariant (IR2)
        ++s->staleResumes;
        if (!r.has_value() && r.error().kind == ErrorKind::Canceled) {
            ++s->canceled;
        }
        ++s->finished; // return silently (D2) — counters only
        co_return;
    }
    ++s->finished;
}

// Batch submission (D6): build every entry, chain the facade parse, launch
// the eager worker coroutine (it runs to its co_await and suspends), keep
// only the child future inside the frame and the handle in the container.
static void submitBatch(Update &u, int n)
{
    u.stats.expected = n;
    u.handles.reserve(n);
    const QByteArray bearer = QByteArrayLiteral("Bearer ") + QByteArray(60, 'x');
    for (int i = 0; i < n; ++i) {
        Entry &e = u.queue.emplace_back();
        const QString tabId = QString::asprintf("%010llx",
                                                static_cast<unsigned long long>(i) * 2654435761ull);
        e.request = QNetworkRequest(
            QUrl(QStringLiteral("https://api.pathofexile.com/stash/Standard/") + tabId));
        e.request.setRawHeader("Authorization", bearer);
        e.request.setRawHeader("User-Agent", "acquisition/spike (batch-scale measurement)");
        e.request.setTransferTimeout(10'000); // the facade's invariant (R5-3)
        e.endpoint = QStringLiteral("getStash");
        e.submitted = QDateTime::currentDateTimeUtc();
        e.token = u.stop.get_token();
        e.promise.start();
        // The facade (D7): chain parsing, return the child, retain no parent.
        QFuture<ParsedOutcome> child = e.promise.future().then(
            [s = &u.stats](RawOutcome r) -> ParsedOutcome {
                if (!r.has_value()) {
                    return std::unexpected(r.error()); // pass-through, no parse
                }
                ++s->parsesRan;
                ParsedTab tab;
                tab.id = QString::fromUtf8(r->left(10));
                tab.name = QStringLiteral("Stash");
                tab.type = QStringLiteral("PremiumStash");
                return tab;
            });
        u.handles.push_back(fetchOne(&u.stats, std::move(child), e.token));
    }
}

// The drain's abort path (D3): pop each entry, the dequeue checkpoint sees
// the stopped token, complete Canceled. The facade's pass-through
// continuation runs synchronously on this stack as each promise finishes.
static void drainCanceled(Update &u)
{
    while (!u.queue.empty()) {
        Entry e = std::move(u.queue.front());
        u.queue.pop_front();
        e.promise.addResult(std::unexpected(FetchError{ErrorKind::Canceled}));
        e.promise.finish();
    } // entry (and its promise) dies here — the child's shared state holds the outcome
}

// ---------------------------------------------------------------------------

static void measureScale(int n, bool warmup)
{
    std::printf("\n[batch-scale measurement, N=%d%s]\n", n, warmup ? " (warm-up)" : "");

    const MemSnapshot base = memNow();
    QElapsedTimer t;

    // --- batch submit: the full standing population comes into existence ---
    t.start();
    auto u = std::make_unique<Update>();
    submitBatch(*u, n);
    const double submitMs = t.nsecsElapsed() / 1e6;
    const MemSnapshot pop = memNow();

    check(u->stats.finished == 0, "no worker resumed during submission (all suspended)");
    check(static_cast<int>(u->handles.size()) == n, "one owned handle per entry");

    // --- abort: request_stop + the drain's synchronous Canceled burst ---
    t.restart();
    u->stop.request_stop();
    drainCanceled(*u);
    const double drainMs = t.nsecsElapsed() / 1e6;
    const bool resumedDuringDrain = u->stats.finished != 0;
    const MemSnapshot afterDrain = memNow();

    check(!resumedDuringDrain,
          "no worker resumed on the completing stack (QFutureWatcher delivery is queued)");

    // --- the event loop delivers the queued resumptions ---
    t.restart();
    QElapsedTimer guard;
    guard.start();
    while (u->stats.finished < n && guard.elapsed() < 30'000) {
        QCoreApplication::processEvents(QEventLoop::AllEvents | QEventLoop::WaitForMoreEvents, 100);
    }
    const double resumeMs = t.nsecsElapsed() / 1e6;
    const MemSnapshot afterResume = memNow();

    check(u->stats.finished == n, "every worker coroutine resumed and finished");
    check(u->stats.canceled == n, "every worker observed a Canceled outcome");
    check(u->stats.staleResumes == n, "every post-await stop check fired (IR2)");
    check(u->stats.parsesRan == 0, "no parse continuation did real work on the abort path");

    bool allReady = true;
    for (const auto &h : u->handles) {
        allReady = allReady && h.isReady();
    }
    check(allReady, "every handle reports ready before the sweep (sweep destroys completed only)");

    // --- deferred sweep (D6): destroy the completed handles ---
    t.restart();
    u->handles.clear();
    const double sweepMs = t.nsecsElapsed() / 1e6;
    const MemSnapshot afterSweep = memNow();

    u.reset(); // remaining update state (empty deque, stop_source, counters)
    const MemSnapshot afterTeardown = memNow();

    const double perEntry = static_cast<double>(pop.heapInUse - base.heapInUse) / n;
    std::printf("       population:  heap +%.2f MB (%.0f bytes/entry), footprint +%.2f MB\n",
                asMB(pop.heapInUse - base.heapInUse),
                perEntry,
                asMB(pop.footprint - base.footprint));
    std::printf("       submit burst:            %8.2f ms  (build + chain + launch, synchronous)\n",
                submitMs);
    std::printf("       abort: Canceled burst:   %8.2f ms  (drain completes %d promises, "
                "continuations inline)\n",
                drainMs,
                n);
    std::printf("       abort: resumption drain: %8.2f ms  (%d queued watcher resumptions)\n",
                resumeMs,
                n);
    std::printf("       abort: handle sweep:     %8.2f ms  (%d completed frames destroyed)\n",
                sweepMs,
                n);
    std::printf("       abort total:             %8.2f ms  (+ one ~22ms pacing-sleep wake per "
                "pump, S1-5)\n",
                drainMs + resumeMs + sweepMs);
    std::printf("       heap deltas vs baseline: after drain %+.2f MB, after resume %+.2f MB, "
                "after sweep %+.2f MB, after teardown %+.2f MB\n",
                asMB(afterDrain.heapInUse - base.heapInUse),
                asMB(afterResume.heapInUse - base.heapInUse),
                asMB(afterSweep.heapInUse - base.heapInUse),
                asMB(afterTeardown.heapInUse - base.heapInUse));
    std::printf("       footprint vs baseline:   after teardown %+.2f MB\n",
                asMB(afterTeardown.footprint - base.footprint));
}

int main(int argc, char **argv)
{
    QCoreApplication app(argc, argv);
#ifdef NDEBUG
    const char *flavor = "release (NDEBUG)";
#else
    const char *flavor = "debug/unoptimized — configure with -DCMAKE_BUILD_TYPE=Release for "
                         "quotable numbers";
#endif
    std::printf("QCoro v0.13.0 batch-scale measurement (Qt %s) — build flavor: %s\n",
                qVersion(),
                flavor);
#ifndef __APPLE__
    std::printf("NOTE: memory instrumentation is macOS-only; sizes will read 0 here\n");
#endif

    // Default: one small warm-up (pages in allocator and code paths, and gives
    // a second point for scaling linearity), then the 2,000-tab scale.
    std::vector<int> scales;
    for (int i = 1; i < argc; ++i) {
        const int n = std::atoi(argv[i]);
        if (n > 0) {
            scales.push_back(n);
        }
    }
    if (scales.empty()) {
        scales = {200, 2000};
    }

    for (size_t i = 0; i < scales.size(); ++i) {
        measureScale(scales[i], scales.size() > 1 && i == 0);
    }

    std::printf("\n%d CHECK failure(s)\n", g_checkFailures);
    return g_checkFailures;
}
