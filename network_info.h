#pragma once

#include "version_defines.h"

#include <boost/predef.h>

#define _STRINGIFY(x)                     #x
#define STRINGIFY(x)                      _STRINGIFY(x)

#ifndef BOOST_COMP_MSVC
#define _CONCAT(x,y)                      x##y
#define CONCAT(x,y)                       _CONCAT(x,y)
#endif

//#define USER_AGENT                        STRINGIFY(CONCAT(CONCAT(Acquisition/, VER_STR), (contact: tom.holz@gmail.com)))
#define USER_AGENT                        VER_PRODUCTNAME_STR "/" VER_STR " (contact: tom.holz@gmail.com)"

