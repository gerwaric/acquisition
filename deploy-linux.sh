#!/bin/bash

# Copyright (C) 2014-2024 Acquisition Contributors
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

export QMAKE=${HOME}/Qt/6.8.1/gcc_64/bin/qmake

LINUXDEPLOY=${HOME}/bin/linuxdeploy-x86_64.AppImage
PROJECT_DIR=${PWD}
BUILD_DIR=${PWD}/build/Desktop_Qt_6_8_1-Release

# Take the version string from version_defines.h
export LINUXDEPLOY_OUTPUT_VERSION=`grep APP_VERSION_STRING ${BUILD_DIR}/version_defines.h | cut -d'"' -f2`

# Prepare a clean deployment directory
rm -rf deploy
mkdir -p deploy/usr/bin
cd deploy

# Copy the executable
cp "${BUILD_DIR}/acquisition" usr/bin/

# Build the app image
${LINUXDEPLOY} --appdir=. \
	--executable=usr/bin/acquisition \
	--desktop-file="${PROJECT_DIR}/acquisition.desktop" \
	--icon-file="${PROJECT_DIR}/assets/icon.svg" \
	--icon-filename=default \
	--plugin=qt \
	--exclude-library=libqsqlmysql.so \
	--exclude-library=libqsqlmimer.so \
	--exclude-library=libqsqlodbc.so \
	--exclude-library=libqsqlpsql.so \
	--output=appimage

