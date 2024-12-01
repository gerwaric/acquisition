#!/bin/bash

MACDEPLOYQT=~/Qt/6.8.0/macos/bin/macdeployqt

TARGET=./build/Qt_6_8_0_for_macOS-Release

pushd $TARGET

$MACDEPLOYQT acquisition.app -verbose=2 -always-overwrite -appstore-compliant -dmg

popd
