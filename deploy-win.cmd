echo -------------------------------------------------------------

set "WINDEPLOYQT=C:\Qt\5.15.2\msvc2019_64\bin\windeployqt.exe"
set "OPENSSL=C:\Program Files\OpenSSL-Win64"
set "ISCC=c:\Program Files (x86)\Inno Setup 6\ISCC.exe"

set "BUILD_DIR=..\build-acquisition-Desktop_Qt_5_15_2_MSVC2019_64bit-Release"

set "DEPLOY_DIR=.\deploy";

rmdir /S /Q "%DEPLOY_DIR%"
mkdir "%DEPLOY_DIR%"

copy "%BUILD_DIR%\release\acquisition.exe" "%DEPLOY_DIR%"
copy "%OPENSSL%\bin\libcrypto-1_1-x64.dll" "%DEPLOY_DIR%"
copy "%OPENSSL%\bin\libssl-1_1-x64.dll" "%DEPLOY_DIR%"

"%WINDEPLOYQT%" "%DEPLOY_DIR%\acquisition.exe" --release --no-compiler-runtime --dir "%DEPLOY_DIR%"

"%ISCC%" ".\installer.iss"
