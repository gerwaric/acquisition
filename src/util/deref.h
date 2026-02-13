// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#pragma once

#include <memory>

#include "util/fatalerror.h"

template<typename T>
T &deref(std::unique_ptr<T> &pointer, const char *context)
{
    if (!pointer) {
        FatalError(QString("%1: tried to dereference a null pointer to '%2'")
                       .arg(context, typeid(T).name()));
    }
    return *pointer;
}
