#pragma once

#include "rapidjson/document.h"

typedef rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator> rapidjson_allocator;

namespace rapidjson {

    template <typename Encoding, typename Allocator>
    typename GenericValue<Encoding, Allocator>::ValueIterator begin(GenericValue<Encoding, Allocator>& v) { return v.Begin(); }
    template <typename Encoding, typename Allocator>
    typename GenericValue<Encoding, Allocator>::ConstValueIterator begin(const GenericValue<Encoding, Allocator>& v) { return v.Begin(); }

    template <typename Encoding, typename Allocator>
    typename GenericValue<Encoding, Allocator>::ValueIterator end(GenericValue<Encoding, Allocator>& v) { return v.End(); }
    template <typename Encoding, typename Allocator>
    typename GenericValue<Encoding, Allocator>::ConstValueIterator end(const GenericValue<Encoding, Allocator>& v) { return v.End(); }

    // Helper functions to simplify parsing code.

    inline static bool HasString(const rapidjson::Value& json, const char* name) {
        return json.HasMember(name) && json[name].IsString();
    };
    inline static bool HasInt(const rapidjson::Value& json, const char* name) {
        return json.HasMember(name) && json[name].IsInt();
    };
    inline static bool HasUint(const rapidjson::Value& json, const char* name) {
        return json.HasMember(name) && json[name].IsUint();
    };
    inline static bool HasBool(const rapidjson::Value& json, const char* name) {
        return json.HasMember(name) && json[name].IsBool();
    };
    inline static bool HasObject(const rapidjson::Value& json, const char* name) {
        return json.HasMember(name) && json[name].IsObject();
    };
    inline static bool HasArray(const rapidjson::Value& json, const char* name) {
        return json.HasMember(name) && json[name].IsArray();
    };

} // namespace rapidjson
