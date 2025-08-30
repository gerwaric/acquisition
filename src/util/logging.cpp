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
