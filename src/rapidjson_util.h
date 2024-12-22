/*
    Copyright (C) 2014-2024 Acquisition Contributors

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
