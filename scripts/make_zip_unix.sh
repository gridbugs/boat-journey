#!/bin/bash
set -euxo pipefail

echo $MODE
echo $ZIP_NAME

TMP=$(mktemp -d)
mkdir $TMP/$ZIP_NAME
cp -v target/$MODE/orbital_decay_graphical $TMP/$ZIP_NAME/orbital-decay-graphical
cp -v target/$MODE/orbital_decay_ansi_terminal $TMP/$ZIP_NAME/orbital-decay-terminal

cp -v extras/unix/* $TMP/$ZIP_NAME

pushd $TMP
zip $ZIP_NAME.zip $ZIP_NAME/*
popd
mv $TMP/$ZIP_NAME.zip .
