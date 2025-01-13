 echo off

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

set "QTBIN=C:\Qt\6.8.1\msvc2022_64\bin"
if not exist "%QTBIN%\." (
    echo ERROR: QTBIN directory not found: "%QTBIN%"
    exit /B
)

set "QTENV2=%QTBIN%\qtenv2.bat"
if not exist "%QTENV2%" (
    echo ERROR: qtenv2.bat not found: "%QTENV2%"
    exit /B
)

set "VCVARSALL=C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat"
if not exist "%VCVARSALL%" (
    echo ERROR: vcvarsall.bat not found: "%VCVARSALL%"
    exit /B
)

set "BUILD_DIR=.\build\Desktop_Qt_6_8_1_MSVC2022_64bit-Release"
if not exist "%BUILD_DIR%\." (
    echo ERROR: build directory not found: "%BUILD_DIR%"
    exit /B
)

rem Need to use pushd before calling these scripts
pushd .
call "%QTENV2%"
call "%VCVARSALL%" x64
popd

cov-build --dir cov-int cmake --build "%BUILD_DIR%" --target all

tar czvf "%BUILD_DIR%\cov-build-win.tgz" "%BUILD_DIR%\cov-int"