// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "util/logging.h"

#include <sentry.h>
#include <spdlog/sinks/callback_sink.h>
#include <spdlog/sinks/rotating_file_sink.h>

#ifdef _WIN32
#include <spdlog/sinks/msvc_sink.h>
#else
#include <spdlog/sinks/stdout_color_sinks.h>
#endif

#include "util/spdlog_qt.h" // IWYU pragma: keep

constexpr int MAX_FILES = 20;

constexpr int MAX_LOG_SIZE = 10 * 1024 * 1024; // 10MB

#ifdef Q_OS_WIN
// Visual Studio debug output on Windows
using DEBUG_SINK = spdlog::sinks::msvc_sink_mt;
#else
// Debug console output sink for macOS/Linux)
using DEBUG_SINK = spdlog::sinks::stdout_color_sink_mt;
#endif

void logging::init(const QString &filename)
{
    // Create a debug sink for the c
    auto debug_sink = std::make_shared<DEBUG_SINK>();
    debug_sink->set_level(spdlog::level::trace);

    // Create a file sink for the log file.
    const auto path = filename.toStdString();
    auto file_sink = std::make_shared<spdlog::sinks::rotating_file_sink_mt>(path,
                                                                            MAX_LOG_SIZE,
                                                                            MAX_FILES);
    file_sink->set_level(spdlog::level::trace);

    /* TBD:
    // Create a sink for sentry (but only capture errors)
    auto sentry_sink = std::make_shared<spdlog::sinks::callback_sink_mt>(
        [](const spdlog::details::log_msg &msg) {
            // Make absolutely sure we are only logging errors and fatal errors,
            // since there seems to be no way to filter sentry logs, e.g. to only
            // report the X latest log messages.
            if (msg.level >= spdlog::level::err) {
                std::string payload{msg.payload.data(), msg.payload.size()};
                sentry_log_error(payload.c_str());
            }
        });
    sentry_sink->set_level(spdlog::level::err);
    */

    // Create sinks vector
    std::vector<spdlog::sink_ptr> sinks = {
        debug_sink,
        file_sink,
        // sentry_sink,
    };

    // Create logger with both sinks
    auto logger = std::make_shared<spdlog::logger>("main", sinks.begin(), sinks.end());

    spdlog::set_default_logger(logger);
    spdlog::set_pattern("[%Y-%m-%d %H:%M:%S.%e] [%l] %v");
}
