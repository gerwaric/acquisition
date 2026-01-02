// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Tom Holz

#include "logging.h"

#include <spdlog/sinks/rotating_file_sink.h>

#ifdef _WIN32
#include <spdlog/sinks/msvc_sink.h>
#else
#include <spdlog/sinks/stdout_color_sinks.h>
#endif

#include <util/spdlog_qt.h>

constexpr int MAX_FILES = 20;

constexpr int MAX_LOG_SIZE = 10 * 1024 * 1024; // 10MB

void logging::init(const QString &filename)
{
    // Create sinks vector
    std::vector<spdlog::sink_ptr> sinks;

#ifdef _WIN32
    // Visual Studio debug output on Windows
    auto debug_sink = std::make_shared<spdlog::sinks::msvc_sink_mt>();
#else
    // Debug console output sink for macOS/Linux)
    auto debug_sink = std::make_shared<spdlog::sinks::stdout_color_sink_mt>();
#endif
    debug_sink->set_level(spdlog::level::trace);
    sinks.push_back(debug_sink);

    const auto path = filename.toStdString();
    auto file_sink = std::make_shared<spdlog::sinks::rotating_file_sink_mt>(path,
                                                                            MAX_LOG_SIZE,
                                                                            MAX_FILES);
    file_sink->set_level(spdlog::level::trace);
    sinks.push_back(file_sink);

    // Create logger with both sinks
    auto logger = std::make_shared<spdlog::logger>("main", sinks.begin(), sinks.end());

    spdlog::set_default_logger(logger);
    spdlog::set_pattern("[%Y-%m-%d %H:%M:%S.%e] [%l] %v");
}
