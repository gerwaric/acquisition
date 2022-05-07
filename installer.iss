#include "src\version_defines.h"

#define AppExeName VER_ORIGINALFILENAME_STR
#define AppName VER_FILEDESCRIPTION_STR
#define AppVersion VER_STR

[Setup]
AppId={{53E25C0C-0305-47BB-9884-F0F202297AF4}
AppName={#AppName}
AppVersion={#AppVersion}
DefaultDirName={commonpf}\{#AppName}
DefaultGroupName={#AppName}
LicenseFile=COPYING
OutputBaseFilename=acquisition_setup_{#AppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes
LZMADictionarySize=1048576
LZMANumFastBytes=273
LZMANumBlockThreads=6
DisableDirPage=auto

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "deploy\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs

[InstallDelete]
Type: filesandordirs; Name: "{app}\bearer"

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{commondesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\vcredist_x86.exe"; Parameters: "/passive /norestart"
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
