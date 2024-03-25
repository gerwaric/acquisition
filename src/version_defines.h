#pragma once

//Make sure to update version.txt file to match the "VER_CODE" value
//The program will look to this file in the GitHub repo, and compare it to
//the VER_CODE in the current installation. If the number in the version.txt
//file is greater than VER_CODE, then the updater will notify the user.
#define VERSION_CODE                57

// These are used by the Inno 6 installer script.
#define APP_NAME                    "acquisition"
#define APP_VERSION                 "0.10.3"
#define APP_VERSION_STRING          "v0.10.3-alpha.1"
#define APP_PUBLISHER               "GERWARIC"
#define APP_PUBLISHER_EMAIL         "gerwaric@gmail.com"
#define APP_URL                     "https://github.com/gerwaric/acquisition"

// These control if this build will expire.
#define TRIAL_VERSION               0
#define TRIAL_EXPIRATION_DAYS       0
