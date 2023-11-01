#!/bin/bash

MACDEPLOYQT=~/Qt/6.5.3/macos/bin/macdeployqt

TARGET=../build-acquisition-Qt_6_5_3_for_macOS-Release/acquisition.app

$MACDEPLOYQT $TARGET -dmg -appstore-compliant
