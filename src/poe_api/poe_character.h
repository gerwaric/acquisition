/*
	Copyright 2023 Gerwaric

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
#pragma once

#include "json_struct/json_struct.h"
#include "poe_api/poe_item.h"
#include "poe_api/poe_passives.h"

#include <optional>
#include <string>
#include <unordered_map>
#include <vector>

class QObject;

namespace PoE {

	// Anonymous member of Character
	struct CharacterPassives {
		std::vector<unsigned int>                   hashes;					// array of uint
		std::vector<unsigned int>                   hashes_ex;				// array of uint
		std::unordered_map<std::string, int>        mastery_effects;		// dictionary of int	the key is the string value of the mastery node skill hashand the value is the selected effect hash
		std::unordered_map<std::string, PoE::PassiveNode> skill_overrides;       // dictionary of PassiveNode	the key is the string value of the node identifier being replaced
		std::optional<std::string>					bandit_choice;			// ? string	one of Kraityn, Alira, Oak, or Eramir
		std::optional<std::string>					pantheon_major;			// ? string	one of TheBrineKing, Arakaali, Solaris, or Lunaris
		std::optional<std::string>					pantheon_minor;			// ? string	one of Abberath, Gruthkul, Yugul, Shakari, Tukohama, Ralakesh, Garukhan, or Ryslatha
		std::unordered_map<std::string, PoE::ItemJewelData> jewel_data;          // dictionary of ItemJewelData	the key is the string value of the x property of an item from the jewels array in this request
		JS_OBJ(hashes, hashes_ex, mastery_effects, skill_overrides, bandit_choice, pantheon_major, pantheon_minor, jewel_data);
	};

	// Anonymous member of Character
	struct CharacterMetadata {
		std::string                                 version;				// game version from the character's realm (not defined in dev docs, Oct 2023)
		JS_OBJ(version);
	};

	// https://www.pathofexile.com/developer/docs/reference#type-Character
	struct Character {
		std::string                                 id;						// string	a unique 64 digit hexadecimal string
		std::string                                 name;					// string
		std::string                                 realm;					// string	pc, xbox, or sony
		std::string                                 class_name;				// string
		std::optional<std::string>					league;					// ? string
		unsigned int                                level = 0;				// uint
		unsigned int                                experience = 0;			// uint
		std::optional<bool>                         ruthless;				// ? bool	always true if present
		std::optional<bool>                         expired;				// ? bool	always true if present
		std::optional<bool>                         deleted;				// ? bool	always true if present
		std::optional<bool>                         current;				// ? bool	always true if present
		std::optional<std::vector<Item>>			equipment;				// ? array of Item
		std::optional<std::vector<Item>>			inventory;				// ? array of Item
		std::optional<std::vector<Item>>			jewels;					// ? array of Item
		std::optional<PoE::CharacterPassives>       passives;				// ? object
		std::optional<PoE::CharacterMetadata>       metadata;				// ? object
		JS_OBJECT(
			JS_MEMBER(id),
			JS_MEMBER(name),
			JS_MEMBER(realm),
			JS_MEMBER_WITH_NAME(class_name, "class"),
			JS_MEMBER(league),
			JS_MEMBER(level),
			JS_MEMBER(experience),
			JS_MEMBER(ruthless),
			JS_MEMBER(expired),
			JS_MEMBER(deleted),
			JS_MEMBER(current),
			JS_MEMBER(equipment),
			JS_MEMBER(inventory),
			JS_MEMBER(jewels),
			JS_MEMBER(passives),
			JS_MEMBER(metadata));
	};

	struct ListCharactersResult {
		std::vector<Character> characters;
		JS_OBJ(characters);
	};
	struct GetCharacterResult {
		Character character;
		JS_OBJ(character);
	};

	typedef std::function<void(const ListCharactersResult&)> ListCharactersCallback;
	typedef std::function<void(const GetCharacterResult&)> GetCharacterCallback;

	void ListCharacters(QObject* object, ListCharactersCallback callback);
	void GetCharacter(QObject* object, GetCharacterCallback callback, const std::string& name);

	struct LegacyCharacter {
		LegacyCharacter(const Character& character);
		std::string name;
		std::string realm;
		std::string class_name;
		std::string league;
		int         level;
		bool        pinnable;
		int         i;
		JS_OBJECT(
			JS_MEMBER(name),
			JS_MEMBER(realm),
			JS_MEMBER_WITH_NAME(class_name, "class"),
			JS_MEMBER(league),
			JS_MEMBER(level),
			JS_MEMBER(pinnable),
			JS_MEMBER(i));
	};

}