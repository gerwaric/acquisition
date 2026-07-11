// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "filters/filtermatchers.h"

#include <QStringList>
#include <QtGlobal>

#include <type_traits>

#include "item.h"
#include "itemconstants.h"

bool MatchesAltart(const Item &item)
{
    static const QStringList altart = {
        // season 1
        "RedBeak2.png",
        "Wanderlust2.png",
        "Ring2b.png",
        "Goldrim2.png",
        "FaceBreaker2.png",
        "Atzirismirror2.png",
        // season 2
        "KaruiWardAlt.png",
        "ShiverstingAlt.png",
        "QuillRainAlt.png",
        "OnyxAmuletAlt.png",
        "DeathsharpAlt.png",
        "CarnageHeartAlt.png",
        "TabulaRasaAlt.png",
        "andvariusAlt.png",
        "AstramentisAlt.png",
        // season 3
        "BlackheartAlt.png",
        "SinTrekAlt.png",
        "ShavronnesPaceAlt.png",
        "Belt3Alt.png",
        "EyeofChayulaAlt.png",
        "SundanceAlt.png",
        "ReapersPursuitAlt.png",
        "WindscreamAlt.png",
        "RainbowStrideAlt.png",
        "TarynsShiverAlt.png",
        // season 4
        "BrightbeakAlt.png",
        "RubyRingAlt.png",
        "TheSearingTouchAlt.png",
        "CloakofFlameAlt.png",
        "AtzirisFoibleAlt.png",
        "DivinariusAlt.png",
        "HrimnorsResolveAlt.png",
        "CarcassJackAlt.png",
        "TheIgnomonAlt.png",
        "HeatShiverAlt.png",
        // season 5
        "KaomsSignAlt.png",
        "StormcloudAlt.png",
        "FairgravesTricorneAlt.png",
        "MoonstoneRingAlt.png",
        "GiftsfromAboveAlt.png",
        "LeHeupofAllAlt.png",
        "QueensDecreeAlt.png",
        "PerandusSignetAlt.png",
        "AuxiumAlt.png",
        "dGlsbGF0ZUFsdCI7czoy",
        // season 6
        "PerandusBlazonAlt.png",
        "AurumvoraxAlt.png",
        "GoldwyrmAlt.png",
        "AmethystAlt.png",
        "DeathRushAlt.png",
        "RingUnique1.png",
        "MeginordsGirdleAlt.png",
        "SidhebreathAlt.png",
        "MingsHeartAlt.png",
        "VoidBatteryAlt.png",
        // season 7
        "Empty-Socket2.png",
        "PrismaticEclipseAlt.png",
        "ThiefsTorment2.png",
        "Amulet5Unique2.png",
        "FurryheadofstarkonjaAlt.png",
        "Headhunter2.png",
        "Belt6Unique2.png",
        "BlackgleamAlt.png",
        "ThousandribbonsAlt.png",
        "IjtzOjI6InNwIjtkOjAu",
        // season 8
        "TheThreeDragonsAlt.png",
        "ImmortalFleshAlt.png",
        "DreamFragmentsAlt2.png",
        "BereksGripAlt.png",
        "SaffellsFrameAlt.png",
        "BereksRespiteAlt.png",
        "LifesprigAlt.png",
        "PillaroftheCagedGodAlt.png",
        "BereksPassAlt.png",
        "PrismaticRingAlt.png",
        // season 9
        "Fencoil.png",
        "TopazRing.png",
        "Cherufe2.png",
        "cy9CbG9ja0ZsYXNrMiI7",
        "BringerOfRain.png",
        "AgateAmuletUnique2.png",
        // season 10
        "StoneofLazhwarAlt.png",
        "SapphireRingAlt.png",
        "CybilsClawAlt.png",
        "DoedresDamningAlt.png",
        "AlphasHowlAlt.png",
        "dCI7czoyOiJzcCI7ZDow",
        // season 11
        "MalachaisArtificeAlt.png",
        "MokousEmbraceAlt.png",
        "RusticSashAlt2.png",
        "MaligarosVirtuosityAlt.png",
        "BinosKitchenKnifeAlt.png",
        "WarpedTimepieceAlt.png",
        // emberwake season
        "UngilsHarmonyAlt.png",
        "LightningColdTwoStoneRingAlt.png",
        "EdgeOfMadnessAlt.png",
        "RashkaldorsPatienceAlt.png",
        "RathpithGlobeAlt.png",
        "EmberwakeAlt.png",
        // bloodgrip season
        "GoreFrenzyAlt.png",
        "BloodGloves.png",
        "BloodAmuletALT.png",
        "TheBloodThornALT.png",
        "BloodJewel.png",
        "BloodRIng.png",
        // soulthirst season
        "ThePrincessAlt.png",
        "EclipseStaff.png",
        "Perandus.png",
        "SoultakerAlt.png",
        "SoulthirstALT.png",
        "bHQiO3M6Mjoic3AiO2Q6",
        // winterheart season
        "AsphyxiasWrathRaceAlt.png",
        "SapphireRingRaceAlt.png",
        "TheWhisperingIceRaceAlt.png",
        "DyadianDawnRaceAlt.png",
        "CallOfTheBrotherhoodRaceAlt.png",
        "WinterHeart.png",
    };

    for (const auto &needle : altart) {
        if (item.icon().contains(needle)) {
            return true;
        }
    }
    return false;
}

bool matches(const Item &item, const TextState &state, const TextPayload &payload)
{
    return payload.value(item).toLower().contains(state.query.toLower());
}
bool matches(const Item &item, const ComboState &state, const ComboPayload &payload)
{
    if (state.value.isEmpty()) {
        return true;
    }

    switch (payload.matchKind) {
    case ComboMatchKind::CategoryContains:
        return item.category().contains(state.value);
    case ComboMatchKind::Rarity: {
        const QString &query = state.value;
        const QStringList &choices = RarityChoices();
        switch (item.frameType()) {
        case FrameType::FRAME_TYPE_NORMAL:
            return (query == choices[1]) || (query == choices[6]);
        case FrameType::FRAME_TYPE_MAGIC:
            return (query == choices[2]) || (query == choices[6]);
        case FrameType::FRAME_TYPE_RARE:
            return (query == choices[3]) || (query == choices[6]);
        case FrameType::FRAME_TYPE_UNIQUE:
            return query == choices[4];
        case FrameType::FRAME_TYPE_FOIL:
        case FrameType::FRAME_TYPE_SUPPORTER_FOIL:
            return query == choices[5];
        default:
            return false;
        }
    }
    }

    Q_ASSERT(false);
    return false;
}
bool matches(const Item &item, const MinMaxState &state, const MinMaxPayload &payload)
{
    if (!payload.present(item)) {
        return !state.min.has_value() && !state.max.has_value();
    }

    const double value = payload.value(item);
    if (state.min.has_value() && *state.min > value) {
        return false;
    }
    if (state.max.has_value() && *state.max < value) {
        return false;
    }
    return true;
}
bool matches(const Item &, const ColorsState &, const ColorsPayload &)
{
    return true;
}
bool matches(const Item &item, const BoolState &state, const BoolPayload &payload)
{
    return !state.checked || payload.predicate(item);
}
bool matches(const Item &, const ModsState &, const ModsPayload &)
{
    return true;
}

bool MatchesFilter(const Item &item, const FilterSpec &spec, const FilterState &state)
{
    return std::visit(
        [&item, &state](const auto &payload) {
            using Payload = std::decay_t<decltype(payload)>;
            if constexpr (std::is_same_v<Payload, LegacyPayload>) {
                Q_ASSERT(std::holds_alternative<LegacyState>(state));
                return true;
            } else {
                using State = std::conditional_t<
                    std::is_same_v<Payload, TextPayload>,
                    TextState,
                    std::conditional_t<
                        std::is_same_v<Payload, ComboPayload>,
                        ComboState,
                        std::conditional_t<
                            std::is_same_v<Payload, MinMaxPayload>,
                            MinMaxState,
                            std::conditional_t<std::is_same_v<Payload, ColorsPayload>,
                                               ColorsState,
                                               std::conditional_t<std::is_same_v<Payload, BoolPayload>,
                                                                  BoolState,
                                                                  ModsState>>>>>;
                const auto *typedState = std::get_if<State>(&state);
                Q_ASSERT(typedState);
                return typedState ? matches(item, *typedState, payload) : false;
            }
        },
        spec.payload);
}
