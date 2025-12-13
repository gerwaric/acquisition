#!/bin/bash

# Copyright (C) 2014-2025 Acquisition Contributors
#
# This file is part of Acquisition.
#
# Acquisition is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
# 
# Acquisition is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
# 
# You should have received a copy of the GNU General Public License
# along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.

export QMAKE=${HOME}/Qt/6.10.1/gcc_64/bin/qmake

# Use commands like this to download the latest linuxdeploy images:
# wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
# wget https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/continuous/linuxdeploy-plugin-qt-x86_64.AppImage


LINUXDEPLOY=${HOME}/bin/linuxdeploy-x86_64.AppImage
LINUXDEPLOY_QT=${HOME}/bin/linuxdeploy-plugin-qt-x86_64.AppImage
PROJECT_DIR=${PWD}
BUILD_DIR=${PWD}/build/Desktop_Qt_6_10_1-Release

# Take the version string from version_defines.h
export LINUXDEPLOY_OUTPUT_VERSION=`grep APP_VERSION_STRING ${BUILD_DIR}/version_defines.h | cut -d'"' -f2`

# Prepare a clean deployment directory
rm -rf deploy
mkdir -p deploy/usr/bin
cd deploy

# Copy the executable
cp "${BUILD_DIR}/acquisition" usr/bin/

# As of August 2025 the official linuxdeploy continuous build now
# properly supports excluding qt libraries, which is needed because
# otherwise linuxdeploy will try to add too many database drivers.
LINUXDEPLOY_EXCLUDED_LIBRARIES="\
libqsqlmysql.so;\
libqsqlmimer.so;\
libqsqlodbc.so;\
libqsqlpsql.so;\
libqsqloci.so;\
libsqloci.so;\
libqsqlibase.so"

export LINUXDEPLOY_EXCLUDED_LIBRARIES

# Build the app image
${LINUXDEPLOY} --appdir=. \
	--executable=usr/bin/acquisition \
	--desktop-file="${PROJECT_DIR}/acquisition.desktop" \
	--icon-file="${PROJECT_DIR}/assets/icon.svg" \
	--icon-filename=default \
	--plugin=qt \
	--output=appimage
