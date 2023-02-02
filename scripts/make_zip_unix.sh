#!/usr/bin/env bash
set -euxo pipefail

echo $MODE
echo $ZIP_NAME

cargo build --release --manifest-path=ggez/Cargo.toml
cargo build --release --manifest-path=wgpu/Cargo.toml
cargo build --release --manifest-path=ansi-terminal/Cargo.toml

TMP=$(mktemp -d)
mkdir $TMP/$ZIP_NAME
cp -v target/$MODE/orbital_decay_wgpu $TMP/$ZIP_NAME/orbital-decay-graphical
cp -v target/$MODE/orbital_decay_ggez $TMP/$ZIP_NAME/orbital-decay-graphical-compatibility
cp -v target/$MODE/orbital_decay_ansi_terminal $TMP/$ZIP_NAME/orbital-decay-terminal

cp -v extras/unix/* $TMP/$ZIP_NAME

pushd $TMP
zip $ZIP_NAME.zip $ZIP_NAME/*
popd
mv $TMP/$ZIP_NAME.zip .
