#!/bin/bash

QMAKE=${HOME}/Qt/6.5.3/gcc_64/bin/qmake
DEPLOYQT=${HOME}/bin/linuxdeployqt-continuous-x86_64.AppImage
BUILD_DIR=../build-acquisition-Desktop_Qt_6_5_3_GCC_64bit-Release

# Take the version string from version_defines.h
export VERSION=`grep APP_VERSION_STRING src/version_defines.h | cut -d'"' -f2`

# Prepare a clean deployment directory
rm -rf ./deploy
mkdir ./deploy
cd deploy

# Copy the executable and build the app image
cp ${BUILD_DIR}/acquisition .
${DEPLOYQT} ../acquisition.desktop -qmake=${QMAKE} -appimage -verbose=1

