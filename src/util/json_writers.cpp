#include "util/json_writers.h"

#include "poe/types/character.h"
#include "poe/types/stashtab.h"
#include "util/glaze_qt.h"
#include "util/spdlog_qt.h"

static_assert(ACQUISITION_USE_GLAZE);
static_assert(ACQUISITION_USE_SPDLOG);

QByteArray writeCharacter(const poe::Character &character)
{
    std::string json;
    const auto ec = glz::write_json(character, json);
    if (ec) {
        const auto msg = glz::format_error(ec);
        spdlog::error("Error writing character: {}", msg);
        return {};
    }
    return QByteArray{json};
}

QByteArray writeCharacterList(const std::vector<poe::Character> &characters)
{
    std::string json;
    const auto ec = glz::write_json(characters, json);
    if (ec) {
        const auto msg = glz::format_error(ec);
        spdlog::error("Error writing character list: {}", msg);
        return {};
    }
    return QByteArray{json};
}

QByteArray writeStash(const poe::StashTab &stash)
{
    std::string json;
    const auto ec = glz::write_json(stash, json);
    if (ec) {
        const auto msg = glz::format_error(ec);
        spdlog::error("Error writing stash tab: {}", msg);
        return {};
    }
    return QByteArray{json};
}

QByteArray writeStashList(const std::vector<poe::StashTab> &stashes)
{
    std::string json;
    const auto ec = glz::write_json(stashes, json);
    if (ec) {
        const auto msg = glz::format_error(ec);
        spdlog::error("Error writing character list: {}", msg);
        return {};
    }
    return QByteArray{json};
}
