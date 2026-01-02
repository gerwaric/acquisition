echo off

rem Copyright (C) 2014-2025 Acquisition Contributors
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

set "QTBIN=C:\Qt\6.10.1\msvc2022_64\bin"
if not exist "%QTBIN%\." (
    echo ERROR: QTBIN directory not found: "%QTBIN%"
    exit /B
)

set "QTENV2=%QTBIN%\qtenv2.bat"
if not exist "%QTENV2%" (
    echo ERROR: qtenv2.bat not found: "%QTENV2%"
    exit /B
)

set "VCVARSALL=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat"
if not exist "%VCVARSALL%" (
    echo ERROR: vcvarsall.bat not found: "%VCVARSALL%"
    exit /B
)

set "WINDEPLOYQT=%QTBIN%\windeployqt.exe"
if not exist "%WINDEPLOYQT%" (
    echo ERROR: windeployqt.exe not found: "%WINDEPLOYQT%"
    exit /B
)

set "ISCC=c:\Program Files (x86)\Inno Setup 6\ISCC.exe"
if not exist "%ISCC%" (
    echo Error: iscc.exe not found: "%ISCC%"
    exit /B
)

set "BUILD_DIR=.\build\Desktop_Qt_6_10_1_MSVC2022_64bit-Release"
if not exist "%BUILD_DIR%\." (
    echo ERROR: build directory not found: "%BUILD_DIR%"
    exit /B
)
if not exist "%BUILD_DIR%\acquisition.exe" (
    echo ERROR: acquisition.exe not found in "%BUILD_DIR%".
    exit /B
)
if not exist "%BUILD_DIR%\acquisition.pdb" (
    echo ERROR: acquisition.pdb not found in "%BUILD_DIR%".
    exit /B
)

rem Need to use pushd before calling these scripts
pushd .
call "%QTENV2%"
call "%VCVARSALL%" x64
popd

set "DEPLOY_DIR=%BUILD_DIR%\deploy";
if exist "%DEPOY_DIR%\NUL" (
    echo Deleting the existing deploy directory.
    rmdir /S /Q "%DEPLOY_DIR%"
)
echo Creating the deploy directory.
mkdir "%DEPLOY_DIR%"

echo Copying acquisition to deploy directory.
copy "%BUILD_DIR%\acquisition.exe" "%DEPLOY_DIR%"

echo Running windeployqt.
"%WINDEPLOYQT%" "%DEPLOY_DIR%\acquisition.exe" --release --compiler-runtime --dir "%DEPLOY_DIR%"

if not exist "%DEPLOY_DIR%\vc_redist.x64.exe" (
    echo ERROR: vc_redist.x64.exe not found in "%DEPLOY_DIR%".
    exit /B
)
vc_redist.x64.exe


echo Calling Inno Setup to create the installer.
"%ISCC%" /DBUILD_DIR="%BUILD_DIR%" /DDEPLOY_DIR="%DEPLOY_DIR%" installer.iss

echo Copying the aquisition executable to the output directory.
copy "%BUILD_DIR%\acquisition.exe" .\Output

echo Copying the acquisition debug symbols to the output directory.
copy "%BUILD_DIR%\acquisition.pdb" .\Output
