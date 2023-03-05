#!/usr/bin/env bash
set -euxo pipefail

echo $MODE
echo $ARCHIVE_NAME

TMP=$(mktemp -d)

trap "rm -rf $TMP" EXIT

mkdir $TMP/$ARCHIVE_NAME
cp -v target/$MODE/boat_journey_wgpu $TMP/$ARCHIVE_NAME/boat-journey-graphical
cp -v target/$MODE/boat_journey_ggez $TMP/$ARCHIVE_NAME/boat-journey-graphical-compatibility
cp -v target/$MODE/boat_journey_ansi_terminal $TMP/$ARCHIVE_NAME/boat-journey-terminal
cp -v extras/unix/* $TMP/$ARCHIVE_NAME

pushd $TMP
zip $ARCHIVE_NAME.zip $ARCHIVE_NAME/*
tar -cvf $ARCHIVE_NAME.tar.gz $ARCHIVE_NAME
popd
mv $TMP/$ARCHIVE_NAME.zip .
mv $TMP/$ARCHIVE_NAME.tar.gz .
