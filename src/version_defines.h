#pragma once

//Make sure to update version.txt file to match the "VER_CODE" value
//The program will look to this file in the GitHub repo, and compare it to
//the VER_CODE in the current installation. If the number in the version.txt
//file is greater than VER_CODE, then the updater will notify the user
#define VER_CODE                    43
#define VER_STR                     "v0.9.2"
#define VER_FILEVERSION             0,9,0,2
#define VER_FILEVERSION_STR         "0.9.0.2"

#define VER_PRODUCTVERSION          VER_FILEVERSION
#define VER_PRODUCTVERSION_STR      VER_FILEVERSION_STR

#define VER_FILEDESCRIPTION_STR     "Acquisition"
#define VER_ORIGINALFILENAME_STR    "acquisition.exe"
#define VER_INTERNALNAME_STR        VER_FILEDESCRIPTION_STR
#define VER_PRODUCTNAME_STR         VER_FILEDESCRIPTION_STR
