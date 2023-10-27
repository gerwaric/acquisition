#!/bin/bash

MACDEPLOYQT=~/Qt/5.15.10/universal/bin/macdeployqt

TARGET=../build-acquisition-Qt_5_15_10_for_macOS-Release/acquisition.app

$MACDEPLOYQT $TARGET -dmg -appstore-compliant
