#pragma once

#include "json_struct/json_struct.h"

#include <QString>

#include <functional>
#include <string>

namespace PoE {

	JS_ENUM(StashType,
		BlightStash,
		CurrencyStash,
		DeliriumStash,
		DelveStash,
		DivinationCardStash,
		EssenceStash,
		FlasksStash,
		FragmentStash,
		GemStash,
		MapStash,
		MetamorphStash,
		PremiumStash,
		QuadStash,
		UniqueStash);

};

JS_ENUM_NAMESPACE_DECLARE_STRING_PARSER(PoE, StashType);

namespace PoE {

	enum class StringType { LEAGUE_NAME, ACCOUNT_NAME, CHARACTER_NAME, STASH_NAME, STASH_ID, ITEM_ID };

	// https://stackoverflow.com/questions/34287842/strongly-typed-using-and-typedef
	template<StringType ID>
	class String {
	public:

		// Construct from std::string.
		String(std::string s) : value_(std::move(s)) {};

		// default constructor is needed for use in STL containers like map.
		String() : value_() {};

		// Provide access to the underlying string value.
		const std::string& value() const { return value_; };

		explicit operator bool() const { return !value_.empty(); };
		//explicit operator const std::string& () const { return value_; };
		explicit operator const QString() const { return QString::fromStdString(value_); };

		// Make the type handler a friend so it can directly read/write value_.
		friend struct JS::TypeHandler<String>;

	private:
		std::string value_;
		friend bool operator<(const String& l, const String& r) {
			return l.value_ < r.value_;
		};
		friend bool operator==(const String& l, const String& r) {
			return l.value_ == r.value_;
		};
		friend const std::string& to_string(const String& r) {
			return r.value_;
		};
		friend std::ostream& operator<<(std::ostream& os, const String& sid) {
			return os << sid.value_;
		};
		friend std::size_t hash_code(const String& sid) {
			return typeid(ID).hash_code() ^ std::hash<std::string>()(sid.value_);
		};
	};

	using LeagueName = String<StringType::LEAGUE_NAME>;
	using AccountName = String<StringType::ACCOUNT_NAME>;
	using CharacterName = String<StringType::CHARACTER_NAME>;
	using StashId = String<StringType::STASH_ID>;
	using StashName = String<StringType::STASH_NAME>;
	using ItemId = String<StringType::ITEM_ID>;
}

template<PoE::StringType ID>
struct std::hash<PoE::String<ID>> {
	using T = PoE::String<ID>;
	std::size_t operator()(const T& arg) const {
		return hash_code(arg);
	};
};

template<PoE::StringType ID>
struct JS::TypeHandler<PoE::String<ID>> {
	using T = PoE::String<ID>;
	static inline JS::Error to(T& to_type, JS::ParseContext& context) {
		return JS::TypeHandler<std::string>::to(to_type.value_, context);
	};
	static void from(const T& from_type, JS::Token& token, JS::Serializer& serializer) {
		JS::TypeHandler<std::string>::from(from_type.value_, token, serializer);
	};
};
