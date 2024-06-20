#!/bin/bash

MACDEPLOYQT=~/Qt/6.5.3/macos/bin/macdeployqt

TARGET=./build/Qt_6_5_3_for_macOS-Release

pushd $TARGET

$MACDEPLOYQT acquisition.app -verbose=2 -always-overwrite -appstore-compliant -dmg

popd
