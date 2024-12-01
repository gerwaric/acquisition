#!/bin/bash

export QMAKE=${HOME}/Qt/6.8.0/gcc_64/bin/qmake

LINUXDEPLOY=${HOME}/bin/linuxdeploy-x86_64.AppImage
PROJECT_DIR=${PWD}
BUILD_DIR=${PWD}/build/Desktop_Qt_6_8_0-Release

# Take the version string from version_defines.h
export LINUXDEPLOY_OUTPUT_VERSION=`grep APP_VERSION_STRING ${BUILD_DIR}/version_defines.h | cut -d'"' -f2`

# Prepare a clean deployment directory
rm -rf deploy
mkdir -p deploy/usr/bin
cd deploy

# Copy the executable
cp "${BUILD_DIR}/acquisition" usr/bin/

# Build the app image
${LINUXDEPLOY} --appdir . \
	--executable usr/bin/acquisition \
	--desktop-file "${PROJECT_DIR}/acquisition.desktop" \
	--icon-file "${PROJECT_DIR}/assets/icon.svg" \
	--icon-filename default \
    --exclude-library libqsqlmimer \
	--exclude-library libmimerapi \
	--plugin qt \
    --output appimage

