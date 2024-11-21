echo -------------------------------------------------------------

setlocal

rem Setup the Qt environment
pushd .
set "PATH=C:\Qt\6.5.3\msvc2019_64\bin;%PATH%"
popd

rem Setup the Visual Studio environment
call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat" amd64

rem Define the locations of windeployqt and inno setup
set "WINDEPLOYQT=C:\Qt\6.5.3\msvc2019_64\bin\windeployqt.exe"
set "ISCC=c:\Program Files (x86)\Inno Setup 6\ISCC.exe"

rem Define the build and deployment directories
set "BUILD_DIR=.\out\build"
set "DEPLOY_DIR=%BUILD_DIR%\deploy";

rmdir /S /Q "%DEPLOY_DIR%"
mkdir "%DEPLOY_DIR%"

rem Build the deployment directory after copying the necessary executables
copy "%BUILD_DIR%\acquisition.exe" "%DEPLOY_DIR%"
copy "%BUILD_DIR%\crashpad_handler.exe" "%DEPLOY_DIR%"
"%WINDEPLOYQT%" "%DEPLOY_DIR%\acquisition.exe" --release --compiler-runtime --dir "%DEPLOY_DIR%"

rem Create the installer and copy the exe and pdb for uploading to BugSplat
"%ISCC%" /DBUILD_DIR="%BUILD_DIR%" /DDEPLOY_DIR="%DEPLOY_DIR%" installer.iss
copy "%BUILD_DIR%\acquisition.exe" .\Output
copy "%BUILD_DIR%\acquisition.pdb" .\Output
