// Phase-0 QCoro integration spike (network-redesign spec, phasing step 0).
//
// Empirically verifies (or falsifies) the QCoro v0.13.0 semantics the frozen
// spec relies on. Each experiment prints CHECK lines (internal validity of the
// experiment itself — a failed CHECK means the experiment is broken) and
// FINDING lines (the observed semantics, phrased as facts). Spec impact is
// analyzed in docs/design/ — this binary only reports what happens.
//
// Exit code: number of failed CHECKs (0 = every experiment ran as designed).

#include <QCoroFuture>
#include <QCoroNetworkReply>
#include <QCoroTask>
#include <QCoroTimer>

#include <QCoreApplication>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QFuture>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QPointer>
#include <QPromise>
#include <QTcpServer>
#include <QTimer>
#include <QUrl>

#include <atomic>
#include <chrono>
#include <cstdio>
#include <functional>
#include <memory>
#include <optional>
#include <stdexcept>
#include <stop_token>

using namespace std::chrono_literals;

static int g_checkFailures = 0;

static void check(bool ok, const char *what)
{
    std::printf("  %s CHECK   %s\n", ok ? "[ok]" : "[!!]", what);
    if (!ok) {
        ++g_checkFailures;
    }
}

static void finding(const char *what)
{
    std::printf("  ==== FINDING %s\n", what);
}

static void section(const char *name)
{
    std::printf("\n[%s]\n", name);
}

// Process events for roughly `ms` milliseconds.
static void spin(int ms)
{
    QEventLoop loop;
    QTimer::singleShot(ms, &loop, &QEventLoop::quit);
    loop.exec();
}

// Process events until `done` returns true, or fail after a generous timeout.
// Condition-driven waiting keeps the experiments robust under load — fixed
// spin windows were an intermittent-flake source.
static bool spinUntil(const std::function<bool()> &done, int maxMs = 5000)
{
    QElapsedTimer timer;
    timer.start();
    while (!done() && timer.elapsed() < maxMs) {
        spin(10);
    }
    return done();
}

// Frame-local RAII sentinel: proves whether a suspended frame's locals'
// destructors ran (the R6-1 reply-release mechanism the spec leans on).
struct Sentinel
{
    explicit Sentinel(bool *flag)
        : m_flag(flag)
    {}
    Sentinel(const Sentinel &) = delete;
    Sentinel &operator=(const Sentinel &) = delete;
    ~Sentinel() { *m_flag = true; }

private:
    bool *m_flag;
};

// Copy/move-counting payload for the result() vs takeResult() experiment.
struct Counted
{
    static inline int copies = 0;
    static inline int moves = 0;
    static void reset()
    {
        copies = 0;
        moves = 0;
    }
    int v = 0;
    Counted() = default;
    explicit Counted(int x)
        : v(x)
    {}
    Counted(const Counted &o)
        : v(o.v)
    {
        ++copies;
    }
    Counted(Counted &&o) noexcept
        : v(o.v)
    {
        ++moves;
    }
    Counted &operator=(const Counted &o)
    {
        v = o.v;
        ++copies;
        return *this;
    }
    Counted &operator=(Counted &&o) noexcept
    {
        v = o.v;
        ++moves;
        return *this;
    }
};

// ---------------------------------------------------------------------------
// E1: awaiting an already-finished QFuture — does the coroutine suspend at all?
// (Worker initialize-before-launch invariant: continuations on ready futures
// run synchronously.)
// ---------------------------------------------------------------------------

struct E1State
{
    bool resumed = false;
    int value = 0;
};

static QCoro::Task<> e1_awaitReady(E1State *s)
{
    QFuture<int> f = QtFuture::makeReadyValueFuture(42);
    s->value = co_await f;
    s->resumed = true;
}

static void e1()
{
    section("E1: co_await on an already-finished QFuture");
    E1State s;
    auto t = e1_awaitReady(&s);
    check(s.resumed && s.value == 42,
          "coroutine ran to completion during the call, no event loop involved");
    check(t.isReady(), "task handle reports ready immediately");
    if (s.resumed) {
        finding("awaiting a finished future never suspends: continuation runs synchronously inside "
                "the call");
    } else {
        finding("awaiting a finished future SUSPENDS (needs the event loop to resume)");
    }

    bool thenRan = false;
    QFuture<int> f = QtFuture::makeReadyValueFuture(1);
    auto f2 = f.then([&thenRan](int) { thenRan = true; });
    check(f2.isFinished(), "contextless .then chain on a finished future is itself finished");
    if (thenRan) {
        finding(
            "QFuture::then (no context) on a finished future runs the continuation synchronously");
    } else {
        finding("QFuture::then (no context) on a finished future did NOT run synchronously");
    }
}

// ---------------------------------------------------------------------------
// E2: promise-completion re-entrancy — does finishing a QPromise resume the
// suspended awaiter synchronously on the finishing stack, or queued?
// (R4-2 requires queued stop wakeups; D5's complete-promise-last rule exists
// to bound re-entrancy if this is synchronous.)
// ---------------------------------------------------------------------------

struct E2State
{
    std::atomic<bool> insideFinish{false};
    bool resumedInsideFinish = false;
    bool resumed = false;
};

static QCoro::Task<> e2_await(E2State *s, QFuture<int> f)
{
    co_await f;
    s->resumedInsideFinish = s->insideFinish.load();
    s->resumed = true;
}

static void e2()
{
    section("E2: resumption when a QPromise finishes while a coroutine is suspended on its future");
    E2State s;
    QPromise<int> p;
    p.start();
    auto t = e2_await(&s, p.future());
    check(!s.resumed, "coroutine is suspended on the unfinished future");
    s.insideFinish = true;
    p.addResult(7);
    p.finish();
    s.insideFinish = false;
    const bool resumedSynchronously = s.resumed;
    check(spinUntil([&] { return s.resumed; }), "coroutine resumed once events were processed");
    if (resumedSynchronously || s.resumedInsideFinish) {
        finding("QPromise::finish() resumed the awaiter SYNCHRONOUSLY on the finishing stack");
    } else {
        finding("QPromise::finish() returned before the awaiter resumed — QCoro's QFuture awaiter "
                "delivers "
                "resumption through the event loop (QFutureWatcher), i.e. queued");
    }
}

// ---------------------------------------------------------------------------
// E3: result() vs takeResult() on the single-consumer path — copy/move counts.
// ---------------------------------------------------------------------------

struct E3State
{
    int value = 0;
    bool done = false;
};

static QCoro::Task<> e3_direct(E3State *s, QFuture<Counted> f)
{
    Counted c = co_await f;
    s->value = c.v;
    s->done = true;
}

static QCoro::Task<> e3_take(E3State *s, QFuture<Counted> f)
{
    Counted c = co_await qCoro(f).takeResult();
    s->value = c.v;
    s->done = true;
}

static QFuture<Counted> e3_makeReady(int v)
{
    QPromise<Counted> p;
    p.start();
    p.addResult(Counted(v));
    p.finish();
    return p.future();
}

static void e3()
{
    section("E3: result() vs takeResult() on the single-consumer path");
    {
        QFuture<Counted> f = e3_makeReady(5);
        Counted::reset();
        E3State s;
        auto t = e3_direct(&s, f);
        check(s.done && s.value == 5, "direct co_await returned the payload");
        std::printf("       plain `co_await future`:            %d copies, %d moves\n",
                    Counted::copies,
                    Counted::moves);
        check(Counted::copies >= 1,
              "plain co_await copies the payload out of the future (QFuture::result())");
    }
    {
        QFuture<Counted> f = e3_makeReady(6);
        Counted::reset();
        E3State s;
        auto t = e3_take(&s, f);
        check(s.done && s.value == 6, "takeResult co_await returned the payload");
        std::printf("       `co_await qCoro(f).takeResult()`:   %d copies, %d moves\n",
                    Counted::copies,
                    Counted::moves);
        check(Counted::copies == 0, "takeResult path performs no copies (moves only)");
        finding("qCoro(future).takeResult() is the move-out path for the worker's single-consumer "
                "payloads");
    }
}

// ---------------------------------------------------------------------------
// E4: stop-interruptible sleep prototype — the primitive D2 requires: wakes on
// stop, resumption queued (request_stop() returns before the sleeper resumes).
// ---------------------------------------------------------------------------

struct SleepShared
{
    QPromise<bool> p;
    std::atomic_flag fired = ATOMIC_FLAG_INIT;
    void complete(bool stopped)
    {
        if (!fired.test_and_set()) {
            p.addResult(stopped);
            p.finish();
        }
    }
};

// Returns true if woken by stop, false if the full duration elapsed.
static QCoro::Task<bool> stopSleep(std::chrono::milliseconds ms, std::stop_token tok, QObject *ctx)
{
    if (tok.stop_requested()) {
        co_return true; // checkpoint: a pre-stopped wait never suspends
    }
    auto st = std::make_shared<SleepShared>();
    st->p.start();
    QFuture<bool> f = st->p.future();
    QTimer::singleShot(ms, ctx, [st] { st->complete(false); });
    std::stop_callback cb(tok, [st, ctx] {
        // Queued on purpose: R4-2 forbids resuming coroutines on the
        // request_stop() stack.
        QMetaObject::invokeMethod(ctx, [st] { st->complete(true); }, Qt::QueuedConnection);
    });
    const bool stopped = co_await f;
    co_return stopped;
}

struct E4State
{
    std::atomic<bool> insideRequestStop{false};
    bool resumedInsideRequestStop = false;
    bool stopped = false;
    bool done = false;
    qint64 elapsedMs = 0;
};

static QCoro::Task<> e4_run(E4State *s,
                            std::chrono::milliseconds ms,
                            std::stop_token tok,
                            QObject *ctx)
{
    QElapsedTimer timer;
    timer.start();
    s->stopped = co_await stopSleep(ms, tok, ctx);
    s->elapsedMs = timer.elapsed();
    s->resumedInsideRequestStop = s->insideRequestStop.load();
    s->done = true;
}

static void e4()
{
    section("E4: stop-interruptible sleep primitive (queued wake on stop)");
    QObject ctx;

    // a) stop interrupts a long sleep
    {
        std::stop_source src;
        E4State s;
        auto t = e4_run(&s, 5000ms, src.get_token(), &ctx);
        spin(20);
        check(!s.done, "5s sleep still suspended after 20ms");
        s.insideRequestStop = true;
        src.request_stop();
        s.insideRequestStop = false;
        const bool resumedDuringRequestStop = s.done;
        check(spinUntil([&] { return s.done; }) && s.stopped,
              "sleeper woke promptly and reported 'stopped'");
        check(!resumedDuringRequestStop && !s.resumedInsideRequestStop,
              "request_stop() returned before the sleeper resumed (queued wakeup, R4-2)");
        check(s.elapsedMs < 1000, "wake latency far below the 5s sleep duration");
        std::printf("       stop wake latency: ~%lldms after request_stop()\n",
                    static_cast<long long>(s.elapsedMs));
    }

    // b) undisturbed sleep completes normally
    {
        std::stop_source src;
        E4State s;
        auto t = e4_run(&s, 50ms, src.get_token(), &ctx);
        check(spinUntil([&] { return s.done; }) && !s.stopped,
              "undisturbed 50ms sleep completed normally");
    }

    // c) pre-stopped token: never suspends
    {
        std::stop_source src;
        src.request_stop();
        E4State s;
        auto t = e4_run(&s, 5000ms, src.get_token(), &ctx);
        check(s.done && s.stopped,
              "sleep with an already-stopped token completed synchronously (checkpoint)");
    }
    finding("a stop-interruptible sleep built on QPromise + queued stop_callback delivers R4-2's "
            "contract");
}

// ---------------------------------------------------------------------------
// E5: destroying the Task handle while suspended on a timer — destroy or
// detach? THE load-bearing question for the spec's shutdown-by-destruction
// (R5-2/R6-1).
// ---------------------------------------------------------------------------

struct E5State
{
    bool localDestroyed = false;
    bool resumed = false;
};

static QCoro::Task<> e5_sleeper(E5State *s, std::chrono::milliseconds duration)
{
    Sentinel guard(&s->localDestroyed);
    co_await QCoro::sleepFor(duration);
    s->resumed = true;
}

static void e5()
{
    section("E5: destroy Task handle while suspended on a timer");
    E5State s;
    {
        auto t = e5_sleeper(&s, 100ms);
        check(!t.isReady(), "sleeper is suspended");
    } // Task handle destroyed here, coroutine still suspended
    const bool destroyedWithHandle = s.localDestroyed;
    if (destroyedWithHandle) {
        finding(
            "destroying the Task handle destroyed the suspended frame (locals' destructors ran)");
    } else {
        finding(
            "destroying the Task handle mid-suspend DETACHES the coroutine: the frame survives and "
            "frame locals' destructors do NOT run (falsifies R5-2/R6-1's mechanism)");
    }
    spinUntil([&] { return s.localDestroyed; });
    if (s.resumed) {
        finding("the detached coroutine RESUMED when its timer later fired — a destroyed handle "
                "does not "
                "prevent resumption while an event loop still runs");
    }
    check(s.localDestroyed,
          "frame was destroyed by the end of the experiment (either with the handle, or at "
          "completion)");
    if (!destroyedWithHandle && s.resumed && s.localDestroyed) {
        finding("a detached coroutine that runs to completion self-destroys at final_suspend "
                "(refcount drops to 0)");
    }
}

// ---------------------------------------------------------------------------
// E6: same question for the QFuture awaitable; plus the never-completed case
// (frame leak) checked at the end of main.
// ---------------------------------------------------------------------------

struct E6State
{
    bool localDestroyed = false;
    bool resumed = false;
};

static QCoro::Task<> e6_awaiter(E6State *s, QFuture<int> f)
{
    Sentinel guard(&s->localDestroyed);
    co_await f;
    s->resumed = true;
}

// E6a state is global: the never-finished frame is deliberately leaked and
// re-checked at the end of main.
static E6State g_e6a;
static QPromise<int> *g_e6aPromise = nullptr;

static void e6()
{
    section("E6: destroy Task handle while suspended on a QFuture");

    // a) promise never finishes, no live handle: what happens to the frame?
    g_e6aPromise = new QPromise<int>();
    g_e6aPromise->start();
    {
        auto t = e6_awaiter(&g_e6a, g_e6aPromise->future());
    }
    if (!g_e6a.localDestroyed) {
        finding("with the handle destroyed and the future never finishing, the suspended frame is "
                "LEAKED — "
                "QCoro 0.13 has no way to destroy a suspended detached coroutine from outside");
    } else {
        finding("frame was destroyed with the handle despite the pending future");
    }

    // b) finishing the promise after the handle died: does the frame resume?
    E6State s;
    QPromise<int> p;
    p.start();
    {
        auto t = e6_awaiter(&s, p.future());
    }
    check(!s.localDestroyed, "frame detached (matches E5)");
    p.addResult(1);
    p.finish();
    spinUntil([&] { return s.localDestroyed; });
    if (s.resumed) {
        finding(
            "finishing the future after the handle was destroyed RESUMED the detached coroutine "
            "(QFutureWatcher lives in the frame, so it survives handle destruction)");
    }
    check(s.localDestroyed, "detached frame self-destroyed after completing");
}

// ---------------------------------------------------------------------------
// E7: the QNetworkReply awaitable — detach, later finished emission, and the
// dispatch-time RAII reply wrapper (D3).
// ---------------------------------------------------------------------------

struct E7State
{
    bool localDestroyed = false;
    bool resumed = false;
    QPointer<QNetworkReply> reply;
};

static QCoro::Task<> e7_fetch(E7State *s, QNetworkAccessManager *nam, QUrl url)
{
    QNetworkReply *r = nam->get(QNetworkRequest(url));
    s->reply = r;
    // D3's dispatch-time RAII wrapper, one level down: the frame owns the reply.
    struct ReplyReleaser
    {
        QNetworkReply *r;
        ~ReplyReleaser() { r->deleteLater(); }
    } releaser{r};
    Sentinel guard(&s->localDestroyed);
    co_await r;
    s->resumed = true;
}

static void e7(QNetworkAccessManager *nam, const QUrl &silentUrl)
{
    section("E7: destroy Task handle while suspended on an in-flight QNetworkReply");
    E7State s;
    {
        auto t = e7_fetch(&s, nam, silentUrl);
        spin(100); // let the request reach the silent server: genuinely in-flight
        check(!t.isReady() && !s.reply.isNull() && !s.reply->isFinished(),
              "reply is in-flight, coroutine suspended");
    }
    check(!s.localDestroyed,
          "frame detached, so the RAII reply wrapper did NOT run at handle destruction");
    check(!s.reply.isNull(),
          "reply still alive (owned by the detached frame's wrapper + QNAM parent)");
    s.reply->abort(); // force a finished emission after the handle is gone
    const bool resumedDuringAbort = s.resumed;
    spinUntil([&] { return s.localDestroyed; });
    check(!resumedDuringAbort,
          "abort()'s finished emission did not resume the coroutine synchronously (queued "
          "connection)");
    if (s.resumed) {
        finding(
            "a finished emission after handle destruction RESUMED the detached coroutine via its "
            "still-live "
            "queued connection — handle destruction does NOT sever the reply-awaiter connection");
    }
    check(s.localDestroyed,
          "detached frame self-destroyed after completing; RAII wrapper released the reply");
    check(spinUntil([&] { return s.reply.isNull(); }),
          "reply deleted via the wrapper's deleteLater once the loop spun");
}

// ---------------------------------------------------------------------------
// E8 (run last, after all event processing has ceased): the post-exec shutdown
// analog — tasks suspended on timer/future/reply, handles destroyed, owners
// destroyed, and NO further event-loop iterations. This is exactly the world
// after a.exec() returns.
// ---------------------------------------------------------------------------

static E5State g_e8Timer;
static E6State g_e8Future;
static E6State g_e8Chain;
static E7State g_e8Reply;
static QPromise<int> *g_e8Promise = nullptr;
static bool g_e8ParseRan = false;
static QFuture<int> g_e8Child;

// The worker's production await (D6): move the payload out through
// qCoro(future).takeResult() — which runs as its own inner QCoro task frame.
static QCoro::Task<> e8_chainAwaiter(E6State *s, QFuture<int> f)
{
    Sentinel guard(&s->localDestroyed);
    co_await qCoro(f).takeResult();
    s->resumed = true;
}

static void e8_setup(QNetworkAccessManager *nam, const QUrl &silentUrl)
{
    section("E8: post-exec shutdown analog (no further event processing after this point)");
    {
        // Duration far beyond the setup spin below — the timer must still be
        // pending when the handles are destroyed, even under sanitizers.
        auto t1 = e5_sleeper(&g_e8Timer, 1h);
        g_e8Promise = new QPromise<int>();
        g_e8Promise->start();
        auto t2 = e6_awaiter(&g_e8Future, g_e8Promise->future());
        // The production topology (D7/D6): the facade returns .then(parse)'s
        // CHILD future and the worker awaits the child via takeResult().
        g_e8Child = g_e8Promise->future().then([](int v) {
            g_e8ParseRan = true;
            return v + 1;
        });
        auto t3 = e8_chainAwaiter(&g_e8Chain, g_e8Child);
        auto t4 = e7_fetch(&g_e8Reply, nam, silentUrl);
        spin(100); // reply becomes in-flight; this is the LAST event processing
        check(!g_e8Reply.reply.isNull() && !g_e8Reply.reply->isFinished(),
              "shutdown-analog reply is in-flight");
    } // all four handles destroyed while suspended → four detached top-level
    // task frames (the sleeper and chain awaiter each also carry a detached
    // inner frame: QCoro::sleepFor's and qCoro().takeResult()'s)
    check(!g_e8Timer.localDestroyed && !g_e8Future.localDestroyed && !g_e8Chain.localDestroyed
              && !g_e8Reply.localDestroyed,
          "all four top-level frames detached (locals alive), matching E5-E7");
}

static void e8_verifyAfterOwnersDestroyed(bool namDestroyedRepliesGone)
{
    check(!g_e8Timer.resumed && !g_e8Future.resumed && !g_e8Chain.resumed && !g_e8Reply.resumed,
          "nothing resumed during owner destruction: no event loop, queued delivery never happens");
    check(!g_e8Future.localDestroyed && !g_e8Chain.localDestroyed && !g_e6a.localDestroyed,
          "destroying the unfinished promises canceled their futures without resuming the detached "
          "awaiters");
    check(!g_e8ParseRan,
          "the parse continuation on the chained child future never ran — promise death broke the "
          "production .then topology without parsing or resuming its awaiter");
    check(g_e8Child.isCanceled() && g_e8Child.isFinished(),
          "the chained child future is canceled and finished after promise destruction — the chain "
          "reached its terminal state synchronously, no event loop involved");
    check(namDestroyedRepliesGone, "QNAM parent destroyed the in-flight reply at owner destruction");
    finding("with no live event loop, detached frames are inert: promises die unfinished (their "
            "futures cancel, "
            ".then chains do not run, awaiters stay suspended), destroying owners (QNAM included) "
            "resumes "
            "nothing — but the frames and their locals are permanently leaked, and the reply "
            "cleanup falls "
            "entirely to the QNAM parent (the RAII wrapper never ran)");
    finding("mid-session the same promise destruction would be ER1's hazard — the watcher would "
            "deliver a "
            "canceled-future resumption into result() once the loop spun — which is why D2 "
            "requires every "
            "live promise to reach a final state and confines unfinished-promise death to "
            "post-loop shutdown");
}

// ---------------------------------------------------------------------------
// E9: exception behavior — unawaited tasks swallow silently (R5-1's premise);
// awaited tasks rethrow.
// ---------------------------------------------------------------------------

static QCoro::Task<> e9_throwerSync()
{
    throw std::runtime_error("boom-sync");
    co_return; // unreachable; makes this a coroutine
}

static QCoro::Task<> e9_throwerAsync()
{
    co_await QCoro::sleepFor(10ms);
    throw std::runtime_error("boom-async");
}

static QCoro::Task<> e9_awaitThrower(bool *caught)
{
    try {
        co_await e9_throwerSync();
    } catch (const std::runtime_error &) {
        *caught = true;
    }
}

static void e9()
{
    section("E9: stored-exception behavior of unawaited tasks");
    bool threwAtCall = false;
    try {
        auto t = e9_throwerSync();
        check(t.isReady(), "task that threw before its first suspension is immediately ready");
    } catch (...) {
        threwAtCall = true;
    }
    check(!threwAtCall, "exception did not propagate out of the coroutine call");

    {
        auto t = e9_throwerAsync();
        check(spinUntil([&] { return t.isReady(); }), "task that threw after resuming is ready");
    } // destroyed unawaited, exception stored inside
    finding("an unawaited task's exception is stored in the promise and vanishes silently when the "
            "handle "
            "dies — no terminate, no log, nothing observes it (confirms R5-1: root coroutines need "
            "catch-alls)");

    bool caught = false;
    auto t = e9_awaitThrower(&caught);
    check(caught, "awaiting a task whose body threw rethrows the stored exception to the awaiter");
}

// ---------------------------------------------------------------------------
// E10: destroying a completed task's handle from inside its own completion
// cascade (the sweep question, R5-1) — self-destruction vs deferred sweep.
// ---------------------------------------------------------------------------

struct E10State
{
    std::optional<QCoro::Task<>> slot;
    bool innerDone = false;
    bool contRan = false;
    bool afterReset = false;
};

static QCoro::Task<> e10_inner(E10State *s)
{
    co_await QCoro::sleepFor(20ms);
    s->innerDone = true;
}

static QCoro::Task<> e10_watcher(E10State *s)
{
    co_await *s->slot; // resumes synchronously inside inner's final_suspend
    s->contRan = true;
    s->slot.reset(); // destroy the completed task's handle from its own completion cascade
    s->afterReset = true;
}

static void e10()
{
    section("E10: destroy a completed task's handle from inside its own completion cascade");
    E10State s;
    s.slot = e10_inner(&s);
    auto w = e10_watcher(&s);
    spinUntil([&] { return s.afterReset; });
    check(s.innerDone && s.contRan && s.afterReset,
          "continuation ran and reset the slot without crashing");
    finding("destroying a completed task's handle from within its own completion cascade did not "
            "crash here "
            "(final_suspend resumes awaiters before releasing its own ref) — but a deferred sweep "
            "remains the "
            "defensive choice; verify under ASAN before relying on this");
}

// ---------------------------------------------------------------------------
// E11: std::stop_callback runs synchronously on the requesting thread — the
// premise that forces queued indirection (R4-2).
// ---------------------------------------------------------------------------

static void e11()
{
    section("E11: std::stop_callback synchrony");
    std::atomic<bool> inside{false};
    bool ranInside = false;
    std::stop_source src;
    std::stop_callback cb(src.get_token(), [&] { ranInside = inside.load(); });
    inside = true;
    src.request_stop();
    inside = false;
    check(ranInside,
          "stop_callback ran synchronously inside request_stop() — raw callbacks must never "
          "resume coroutines directly");
}

// ---------------------------------------------------------------------------

int main(int argc, char **argv)
{
    QCoreApplication app(argc, argv);
    std::printf("QCoro v0.13.0 phase-0 spike (Qt %s)\n", qVersion());

    // Silent HTTP server: accepts connections, never responds — keeps replies
    // in-flight indefinitely.
    QTcpServer server;
    if (!server.listen(QHostAddress::LocalHost, 0)) {
        std::printf("FATAL: could not listen on localhost\n");
        return 1;
    }
    QObject::connect(&server, &QTcpServer::newConnection, &server, [&server] {
        server.nextPendingConnection(); // parented to the server; never respond
    });
    const QUrl silentUrl(QStringLiteral("http://127.0.0.1:%1/").arg(server.serverPort()));

    auto nam = std::make_unique<QNetworkAccessManager>();

    e1();
    e2();
    e3();
    e9();
    e10();
    e11();
    e4();
    e5();
    e6();
    e7(nam.get(), silentUrl);

    // Shutdown analog last: after e8_setup, no more event processing happens.
    e8_setup(nam.get(), silentUrl);
    QPointer<QNetworkReply> e8Reply = g_e8Reply.reply;

    // Real shutdown destroys queued entries' unfinished QPromises (the hub's
    // deques die with it). Destroy them here — the parent future, its
    // .then(parse) chain, and the child future the chain awaiter suspends on
    // must all cancel without parsing or resuming anything.
    delete g_e8Promise;
    g_e8Promise = nullptr;
    delete g_e6aPromise;
    g_e6aPromise = nullptr;

    nam.reset(); // QNAM destroyed with an in-flight reply, loop never spins again
    e8_verifyAfterOwnersDestroyed(e8Reply.isNull());

    // Release main's own copy of the child future before leak accounting so
    // the only remaining retainers of its shared state are the detached
    // frames themselves.
    g_e8Child = QFuture<int>();

    section("end-of-run leak accounting");
    check(!g_e6a.localDestroyed,
          "E6a frame (future never finished) is still leaked at exit, as predicted");
    check(!g_e8Timer.localDestroyed && !g_e8Future.localDestroyed && !g_e8Chain.localDestroyed
              && !g_e8Reply.localDestroyed,
          "E8 detached frames are still leaked at exit, as predicted");
    std::printf("\n%d CHECK failure(s). Deliberate detached-frame leaks at exit: seven coroutine "
                "frame allocations — five top-level task frames (E6a + 4x E8) plus two inner task "
                "frames (QCoro::sleepFor's, qCoro().takeResult()'s) — and whatever only they "
                "retain, future shared state included. The sentinel checks above are the "
                "authoritative accounting; what `leaks` roots varies by run — the reply frame is "
                "a stable root, timer frames surface as root cycles, and the future frames "
                "usually stay hidden behind Qt thread-data (pending cancel call-outs).\n",
                g_checkFailures);
    return g_checkFailures;
}
