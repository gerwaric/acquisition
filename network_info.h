#pragma once

#include "version_defines.h"

#define _STRINGIFY(x)                     #x
#define STRINGIFY(x)                      _STRINGIFY(x)
#define _CONCAT(x,y)                      x##y
#define CONCAT(x,y)                       _CONCAT(x,y)

//#define USER_AGENT                        STRINGIFY(CONCAT(CONCAT(Acquisition/, VER_STR), (contact: tom.holz@gmail.com)))
#define USER_AGENT                        VER_PRODUCTNAME_STR "/" VER_STR " (contact: tom.holz@gmail.com)"

