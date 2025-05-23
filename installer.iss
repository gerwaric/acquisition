; Acquistion installer script

;-----------------------------------------------------------------------------------------
; Check command-line arguments.

#ifndef BUILD_DIR
#error "BUILD_DIR must be defined using the /D command-line parameter to run this script."
#endif

#ifndef DEPLOY_DIR
#error "DEPLOY_DIR must be defined using the /D command-line parameter to run this script."
#endif

;-----------------------------------------------------------------------------------------
; Load and check the version defines.

#define VERSION_H BUILD_DIR + "\version_defines.h"
#include VERSION_H

#ifndef APP_NAME
#error "APP_NAME not defined in " + VERSION_H
#endif

#ifndef APP_VERSION_STRING
#error "APP_VERSION_STRING not defined in " + VERSION_H
#endif

#ifndef APP_PUBLISHER
#error "APP_PUBLISHER not defined in " + VERSION_H
#endif

#ifndef APP_URL
#error "APP_URL not defined in " + VERSION_H
#endif

;-----------------------------------------------------------------------------------------

#pragma message "BUILD_DIR = " + BUILD_DIR
#pragma message "DEPLY_DIR = " + DEPLOY_DIR
#pragma message "VERSION_H = " + VERSION_H

#pragma message "APP_NAME           = " + APP_NAME
#pragma message "APP_VERSION_STRING = " + APP_VERSION_STRING
#pragma message "APP_PUBLISHER      = " + APP_PUBLISHER
#pragma message "APP_URL            = " + APP_URL

;-----------------------------------------------------------------------------------------
; Script generated by the Inno Setup Script Wizard.
; SEE THE DOCUMENTATION FOR DETAILS ON CREATING INNO SETUP SCRIPT FILES!


[Setup]
; NOTE: The value of AppId uniquely identifies this application. Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
;
; This is the same GUID as the testpushpleaseignore fork of Acquisition:
AppId={{53E25C0C-0305-47BB-9884-F0F202297AF4} 
AppName={#APP_NAME}
AppVersion={#APP_VERSION_STRING}
AppPublisher={#APP_PUBLISHER}
AppPublisherURL={#APP_URL}
;AppSupportURL={#AppURL}
AppUpdatesURL={#APP_URL}
DefaultDirName={autopf}\{#APP_NAME}
DisableProgramGroupPage=yes
LicenseFile=LICENSE
OutputBaseFilename={#APP_NAME}_setup_{#APP_VERSION_STRING}
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
RestartIfNeededByRun=yes
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
MinVersion=10.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "vc_redist"; Description: "Update/Install Microsoft Visual C++ 2015-2022 Runtime";
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; Flags: unchecked

[Files]
Source: "{#DEPLOY_DIR}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
Name: "{autoprograms}\{#APP_NAME}"; Filename: "{app}\{#APP_NAME}.exe"
Name: "{autodesktop}\{#APP_NAME}"; Filename: "{app}\{#APP_NAME}.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\{#APP_NAME}.exe"; Description: "{cm:LaunchProgram,{#StringChange(APP_NAME, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
Filename: "{app}\vc_redist.x64.exe"; Parameters: "/install /passive /norestart"; Tasks: vc_redist;
