// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include "legacy/legacyfolderimporter.h"

#include "datastore/userstore.h"

LegacyFolderImporter::LegacyFolderImporter(UserStore &store)
    : m_store(store)
{}
