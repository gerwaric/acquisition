rem Copyright (C) 2014-2024 Acquisition Contributors
rem
rem This file is part of Acquisition.
rem
rem Acquisition is free software: you can redistribute it and/or modify
rem it under the terms of the GNU General Public License as published by
rem the Free Software Foundation, either version 3 of the License, or
rem (at your option) any later version.
rem 
rem Acquisition is distributed in the hope that it will be useful,
rem but WITHOUT ANY WARRANTY; without even the implied warranty of
rem MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
rem GNU General Public License for more details.
rem 
rem You should have received a copy of the GNU General Public License
rem along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.

echo -------------------------------------------------------------

setlocal

rem Setup the Qt environment
pushd .
set "PATH=C:\Qt\6.8.1\msvc2019_64\bin;%PATH%"
popd

rem Define the locations of windeployqt and inno setup
set "WINDEPLOYQT=C:\Qt\6.8.1\msvc2022_64\bin\windeployqt.exe"
set "ISCC=c:\Program Files (x86)\Inno Setup 6\ISCC.exe"

rem Define the build and deployment directories
set "BUILD_DIR=.\build\Desktop_Qt_6_8_1_MSVC2022_64bit-Release"
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
