#pragma once

#include "boost/predef.h"

#include "version_defines.h"

#define _STRINGIFY(x)                     #x
#define STRINGIFY(x)                      _STRINGIFY(x)

#ifdef BOOST_COMP_MSVC
#define CONCAT(x,y)                       _CONCATX(x,y)
#else
#define _CONCAT(x,y)                      x##y
#define CONCAT(x,y)                       _CONCAT(x,y)
#endif

#define USER_AGENT                        VER_PRODUCTNAME_STR "/" VER_STR " (contact: gerwaric@gmail.com)"

