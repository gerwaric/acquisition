echo -------------------------------------------------------------

set "WINDEPLOYQT=C:\Qt\6.5.3\msvc2019_64\bin\windeployqt.exe"
set "ISCC=c:\Program Files (x86)\Inno Setup 6\ISCC.exe"

set "BUILD_DIR=.\build\Desktop_Qt_6_5_3_MSVC2019_64bit-Release"
set "DEPLOY_DIR=%BUILD_DIR%\deploy";

rmdir /S /Q "%DEPLOY_DIR%"
mkdir "%DEPLOY_DIR%"

copy "%BUILD_DIR%\acquisition.exe" "%DEPLOY_DIR%"

"%WINDEPLOYQT%" "%DEPLOY_DIR%\acquisition.exe" --release --no-compiler-runtime --dir "%DEPLOY_DIR%"

"%ISCC%" /DBUILD_DIR="%BUILD_DIR%" /DDEPLOY_DIR="%DEPLOY_DIR%" installer.iss
